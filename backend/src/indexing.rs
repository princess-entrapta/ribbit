pub trait InsertHandle<Tag, ItemRef, Item, DbError>
where
    ItemRef: Clone,
    Tag: Clone,
{
    fn insert_tags(
        &self,
        tags: Vec<Tag>,
        item_ref: ItemRef,
    ) -> impl std::future::Future<Output = Result<(), DbError>> + std::marker::Send;
    fn insert_item(
        &self,
        item: Item,
    ) -> impl std::future::Future<Output = Result<(), DbError>> + std::marker::Send;
    fn insert_alias(
        &self,
        phrase: String,
        tags: Vec<Tag>,
    ) -> impl std::future::Future<Output = Result<(), DbError>> + std::marker::Send;
}

pub async fn insert_and_index_item<Tag, ItemRef, Item, DbError>(
    handler: &impl InsertHandle<Tag, ItemRef, Item, DbError>,
    item_ref: ItemRef,
    item: Item,
    tags: Vec<Tag>,
) -> Result<(), DbError>
where
    Item: Clone,
    ItemRef: Clone,
    Tag: Clone,
{
    let _ = tokio::try_join!(
        { handler.insert_item(item.clone()) },
        handler.insert_tags(tags, item_ref)
    )?;
    Ok(())
}
