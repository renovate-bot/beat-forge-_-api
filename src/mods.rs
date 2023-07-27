use actix_web::{web, Responder, HttpRequest, post, HttpResponse};
use chrono::{DateTime, Utc};

use juniper::{FieldError, FieldResult, GraphQLObject};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use entity::prelude::*;

use crate::{Database, versions::{GVersion, self}};

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct Mod {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub cover: Option<String>,
    pub author: ModAuthor,
    pub category: ModCategory,
    pub stats: GModStats,
    pub versions: Vec<GVersion>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct ModAuthor {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,

    pub bio: Option<String>,
    pub permissions: i32,
    pub avatar: Option<String>,
    pub banner: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Mod {
    async fn from_db_mod(db: &Database, m: entity::mods::Model) -> Result<Self, FieldError> {
        let category = Categories::find_by_id(m.category)
            .one(&db.pool)
            .await?
            .unwrap();
        let stats = ModStats::find_by_id(m.stats).one(&db.pool).await?.unwrap();
        let author = Users::find_by_id(m.author).one(&db.pool).await?.unwrap();
        Ok(Mod {
            id: Uuid::from_bytes(*m.id.as_bytes()),
            slug: m.slug,
            name: m.name,
            description: m.description,
            icon: m.icon,
            cover: m.cover,
            author: ModAuthor { 
                id: Uuid::from_bytes(*author.id.as_bytes()),
                username: author.username,
                display_name: author.display_name,
                bio: author.bio,
                permissions: author.permissions,
                avatar: author.avatar,
                banner: author.banner,
                created_at: author.created_at.and_utc(),
                updated_at: author.updated_at.and_utc(),
            },
            category: ModCategory {
                name: category.name,
                desc: category.description,
            },
            stats: GModStats {
                downloads: stats.downloads,
            },
            versions: versions::find_by_mod_id(db, Uuid::from_bytes(*m.id.as_bytes())).await?,
            updated_at: m.updated_at.and_utc(),
            created_at: m.created_at.and_utc(),
        })
    }
}

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct GModStats {
    pub downloads: Option<i32>,
    // pub rating: f32,
    // pub rating_count: i32,
}

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct ModCategory {
    pub name: String,
    pub desc: Option<String>,
}

pub async fn find_all(db: &Database, limit: i32, offset: i32, version: Option<String>) -> FieldResult<Vec<Mod>> {
    let limit = limit as u64;
    let offset = offset as u64;

    let mods = Mods::find()
        .limit(limit)
        .offset(offset)
        .all(&db.pool)
        .await?;

    let mods = mods
        .into_iter()
        .map(|m| async move { Mod::from_db_mod(db, m).await.unwrap() })
        .collect::<Vec<_>>();

    Ok(futures::future::join_all(mods).await)
}

pub async fn find_by_id(db: &Database, id: Uuid) -> FieldResult<Mod> {
    let id = sea_orm::prelude::Uuid::from_bytes(*id.as_bytes());
    let m = Mods::find_by_id(id).one(&db.pool).await?.unwrap();

    Mod::from_db_mod(db, m).await
}

pub async fn find_by_author(db: &Database, author: Uuid) -> FieldResult<Vec<Mod>> {
    let author = sea_orm::prelude::Uuid::from_bytes(*author.as_bytes());

    let mods = Mods::find()
        .filter(entity::mods::Column::Author.eq(author))
        .all(&db.pool)
        .await?;

    let mods = mods
        .into_iter()
        .map(|m| async move { Mod::from_db_mod(db, m).await.unwrap() })
        .collect::<Vec<_>>();

    Ok(futures::future::join_all(mods).await)
}

#[post("/mods")]
pub async fn create_mod(_db: web::Data<Database>, _payload: web::Payload, _req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}