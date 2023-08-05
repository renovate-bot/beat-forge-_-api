use sea_orm_migration::{prelude::*, sea_orm::{Set, TransactionTrait, ActiveModelTrait, EntityTrait, ColumnTrait, QueryFilter}};
use entity::{prelude::*, beat_saber_versions, categories};

const BEAT_SABER_VERSIONS: [&'static str; 73] = [
	"0.10.1",
	"0.10.2",
	"0.10.2-p1",
	"0.11.0-b1",
	"0.11.0",
	"0.11.1",
	"0.11.2",
	"0.12.0",
	"0.12.0-p1",
	"0.12.1",
	"0.12.2",
	"0.13.0",
	"0.13.0-p1",
	"0.13.1",
	"0.13.2",
	"1.0.0",
	"1.0.1",
	"1.1.0",
	"1.1.0-p1",
	"1.2.0",
	"1.3.0",
	"1.4.0",
	"1.4.2",
	"1.5.0",
	"1.6.0",
	"1.6.1",
	"1.6.2",
	"1.7.0",
	"1.8.0",
	"1.9.0",
	"1.9.1",
	"1.10.0",
	"1.11.0",
	"1.11.1",
	"1.12.1",
	"1.12.2",
	"1.13.0",
	"1.13.2",
	"1.13.4",
	"1.13.5",
	"1.14.0",
	"1.15.0",
	"1.16.0",
	"1.16.1",
	"1.16.2",
	"1.16.3",
	"1.16.4",
	"1.17.0",
	"1.17.1",
	"1.18.0",
	"1.18.1",
	"1.18.2",
	"1.18.3",
	"1.19.0",
	"1.19.1",
	"1.20.0",
	"1.21.0",
	"1.22.0",
	"1.22.1",
	"1.23.0",
	"1.24.0",
	"1.24.1",
	"1.25.0",
	"1.25.1",
	"1.26.0",
	"1.27.0",
	"1.28.0",
	"1.29.0",
	"1.29.1",
	"1.29.4",
	"1.30.0",
	"1.30.2",
	"1.31.0",
];

const CATEGORY_DES: [(&'static str, &'static str); 14] = [
    ("core", "Mods that only depend on other core mods."),
    ("libraries", "Mods that are used by other mods."),
    ("cosmetic", "Mods that affect the appearance of the game."),
    ("gameplay", "Mods that affect gameplay."),
    ("leaderboards", "Mods that affect leaderboards."),
    ("lighting", "Mods that affect lighting."),
    ("multiplayer", "Mods that change online play."),
    ("accessibility", "Mods that affect accessibility."),
    ("practice", "Mods that are used for practice."),
    ("streaming", "Mods that affect live streams."),
    ("text", "Mods that change how text is displayed."),
    ("tweaks", "Mods that tweak the gameplay experience."),
    ("ui", "Mods that affect the ui."),
    ("other", "Mods that do not fit into other categories."),
];

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // start transaction
        let db = manager.get_connection();
        let trans = db.begin().await?;
        
        // insert beat saber versions
        let vers = BEAT_SABER_VERSIONS
        .iter()
        .map(|v| beat_saber_versions::ActiveModel {
            ver: Set(v.to_string()),
            ..Default::default()
        }.insert(&trans))
        .collect::<Vec<_>>();

        let cata = CATEGORY_DES
        .iter()
        .map(|(n, d)| categories::ActiveModel {
            name: Set(n.to_string()),
            description: Set(d.to_string()),
            ..Default::default()
        }.insert(&trans))
        .collect::<Vec<_>>();

        let vres = futures::future::join_all(vers).await;
        let cres = futures::future::join_all(cata).await;
        //propagate errors
        vres.iter().map(|r| r.as_ref().map(|_|())).collect::<Result<Vec<_>, _>>().unwrap();
        cres.iter().map(|r| r.as_ref().map(|_|())).collect::<Result<Vec<_>, _>>().unwrap();
        
        trans.commit().await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // remove beat saber versions
        let db = manager.get_connection();
        let trans = db.begin().await?;
        
        for v in BEAT_SABER_VERSIONS.iter() {
            BeatSaberVersions::delete_by_id(BeatSaberVersions::find()
                .filter(beat_saber_versions::Column::Ver.eq(*v))
                .one(&trans)
                .await?.unwrap().id);
        }

        for c in CATEGORY_DES.iter() {
            Categories::delete_by_id(Categories::find()
                .filter(categories::Column::Name.eq(c.0))
                .one(&trans)
                .await?.unwrap().id);
        }

        trans.commit().await?;
        Ok(())
    }
}