use core::hash::Hash;
use futures::future::try_join_all;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;

pub trait SearchCache<ItemRef, DbError>
where
    Self: Sync + Send,
{
    fn cache_search(
        &self,
        search_query: &str,
        results: Vec<ItemRef>,
    ) -> impl std::future::Future<Output = Result<(), DbError>> + std::marker::Send;
    fn get_cached_search(
        &self,
        search_query: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ItemRef>, DbError>> + std::marker::Send;
}

pub trait SearchDb<Tag, ItemRef, Item, DbError>
where
    Self: Sync + Send,
{
    fn get_item_refs_from_tag(
        &self,
        tag: Tag,
    ) -> impl std::future::Future<Output = Result<Vec<ItemRef>, DbError>> + std::marker::Send;
    fn get_item_from_ref(
        &self,
        item_ref: ItemRef,
    ) -> impl std::future::Future<Output = Result<Item, DbError>> + std::marker::Send;
    fn get_tags_from_phrase(
        &self,
        phrase: &str,
    ) -> impl std::future::Future<Output = Result<Vec<Tag>, DbError>> + std::marker::Send;
}

pub trait ItemRepo<Tag, ItemRef, Item, DbError>
where
    ItemRef: Ord + Eq + Hash + Clone + Sync + Send + std::fmt::Debug,
    Item: Send,
    Tag: Clone + std::fmt::Debug,
    DbError: Send,
    Self: Sync,
{
    fn get_cache(&self) -> impl SearchCache<ItemRef, DbError>;
    fn get_db(&self) -> impl SearchDb<Tag, ItemRef, Item, DbError>;

    fn get_item_refs_for_phrase(
        &self,
        phrase: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ItemRef>, DbError>> + Send {
        async {
            let tags = self.get_db().get_tags_from_phrase(phrase).await?;
            let db = self.get_db();
            let hashset: HashSet<_, RandomState> = HashSet::from_iter(
                try_join_all(tags.into_iter().map(|t| db.get_item_refs_from_tag(t)))
                    .await?
                    .into_iter()
                    .flatten(),
            );

            Ok(hashset.into_iter().collect())
        }
    }
    fn get_item_refs_search_query(
        &self,
        search_query: &str,
        word_max: usize,
        phrase_max: usize,
    ) -> impl std::future::Future<Output = Result<Vec<ItemRef>, DbError>> + Send {
        async move {
            let mut counter: HashMap<ItemRef, usize> = HashMap::new();
            let to_process_words = search_query
                .split(" ")
                .filter(|w| w.len() > 0)
                .take(word_max)
                .collect::<Vec<_>>();
            let to_process_phrases = to_process_words
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, _word)| {
                    let mut v = vec![];
                    for k in 1..min(phrase_max + 1, to_process_words.len() + 1 - i) {
                        let phrase: Vec<_> = to_process_words
                            .clone()
                            .into_iter()
                            .skip(i)
                            .take(k)
                            .collect();
                        if phrase.len() > 0 {
                            v.push((phrase.join(" "), (10 + k) * k))
                        }
                    }
                    v
                })
                .collect::<Vec<_>>();

            let listings = try_join_all(to_process_phrases.into_iter().flatten().map(
                |(phrase, score)| async move {
                    Ok((self.get_item_refs_for_phrase(&phrase).await?, score))
                },
            ))
            .await?;
            listings.into_iter().for_each(|(ref_list, index)| {
                ref_list.into_iter().for_each(|item_ref| {
                    if counter.contains_key(&item_ref) {
                        counter.insert(item_ref.clone(), counter.get(&item_ref).unwrap() + index);
                    } else {
                        counter.insert(item_ref.clone(), index);
                    }
                })
            });

            let max_pertinence = counter.values().max().unwrap_or(&1).clone();
            let ratings = counter
                .into_iter()
                .filter(|(_slug, weight)| weight.clone() == max_pertinence)
                .collect::<Vec<_>>();

            let mut results: Vec<_> = ratings.into_iter().map(|(slug, _rating)| slug).collect();
            results.sort();
            self.get_cache()
                .cache_search(search_query, results.clone())
                .await?;
            Ok(results)
        }
    }

    fn get_items_for_search(
        &self,
        search_query: &str,
        word_max: usize,
        phrase_max: usize,
        result_max: usize,
        page_num: usize,
    ) -> impl std::future::Future<Output = Result<(Vec<Item>, usize), DbError>> + Send {
        async move {
            let search = self.get_cache().get_cached_search(search_query).await?;
            let results = if search.is_empty() {
                self.get_item_refs_search_query(search_query, word_max, phrase_max)
                    .await?
            } else {
                search
            };
            let total_result = results.len();

            let db = self.get_db();
            let futures = results
                .into_iter()
                .skip((page_num - 1) * result_max)
                .take(result_max)
                .map(|item_ref| db.get_item_from_ref(item_ref))
                .collect::<Vec<_>>();
            Ok((try_join_all(futures).await?, total_result))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct TestItem {
        pub name: String,
        pub description: String,
    }

    #[derive(Clone)]
    struct TestCache {
        pub correct_input: Option<String>,
        pub retval: Vec<u64>,
    }

    #[derive(Clone)]
    struct TestDB {
        pub aliases: HashMap<String, Vec<u8>>,
        pub tags: HashMap<u8, Vec<u64>>,
        pub items: HashMap<u64, TestItem>,
    }

    #[derive(Debug)]
    struct TestError {}

    struct TestRepo {
        pub db: TestDB,
        pub cache: TestCache,
    }
    impl SearchCache<u64, TestError> for TestCache {
        fn cache_search(
            &self,
            search_query: &str,
            _results: Vec<u64>,
        ) -> impl std::future::Future<Output = Result<(), TestError>> + std::marker::Send {
            async {
                match &self.correct_input {
                    None => {}
                    Some(string) => {
                        assert!(search_query.to_string() == string.clone())
                    }
                }
                Ok(())
            }
        }

        fn get_cached_search(
            &self,
            search_query: &str,
        ) -> impl std::future::Future<Output = Result<Vec<u64>, TestError>> + std::marker::Send
        {
            async {
                match &self.correct_input {
                    None => {}
                    Some(string) => {
                        assert!(search_query.to_string() == string.clone())
                    }
                }
                Ok(self.retval.clone())
            }
        }
    }

    impl SearchDb<u8, u64, TestItem, TestError> for TestDB {
        fn get_item_from_ref(
            &self,
            item_ref: u64,
        ) -> impl std::future::Future<Output = Result<TestItem, TestError>> + std::marker::Send
        {
            println!("{}", item_ref);
            async move { Ok(self.items.get(&item_ref).cloned().unwrap()) }
        }
        fn get_item_refs_from_tag(
            &self,
            tag: u8,
        ) -> impl std::future::Future<Output = Result<Vec<u64>, TestError>> + std::marker::Send
        {
            println!("{}", tag);
            async move { Ok(self.tags.get(&tag).cloned().unwrap_or(vec![])) }
        }
        fn get_tags_from_phrase(
            &self,
            phrase: &str,
        ) -> impl std::future::Future<Output = Result<Vec<u8>, TestError>> + std::marker::Send
        {
            println!("{}", phrase);
            async move { Ok(self.aliases.get(phrase).cloned().unwrap_or(vec![])) }
        }
    }

    impl ItemRepo<u8, u64, TestItem, TestError> for TestRepo {
        fn get_cache(&self) -> impl SearchCache<u64, TestError> {
            self.cache.clone()
        }
        fn get_db(&self) -> impl SearchDb<u8, u64, TestItem, TestError> {
            self.db.clone()
        }
    }
    #[tokio::test]
    async fn test_search_cached_single() {
        let query = "butter";
        let item = TestItem {
            name: "Butter".to_string(),
            description: "Some butter".to_string(),
        };
        let mut searcher = TestRepo {
            db: TestDB {
                aliases: HashMap::new(),
                tags: HashMap::new(),
                items: HashMap::new(),
            },
            cache: TestCache {
                correct_input: Some(query.to_owned()),
                retval: vec![1001],
            },
        };
        searcher.db.items.insert(1001, item.clone());

        let research = searcher
            .get_items_for_search(query, 1, 1, 1, 1)
            .await
            .unwrap();

        assert_eq!(research, (vec![item], 1));
    }

    #[tokio::test]
    async fn test_search_single_cache_miss() {
        let query = "butter";
        let item = TestItem {
            name: "Butter".to_string(),
            description: "Some butter".to_string(),
        };
        let mut searcher = TestRepo {
            db: TestDB {
                aliases: HashMap::new(),
                tags: HashMap::new(),
                items: HashMap::new(),
            },
            cache: TestCache {
                correct_input: Some(query.to_owned()),
                retval: vec![],
            },
        };
        searcher.db.aliases.insert("butter".to_owned(), vec![1]);
        searcher.db.tags.insert(1, vec![1001]);
        searcher.db.items.insert(1001, item.clone());

        let research = searcher
            .get_items_for_search(query, 1, 1, 1, 1)
            .await
            .unwrap();

        assert_eq!(research, (vec![item], 1));
    }

    #[tokio::test]
    async fn test_two_words_exact_match() {
        let query = "butter flour";
        let item = TestItem {
            name: "Butter flour".to_string(),
            description: "Some butter flour".to_string(),
        };
        let mut searcher = TestRepo {
            db: TestDB {
                aliases: HashMap::new(),
                tags: HashMap::new(),
                items: HashMap::new(),
            },
            cache: TestCache {
                correct_input: Some(query.to_owned()),
                retval: vec![],
            },
        };
        searcher.db.aliases.insert("butter".to_owned(), vec![1]); // Bad, should not use
        searcher.db.aliases.insert("flour".to_owned(), vec![2]); // Bad, should not use
        searcher
            .db
            .aliases
            .insert("butter flour".to_owned(), vec![3]);

        searcher.db.tags.insert(3, vec![1001]);
        searcher.db.items.insert(1001, item.clone());

        let research = searcher
            .get_items_for_search(query, 2, 2, 1, 1)
            .await
            .unwrap();

        assert_eq!(research, (vec![item], 1));
    }

    #[tokio::test]
    async fn test_two_words_partial_match_join() {
        let query = "butter flour";
        let first_item = TestItem {
            name: "Butter".to_string(),
            description: "Some butter".to_string(),
        };
        let second_item = TestItem {
            name: "Flour".to_string(),
            description: "Some flour".to_string(),
        };
        let mut searcher = TestRepo {
            db: TestDB {
                aliases: HashMap::new(),
                tags: HashMap::new(),
                items: HashMap::new(),
            },
            cache: TestCache {
                correct_input: Some(query.to_owned()),
                retval: vec![],
            },
        };
        searcher.db.aliases.insert("butter".to_owned(), vec![1]);
        searcher.db.aliases.insert("flour".to_owned(), vec![2]);

        searcher.db.tags.insert(1, vec![1001]);
        searcher.db.tags.insert(2, vec![1002]);

        searcher.db.items.insert(1001, first_item.clone());
        searcher.db.items.insert(1002, second_item.clone());

        let research = searcher
            .get_items_for_search(query, 2, 2, 2, 1)
            .await
            .unwrap();

        assert_eq!(research, (vec![first_item, second_item], 2));
    }
}
