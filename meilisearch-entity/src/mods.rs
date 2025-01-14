use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: sea_orm::prelude::Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub versions: Vec<super::version::Entity>,
    pub category: String,
    pub author: super::user::Entity,
    pub stats: super::mod_stats::Entity,
    pub supported_versions: Vec<Version>,
    pub created_at: i64,
    pub updated_at: i64
}