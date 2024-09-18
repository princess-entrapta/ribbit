use redis::AsyncCommands;
use spow::pow::Pow;

use crate::pow::PowValidator;
use crate::schemas::PostEntity;
use crate::search::{ItemRepo, SearchDb};
use crate::{schemas::AppError, search::SearchCache};

#[derive(Debug, Clone)]
pub struct RepositoryDb {
    pub client: redis::aio::MultiplexedConnection,
}

impl From<redis::RedisError> for AppError {
    fn from(value: redis::RedisError) -> Self {
        return Self {
            reason: value.to_string(),
        };
    }
}

impl RepositoryDb {
    pub async fn new(redka_host: &str) -> Self {
        Self {
            client: redis::Client::open(format!("redis://{redka_host}").as_str())
                .unwrap()
                .get_multiplexed_tokio_connection()
                .await
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryCache {
    pub cache: redis::aio::MultiplexedConnection,
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub redis: RepositoryCache,
    pub redka: RepositoryDb,
}

impl Repository {
    pub async fn new(redis_host: &str, redka_host: &str) -> Self {
        Self {
            redis: RepositoryCache::new(redis_host).await,
            redka: RepositoryDb::new(redka_host).await,
        }
    }
}

impl RepositoryCache {
    pub async fn new(redis_host: &str) -> Self {
        Self {
            cache: redis::Client::open(format!("redis://{redis_host}").as_str())
                .unwrap()
                .get_multiplexed_tokio_connection()
                .await
                .unwrap(),
        }
    }
}

impl SearchCache<String, AppError> for RepositoryCache {
    async fn get_cached_search(&self, _search_tags: &str) -> Result<Vec<String>, AppError> {
        // Enable cache
        /*

        Ok(self
            .cache
            .clone()
            .smembers::<_, Option<Vec<String>>>(format!("search.{search_tags}"))
            .await?
            .unwrap_or(vec![]))
        */
        //Disabled cache
        Ok(vec![])
    }

    async fn cache_search(&self, search_tags: &str, results: Vec<String>) -> Result<(), AppError> {
        if results.is_empty() {
            return Ok(());
        }
        self.cache
            .clone()
            .sadd(format!("search.{search_tags}"), results.clone())
            .await?;
        self.cache
            .clone()
            .expire::<_, String>(format!("search.{search_tags}"), 600)
            .await?;
        Ok(())
    }
}

impl SearchDb<String, String, PostEntity, AppError> for RepositoryDb {
    async fn get_item_refs_from_tag(&self, tag: String) -> Result<Vec<String>, AppError> {
        let key = format!("tag.{tag}");
        let members = self
            .client
            .clone()
            .smembers::<&str, Vec<String>>(key.as_str())
            .await?;
        Ok(members)
    }

    async fn get_item_from_ref(&self, slug: String) -> Result<PostEntity, AppError> {
        let key = format!("post.{slug}");
        let json_str: String = self
            .client
            .clone()
            .get::<&str, String>(key.as_str())
            .await?;
        Ok(serde_json::from_str(json_str.as_str()).unwrap())
    }

    async fn get_tags_from_phrase(&self, w: &str) -> Result<Vec<String>, AppError> {
        let tags = self
            .client
            .clone()
            .smembers::<&str, Vec<String>>(format!("aliases.{w}").as_str())
            .await?;
        if tags.is_empty() {
            return Ok(vec![w.to_string()]);
        }
        Ok(tags)
    }
}

impl ItemRepo<String, String, PostEntity, AppError> for Repository {
    fn get_cache(&self) -> impl SearchCache<String, AppError> {
        self.redis.clone()
    }

    fn get_db(&self) -> impl SearchDb<String, String, PostEntity, AppError> {
        self.redka.clone()
    }
}

impl PowValidator for RepositoryCache {
    async fn is_valid_pow(&self, challenges: [String; 16]) -> bool {
        for challenge in challenges {
            if Pow::validate(&challenge).is_err() {
                return false;
            }
            if self
                .cache
                .clone()
                .get(challenge.clone())
                .await
                .unwrap_or(true)
            {
                return false;
            }
            let _ = self
                .cache
                .clone()
                .set_ex::<_, _, String>(challenge, true, 600)
                .await;
        }
        true
    }
}

impl Repository {
    pub fn get_pow_validator(&self) -> impl PowValidator {
        return self.redis.clone();
    }
}
