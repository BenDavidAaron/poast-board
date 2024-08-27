use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result as SqliteResult};
use serde_json::json;

type DbPool = r2d2::Pool<SqliteConnectionManager>;

fn initialize_db(db_path: &str) -> SqliteResult<DbPool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool");

    let conn = pool.get().expect("Failed to get connection from pool");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS poasts (
            path TEXT PRIMARY KEY,
            body TEXT NOT NULL
        )",
        [],
    )?;
    Ok(pool)
}

async fn put_blob(
    path: web::Path<String>,
    body: String,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT body FROM poasts WHERE path = ?1", params![path.as_str()], |row| row.get(0)
    );
    match result {
        Ok(_) => {
            println!("Blob already exists at path: {}", path);
            return HttpResponse::Conflict().json(json!({"error": "Blob already exists"}));
        }
        Err(_e) => {
            // ignore the error and continue
        }
    }

    match conn.execute(
        "INSERT INTO poasts (path, body) VALUES (?1, ?2)",
        params![path.as_str(), body.as_str()],
    ) {
        Ok(_) => {
            println!("Inserted blob at path: {}", path);
            HttpResponse::Ok().json(json!({"status": "u made a post"}))
        }
        Err(e) => {
            println!("Failed to insert blob at path: {}: {}", path, e);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to post"}))
        }
    }
}

async fn get_blob(path: web::Path<String>, pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT body FROM poasts WHERE path = ?1",
        params![path.as_str()],
        |row| row.get(0),
    );

    match result {
        Ok(body) => {
            println!("Got blob at path: {}", path);
            HttpResponse::Ok().body(body)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            println!("No blob found at path: {}", path);
            HttpResponse::NotFound().body("Not Found")
        }
        Err(e) => {
            println!("Failed to get blob at path: {}: {}", path, e);
            HttpResponse::NotFound().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = initialize_db("./poasts.db").expect("Failed to create database pool");
    let pool = web::Data::new(pool);

    println!("Database initialized!");
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .route("/{path:.*}", web::put().to(put_blob))
            .route("/{path:.*}", web::get().to(get_blob))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}