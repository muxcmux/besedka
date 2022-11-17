use crate::{
    api::{Context, Result},
    db::{configs::Config, pages::{find_by_site_and_path, create_or_find_by_site_and_path}, users::get_commenter},
};
use axum::{extract::Path, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::{query_as, FromRow, SqlitePool};

use super::{Cursor, Error,  AuthenticatedConfig, extractors::AuthenticatedModerator};

pub fn router() -> Router {
    Router::new()
        .route("/api/comments", post(comments))
        .route("/api/comment", post(post_comment))
        .route("/api/thread/:comment_id", post(thread))
}

#[derive(FromRow, Clone, Debug, Serialize)]
struct Comment {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    body: String,
    avatar: Option<String>,
    replies_count: i64,
    locked: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct CommentWithReplies {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    body: String,
    avatar: Option<String>,
    replies_count: i64,
    locked: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
}

#[derive(Deserialize, Debug)]
struct CommentsRequest {
    site: String,
    path: String,
    config: Option<String>,
    signature: Option<String>,
}

impl AuthenticatedConfig for CommentsRequest {
    fn site(&self)      -> String { self.site.clone() }
    fn json(&self)      -> Option<String> { self.config.clone() }
    fn signature(&self) -> Option<String> { self.signature.clone() }
}

#[derive(Deserialize)]
struct PostCommentRequest {
    site: String,
    path: String,
    comment: CommentData,
    config: Option<String>,
    signature: Option<String>,
}

impl AuthenticatedConfig for PostCommentRequest {
    fn site(&self)      -> String { self.site.clone() }
    fn json(&self)      -> Option<String> { self.config.clone() }
    fn signature(&self) -> Option<String> { self.signature.clone() }
}

#[derive(Deserialize)]
struct CommentData {
    name: Option<String>,
    body: String,
}

async fn parents(
    db: &SqlitePool,
    page_id: i64,
    limit: i64,
    cursor: Option<Cursor>,
) -> Result<Vec<Comment>> {
    let query = match cursor {
        Some(cur) => {
            query_as::<_, Comment>(
                r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, created_at, updated_at
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed_at NOT NULL
                    AND parent_id IS NULL
                    AND (datetime(created_at) < datetime(?) OR (datetime(created_at) = datetime(?) AND id < ?))
                    ORDER BY datetime(created_at) DESC
                    LIMIT ?
                "#,
             )
             .bind(page_id)
             .bind(cur.created_at)
             .bind(cur.created_at)
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
                r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, created_at, updated_at
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed_at NOT NULL
                    AND parent_id IS NULL
                    ORDER BY datetime(created_at) DESC
                    LIMIT ?
                "#,
            )
            .bind(page_id)
            .bind(limit)
        }
    };

    Ok(query.fetch_all(db).await?)
}

async fn nested_replies(
    db: &SqlitePool,
    limit: i64,
    parents: &Vec<Comment>,
) -> Result<Vec<Comment>> {
    let parent_ids: Vec<String> = parents.iter().map(|p| p.id.to_string()).collect();
    let ids = parent_ids.join(",");

    let query = format!(
        r#"
        SELECT
            id, parent_id, name, body, avatar, replies_count,
            locked,  created_at, updated_at
        FROM (
            SELECT
                r.id AS id,
                r.parent_id AS parent_id,
                r.name AS name,
                r.body AS body,
                r.avatar AS avatar,
                r.replies_count AS replies_count,
                r.locked AS locked,
                r.created_at AS created_at,
                r.updated_at AS updated_at,
                row_number() OVER (PARTITION BY c.id ORDER BY datetime(r.created_at), r.id) AS rn
            FROM comments c
            LEFT JOIN comments r
            ON r.parent_id = c.id
            WHERE r.reviewed_at NOT NULL
        )
        WHERE parent_id IN({ids})
        AND id NOT NULL
        AND rn <= {limit}
        ORDER BY datetime(created_at) ASC;
    "#,
        ids = ids,
        limit = limit
    );

    Ok(query_as::<_, Comment>(&query).fetch_all(db).await?)
}

fn comments_page(
    parents: Vec<Comment>,
    all_replies: Vec<Comment>,
    total: i64,
    config: Config,
) -> CommentsPage {
    let parents_len = parents.len() as i64;
    let mut comments = vec![];
    let mut count = 0_i64;

    for parent in parents {
        if count == config.comments_per_page {
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
            if reply_count == config.replies_per_comment {
                break;
            }
            replies.push(reply);
            reply_count += 1;
        }

        let cursor = if config.replies_per_comment < replies_len {
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
            parent_id: parent.parent_id,
            name: parent.name,
            body: parent.body,
            avatar: parent.avatar,
            replies_count: parent.replies_count,
            locked: parent.locked,
            created_at: parent.created_at,
            updated_at: parent.updated_at,
            thread: Thread { replies, cursor },
        });
        count += 1;
    }

    let cursor = if config.comments_per_page < parents_len {
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

/// POST /api/comments
async fn comments(
    ctx: Context,
    cursor: Option<Cursor>,
    moderator: Option<AuthenticatedModerator>,
    Json(comments_request): Json<CommentsRequest>,
) -> Result<Json<CommentsPage>> {
    let (config, authenticated_user) = comments_request.authenticated_config(&ctx.db).await?;

    if config.private && authenticated_user.is_none() && moderator.is_none() { return Err(Error::Unauthorized) }

    let page = find_by_site_and_path(&ctx.db, &comments_request.site, &comments_request.path).await?;
    match page {
        None => Err(Error::NotFound),
        Some(p) => {
            // We need the fetch limit + 1 in order
            // to work out if there is a next page or not
            let parents = parents(&ctx.db, p.id, config.comments_per_page + 1, cursor).await?;
            let replies = nested_replies(&ctx.db, config.replies_per_comment + 1, &parents).await?;

            Ok(Json(comments_page(
                parents,
                replies,
                p.comments_count,
                config,
            )))
        }
    }
}

async fn replies(
    db: &SqlitePool,
    parent_id: i64,
    limit: i64,
    cursor: Option<Cursor>,
) -> Result<Vec<Comment>> {
    let query = match cursor {
        Some(cur) => {
            query_as::<_, Comment>(
                r#"
                     SELECT id, parent_id, name, body, avatar, replies_count,
                     locked, created_at, updated_at
                     FROM comments
                     WHERE parent_id = ?
                     AND reviewed_at NOT NULL
                     AND (datetime(created_at) > datetime(?) OR (datetime(created_at) = datetime(?) AND id > ?))
                     ORDER BY datetime(created_at) ASC
                     LIMIT ?
                 "#,
             )
             .bind(parent_id)
             .bind(cur.created_at)
             .bind(cur.created_at)
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
               r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, created_at, updated_at
                    FROM comments
                    WHERE parent_id = ?
                    AND reviewed_at NOT NULL
                    ORDER BY datetime(created_at) ASC
                    LIMIT ?
                "#,
            )
            .bind(parent_id)
            .bind(limit)
        }
    };

    Ok(query.fetch_all(db).await?)
}

/// POST /api/thread/42
async fn thread(
    ctx: Context,
    cursor: Option<Cursor>,
    moderator: Option<AuthenticatedModerator>,
    Path(comment_id): Path<i64>,
    Json(comments_request): Json<CommentsRequest>,
) -> Result<Json<Thread>> {
    let (config, authenticated_user) = comments_request.authenticated_config(&ctx.db).await?;

    if config.private && authenticated_user.is_none() && moderator.is_none() { return Err(Error::Unauthorized) }

    let all_replies = replies(&ctx.db, comment_id, config.replies_per_comment + 1, cursor).await?;

    let replies_len = all_replies.len() as i64;
    let mut replies = vec![];
    let mut count = 0_i64;

    for reply in all_replies {
        if count == config.replies_per_comment {
            break;
        }

        replies.push(reply);
        count += 1;
    }

    let cursor = if config.replies_per_comment < replies_len {
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

// /// POST /api/comment
async fn post_comment(
    ctx: Context,
    moderator: Option<AuthenticatedModerator>,
    Json(comment_request): Json<PostCommentRequest>,
) -> Result<Json<Comment>> {
    let (config, authenticated_user) = comment_request.authenticated_config(&ctx.db).await?;

    let requires_user = config.private || !config.anonymous_comments;
    let no_user = moderator.is_none() && authenticated_user.is_none();

    if requires_user && no_user { return Err(Error::Unauthorized) }

    let page = create_or_find_by_site_and_path(&ctx.db, &comment_request.site, &comment_request.path).await?;
    if page.locked { return Err(Error::Forbidden) }

    let mut q = String::from("INSERT INTO comments (page_id");
    let mut v = String::from(" VALUES (?");


    let commenter = get_commenter(&ctx.db, &comment_request.site, authenticated_user, moderator).await?;

    if commenter.is_some() {
        q.push_str(", user_id");
        v.push_str(", ?");
    }



    Err(Error::NotFound)
}
