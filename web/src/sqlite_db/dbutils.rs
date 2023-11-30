use crate::monitor::monitoring_data::MonitoringData;
use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::path::PathBuf;

pub struct SQLiteDB {
    sqlite_conn: Connection,
}
impl SQLiteDB {
    pub fn new(db_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            sqlite_conn: Connection::open(db_path)?,
        })
    }

    pub fn crate_table(&self, table_name: &str) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                timestamp   INTERGER PRIMARY KEY,
                load_1      REAL,
                load_5      REAL,
                load_15     REAL,
                cpu_usage   REAL,
                memory_used INTEGER,
                memory_free INTEGER,
                swap_used   INTEGER,
                swap_free   INTEGER,
                disk_used   INTEGER,
                disk_read   INTEGER,
                disk_write  INTEGER,
                network_rx  INTEGER,
                network_tx  INTEGER
        )",
            table_name
        );
        self.sqlite_conn.execute(&sql, [])?;

        Ok(())
    }

    pub fn insert_data(&self, table_name: &str, data: MonitoringData) -> Result<()> {
        let sql = format!(
            "INSERT INTO {} 
            (timestamp, load_1, load_5, load_15, cpu_usage, memory_used, memory_free,
            swap_used, swap_free, disk_used, disk_read, disk_write, network_rx, network_tx) 
            VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            table_name
        );

        self.sqlite_conn.execute(
            &sql,
            params![
                data.timestamp,
                data.load_1,
                data.load_5,
                data.load_15,
                data.cpu_usage,
                data.memory_used,
                data.memory_free,
                data.swap_used,
                data.swap_free,
                data.disk_used,
                data.disk_read,
                data.disk_write,
                data.network_rx,
                data.network_tx,
            ],
        )?;

        Ok(())
    }
}
