use anyhow::Result;
use serde::{Serialize, Deserialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::strategy::{PortfolioPosition, PartialSell};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub token_address: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub entry_timestamp: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub partial_sells: Vec<PartialSell>,
    pub status: PositionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialSell {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    PartiallyExited,
}

pub struct PositionsCollection {
    pool: Pool<Postgres>,
}

impl PositionsCollection {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn create_position(&self, position: &Position) -> Result<Uuid> {
        let json = serde_json::to_value(position)?;

        sqlx::query!(
            "INSERT INTO positions (id, document) VALUES ($1, $2)",
            position.id,
            json
        )
        .execute(&self.pool)
        .await?;

        Ok(position.id)
    }

    pub async fn get_position(&self, id: Uuid) -> Result<Option<Position>> {
        let row = sqlx::query!(
            "SELECT document FROM positions WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(serde_json::from_value(row.document)?)),
            None => Ok(None),
        }
    }

    pub async fn get_position_by_token(&self, token_address: &str) -> Result<Option<Position>> {
        let row = sqlx::query!(
            "SELECT document FROM positions WHERE document->>'token_address' = $1",
            token_address
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(serde_json::from_value(row.document)?)),
            None => Ok(None),
        }
    }

    pub async fn update_position(&self, position: &Position) -> Result<bool> {
        let json = serde_json::to_value(position)?;

        let result = sqlx::query!(
            "UPDATE positions SET document = $1 WHERE id = $2",
            json,
            position.id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_partial_sell(
        &self,
        token_address: &str,
        price: f64,
        quantity: f64,
    ) -> Result<bool> {
        let mut position = match self.get_position_by_token(token_address).await? {
            Some(p) => p,
            None => return Ok(false),
        };

        let partial_sell = PartialSell {
            price,
            quantity,
            timestamp: Utc::now(),
        };

        position.partial_sells.push(partial_sell);
        position.status = PositionStatus::PartiallyExited;
        position.last_update = Utc::now();

        self.update_position(&position).await
    }

    pub async fn close_position(&self, token_address: &str) -> Result<bool> {
        let mut position = match self.get_position_by_token(token_address).await? {
            Some(p) => p,
            None => return Ok(false),
        };

        position.status = PositionStatus::Closed;
        position.last_update = Utc::now();

        self.update_position(&position).await
    }

    pub async fn get_open_positions(&self) -> Result<Vec<Position>> {
        let rows = sqlx::query!(
            "SELECT document FROM positions WHERE document->>'status' = 'Open'"
        )
        .fetch_all(&self.pool)
        .await?;

        let positions = rows
            .into_iter()
            .map(|row| serde_json::from_value(row.document))
            .collect::<Result<Vec<Position>, _>>()?;

        Ok(positions)
    }

    pub async fn get_portfolio_stats(&self) -> Result<PortfolioStats> {
        let positions = self.get_open_positions().await?;
        
        let mut stats = PortfolioStats {
            total_value_sol: 0.0,
            total_value_usd: 0.0,
            total_realized_pnl_sol: 0.0,
            total_unrealized_pnl_sol: 0.0,
            position_count: positions.len(),
            profitable_positions: 0,
        };

        for pos in positions {
            stats.total_value_sol += pos.quantity * pos.entry_price;
            stats.total_value_usd += pos.quantity * pos.entry_price;
            stats.total_realized_pnl_sol += pos.partial_sells.iter()
                .map(|sell| (sell.price - pos.entry_price) * sell.quantity)
                .sum();
            stats.total_unrealized_pnl_sol += (pos.entry_price - pos.entry_price) * pos.quantity;
            
            if pos.partial_sells.iter()
                .map(|sell| (sell.price - pos.entry_price) * sell.quantity)
                .sum::<f64>() > 0.0 {
                stats.profitable_positions += 1;
            }
        }

        Ok(stats)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub total_value_sol: f64,
    pub total_value_usd: f64,
    pub total_realized_pnl_sol: f64,
    pub total_unrealized_pnl_sol: f64,
    pub position_count: usize,
    pub profitable_positions: usize,
} 