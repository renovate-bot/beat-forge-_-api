//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "versions_conflicts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub version_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub dependent: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::versions::Entity",
        from = "Column::Dependent",
        to = "super::versions::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Versions2,
    #[sea_orm(
        belongs_to = "super::versions::Entity",
        from = "Column::VersionId",
        to = "super::versions::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Versions1,
}

impl ActiveModelBehavior for ActiveModel {}
