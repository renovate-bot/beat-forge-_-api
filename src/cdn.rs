use actix_web::{get, post, web, Error, HttpResponse, Responder};
use entity::prelude::*;
use forge_lib::structs::forgemod::ForgeMod;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::Database;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CdnType {
    Dll,
    Package,
}

async fn cdn_handler(
    db: web::Data<Database>,
    slug: String,
    version: String,
    dl_type: CdnType,
) -> impl Responder {
    let db_mod = Mods::find()
        .filter(entity::mods::Column::Slug.eq(&slug))
        .one(&db.pool)
        .await
        .unwrap();

    if let Some(db_mod) = db_mod {
        let db_version = Versions::find()
            .filter(entity::versions::Column::ModId.eq(db_mod.id))
            .filter(entity::versions::Column::Version.eq(&version))
            .one(&db.pool)
            .await
            .unwrap();

        if let Some(db_version) = db_version {
            let file = match std::fs::read(format!(
                "./data/cdn/{}/{}.forgemod",
                db_mod.id, db_version.id
            )) {
                Ok(file) => file,
                Err(_) => return HttpResponse::NotFound().finish(),
            };
            match dl_type {
                CdnType::Dll => {
                    let dll = ForgeMod::try_from(&*file).unwrap().artifact_data;

                    return HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}-v{}.dll\"", slug, version),
                        ))
                        .body(dll);
                }
                CdnType::Package => {
                    return HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}-v{}.beatforge\"", slug, version),
                        ))
                        .body(file);
                }
            }
        }
    }

    HttpResponse::NotFound().finish()
}

#[get("/cdn/{slug}@{version}/{type}")]
async fn cdn_get(
    db: web::Data<Database>,
    path: web::Path<(String, String, CdnType)>,
) -> impl Responder {
    let (slug, version, dl_type) = path.into_inner();

    cdn_handler(db, slug, version, dl_type).await
}

#[get("/cdn/{slug}@{version}")]
async fn cdn_get_typeless(
    db: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (slug, version) = path.into_inner();
    
    cdn_handler(db, slug, version, CdnType::Package).await
}
