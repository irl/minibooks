use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;
use tera::Tera;

mod ledger;
mod settings;
mod db;
mod services;
mod error;

struct AppState {
    db: Pool<Sqlite>,
    tmpl: Tera,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePoolOptions::new()
        .connect(database_url.as_str())
        .await
        .expect("database connection is successful");

    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "\\src\\templates\\**\\*")).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState{
                db: pool.clone(),
                tmpl: tera.clone(),
            }))
            .service(web::scope("/api/v1")
                .service(services::account_list)
                .service(services::account_detail)
                .service(services::account_new)
                .service(services::journal_new)
                .service(services::report_balance_sheet)
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
