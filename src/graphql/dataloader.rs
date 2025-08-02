use async_graphql::dataloader::{DataLoader, Loader};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::PermissionService;

/// DataLoader for batching permission requests
#[derive(Clone)]
pub struct PermissionLoader {
    permission_service: PermissionService,
    resource_name: String,
}

impl PermissionLoader {
    pub fn new(db: DatabaseConnection, resource_name: String) -> Self {
        Self {
            permission_service: PermissionService::new(db),
            resource_name,
        }
    }
}

impl Loader<Uuid> for PermissionLoader {
    type Value = Vec<String>;
    type Error = String;

    /// Batch load permissions for multiple users
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let permissions_map = self
            .permission_service
            .get_users_permissions_batch(keys, &self.resource_name)
            .await
            .map_err(|e| format!("Failed to load permissions: {}", e))?;

        Ok(permissions_map)
    }
}

/// DataLoader for batching specific permission checks
#[derive(Clone)]
pub struct PermissionCheckLoader {
    permission_service: PermissionService,
    resource_name: String,
    action: String,
}

impl PermissionCheckLoader {
    pub fn new(db: DatabaseConnection, resource_name: String, action: String) -> Self {
        Self {
            permission_service: PermissionService::new(db),
            resource_name,
            action,
        }
    }
}

impl Loader<Uuid> for PermissionCheckLoader {
    type Value = bool;
    type Error = String;

    /// Batch check specific permission for multiple users
    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let result = self
            .permission_service
            .users_have_permission_batch(keys, &self.resource_name, &self.action)
            .await
            .map_err(|e| format!("Failed to check permissions: {}", e))?;

        Ok(result)
    }
}

/// DataLoader context for GraphQL resolvers
#[derive(Clone)]
pub struct DataLoaderContext {
    pub permission_loader: Arc<DataLoader<PermissionLoader>>,
    pub invite_users_loader: Arc<DataLoader<PermissionCheckLoader>>,
    pub user_management_loader: Arc<DataLoader<PermissionCheckLoader>>,
    pub admin_loader: Arc<DataLoader<PermissionCheckLoader>>,
    pub system_admin_loader: Arc<DataLoader<PermissionCheckLoader>>,
}

impl DataLoaderContext {
    /// Create new DataLoader context with optimized batch sizes
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            permission_loader: Arc::new(DataLoader::new(
                PermissionLoader::new(db.clone(), "freshapi".to_string()),
                tokio::spawn,
            )
            .max_batch_size(100)), // Batch up to 100 permission requests
            
            invite_users_loader: Arc::new(DataLoader::new(
                PermissionCheckLoader::new(db.clone(), "freshapi".to_string(), "invite_users".to_string()),
                tokio::spawn,
            )
            .max_batch_size(100)),
            
            user_management_loader: Arc::new(DataLoader::new(
                PermissionCheckLoader::new(db.clone(), "freshapi".to_string(), "user_management".to_string()),
                tokio::spawn,
            )
            .max_batch_size(100)),
            
            admin_loader: Arc::new(DataLoader::new(
                PermissionCheckLoader::new(db.clone(), "freshapi".to_string(), "admin".to_string()),
                tokio::spawn,
            )
            .max_batch_size(100)),
            
            system_admin_loader: Arc::new(DataLoader::new(
                PermissionCheckLoader::new(db, "freshapi".to_string(), "system_admin".to_string()),
                tokio::spawn,
            )
            .max_batch_size(100)),
        }
    }

    /// Load permissions for a single user (with caching)
    pub async fn load_user_permissions(&self, user_id: Uuid) -> Result<Vec<String>, String> {
        self.permission_loader
            .load_one(user_id)
            .await?
            .ok_or_else(|| "User not found".to_string())
    }

    /// Check if user can invite users (with caching)
    pub async fn can_invite_users(&self, user_id: Uuid) -> Result<bool, String> {
        Ok(self.invite_users_loader.load_one(user_id).await?.unwrap_or(false))
    }

    /// Check if user can manage users (with caching)
    pub async fn can_manage_users(&self, user_id: Uuid) -> Result<bool, String> {
        Ok(self.user_management_loader.load_one(user_id).await?.unwrap_or(false))
    }

    /// Check if user is admin (with caching)
    pub async fn is_admin(&self, user_id: Uuid) -> Result<bool, String> {
        Ok(self.admin_loader.load_one(user_id).await?.unwrap_or(false))
    }

    /// Check if user is system admin (with caching)
    pub async fn is_system_admin(&self, user_id: Uuid) -> Result<bool, String> {
        Ok(self.system_admin_loader.load_one(user_id).await?.unwrap_or(false))
    }

    /// Clear all caches (useful for testing or when permissions change)
    pub fn clear_all(&self) {
        self.permission_loader.clear();
        self.invite_users_loader.clear();
        self.user_management_loader.clear();
        self.admin_loader.clear();
        self.system_admin_loader.clear();
    }
}