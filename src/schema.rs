use entity::prelude::*;
use juniper::{
    graphql_value, EmptyMutation, EmptySubscription, FieldResult, GraphQLEnum, GraphQLObject,
    RootNode,
};

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::auth::Authorization;
use crate::mods::Mod;
use crate::users::User;
use crate::{mods, users, Database};

pub struct QueryRoot;

#[juniper::graphql_object(context = Database)]
impl QueryRoot {
    #[cached::cached(time=120)]
    async fn user_by_id(db: &Database, id: Uuid, auth: Option<String>) -> FieldResult<User> {
        let db = db.pool.clone();
        let r = users::find_by_id(&db, id, Authorization::parse(auth)).await;
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn users(
        db: &Database,
        limit: Option<i32>,
        offset: Option<i32>,
        auth: Option<String>,
    ) -> FieldResult<Vec<User>> {
        if limit > Some(10) {
            return Err(juniper::FieldError::new(
                "Limit must be less than 10",
                graphql_value!({ "limit": "Limit must be less than 10" }),
            ));
        }
        let db = db.pool.clone();
        let r = users::find_all(
            &db,
            limit.unwrap_or(10),
            offset.unwrap_or(0),
            Authorization::parse(auth),
        )
        .await;
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn mods(
        db: &Database,
        limit: Option<i32>,
        offset: Option<i32>,
        version: Option<String>,
    ) -> FieldResult<Vec<Mod>> {
        if limit > Some(10) {
            return Err(juniper::FieldError::new(
                "Limit must be less than 10",
                graphql_value!({ "limit": "Limit must be less than 10" }),
            ));
        }
        let db = db.pool.clone();

        let r = mods::find_all(&db, limit.unwrap_or(10), offset.unwrap_or(0), version).await;
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn mod_by_id(db: &Database, id: Uuid) -> FieldResult<Mod> {
        let db = db.pool.clone();

        let r = mods::find_by_id(&db, id).await;
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn mod_by_author(db: &Database, id: Uuid) -> FieldResult<Vec<Mod>> {
        let db = db.pool.clone();

        let r = mods::find_by_author(&db, id).await;
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn categories(db: &Database) -> FieldResult<Vec<GCategory>> {
        let db = db.pool.clone();

        let r = Ok(Categories::find()
            .all(&db)
            .await
            .unwrap()
            .iter()
            .map(|c| GCategory {
                name: c.name.clone(),
                description: c.description.clone(),
            })
            .collect::<Vec<_>>());
        db.close().await.unwrap();
        r
    }

    #[cached::cached(time=120)]
    async fn beat_saber_versions(db: &Database) -> FieldResult<Vec<String>> {
        let db = db.pool.clone();

        let r = Ok(BeatSaberVersions::find()
            .all(&db)
            .await
            .unwrap()
            .iter()
            .map(|v| v.ver.clone())
            .collect::<Vec<_>>());
        db.close().await.unwrap();
        r
    }
}

#[derive(GraphQLObject)]
pub struct GCategory {
    name: String,
    description: String,
}

pub type Schema =
    RootNode<'static, QueryRoot, EmptyMutation<Database>, EmptySubscription<Database>>;

pub fn create_schema() -> Schema {
    // let sub = EmptySubscription::<Database>::new();
    Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new())
}
