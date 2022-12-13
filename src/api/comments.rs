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

use super::{User, Base64, generate_random_token, verify_read_permission, require_moderator};

pub fn router() -> Router {
    Router::new()
        .route("/api/comments", post(index))
        .route("/api/comment", post(create))
        .route(
            "/api/comment/:comment_id",
            post(reply)
                .patch(approve)
                .delete(destroy)
                .put(update)
        )
}

#[derive(Serialize)]
struct CommentWithReplies {
    id: i64,
    name: String,
    html_body: String,
    body: String,
    avatar: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    reviewed: bool,
    owned: bool,
    edited: bool,
    replies: Vec<OwnedComment>,
}

#[derive(Serialize)]
struct OwnedComment {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    html_body: String,
    body: String,
    avatar: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    reviewed: bool,
    owned: bool,
    edited: bool,
}

#[derive(Serialize)]
struct CommentsPage {
    total: i64,
    cursor: Option<String>,
    comments: Vec<CommentWithReplies>,
}

#[derive(Deserialize)]
struct CommentData {
    body: String,
    name: Option<String>,
    token: Option<Base64>,
}

const COMMENTS_PER_PAGE: i64 = 42;

fn comments_page(
    parents: Vec<Comment>,
    all_replies: Vec<Comment>,
    total: i64,
    token: &Option<Base64>,
) -> CommentsPage {
    let parents_len = parents.len() as i64;
    let mut comments = vec![];
    let mut count = 0_i64;

    for parent in parents {
        if count == COMMENTS_PER_PAGE {
            break;
        }

        let mut replies = vec![];

        let comment_replies: Vec<Comment> = all_replies
            .iter()
            .filter(|r| r.parent_id == Some(parent.id))
            .cloned()
            .collect();

        for r in comment_replies {
            let owned = match token {
                None => false,
                Some(t) => t == &r.token
            };
            replies.push(OwnedComment {
                id: r.id,
                parent_id: r.parent_id,
                name: r.name,
                html_body: r.html_body,
                body: r.body,
                avatar: r.avatar,
                created_at: r.created_at,
                updated_at: r.updated_at,
                reviewed: r.reviewed,
                edited: r.created_at != r.updated_at,
                owned,
            })
        }

        let owned = match token {
            None => false,
            Some(t) => t == &parent.token
        };

        comments.push(CommentWithReplies {
            id: parent.id,
            name: parent.name,
            html_body: parent.html_body,
            body: parent.body,
            avatar: parent.avatar,
            created_at: parent.created_at,
            updated_at: parent.updated_at,
            reviewed: parent.reviewed,
            edited: parent.created_at != parent.updated_at,
            owned,
            replies,
        });

        count += 1;
    }

    let cursor = if COMMENTS_PER_PAGE < parents_len {
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
    }
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
        COMMENTS_PER_PAGE + 1,
        show_only_reviewed,
        req.payload.as_ref().map_or(&None, |p| &p.token),
        cursor
    ).await?;

    let replies = comments::nested_replies(
        &ctx.db,
        show_only_reviewed,
        req.payload.as_ref().map_or(&None, |p| &p.token),
        &parents
    ).await?;

    Ok(Json(comments_page(
        parents,
        replies,
        total,
        req.payload.as_ref().map_or(&None, |p| &p.token),
    )))
}

#[derive(Serialize)]
struct PostCommentResponse {
    token: Base64,
    comment: OwnedComment,
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

fn get_markdown(data: &str) -> Result<String> {
    Ok(
        markdown::to_html_with_options(data, &markdown::Options::gfm())
            .map_err(|_| Error::UnprocessableEntity("Your comment contains invalid markdown"))?
    )
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
                db, page.id, parent_id, &name, &get_markdown(&data.body)?, &data.body, &avatar,
                reviewed_at, data.token.as_ref().unwrap_or(&generate_random_token())
            ).await?;

            let owned = match &data.token {
                None => false,
                Some(t) => t == &comment.token
            };

            Ok(Json({
                PostCommentResponse {
                    token: comment.token,
                    comment: OwnedComment {
                        id: comment.id,
                        parent_id: comment.parent_id,
                        html_body: comment.html_body,
                        name: comment.name,
                        body: comment.body,
                        avatar: comment.avatar,
                        created_at: comment.created_at,
                        updated_at: comment.updated_at,
                        reviewed: comment.reviewed,
                        edited: comment.created_at != comment.updated_at,
                        owned,
                    }
                }
            }))
        }
    }
}

fn ensure_modifiable(user: Option<&User>, token: Option<&Base64>, comment: &Comment) -> Result<()> {
    match user {
        Some(u) if u.moderator => Ok(()),
        _ => match token {
            None => Err(Error::Forbidden),
            Some(t) => {
                let now = Utc::now();
                let created = &comment.created_at;
                let diff = now - *created;
                if t == &comment.token && diff.num_minutes() <= 3 { return Ok(()) }

                Err(Error::Forbidden)
            }
        }
    }
}

/// PUT /api/comment/42
async fn update(
    ctx: Context,
    Path(comment_id): Path<i64>,
    Json(req): Json<ApiRequest<CommentData>>,
) -> Result<Json<PostCommentResponse>> {
    match req.payload {
        None => Err(Error::UnprocessableEntity("Payload can't be blank")),
        Some(ref data) => {
            if data.body.trim().len() < 1 { return Err(Error::UnprocessableEntity("Comment can't be blank")) }

            let (_, user) = req.extract_verified(&ctx.db).await?;

            let comment = comments::find(&ctx.db, comment_id).await?;

            ensure_modifiable(
                user.as_ref(),
                req.payload.as_ref().and_then(|p| p.token.as_ref()),
                &comment
            )?;

            let updated_comment = comments::update(&ctx.db, comment_id, &get_markdown(&data.body)?, &data.body).await?;

            let owned = match &data.token {
                None => false,
                Some(t) => t == &comment.token
            };

            Ok(
                Json(PostCommentResponse {
                    token: updated_comment.token,
                    comment: OwnedComment {
                        id: updated_comment.id,
                        parent_id: updated_comment.parent_id,
                        html_body: updated_comment.html_body,
                        name: updated_comment.name,
                        body: updated_comment.body,
                        avatar: updated_comment.avatar,
                        created_at: updated_comment.created_at,
                        updated_at: updated_comment.updated_at,
                        reviewed: updated_comment.reviewed,
                        edited: updated_comment.created_at != updated_comment.updated_at,
                        owned,
                    }
                })
            )
        }
    }
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

/// DELETE /api/comment/42
async fn destroy(
    ctx: Context,
    Path(comment_id): Path<i64>,
    Json(req): Json<ApiRequest<Base64>>,
) -> Result<String> {
    let comment = comments::find(&ctx.db, comment_id).await?;

    let (_, user) = req.extract_verified(&ctx.db).await?;

    ensure_modifiable(
        user.as_ref(),
        req.payload.as_ref().and_then(|p| Some(p)),
        &comment
    )?;

    let _ = comments::delete(&ctx.db, comment_id).await?;
    Ok("Success".to_string())
}

