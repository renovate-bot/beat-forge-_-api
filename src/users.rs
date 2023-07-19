use chrono::{DateTime, Utc};
use entity::prelude::*;
use juniper::{graphql_value, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, FieldError};
use sea_orm::{EntityTrait, QuerySelect};
use uuid::Uuid;

use crate::{Database, mods::{Mod, self}};

#[derive(GraphQLObject)]
pub struct User {
    pub id: Uuid,
    pub github_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: String,
    pub bio: Option<String>,
    pub mods: Vec<Mod>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    async fn from_db_user(db: &Database, u: entity::users::Model) -> Result<Self, FieldError> {
        Ok(User {
            id: Uuid::from_bytes(*u.id.as_bytes()),
            github_id: u.github_id.to_string(),
            username: u.username,
            display_name: u.display_name,
            email: u.email,
            bio: u.bio,
            mods: mods::find_by_author(db, Uuid::from_bytes(*u.id.as_bytes())).await.unwrap(),
            avatar: u.avatar,
            banner: u.banner,
            api_key: u.api_key,
            created_at: u.created_at.and_utc(),
            updated_at: u.updated_at.and_utc(),
        })
    }
}

pub async fn find_all(db: &Database, limit: i32, offset: i32) -> FieldResult<Vec<User>> {
    let limit = limit as u64;
    let offset = offset as u64;

    let users = Users::find()
    .limit(Some(limit))
        .offset(Some(offset))
        .all(&db.pool)
        .await?;

    let users = users
        .into_iter()
        .map(|user| async move { User::from_db_user(&db, user).await.unwrap() })
        .collect::<Vec<_>>();

    Ok(futures::future::join_all(users).await)
}

pub async fn find_by_id(db: &Database, _id: Uuid) -> FieldResult<User> {
    let id = sea_orm::prelude::Uuid::from_bytes(*_id.as_bytes());

    let user = Users::find_by_id(id).one(&db.pool).await?;

    if user.is_none() {
        return Err(juniper::FieldError::new(
            "User not found",
            graphql_value!({ "notFound": "User not found" }),
        ));
    }

    let user = user.unwrap();

    Ok(User::from_db_user(&db, user).await?)
}
