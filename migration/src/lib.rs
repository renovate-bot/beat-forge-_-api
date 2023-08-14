pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230727_015400_load_default_data;
mod m20230727_063415_create_mod_beat_saber_versions;
mod m20230806_034429_melilisearch;
mod m20230813_235044_reclean_download_urls;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230727_015400_load_default_data::Migration),
            Box::new(m20230727_063415_create_mod_beat_saber_versions::Migration),
            Box::new(m20230806_034429_melilisearch::Migration),
            Box::new(m20230813_235044_reclean_download_urls::Migration),
        ]
    }
}
