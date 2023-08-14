use meilisearch_sdk::settings::Settings;
use sea_orm_migration::{prelude::*, sea_orm::{EntityTrait, ColumnTrait, QueryFilter}};
use entity::prelude::*;
use meilisearch_entity::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        // see if env vars are set
        std::env::var("MEILI_URL").unwrap();
        std::env::var("MEILI_KEY").unwrap();

        let db = manager.get_connection();
        let mods = Mods::find().all(db).await.unwrap();


        let mut meili_mods = Vec::new();
        for m in mods {
            let vers = ModVersions::find()
                .filter(entity::mod_versions::Column::ModId.eq(m.id))
                .find_also_related(Versions)
                .all(db)
                .await
                .unwrap().into_iter().map(|(_, v)| v.unwrap()).collect::<Vec<_>>();

            let category = Categories::find()
                .filter(entity::categories::Column::Id.eq(m.category))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let author = Users::find()
                .filter(entity::users::Column::Id.eq(m.author))
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let stats = ModStats::find_by_id(m.stats)
                .one(db)
                .await
                .unwrap()
                .unwrap();

            let supported_versions = ModBeatSaberVersions::find()
                .filter(entity::mod_beat_saber_versions::Column::ModId.eq(m.id))
                .find_also_related(BeatSaberVersions)
                .all(db)
                .await
                .unwrap().into_iter().map(|(_, v)| v.unwrap()).collect::<Vec<_>>();

            let mm = MeiliMod {
                id: m.id,
                name: m.name,
                description: m.description.unwrap_or("".to_string()),
                category: category.name,
                versions: vers.into_iter().map(|v| MeiliVersion {
                    version: semver::Version::parse(&v.version).unwrap(),
                }).collect(),
                author: MeiliUser {
                    username: author.username.clone(),
                    display_name: author.display_name.unwrap_or(author.username),
                },
                stats: MeiliModStats {
                    downloads: stats.downloads as u64,
                },
                supported_versions: supported_versions.into_iter().map(|v| semver::Version::parse(&v.ver).unwrap()).collect(),
                created_at: m.created_at.and_utc(),
                updated_at: m.updated_at.and_utc(),
            };
            meili_mods.push(mm);
        }

        let client = meilisearch_sdk::client::Client::new(std::env::var("MEILI_URL").unwrap(), Some(std::env::var("MEILI_KEY").unwrap()));

        let settings = Settings::new().with_filterable_attributes(&["category"]).with_searchable_attributes(&["name", "description"]).with_sortable_attributes(&["stats.downloads", "created_at", "updated_at"]);
        client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).set_settings(&settings).await.unwrap();
        
        client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).add_documents(&meili_mods, None).await.unwrap();
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let client = meilisearch_sdk::client::Client::new(std::env::var("MEILI_URL").unwrap(), Some(std::env::var("MEILI_KEY").unwrap()));
        client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).delete().await.unwrap();
        Ok(())
    }
}
