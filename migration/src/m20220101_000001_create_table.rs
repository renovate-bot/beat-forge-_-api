use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::GithubId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::DisplayName).string().null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Bio).string().null())
                    .col(ColumnDef::new(Users::Avatar).string().null())
                    .col(ColumnDef::new(Users::Banner).string().null())
                    .col(
                        ColumnDef::new(Users::Permissions)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Users::ApiKey)
                            .uuid()
                            .not_null()
                            .unique_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Categories::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Categories::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Categories::Description)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ModStats::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ModStats::Downloads)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VersionStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VersionStats::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(VersionStats::Downloads)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BeatSaberVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BeatSaberVersions::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BeatSaberVersions::Ver)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Mods::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Mods::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Mods::Slug).string().not_null().unique_key())
                    .col(ColumnDef::new(Mods::Name).string().not_null())
                    .col(ColumnDef::new(Mods::Description).string().null())
                    .col(ColumnDef::new(Mods::Icon).string().null())
                    .col(ColumnDef::new(Mods::Cover).string().null())
                    .col(ColumnDef::new(Mods::Author).uuid().not_null())
                    .col(ColumnDef::new(Mods::Category).uuid().not_null())
                    .col(ColumnDef::new(Mods::Stats).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Mods::Website).string().null())
                    .col(
                        ColumnDef::new(Mods::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Mods::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mods_category")
                            .from(Mods::Table, Mods::Category)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mods_stats")
                            .from(Mods::Table, Mods::Stats)
                            .to(ModStats::Table, ModStats::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mods_author")
                            .from(Mods::Table, Mods::Author)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Versions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Versions::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Versions::ModId).uuid().not_null())
                    .col(ColumnDef::new(Versions::Version).string().not_null())
                    .col(
                        ColumnDef::new(Versions::Approved)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Versions::Stats)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Versions::ArtifactHash).string().not_null())
                    .col(ColumnDef::new(Versions::DownloadUrl).string().not_null())
                    .col(
                        ColumnDef::new(Versions::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_versions_mod")
                            .from(Versions::Table, Versions::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_versions_stats")
                            .from(Versions::Table, Versions::Stats)
                            .to(VersionStats::Table, VersionStats::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VersionDependents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VersionDependents::VersionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VersionDependents::Dependent)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_dependents_version")
                            .from(VersionDependents::Table, VersionDependents::VersionId)
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_dependents_dependent")
                            .from(VersionDependents::Table, VersionDependents::Dependent)
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VersionConflicts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VersionConflicts::VersionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VersionConflicts::Dependent)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_conflicts_version")
                            .from(VersionConflicts::Table, VersionConflicts::VersionId)
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_conflicts_dependent")
                            .from(VersionConflicts::Table, VersionConflicts::Dependent)
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModVersions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ModVersions::ModId).uuid().not_null())
                    .col(ColumnDef::new(ModVersions::VersionId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mod_versions_mod")
                            .from(ModVersions::Table, ModVersions::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mod_versions_version")
                            .from(ModVersions::Table, ModVersions::VersionId)
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserMods::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserMods::UserId).uuid().not_null())
                    .col(ColumnDef::new(UserMods::ModId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_mods_user")
                            .from(UserMods::Table, UserMods::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_mods_mod")
                            .from(UserMods::Table, UserMods::ModId)
                            .to(Mods::Table, Mods::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VersionBeatSaberVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VersionBeatSaberVersions::VersionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VersionBeatSaberVersions::BeatSaberVersionId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_beat_saber_versions_version")
                            .from(
                                VersionBeatSaberVersions::Table,
                                VersionBeatSaberVersions::VersionId,
                            )
                            .to(Versions::Table, Versions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_version_beat_saber_versions_beat_saber_version")
                            .from(
                                VersionBeatSaberVersions::Table,
                                VersionBeatSaberVersions::BeatSaberVersionId,
                            )
                            .to(BeatSaberVersions::Table, BeatSaberVersions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ModStats::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(VersionStats::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BeatSaberVersions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Mods::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Versions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(VersionDependents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(VersionConflicts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ModVersions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserMods::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(VersionBeatSaberVersions::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Users {
    Table,
    Id,
    GithubId,
    Username,
    DisplayName,
    Email,
    Bio,
    Avatar,
    Banner,
    Permissions,
    ApiKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
    Name,
    Description,
}

#[derive(Iden)]
enum ModStats {
    Table,
    Id,
    Downloads,
}

#[derive(Iden)]
enum VersionStats {
    Table,
    Id,
    Downloads,
}

#[derive(Iden)]
enum BeatSaberVersions {
    Table,
    Id,
    Ver,
}

#[derive(Iden)]
enum Mods {
    Table,
    Id,
    Slug,
    Name,
    Description,
    Icon,
    Cover,
    Author,
    Website,
    Category,
    Stats,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Versions {
    Table,
    Id,
    ModId,
    Version,
    Approved,
    ArtifactHash,
    DownloadUrl,
    Stats,
    CreatedAt,
}

#[derive(Iden)]
enum VersionDependents {
    Table,
    VersionId,
    Dependent,
}

#[derive(Iden)]
enum VersionConflicts {
    Table,
    VersionId,
    Dependent,
}

#[derive(Iden)]
enum ModVersions {
    Table,
    ModId,
    VersionId,
}

#[derive(Iden)]
enum UserMods {
    Table,
    UserId,
    ModId,
}

#[derive(Iden)]
enum VersionBeatSaberVersions {
    Table,
    VersionId,
    BeatSaberVersionId,
}
