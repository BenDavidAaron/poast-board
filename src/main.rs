use rusqlite::{Connection, Result as SqliteResult};

fn initalize_db() -> SqliteResult<Connection> {
    let conn = Connection::open("poasts.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS poasts(
            title TEXT PRIMARY KEY,
            body TEXT NOT NULL,
            timestamp DATETIEME DEFAULT CURRENT_TIMESTAMP
    )",
        [],
    )?;
    Ok(conn)
}

fn main() -> SqliteResult<()> {
    let conn = initalize_db()?;
    println!("Database initalized!");
    Ok(())
}
