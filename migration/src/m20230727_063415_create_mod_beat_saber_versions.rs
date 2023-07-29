use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ModBeatSaberVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ModBeatSaberVersions::ModId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModBeatSaberVersions::BeatSaberVersionId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mod_beat_saber_versions_mods_mod_id")
                            .from(
                                ModBeatSaberVersions::Table,
                                ModBeatSaberVersions::ModId,
                            )
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mod_beat_saber_versions_beat_saber_versions_beat_saber_version_id")
                            .from(
                                ModBeatSaberVersions::Table,
                                ModBeatSaberVersions::BeatSaberVersionId,
                            )
                            .to(BeatSaberVersions::Table, BeatSaberVersions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ModBeatSaberVersions::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ModBeatSaberVersions {
    Table,
    ModId,
    BeatSaberVersionId,
}

#[derive(Iden)]
enum Mods {
    Table,
    Id,
}

#[derive(Iden)]
enum BeatSaberVersions {
    Table,
    Id,
}