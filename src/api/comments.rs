use crate::{
    api::{ApiRequest, Cursor, Error, Context, Result},
    db::{
        comments::{Comment, self},
        pages::{Page, self},
        sites::Site,
    },
};
use axum::{extract::Path, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{User, Base64, generate_random_token};

pub fn router() -> Router {
    Router::new()
        .route("/api/comments", post(index))
        .route("/api/comments/:comment_id", post(thread))
        .route("/api/comment", post(create))
        .route(
            "/api/comment/:comment_id",
            post(reply)
                .patch(approve)
                // .delete(destroy)
                // .put(update)
        )
}

#[derive(Serialize)]
struct CommentWithReplies {
    id: i64,
    name: String,
    body: String,
    avatar: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    reviewed: bool,
    thread: Thread,
}

#[derive(Serialize)]
struct Thread {
    cursor: Option<String>,
    replies: Vec<Comment>,
}

#[derive(Serialize)]
struct CommentsPage {
    total: i64,
    cursor: Option<String>,
    comments: Vec<CommentWithReplies>,
    site: Site,
}

#[derive(Deserialize)]
struct CommentData {
    body: String,
    name: Option<String>,
    token: Option<Base64>,
}

fn comments_page(
    parents: Vec<Comment>,
    all_replies: Vec<Comment>,
    total: i64,
    site: Site,
) -> CommentsPage {
    let parents_len = parents.len() as i64;
    let mut comments = vec![];
    let mut count = 0_i64;

    for parent in parents {
        if count == site.comments_per_page {
            break;
        }

        let mut replies = vec![];

        let comment_replies: Vec<Comment> = all_replies
            .iter()
            .filter(|r| r.parent_id == Some(parent.id))
            .cloned()
            .collect();

        let replies_len = comment_replies.len() as i64;

        let mut reply_count = 0_i64;
        for reply in comment_replies {
            if reply_count == site.replies_per_comment {
                break;
            }
            replies.push(reply);
            reply_count += 1;
        }

        let cursor = if site.replies_per_comment < replies_len {
            Some(
                Cursor {
                    id: replies.last().unwrap().id,
                    created_at: replies.last().unwrap().created_at,
                }
                .encode(),
            )
        } else {
            None
        };

        comments.push(CommentWithReplies {
            id: parent.id,
            name: parent.name,
            body: parent.body,
            avatar: parent.avatar,
            created_at: parent.created_at,
            updated_at: parent.updated_at,
            reviewed: parent.reviewed,
            thread: Thread { replies, cursor },
        });
        count += 1;
    }

    let cursor = if site.comments_per_page < parents_len {
        Some(
            Cursor {
                id: comments.last().unwrap().id,
                created_at: comments.last().unwrap().created_at,
            }
            .encode(),
        )
    } else {
        None
    };

    CommentsPage {
        comments,
        cursor,
        total,
        site,
    }
}

fn verify_read_permission(site: &Site, user: &Option<User>, page: Option<&Page>) -> Result<()> {
    if site.private && user.is_none() { return Err(Error::Unauthorized) }

    if let Some(p) = page {
        if p.site != site.site { return Err(Error::BadRequest("Wrong site requested")) }
    }

    Ok(())
}

async fn comment_and_page(ctx: &Context, comment_id: i64) -> Result<(Comment, Page)> {
    let comment = comments::find_root(&ctx.db, comment_id).await?;
    let page_id = comment.page_id;
    Ok((comment, pages::find(&ctx.db, page_id).await?))
}

#[derive(Deserialize)]
struct ListCommentsRequest {
    token: Option<Base64>
}

/// POST /api/comments
async fn index(
    ctx: Context,
    cursor: Option<Cursor>,
    Json(req): Json<ApiRequest<ListCommentsRequest>>,
) -> Result<Json<CommentsPage>> {
    let (site, user) = req.extract_verified(&ctx.db).await?;

    verify_read_permission(&site, &user, None)?;

    let page = pages::find_by_site_and_path(&ctx.db, &req.site, &req.path).await?;

    let show_only_reviewed = user
        .as_ref()
        .map_or(true, |u| !u.moderator);
    // We need the fetch limit + 1 in order
    // to work out if there is a next page or not
    let (total, parents) = comments::root_comments(
        &ctx.db,
        page.id,
        site.comments_per_page + 1,
        show_only_reviewed,
        req.payload.as_ref().map_or(&None, |p| &p.token),
        cursor
    ).await?;

    let replies = comments::nested_replies(
        &ctx.db,
        site.replies_per_comment + 1,
        show_only_reviewed,
        req.payload.as_ref().map_or(&None, |p| &p.token),
        &parents
    ).await?;

    Ok(Json(comments_page(
        parents,
        replies,
        total,
        site,
    )))
}

/// POST /api/thread/42
async fn thread(
    ctx: Context,
    cursor: Option<Cursor>,
    Path(comment_id): Path<i64>,
    Json(req): Json<ApiRequest<ListCommentsRequest>>,
) -> Result<Json<Thread>> {
    let (site, user) = req.extract_verified(&ctx.db).await?;
    let (_, page) = comment_and_page(&ctx, comment_id).await?;
    verify_read_permission(&site, &user, Some(&page))?;

    let show_only_reviewed = user
        .as_ref()
        .map_or(true, |u| !u.moderator);

    let all_replies = comments::replies(
        &ctx.db,
        comment_id,
        site.replies_per_comment + 1,
        cursor,
        show_only_reviewed,
        req.payload.as_ref().map_or(&None, |p| &p.token),
    ).await?;

    let replies_len = all_replies.len() as i64;
    let mut replies = vec![];
    let mut count = 0_i64;

    for reply in all_replies {
        if count == site.replies_per_comment {
            break;
        }

        replies.push(reply);
        count += 1;
    }

    let cursor = if site.replies_per_comment < replies_len {
        Some(
            Cursor {
                id: replies.last().unwrap().id,
                created_at: replies.last().unwrap().created_at,
            }
            .encode(),
        )
    } else {
        None
    };

    Ok(Json(Thread { replies, cursor }))
}

#[derive(Serialize)]
struct PostCommentResponse {
    token: Base64,
    comment: Comment,
}
/// POST /api/comment
async fn create(
    ctx: Context,
    Json(req): Json<ApiRequest<CommentData>>,
) -> Result<Json<PostCommentResponse>> {
    Ok(post_comment(&ctx.db, req, None).await?)
}

/// POST /api/comment/42
async fn reply(
    ctx: Context,
    Path(comment_id): Path<i64>,
    Json(req): Json<ApiRequest<CommentData>>,
) -> Result<Json<PostCommentResponse>> {
    Ok(post_comment(&ctx.db, req, Some(comment_id)).await?)
}

fn authorize_posting(site: &Site, user: &Option<User>, page: &Page) -> Result<()> {
    verify_read_permission(site, user, Some(page))?;
    if user.is_none() && !site.anonymous { return Err(Error::Unauthorized) }
    if page.locked { return Err(Error::Forbidden) }
    Ok(())
}

async fn post_comment(
    db: &SqlitePool,
    req: ApiRequest<CommentData>,
    parent_id: Option<i64>
) -> Result<Json<PostCommentResponse>> {
    match req.payload {
        None => Err(Error::UnprocessableEntity("Payload can't be blank")),
        Some(ref data) => {
            if data.body.trim().len() < 1 { return Err(Error::UnprocessableEntity("Comment can't be blank")) }

            let (site, user) = req.extract_verified(db).await?;
            let page = match parent_id {
                None => pages::create_or_find_by_site_and_path(db, &req.site, &req.path).await?,
                Some(pid) => {
                    let parent = comments::find_root(db, pid).await?;
                    pages::find(db, parent.page_id).await?
                }
            };

            authorize_posting(&site, &user, &page)?;

            let anon = String::from("Anonymous");
            let (mut name, avatar) = user
                .as_ref()
                .map_or((data.name.as_ref().unwrap_or(&anon), None), |c| {
                    (&c.name, c.avatar.as_ref())
                });
            if name.trim() == "" { name = &anon }

            let reviewed_at = !site.moderated || (user.is_some() && user.as_ref().unwrap().moderator);

            let comment = comments::create(
                db, page.id, parent_id, &name, &data.body, &avatar,
                reviewed_at, data.token.as_ref().unwrap_or(&generate_random_token())
            ).await?;

            Ok(Json({
                PostCommentResponse { token: comment.token.clone(), comment }
            }))
        }
    }
}

fn require_moderator(user: &Option<User>) -> Result<()> {
    match user {
        None => return Err(Error::Unauthorized),
        Some(u) => if !u.moderator { return Err(Error::Forbidden) }
    };
    Ok(())
}

/// PATCH /api/comment/42
async fn approve(
    ctx: Context,
    Path(comment_id): Path<i64>,
    Json(req): Json<ApiRequest<()>>,
) -> Result<String> {
    let (_, user) = req.extract_verified(&ctx.db).await?;
    require_moderator(&user)?;

    comments::approve(&ctx.db, comment_id).await?;

    Ok("Success".to_string())
}

// fn modifiable(user: Option<&User>, sid: Option<&Base64>, cResult<_, _> {omment_sid: &Base64) -> bool {
//     match user {
//         Some(u) if u.moderator => true,
//         _ => match sid {
//             None => false,
//             Some(s) => s == comment_sid,
//         }
//     }
// }
//
// #[derive(Serialize)]
// struct DeleteCommentData { sid: Option<Base64> }
// /// DELETE /api/comment/42
// async fn destroy(
//     ctx: Context,
//     Path(comment_id): Path<i64>,
//     Json(req): Json<ApiRequest<DeleteCommentData>>,
// ) -> Result<String> {
//     let comment = find(&ctx.db, comment_id).await?;
//
//     let (_, user) = req.extract_verified(&ctx.db).await?;
//
//     if !modifiable(user.as_ref(), req.payload.as_ref().and_then(|p| p.sid.as_ref()), &comment.sid) { return Err(Error::Forbidden) }
//
//     let _payload = &req.payload;
//
//     let _ = comments::delete(&ctx.db, comment_id).await?;
//     Ok("Success".to_string())
// }

