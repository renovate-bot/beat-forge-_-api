use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: sea_orm::prelude::Uuid,
    pub name: String,
    pub description: String,
    pub versions: Vec<super::version::Entity>,
    pub category: String,
    pub author: super::user::Entity,
    pub stats: super::mod_stats::Entity,
    pub supported_versions: Vec<Version>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}