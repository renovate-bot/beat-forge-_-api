use chrono::{DateTime, Utc};
use entity::prelude::*;
use juniper::{
    FieldError, FieldResult, GraphQLObject,
};
use serde::{Serialize, Deserialize};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    Database,
};

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct GVersion {
    pub id: Uuid,
    pub mod_id: Uuid,
    pub version: String,
    pub approved: bool,
    pub supported_game_versions: Vec<String>,
    pub stats: GVersionStats,
    pub created_at: DateTime<Utc>,
}

#[derive(GraphQLObject, Debug, Deserialize, Serialize)]
pub struct GVersionStats {
    pub downloads: Option<i32>,
    // pub rating: f32,
    // pub rating_count: i32,
}

impl GVersion {
    pub async fn from_db_version(
        db: &Database,
        v: entity::versions::Model,
    ) -> Result<Self, FieldError> {
        let versions = VersionBeatSaberVersions::find()
            .filter(entity::version_beat_saber_versions::Column::VersionId.eq(v.id))
            .find_also_related(BeatSaberVersions)
            .all(&db.pool)
            .await
            .unwrap()
            .iter()
            .map(|v| v.1.clone().unwrap().ver)
            .collect::<Vec<_>>();

        let stats = VersionStats::find_by_id(v.stats)
            .one(&db.pool)
            .await
            .unwrap()
            .unwrap();

        Ok(GVersion {
            id: Uuid::from_bytes(*v.id.as_bytes()),
            mod_id: Uuid::from_bytes(*v.mod_id.as_bytes()),
            version: v.version,
            supported_game_versions: versions,
            created_at: v.created_at.and_utc(),
            approved: v.approved.unwrap_or(false),
            stats: GVersionStats {
                downloads: stats.downloads,
            },
        })
    }
}

pub async fn find_by_mod_id(db: &Database, id: Uuid) -> FieldResult<Vec<GVersion>> {
    let id = sea_orm::prelude::Uuid::from_bytes(*id.as_bytes());

    let versions = Versions::find()
        .filter(entity::versions::Column::ModId.eq(id))
        .all(&db.pool)
        .await
        .unwrap();

    let versions = futures::future::join_all(
        versions
            .iter()
            .map(|v| async move { GVersion::from_db_version(db, v.clone()).await.unwrap() }),
    )
    .await;

    Ok(versions)
}
