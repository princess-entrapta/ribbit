use redis::AsyncCommands;

use crate::indexing::InsertHandle;
use crate::schemas::{AppError, PostEntity};
use crate::searchdb::Repository;
use futures::future::try_join_all;

impl InsertHandle<String, String, PostEntity, AppError> for Repository {
    async fn insert_tags(&self, tags: Vec<String>, item_ref: String) -> Result<(), AppError> {
        let item_ref_str = item_ref.as_str();
        try_join_all(tags.into_iter().map(|tag| async move {
            self.redka
                .clone()
                .client
                .sadd::<String, String, Vec<String>>(
                    format!("tag.{}", tag.as_str()),
                    item_ref_str.to_owned(),
                )
                .await
        }))
        .await?;
        Ok(())
    }

    async fn insert_item(&self, item: PostEntity) -> Result<(), AppError> {
        self.redka
            .clone()
            .client
            .set::<&str, String, String>(
                format!("post.{}", item.slug).as_str(),
                serde_json::to_string(&item).unwrap(),
            )
            .await?;
        Ok(())
    }

    async fn insert_alias(&self, phrase: String, tags: Vec<String>) -> Result<(), AppError> {
        self.redka
            .clone()
            .client
            .sadd::<_, _, Vec<String>>(format!("aliases.{phrase}"), tags)
            .await?;
        Ok(())
    }
}
