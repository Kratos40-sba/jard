use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use crate::api::{Order, OrderItem, ScanRecord};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Initialize Tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS order_items (
                order_id TEXT,
                barcode TEXT,
                name TEXT,
                target_qty INTEGER,
                packed_qty INTEGER,
                PRIMARY KEY (order_id, barcode)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS scans (
                barcode TEXT PRIMARY KEY,
                count INTEGER,
                last_worker TEXT,
                is_anomaly BOOLEAN,
                anomaly_reason TEXT
            )",
            [],
        )?;

        Ok(Self { conn: Arc::new(Mutex<Connection>::new(conn)) })
    }

    pub fn save_order(&self, order: &Order) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO orders (id, status) VALUES (?1, ?2)",
            params![order.id, order.status],
        )?;

        for item in &order.items {
            conn.execute(
                "INSERT OR REPLACE INTO order_items (order_id, barcode, name, target_qty, packed_qty) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![order.id, item.barcode, item.name, item.target_qty, item.packed_qty],
            )?;
        }
        Ok(())
    }

    pub fn load_orders(&self) -> Result<Vec<Order>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, status FROM orders")?;
        let order_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut orders = Vec::new();
        for row in order_rows {
            let (id, status) = row?;
            let mut item_stmt = conn.prepare("SELECT barcode, name, target_qty, packed_qty FROM order_items WHERE order_id = ?1")?;
            let items = item_stmt.query_map(params![id], |ir| {
                Ok(OrderItem {
                    barcode: ir.get(0)?,
                    name: ir.get(1)?,
                    target_qty: ir.get(2)?,
                    packed_qty: ir.get(3)?,
                })
            })?.collect::<Result<Vec<_>>>()?;

            orders.push(Order { id, status, items });
        }
        Ok(orders)
    }

    pub fn save_scan(&self, barcode: &str, record: &ScanRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO scans (barcode, count, last_worker, is_anomaly, anomaly_reason) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![barcode, record.count, record.last_worker, record.is_anomaly, record.anomaly_reason],
        )?;
        Ok(())
    }

    pub fn load_scans(&self) -> Result<std::collections::HashMap<String, ScanRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT barcode, count, last_worker, is_anomaly, anomaly_reason FROM scans")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ScanRecord {
                    count: row.get(1)?,
                    last_worker: row.get(2)?,
                    is_anomaly: row.get(3)?,
                    anomaly_reason: row.get(4)?,
                }
            ))
        })?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (barcode, record) = row?;
            map.insert(barcode, record);
        }
        Ok(map)
    }
}
