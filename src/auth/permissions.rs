use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use uuid::Uuid;
use std::collections::{HashSet, HashMap};

use crate::entities::{prelude::*, user_permission, resource};

#[derive(Clone)]
pub struct PermissionService {
    db: DatabaseConnection,
}

impl PermissionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get all permissions for a user (both from role and direct assignments) - OPTIMIZED
    pub async fn get_user_permissions(
        &self,
        user_id: Uuid,
        resource_name: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        use crate::entities::{role_permission};
        
        let mut permissions = HashSet::new();

        // Get target resource first
        let target_resource = Resource::find()
            .filter(resource::Column::Name.eq(resource_name))
            .filter(resource::Column::IsActive.eq(true))
            .one(&self.db)
            .await?;

        let resource_id = match target_resource {
            Some(resource) => resource.id,
            None => return Ok(Vec::new()), // Resource doesn't exist
        };

        // Get user with role in single query
        let user_with_role = User::find_by_id(user_id)
            .find_also_related(Role)
            .one(&self.db)
            .await?;

        if let Some((_, role_opt)) = user_with_role {
            // Get role permissions if user has a role
            if let Some(role) = role_opt {
                let role_permissions = RolePermission::find()
                    .filter(role_permission::Column::RoleId.eq(role.id))
                    .find_also_related(Permission)
                    .all(&self.db)
                    .await?;

                for (_, permission_opt) in role_permissions {
                    if let Some(permission) = permission_opt {
                        if permission.resource_id == resource_id && permission.is_active {
                            permissions.insert(permission.action);
                        }
                    }
                }
            }
        }

        // Get direct user permissions (can override role permissions)
        let user_permissions = UserPermission::find()
            .filter(user_permission::Column::UserId.eq(user_id))
            .find_also_related(Permission)
            .all(&self.db)
            .await?;

        for (user_perm, permission_opt) in user_permissions {
            if let Some(permission) = permission_opt {
                if permission.resource_id == resource_id && permission.is_active {
                    if user_perm.is_granted {
                        permissions.insert(permission.action);
                    } else {
                        // Deny permission explicitly removes it
                        permissions.remove(&permission.action);
                    }
                }
            }
        }

        Ok(permissions.into_iter().collect())
    }

    /// Check if user has a specific permission
    pub async fn user_has_permission(
        &self,
        user_id: Uuid,
        resource_name: &str,
        action: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let permissions = self.get_user_permissions(user_id, resource_name).await?;
        Ok(permissions.contains(&action.to_string()))
    }

    /// Check if user has admin permissions
    pub async fn user_is_admin(
        &self,
        user_id: Uuid,
        resource_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.user_has_permission(user_id, resource_name, "admin").await
    }

    /// Check if user has system admin permissions (super admin)
    pub async fn user_is_system_admin(
        &self,
        user_id: Uuid,
        resource_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.user_has_permission(user_id, resource_name, "system_admin").await
    }

    /// Get user's role level (higher = more permissions) - OPTIMIZED
    pub async fn get_user_role_level(
        &self,
        user_id: Uuid,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let role_level = User::find_by_id(user_id)
            .find_also_related(Role)
            .one(&self.db)
            .await?
            .and_then(|(_, role_opt)| role_opt)
            .map(|role| role.level)
            .unwrap_or(0);
        
        Ok(role_level)
    }

    /// Check if user can manage another user (based on role hierarchy)
    pub async fn user_can_manage_user(
        &self,
        manager_id: Uuid,
        target_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let manager_level = self.get_user_role_level(manager_id).await?;
        let target_level = self.get_user_role_level(target_id).await?;
        
        // Manager must have higher level than target
        Ok(manager_level > target_level)
    }

    /// Grant permission directly to user
    pub async fn grant_user_permission(
        &self,
        user_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use sea_orm::{ActiveModelTrait, Set};
        use chrono::Utc;

        // Check if permission already exists
        if let Some(existing) = UserPermission::find()
            .filter(user_permission::Column::UserId.eq(user_id))
            .filter(user_permission::Column::PermissionId.eq(permission_id))
            .one(&self.db)
            .await?
        {
            // Update existing permission
            let mut active_model: user_permission::ActiveModel = existing.into();
            active_model.is_granted = Set(true);
            active_model.updated_at = Set(Utc::now().into());
            active_model.update(&self.db).await?;
        } else {
            // Create new permission
            let new_permission = user_permission::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                permission_id: Set(permission_id),
                is_granted: Set(true),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            new_permission.insert(&self.db).await?;
        }

        Ok(())
    }

    /// Revoke permission from user
    pub async fn revoke_user_permission(
        &self,
        user_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use sea_orm::{ActiveModelTrait, Set};
        use chrono::Utc;

        // Check if permission already exists
        if let Some(existing) = UserPermission::find()
            .filter(user_permission::Column::UserId.eq(user_id))
            .filter(user_permission::Column::PermissionId.eq(permission_id))
            .one(&self.db)
            .await?
        {
            // Update existing permission to deny
            let mut active_model: user_permission::ActiveModel = existing.into();
            active_model.is_granted = Set(false);
            active_model.updated_at = Set(Utc::now().into());
            active_model.update(&self.db).await?;
        } else {
            // Create new denial permission
            let new_permission = user_permission::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                permission_id: Set(permission_id),
                is_granted: Set(false),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            new_permission.insert(&self.db).await?;
        }

        Ok(())
    }

    /// Batch load permissions for multiple users - OPTIMIZED for GraphQL resolvers
    pub async fn get_users_permissions_batch(
        &self,
        user_ids: &[Uuid],
        resource_name: &str,
    ) -> Result<HashMap<Uuid, Vec<String>>, Box<dyn std::error::Error>> {
        use crate::entities::{role_permission, user};
        
        let mut result: HashMap<Uuid, HashSet<String>> = HashMap::new();
        
        // Initialize result map
        for &user_id in user_ids {
            result.insert(user_id, HashSet::new());
        }

        // Get target resource
        let target_resource = Resource::find()
            .filter(resource::Column::Name.eq(resource_name))
            .filter(resource::Column::IsActive.eq(true))
            .one(&self.db)
            .await?;

        let resource_id = match target_resource {
            Some(resource) => resource.id,
            None => return Ok(HashMap::new()), // Resource doesn't exist
        };

        // Get all users with their roles
        let users_with_roles = User::find()
            .filter(user::Column::Id.is_in(user_ids.iter().cloned()))
            .find_also_related(Role)
            .all(&self.db)
            .await?;

        // Track users with roles for role permission lookup
        let mut users_with_roles_map: HashMap<Uuid, Uuid> = HashMap::new();

        for (user, role_opt) in &users_with_roles {
            if let Some(role) = role_opt {
                users_with_roles_map.insert(user.id, role.id);
            }
        }

        // Batch load role permissions for all roles at once
        if !users_with_roles_map.is_empty() {
            let role_ids: Vec<Uuid> = users_with_roles_map.values().cloned().collect();
            
            let role_permissions = RolePermission::find()
                .filter(role_permission::Column::RoleId.is_in(role_ids))
                .find_also_related(Permission)
                .all(&self.db)
                .await?;

            // Group role permissions by role_id
            let mut role_perms_map: HashMap<Uuid, Vec<String>> = HashMap::new();
            for (role_perm, permission_opt) in role_permissions {
                if let Some(permission) = permission_opt {
                    if permission.resource_id == resource_id && permission.is_active {
                        role_perms_map
                            .entry(role_perm.role_id)
                            .or_insert_with(Vec::new)
                            .push(permission.action);
                    }
                }
            }

            // Apply role permissions to users
            for (user_id, role_id) in &users_with_roles_map {
                if let Some(role_permissions) = role_perms_map.get(role_id) {
                    let user_permissions = result.entry(*user_id).or_insert_with(HashSet::new);
                    for perm in role_permissions {
                        user_permissions.insert(perm.clone());
                    }
                }
            }
        }

        // Get direct user permissions for all users
        let user_permissions = UserPermission::find()
            .filter(user_permission::Column::UserId.is_in(user_ids.iter().cloned()))
            .find_also_related(Permission)
            .all(&self.db)
            .await?;

        for (user_perm, permission_opt) in user_permissions {
            if let Some(permission) = permission_opt {
                if permission.resource_id == resource_id && permission.is_active {
                    let user_permissions = result.entry(user_perm.user_id).or_insert_with(HashSet::new);
                    if user_perm.is_granted {
                        user_permissions.insert(permission.action);
                    } else {
                        // Deny permission explicitly removes it
                        user_permissions.remove(&permission.action);
                    }
                }
            }
        }

        // Convert HashSets to Vecs
        let final_result = result
            .into_iter()
            .map(|(user_id, perms)| (user_id, perms.into_iter().collect()))
            .collect();

        Ok(final_result)
    }

    /// Batch check permissions for multiple users
    pub async fn users_have_permission_batch(
        &self,
        user_ids: &[Uuid],
        resource_name: &str,
        action: &str,
    ) -> Result<HashMap<Uuid, bool>, Box<dyn std::error::Error>> {
        let permissions_map = self.get_users_permissions_batch(user_ids, resource_name).await?;
        
        let result = permissions_map
            .into_iter()
            .map(|(user_id, permissions)| {
                let has_permission = permissions.contains(&action.to_string());
                (user_id, has_permission)
            })
            .collect();

        Ok(result)
    }
}