// src/repository/user_repository.rs
use crate::infrastructure::db::Db;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub status: String,
}

pub struct OrderRepository {
    db: Db,
}

impl OrderRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn create_order(&self, user_id: i64, amount: f64) -> Result<Order> {
        let rec = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO orders (user_id, amount, status)
            VALUES (?, ?, 'pending')
            RETURNING id, user_id, amount, status
            "#,
            user_id,
            amount
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(rec)
    }

    pub async fn get_order(&self, id: i64) -> Result<Order> {
        let rec = sqlx::query_as!(
            Order,
            r#"
            SELECT id, user_id, amount, status
            FROM orders
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(rec)
    }
}