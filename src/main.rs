use std::sync::Mutex;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_json::json;

fn initialize_db() -> SqliteResult<Connection> {
    let conn = Connection::open("./poasts.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS poasts (
            path TEXT PRIMARY KEY,
            body TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

async fn put_blob(
    path: web::Path<String>,
    body: String,
    db: web::Data<Connection>,
) -> impl Responder {
    match db.execute(
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

async fn get_blob(path: web::Path<String>, db: web::Data<Connection>) -> impl Responder {
    let result: Result<String, rusqlite::Error> = db.query_row(
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

async fn index() -> impl Responder {
    HttpResponse::Ok().body("This is Poast Trough!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_client = initialize_db().expect("Failed to connect to the database");
    let db_client = web::Data::new(Mutex::new(db_client));

    // println!("Database initalized!");
    HttpServer::new(move || {
        App::new()
            .app_data(db_client.clone())
            // .route("/", web::get().to(index))
            .route("/{path:.*}", web::put().to(put_blob))
            .route("/{path:.*}", web::get().to(get_blob))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
