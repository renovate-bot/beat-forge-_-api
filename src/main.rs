use std::{io, sync::Arc, path::Path};

use actix_cors::Cors;
use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Error,
};
use migration::MigratorTrait;
use rand::Rng;

mod schema;
mod users;
mod mods;
mod versions;
mod auth;
mod cdn;

use crate::schema::{create_schema, Schema};

/// GraphiQL playground UI
async fn graphiql_route() -> Result<HttpResponse, Error> {
    juniper_actix::graphiql_handler("/graphgl", None).await
}

async fn playground_route() -> Result<HttpResponse, Error> {
    juniper_actix::playground_handler("/graphgl", None).await
}

async fn graphql_route(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    data: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    let database = Database {
        pool: sea_orm::Database::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap(),
    };
    juniper_actix::graphql_handler(&data, &database, req, payload).await
}

#[derive(Clone)]
pub struct Database {
    pool: sea_orm::DatabaseConnection,
}

#[derive(Clone, Copy)]
pub struct Key([u8; 1024]);

lazy_static::lazy_static! {
    pub static ref KEY: Arc<Key> = {
        if !Path::new("./data/secret.key").exists() {
            let _ = std::fs::create_dir(Path::new("./data"));
            let mut rng = rand::thread_rng();
            let key: Vec<u8> = (0..1024).map(|_| rng.gen::<u8>()).collect();
            std::fs::write("./data/secret.key", key).unwrap();
    
            println!("Generated secret key (first run)");
        }
    
        Arc::new(Key(std::fs::read("./data/secret.key").unwrap().try_into().unwrap()))
    };
}

// #[derive(Clone)]
// pub struct AppState {
//     db: Database,
//     schema: Arc<Schema>,
// }

impl juniper::Context for Database {}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server on port 8080");
    log::info!("GraphiQL playground: http://localhost:8080/graphiql");
    log::info!("Playground: http://localhost:8080/playground");

    let db_conn = sea_orm::Database::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _ = std::fs::create_dir(Path::new("./data/cdn"));

    //migrate
    migration::Migrator::up(&db_conn, None).await.unwrap();

    // Start HTTP server
    HttpServer::new( move || {
        App::new()
            .app_data(Data::new(create_schema()))
            .app_data(Data::new(Database {
                pool: db_conn.clone(),
            }))
            .service(
                web::resource("/graphgl")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
            .service(users::user_auth)
            .service(mods::create_mod)
            .service(cdn::cdn_get)
            // the graphiql UI requires CORS to be enabled
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}