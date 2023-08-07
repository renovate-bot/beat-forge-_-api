use std::vec;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};

use forge_lib::structs::forgemod::ForgeMod;
use futures::StreamExt;
use juniper::{graphql_value, FieldError, FieldResult, GraphQLObject};
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::prelude::*;
use meilisearch_entity::prelude::*;

use crate::{
    auth::{validate_permissions, Authorization, Permission},
    versions::{self, GVersion},
    Database,
};

#[derive(GraphQLObject, Debug, Deserialize, Serialize, Clone)]
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

#[derive(GraphQLObject, Debug, Deserialize, Serialize, Clone)]
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
    async fn from_db_mod(
        db: &DatabaseConnection,
        m: entity::mods::Model,
    ) -> Result<Self, FieldError> {
        let category = Categories::find_by_id(m.category).one(db).await?.unwrap();
        let stats = ModStats::find_by_id(m.stats).one(db).await?.unwrap();
        let author = Users::find_by_id(m.author).one(db).await?.unwrap();
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

#[derive(GraphQLObject, Debug, Deserialize, Serialize, Clone)]
pub struct GModStats {
    pub downloads: i32,
    // pub rating: f32,
    // pub rating_count: i32,
}

#[derive(GraphQLObject, Debug, Deserialize, Serialize, Clone)]
pub struct ModCategory {
    pub name: String,
    pub desc: String,
}

pub async fn find_all(
    db: &DatabaseConnection,
    limit: i32,
    offset: i32,
    version: Option<String>,
) -> FieldResult<Vec<Mod>> {
    let limit = limit as u64;
    let offset = offset as u64;

    if let Some(version) = version {
        let verid = BeatSaberVersions::find()
            .filter(entity::beat_saber_versions::Column::Ver.eq(version))
            .one(db)
            .await?
            .unwrap()
            .id;

        let mods = ModBeatSaberVersions::find()
            .filter(entity::mod_beat_saber_versions::Column::BeatSaberVersionId.eq(verid))
            .find_also_related(Mods)
            .all(db)
            .await?
            .iter()
            .map(|v| v.1.clone().unwrap())
            .collect::<Vec<_>>();

        let mut r = vec![];
        for m in mods {
            r.push(Mod::from_db_mod(db, m).await.unwrap());
        }
        Ok(r)
    } else {
        let mods = Mods::find().limit(limit).offset(offset).all(db).await?;

        let mut r = vec![];
        for m in mods {
            r.push(Mod::from_db_mod(db, m).await.unwrap());
        }
        Ok(r)
    }
}

pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> FieldResult<Mod> {
    let id = sea_orm::prelude::Uuid::from_bytes(*id.as_bytes());
    let m = Mods::find_by_id(id).one(db).await?;

    if let Some(m) = m {
        Mod::from_db_mod(db, m).await
    } else {
        Err(FieldError::new(
            "Mod not found",
            graphql_value!({ "internal_error": "Mod not found" }),
        ))
    }
}

pub async fn find_by_slug(db: &DatabaseConnection, slug: String) -> FieldResult<Mod> {
    let m = Mods::find()
        .filter(entity::mods::Column::Slug.eq(slug))
        .one(db)
        .await?;

    if let Some(m) = m {
        Mod::from_db_mod(db, m).await
    } else {
        Err(FieldError::new(
            "Mod not found",
            graphql_value!({ "internal_error": "Mod not found" }),
        ))
    }
}

pub async fn find_by_author(db: &DatabaseConnection, author: Uuid) -> FieldResult<Vec<Mod>> {
    let author = sea_orm::prelude::Uuid::from_bytes(*author.as_bytes());

    let mods = Mods::find()
        .filter(entity::mods::Column::Author.eq(author))
        .all(db)
        .await?;

    let mut r = vec![];
    for m in mods {
        r.push(Mod::from_db_mod(db, m).await.unwrap());
    }
    Ok(r)
}

#[post("/mods")]
pub async fn create_mod(
    db: web::Data<Database>,
    mut payload: web::Payload,
    req: HttpRequest,
) -> impl Responder {
    let auth = req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap();
    let auser;
    if auth.starts_with("Bearer") {
        let auth = Authorization::parse(Some(auth.split(" ").collect::<Vec<_>>()[1].to_string()));
        let user = auth.get_user(&db.pool).await.unwrap();
        if !validate_permissions(&user, Permission::CREATE_MOD).await {
            return HttpResponse::Unauthorized().body("Unauthorized");
        }
        auser = user;
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let mut buf = Vec::new();

    while let Some(item) = payload.next().await {
        let item = item.unwrap();
        buf.extend_from_slice(&item);
    }

    let forgemod = ForgeMod::try_from(&buf[..]).unwrap();

    let db_cata = Categories::find()
        .filter(entity::categories::Column::Name.eq(forgemod.manifest.category.clone().to_string()))
        .one(&db.pool)
        .await
        .unwrap();

    // if cata does not exist, default to other
    let db_cata = if let Some(db_cata) = db_cata {
        db_cata
    } else {
        Categories::find()
            .filter(entity::categories::Column::Name.eq("other"))
            .one(&db.pool)
            .await
            .unwrap()
            .unwrap()
    };

    let v_req = forgemod.manifest.game_version.clone();
    let vers = BeatSaberVersions::find()
        .all(&db.pool)
        .await
        .unwrap()
        .into_iter()
        .filter(|v| v_req.matches(&Version::parse(&v.ver).unwrap()))
        .collect::<Vec<_>>();

    if vers.len() == 0 {
        return HttpResponse::BadRequest().body("Invalid game version");
    }

    // see if mod exists; if it does add a new version; if it doesn't create a new mod
    let mby_mod = Mods::find()
        .filter(entity::mods::Column::Slug.eq(forgemod.manifest._id.clone()))
        .one(&db.pool)
        .await
        .unwrap();

    let v_id;

    let trans = db.pool.begin().await.unwrap();

    if let Some(db_mod) = mby_mod {
        let db_mod = db_mod.id;
        for v in &vers {
            let vm = entity::mod_beat_saber_versions::ActiveModel {
                mod_id: Set(db_mod),
                beat_saber_version_id: Set(v.id),
            };

            //see if vm exists
            if ModBeatSaberVersions::find()
                .filter(entity::mod_beat_saber_versions::Column::ModId.eq(db_mod))
                .filter(entity::mod_beat_saber_versions::Column::BeatSaberVersionId.eq(v.id))
                .one(&trans)
                .await
                .unwrap()
                .is_none()
            {
                vm.insert(&trans).await.unwrap();
            }
        }

        let version_stats = entity::version_stats::ActiveModel {
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        let version = entity::versions::ActiveModel {
            mod_id: Set(db_mod),
            version: Set(forgemod.manifest.version.clone().to_string()),
            stats: Set(version_stats),
            //todo: artifact hash
            artifact_hash: Set("".to_string()),
            //todo: download url
            download_url: Set(format!(
                "{}/cdn/{}@{}",
                std::env::var("DOWNLOAD_URL").unwrap(),
                forgemod.manifest._id,
                forgemod.manifest.version.clone().to_string()
            )),
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        for v in &vers {
            let _ = entity::version_beat_saber_versions::ActiveModel {
                version_id: Set(version),
                beat_saber_version_id: Set(v.id),
            }
            .insert(&trans)
            .await
            .unwrap();
        }

        for conflict in forgemod.manifest.conflicts {
            let c_ver = Versions::find()
                .filter(entity::versions::Column::ModId.eq(db_mod))
                .all(&trans)
                .await
                .unwrap()
                .into_iter()
                .filter(|c| {
                    conflict
                        .version
                        .matches(&Version::parse(&c.version).unwrap())
                })
                .collect::<Vec<_>>();

            for c in c_ver {
                let _ = entity::version_conflicts::ActiveModel {
                    version_id: Set(version),
                    dependent: Set(c.id),
                }
                .insert(&trans)
                .await
                .unwrap();
            }
        }

        for dependent in forgemod.manifest.depends {
            let d_ver = Versions::find()
                .filter(entity::versions::Column::ModId.eq(db_mod))
                .all(&trans)
                .await
                .unwrap()
                .into_iter()
                .filter(|d| {
                    dependent
                        .version
                        .matches(&Version::parse(&d.version).unwrap())
                })
                .collect::<Vec<_>>();

            for d in d_ver {
                let _ = entity::version_dependents::ActiveModel {
                    version_id: Set(version),
                    dependent: Set(d.id),
                }
                .insert(&trans)
                .await
                .unwrap();
            }
        }
        v_id = version;
    } else {
        let mod_stats = entity::mod_stats::ActiveModel {
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        let db_mod = entity::mods::ActiveModel {
            slug: Set(forgemod.manifest._id.clone()),
            name: Set(forgemod.manifest.name.clone()),
            author: Set(auser.id),
            description: Set(Some(forgemod.manifest.description.clone())),
            website: Set(Some(forgemod.manifest.website.clone())),
            category: Set(db_cata.id),
            stats: Set(mod_stats),
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        entity::user_mods::ActiveModel {
            user_id: Set(auser.id),
            mod_id: Set(db_mod),
        }
        .insert(&trans)
        .await
        .unwrap();

        for v in &vers {
            let _ = entity::mod_beat_saber_versions::ActiveModel {
                mod_id: Set(db_mod),
                beat_saber_version_id: Set(v.id),
            }
            .insert(&trans)
            .await
            .unwrap();
        }

        let version_stats = entity::version_stats::ActiveModel {
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        let version = entity::versions::ActiveModel {
            mod_id: Set(db_mod),
            version: Set(forgemod.manifest.version.clone().to_string()),
            stats: Set(version_stats),
            //todo: artifact hash
            artifact_hash: Set("".to_string()),
            //todo: download url
            download_url: Set(format!(
                "{}/cdn/{}@{}",
                std::env::var("DOWNLOAD_URL").unwrap(),
                forgemod.manifest._id,
                forgemod.manifest.version.clone().to_string()
            )),
            ..Default::default()
        }
        .insert(&trans)
        .await
        .unwrap()
        .id;

        for v in &vers {
            let _ = entity::version_beat_saber_versions::ActiveModel {
                version_id: Set(version),
                beat_saber_version_id: Set(v.id),
            }
            .insert(&trans)
            .await
            .unwrap();
        }

        entity::mod_versions::ActiveModel {
            mod_id: Set(db_mod),
            version_id: Set(version),
        }
        .insert(&trans)
        .await
        .unwrap();

        for conflict in forgemod.manifest.conflicts {
            let c_ver = Versions::find()
                .filter(entity::versions::Column::ModId.eq(db_mod))
                .all(&trans)
                .await
                .unwrap()
                .into_iter()
                .filter(|c| {
                    conflict
                        .version
                        .matches(&Version::parse(&c.version).unwrap())
                })
                .collect::<Vec<_>>();

            for c in c_ver {
                let _ = entity::version_conflicts::ActiveModel {
                    version_id: Set(version),
                    dependent: Set(c.id),
                }
                .insert(&trans)
                .await
                .unwrap();
            }
        }

        for dependent in forgemod.manifest.depends {
            let d_ver = Versions::find()
                .filter(entity::versions::Column::ModId.eq(db_mod))
                .all(&trans)
                .await
                .unwrap()
                .into_iter()
                .filter(|d| {
                    dependent
                        .version
                        .matches(&Version::parse(&d.version).unwrap())
                })
                .collect::<Vec<_>>();

            for d in d_ver {
                let _ = entity::version_dependents::ActiveModel {
                    version_id: Set(version),
                    dependent: Set(d.id),
                }
                .insert(&trans)
                .await
                .unwrap();
            }
        }
        v_id = version;
    }

    let db_mod = Mods::find()
        .filter(entity::mods::Column::Slug.eq(forgemod.manifest._id.clone()))
        .one(&trans)
        .await
        .unwrap()
        .unwrap();

    let _ = std::fs::create_dir(format!("./data/cdn/{}", &db_mod.id));
    std::fs::write(format!("./data/cdn/{}/{}.forgemod", &db_mod.id, v_id), buf).unwrap();

    trans.commit().await.unwrap();

    // add to meilisearch
    let client = meilisearch_sdk::client::Client::new(
        std::env::var("MEILI_URL").unwrap(),
        Some(std::env::var("MEILI_KEY").unwrap()),
    );

    let mod_vers = ModVersions::find()
        .filter(entity::mod_versions::Column::ModId.eq(db_mod.id))
        .find_also_related(Versions)
        .all(&db.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|(_, v)| Version::parse(&v.unwrap().version).unwrap())
        .collect::<Vec<_>>();

    let supported_versions = ModBeatSaberVersions::find().filter(entity::mod_beat_saber_versions::Column::ModId.eq(db_mod.id))
        .find_also_related(BeatSaberVersions)
        .all(&db.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|(_, v)| Version::parse(&v.unwrap().ver).unwrap())
        .collect::<Vec<_>>();

    let mod_stats = ModStats::find_by_id(db_mod.stats)
        .one(&db.pool)
        .await
        .unwrap()
        .unwrap();

    let meilimod = MeiliMod {
        id: db_mod.id,
        name: db_mod.name,
        description: db_mod.description.unwrap_or("".to_string()),
        category: db_cata.name,
        author: MeiliUser {
            username: auser.username.clone(),
            display_name: auser.display_name.unwrap_or(auser.username),
        },
        stats: MeiliModStats {
            downloads: mod_stats.downloads as u64,
        },
        versions: mod_vers
            .into_iter()
            .map(|v| MeiliVersion { version: v })
            .collect(),
        supported_versions,
    };
    client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).set_filterable_attributes(["category"]).await.unwrap();
    client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).set_searchable_attributes(["name", "description", "author.display_name", "author.username"]).await.unwrap();
    client.index(format!("{}_mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).set_sortable_attributes(["stats.downloads"]).await.unwrap();

    client.index(format!("{}mods", std::env::var("MEILI_PREFIX").unwrap_or("".to_string()))).add_or_replace(&[meilimod], None).await.unwrap();

    HttpResponse::Created().finish()
}
