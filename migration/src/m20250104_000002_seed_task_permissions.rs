use sea_orm_migration::prelude::*;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter, DbErr};
use chrono::Utc;
use uuid::Uuid;
use crate::rbac_helpers::create_resource_with_admin_permissions;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        println!("🌱 Seeding task system RBAC permissions...");

        // Define task system permissions
        let permissions = vec![
            ("project_create", "Create new projects"),
            ("project_read", "View projects and their details"),
            ("project_admin", "Full project administration"),
            ("project_invite", "Invite users to projects"),
            ("task_create", "Create tasks in projects"),
            ("task_read", "View tasks in projects"),
            ("task_write", "Edit and update tasks"),
            ("task_assign", "Assign tasks to users"),
            ("task_delete", "Delete tasks"),
        ];

        // Define which permissions admin role should get (all in this case)
        let admin_permissions = vec![
            "project_create",
            "project_read", 
            "project_admin",
            "project_invite",
            "task_create",
            "task_read",
            "task_write",
            "task_assign",
            "task_delete",
        ];

        // Create resource with permissions and automatically assign to admin roles
        let task_system_resource_id = create_resource_with_admin_permissions(
            db,
            "task_system",
            "Task Management System",
            &permissions,
            &admin_permissions,
        ).await?;

        // Get user role for basic permissions
        let user_role = freshapi::entities::role::Entity::find()
            .filter(freshapi::entities::role::Column::Name.eq("user"))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("User role not found".to_string()))?;

        // Assign permissions to user (basic project participation)
        let user_permissions = vec![
            "project_create",  // Users can create their own projects
            "project_read",
            "task_create",     // Create tasks in projects they're members of
            "task_read",
            "task_write",      // Edit tasks they created or are assigned to
        ];

        // Get all permissions for the task_system resource
        let all_permissions = freshapi::entities::permission::Entity::find()
            .filter(freshapi::entities::permission::Column::ResourceId.eq(task_system_resource_id))
            .all(db)
            .await?;

        for action in &user_permissions {
            if let Some(permission) = all_permissions.iter().find(|p| p.action == *action) {
                // Check if assignment already exists
                let existing = freshapi::entities::role_permission::Entity::find()
                    .filter(freshapi::entities::role_permission::Column::RoleId.eq(user_role.id))
                    .filter(freshapi::entities::role_permission::Column::PermissionId.eq(permission.id))
                    .one(db)
                    .await?;

                if existing.is_none() {
                    let role_permission = freshapi::entities::role_permission::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        role_id: Set(user_role.id),
                        permission_id: Set(permission.id),
                        created_at: Set(Utc::now().into()),
                    };
                    role_permission.insert(db).await?;
                    println!("✅ Assigned {} to user", action);
                }
            }
        }

        println!("🎉 Task system RBAC permissions seeded successfully!");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        println!("🗑 Removing task system RBAC permissions...");

        // Get task_system resource
        if let Some(resource) = freshapi::entities::resource::Entity::find()
            .filter(freshapi::entities::resource::Column::Name.eq("task_system"))
            .one(db)
            .await?
        {
            // Delete role permissions for task_system permissions
            let permissions = freshapi::entities::permission::Entity::find()
                .filter(freshapi::entities::permission::Column::ResourceId.eq(resource.id))
                .all(db)
                .await?;

            for permission in permissions {
                // Delete role permissions
                freshapi::entities::role_permission::Entity::delete_many()
                    .filter(freshapi::entities::role_permission::Column::PermissionId.eq(permission.id))
                    .exec(db)
                    .await?;
                
                // Delete user permissions
                freshapi::entities::user_permission::Entity::delete_many()
                    .filter(freshapi::entities::user_permission::Column::PermissionId.eq(permission.id))
                    .exec(db)
                    .await?;
            }

            // Delete permissions
            freshapi::entities::permission::Entity::delete_many()
                .filter(freshapi::entities::permission::Column::ResourceId.eq(resource.id))
                .exec(db)
                .await?;

            // Delete resource
            freshapi::entities::resource::Entity::delete_by_id(resource.id)
                .exec(db)
                .await?;

            println!("✅ Removed task system RBAC permissions");
        }

        Ok(())
    }
}