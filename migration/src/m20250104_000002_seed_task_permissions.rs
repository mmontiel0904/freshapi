use sea_orm_migration::prelude::*;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter, DbErr};
use chrono::Utc;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        println!("🌱 Seeding task system RBAC permissions...");

        // Create task_system resource
        let task_system_resource_id = Uuid::new_v4();
        let task_system_resource = freshapi::entities::resource::ActiveModel {
            id: Set(task_system_resource_id),
            name: Set("task_system".to_string()),
            description: Set(Some("Task Management System".to_string())),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        task_system_resource.insert(db).await?;
        println!("✅ Created resource: task_system");

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

        let mut permission_ids = Vec::new();

        for (action, description) in &permissions {
            let permission_id = Uuid::new_v4();
            let permission = freshapi::entities::permission::ActiveModel {
                id: Set(permission_id),
                action: Set(String::from(*action)),
                resource_id: Set(task_system_resource_id),
                description: Set(Some(String::from(*description))),
                is_active: Set(true),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            permission.insert(db).await?;
            permission_ids.push((action, permission_id));
            println!("✅ Created permission: {}", action);
        }

        // Get existing roles
        let super_admin_role = freshapi::entities::role::Entity::find()
            .filter(freshapi::entities::role::Column::Name.eq("super_admin"))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Super admin role not found".to_string()))?;

        let admin_role = freshapi::entities::role::Entity::find()
            .filter(freshapi::entities::role::Column::Name.eq("admin"))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Admin role not found".to_string()))?;

        let user_role = freshapi::entities::role::Entity::find()
            .filter(freshapi::entities::role::Column::Name.eq("user"))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("User role not found".to_string()))?;

        // Assign permissions to super_admin (all permissions)
        for (action, permission_id) in &permission_ids {
            let role_permission = freshapi::entities::role_permission::ActiveModel {
                id: Set(Uuid::new_v4()),
                role_id: Set(super_admin_role.id),
                permission_id: Set(*permission_id),
                created_at: Set(Utc::now().into()),
            };
            role_permission.insert(db).await?;
            println!("✅ Assigned {} to super_admin", action);
        }

        // Assign permissions to admin (project management)
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

        for action in &admin_permissions {
            if let Some((_, permission_id)) = permission_ids.iter().find(|(a, _)| *a == action) {
                let role_permission = freshapi::entities::role_permission::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    role_id: Set(admin_role.id),
                    permission_id: Set(*permission_id),
                    created_at: Set(Utc::now().into()),
                };
                role_permission.insert(db).await?;
                println!("✅ Assigned {} to admin", action);
            }
        }

        // Assign permissions to user (basic project participation)
        let user_permissions = vec![
            "project_create",  // Users can create their own projects
            "project_read",
            "task_create",     // Create tasks in projects they're members of
            "task_read",
            "task_write",      // Edit tasks they created or are assigned to
        ];

        for action in &user_permissions {
            if let Some((_, permission_id)) = permission_ids.iter().find(|(a, _)| *a == action) {
                let role_permission = freshapi::entities::role_permission::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    role_id: Set(user_role.id),
                    permission_id: Set(*permission_id),
                    created_at: Set(Utc::now().into()),
                };
                role_permission.insert(db).await?;
                println!("✅ Assigned {} to user", action);
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