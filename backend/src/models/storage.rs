use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

/// Storage configuration that can be adjusted based on environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Total database storage in bytes (default: 15GB - Render Basic plan)
    pub total_db_storage: i64,
    
    /// Default max storage per user in bytes (default: 10MB)
    pub default_user_quota: i64,
    
    /// Maximum number of users we expect to support
    pub expected_max_users: i32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let total_storage_gb = env::var("TOTAL_DB_STORAGE_GB")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(15); // Default: 15GB (Render Basic plan)
            
        let user_quota_mb = env::var("USER_STORAGE_QUOTA_MB")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(10); // Default: 10MB per user
            
        let expected_max_users = env::var("EXPECTED_MAX_USERS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1000); // Default expectation: 1000 users
            
        StorageConfig {
            total_db_storage: total_storage_gb * 1024 * 1024 * 1024, // Convert GB to bytes
            default_user_quota: user_quota_mb * 1024 * 1024, // Convert MB to bytes
            expected_max_users,
        }
    }
}

pub struct StorageManager;

impl StorageManager {
    /// Get current database size in bytes
    pub async fn get_db_size(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT pg_database_size(current_database()) as size
            "#
        )
        .fetch_one(pool)
        .await?;
        
        Ok(result.size.unwrap_or(0))
    }
    
    /// Get total database allocated size (from config)
    pub fn get_total_db_allocated() -> i64 {
        StorageConfig::default().total_db_storage
    }
    
    /// Get user storage quota
    pub fn get_user_quota() -> i64 {
        StorageConfig::default().default_user_quota
    }
    
    /// Get current database usage percentage
    pub async fn get_db_usage_percentage(pool: &PgPool) -> Result<f64, sqlx::Error> {
        let current_size = Self::get_db_size(pool).await?;
        let total_size = Self::get_total_db_allocated();
        
        Ok((current_size as f64 / total_size as f64) * 100.0)
    }
} 