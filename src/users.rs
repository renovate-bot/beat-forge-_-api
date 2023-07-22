use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use entity::prelude::*;
use juniper::{
    graphql_value, FieldError, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    auth::{self, JWTAuth},
    mods::{self, Mod},
    Database, Key,
};

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub github_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: String,
    pub bio: Option<String>,
    pub mods: Vec<Mod>,
    pub permissions: i32,
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
            mods: mods::find_by_author(db, Uuid::from_bytes(*u.id.as_bytes()))
                .await
                .unwrap(),
            avatar: u.avatar,
            banner: u.banner,
            permissions: u.permissions,
            api_key: u.api_key.to_string(),
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

#[derive(Deserialize, Serialize)]
pub struct UserAuthReq {
    pub code: String,
}

#[post("/auth/github")]
pub async fn user_auth(
    req: HttpRequest,
    data: web::Data<Database>,
    key: web::Data<Key>,
    info: web::Query<UserAuthReq>,
) -> impl Responder {
    let code = &info.code;

    let gat = minreq::post("https://github.com/login/oauth/access_token")
        .with_header("User-Agent", "forge-registry")
        .with_json(&json!({
            "client_id": std::env::var("GITHUB_CLIENT_ID").unwrap(),
            "client_secret": std::env::var("GITHUB_CLIENT_SECRET").unwrap(),
            "code": code,
        }))
        .unwrap()
        .send()
        .unwrap(); //.json::<Value>().unwrap()["access_token"].to_string();
    let gat = gat.as_str().unwrap().split("&").collect::<Vec<_>>()[0]
        .split("=")
        .collect::<Vec<_>>()[1]
        .to_string();

    let github_user = minreq::get("https://api.github.com/user")
        .with_header("User-Agent", "forge-registry")
        .with_header("Authorization", format!("Bearer {}", gat))
        .send()
        .unwrap();
    dbg!(github_user.as_str().unwrap());
    let github_user = serde_json::from_str::<GithubUser>(github_user.as_str().unwrap()).unwrap();

    // let user = match Users::find().filter(entity::users::Column::GithubId.eq(github_user.id as i32)).one(&data.pool).await.unwrap() {
    //     Some(user) => user,
    //     None => {
    //         let usr = entity::users::ActiveModel {
    //             github_id: Set(github_user.id as i32),
    //             username: Set(github_user.login),
    //             email: Set(github_user.email),
    //             bio: Set(Some(github_user.bio)),
    //             avatar: Set(Some(github_user.avatar_url)),
    //             permissions: Set(7),
    //             ..Default::default()
    //         };

    //         let user = Users::insert(usr).exec(&data.pool).await.unwrap();
    //     }
    // };

    let mby_user = Users::find()
        .filter(entity::users::Column::GithubId.eq(github_user.id as i32))
        .one(&data.pool)
        .await
        .unwrap();

    if mby_user.is_none() {
        let usr = entity::users::ActiveModel {
            github_id: Set(github_user.id as i32),
            username: Set(github_user.login),
            email: Set(github_user.email),
            bio: Set(Some(github_user.bio)),
            avatar: Set(Some(github_user.avatar_url)),
            permissions: Set(7),
            ..Default::default()
        };

        Users::insert(usr).exec(&data.pool).await.unwrap();
    }

    let user = Users::find()
        .filter(entity::users::Column::GithubId.eq(github_user.id as i32))
        .one(&data.pool)
        .await
        .unwrap()
        .unwrap();

    let jwt = JWTAuth::new(user).encode(&*key.0);

    let mut res = HttpResponse::Ok();
    res.cookie(
        actix_web::cookie::Cookie::build("jwt", jwt)
            .path("/")
            .secure(true)
            .http_only(true)
            .finish(),
    );

    res.finish()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubUser {
    pub avatar_url: String,
    pub bio: String,
    pub blog: String,
    pub email: String,
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub twitter_username: String,
    pub updated_at: DateTime<Utc>,
    pub url: String,
}
