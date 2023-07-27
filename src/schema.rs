use juniper::{graphql_value, EmptyMutation, EmptySubscription, FieldResult, RootNode};

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use uuid::Uuid;

use crate::auth::Authorization;
use crate::mods::Mod;
use crate::users::User;
use crate::{mods, users, Database};

pub struct QueryRoot;

#[juniper::graphql_object(context = Database)]
impl QueryRoot {
    async fn user_by_id(db: &Database, id: Uuid, auth: Option<String>) -> FieldResult<User> {
        users::find_by_id(&db, id, Authorization::parse(auth)).await
    }
    
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
        users::find_all(&db, limit.unwrap_or(10), offset.unwrap_or(0), Authorization::parse(auth)).await
    }
    async fn mods(db: &Database, limit: Option<i32>, offset: Option<i32>) -> FieldResult<Vec<Mod>> {
        if limit > Some(10) {
            return Err(juniper::FieldError::new(
                "Limit must be less than 10",
                graphql_value!({ "limit": "Limit must be less than 10" }),
            ));
        }
        mods::find_all(&db, limit.unwrap_or(10), offset.unwrap_or(0)).await
    }
    async fn mod_by_id(db: &Database, id: Uuid) -> FieldResult<Mod> {
        mods::find_by_id(&db, id).await
    }
    async fn mod_by_author(db: &Database, id: Uuid) -> FieldResult<Vec<Mod>> {
        mods::find_by_author(&db, id).await
    }
}

pub type Schema =
    RootNode<'static, QueryRoot, EmptyMutation<Database>, EmptySubscription<Database>>;

pub fn create_schema() -> Schema {
    // let sub = EmptySubscription::<Database>::new();
    Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new())
}
