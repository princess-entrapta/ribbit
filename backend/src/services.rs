use std::usize;

use crate::indexing::{insert_and_index_item, InsertHandle};
use crate::schemas::{AppError, AuthorInfo, Page, Post};
use crate::schemas::{AuthorEntity, PostEntity};
use crate::search::{ItemRepo, SearchDb};

use tokio::task::JoinError;

impl From<JoinError> for AppError {
    fn from(value: JoinError) -> Self {
        Self {
            reason: value.to_string(),
        }
    }
}
impl Post {
    pub fn from_store(entity: PostEntity, author: AuthorEntity) -> Self {
        Self {
            title: entity.title.clone(),
            slug: entity.slug.clone(),
            author: AuthorInfo {
                name: author.name,
                profile_picture: author.profile_picture,
            },
            body: entity.body.clone(),
            can_reply: false,
        }
    }
}

pub async fn find_posts(
    db: impl ItemRepo<String, String, PostEntity, AppError>,
    search_query: &str,
    page_num: usize,
) -> Result<Page<Post>, AppError> {
    let (posts, nb_items) = db
        .get_items_for_search(search_query, 20, 1, 20, page_num)
        .await?;

    Ok(Page {
        objects: posts
            .into_iter()
            .map(|r| {
                Post::from_store(
                    r.clone(),
                    AuthorEntity {
                        author_id: r.author,
                        name: "sample name".to_string(),
                        profile_picture: "https://example.com".to_string(),
                    },
                )
            })
            .collect(),
        total_objects: nb_items,
        current_page: page_num,
        per_page: 20,
    })
}

pub async fn find_post(
    db: impl SearchDb<String, String, PostEntity, AppError>,
    slug: String,
) -> Result<Post, AppError> {
    let entity = db.get_item_from_ref(slug).await?;
    Ok(Post::from_store(
        entity.clone(),
        AuthorEntity {
            author_id: entity.author,
            name: "sample name".to_string(),
            profile_picture: "https://example.com".to_string(),
        },
    ))
}

pub async fn register_post(
    db: impl InsertHandle<String, String, PostEntity, AppError>,
    form: PostEntity,
) -> Result<(), AppError> {
    insert_and_index_item(&db, form.slug.clone(), form.clone(), form.search_tags()).await
}
