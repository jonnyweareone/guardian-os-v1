mod accounts;
mod auth;
mod devices;
mod families;
mod files;
mod settings;

pub use accounts::*;
pub use auth::*;
pub use devices::*;
pub use families::*;
pub use files::*;
pub use settings::*;

use sqlx::mysql::MySqlPool;
use crate::error::Result;

#[derive(Clone)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = MySqlPool::connect(database_url).await?;
        Ok(Self { pool })
    }
    
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
}
