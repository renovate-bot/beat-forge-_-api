use actix_web::{get, post, web, Error, HttpResponse, Responder};
use entity::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::Database;

#[get("/cdn/{slug}@{version}")]
async fn cdn_get(db: web::Data<Database>, path: web::Path<(String, String)>) -> impl Responder {
    let db = db.pool.clone();

    let (slug, version) = path.into_inner();
    let db_mod = Mods::find()
        .filter(entity::mods::Column::Slug.eq(&slug))
        .one(&db)
        .await
        .unwrap();

    if let Some(db_mod) = db_mod {
        let db_version = Versions::find()
            .filter(entity::versions::Column::ModId.eq(db_mod.id))
            .filter(entity::versions::Column::Version.eq(&version))
            .one(&db)
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

            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}-v{}.beatforge\"", slug, version),
                ))
                .body(file);
        }
    }

    HttpResponse::NotFound().finish()
}
