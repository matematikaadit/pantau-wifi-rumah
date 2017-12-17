use rusqlite::{self, Connection};
use rusqlite::types::ToSql;

#[derive(Debug, Serialize)]
pub struct Item {
    mac: String,
    owner: Option<String>,
}

/// Holds rusqlite connection
pub struct Db {
    conn: Connection
}

impl Db {
    /// Create new db
    pub fn new(db_file: &str) -> Self {
        Db { conn: Connection::open(db_file).expect("sqlite3 database file") }
    }

    /// Create items for the template
    pub fn query(&self, mac_addresses: &[String]) -> Vec<Item> {
        if mac_addresses.len() == 0 {
            return Vec::new();
        }
        let qmarks = vec!["(?)"; mac_addresses.len()].join(",");
        let statement = format!("
WITH visible (mac) AS (VALUES {})
SELECT visible.mac AS mac, known.owner AS owner
FROM visible LEFT JOIN known ON visible.mac = known.mac
ORDER BY owner", qmarks);

        let mut statement = self.conn.prepare(&statement)
            .expect("SQL statement");

        let mut params = Vec::with_capacity(mac_addresses.len());
        for mac in mac_addresses {
            params.push(mac as &ToSql);
        }

        let result = statement.query_map(&params, |row| {
            Item { mac: row.get(0), owner: row.get(1) }
        }).expect("Resulting rows from db");

        result.collect::<rusqlite::Result<Vec<_>>>().unwrap_or(Vec::new())
    }
}
