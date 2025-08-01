use sea_orm_migration::{prelude::*, schema::*};
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter};
use chrono::Utc;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        println!("🌱 Seeding RBAC data...");

        // Create default resource (FreshAPI)
        let freshapi_resource_id = Uuid::new_v4();
        let freshapi_resource = freshapi::entities::resource::ActiveModel {
            id: Set(freshapi_resource_id),
            name: Set("freshapi".to_string()),
            description: Set(Some("FreshAPI Core Application".to_string())),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        freshapi_resource.insert(db).await?;
        println!("✅ Created resource: freshapi");

        // Create default roles
        let super_admin_role_id = Uuid::new_v4();
        let super_admin_role = freshapi::entities::role::ActiveModel {
            id: Set(super_admin_role_id),
            name: Set("super_admin".to_string()),
            description: Set(Some("Super Administrator with full system access".to_string())),
            level: Set(100),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        super_admin_role.insert(db).await?;
        println!("✅ Created role: super_admin (level 100)");

        let admin_role_id = Uuid::new_v4();
        let admin_role = freshapi::entities::role::ActiveModel {
            id: Set(admin_role_id),
            name: Set("admin".to_string()),
            description: Set(Some("Administrator with user management access".to_string())),
            level: Set(50),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        admin_role.insert(db).await?;
        println!("✅ Created role: admin (level 50)");

        let user_role_id = Uuid::new_v4();
        let user_role = freshapi::entities::role::ActiveModel {
            id: Set(user_role_id),
            name: Set("user".to_string()),
            description: Set(Some("Regular user with basic access".to_string())),
            level: Set(10),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        user_role.insert(db).await?;
        println!("✅ Created role: user (level 10)");

        // Create default permissions for FreshAPI resource
        let permissions = vec![
            ("read", "Read access to basic data"),
            ("write", "Write access to own data"),
            ("admin", "Administrative access"),
            ("user_management", "Manage users and roles"),
            ("invite_users", "Create user invitations"),
            ("system_admin", "Full system administration"),
        ];

        let mut permission_ids = Vec::new();
        for (action, description) in permissions {
            let permission_id = Uuid::new_v4();
            let permission = freshapi::entities::permission::ActiveModel {
                id: Set(permission_id),
                action: Set(action.to_string()),
                resource_id: Set(freshapi_resource_id),
                description: Set(Some(description.to_string())),
                is_active: Set(true),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            permission.insert(db).await?;
            permission_ids.push((action, permission_id));
            println!("✅ Created permission: {} for freshapi", action);
        }

        // Assign permissions to roles
        
        // Super Admin gets all permissions
        for (_, permission_id) in &permission_ids {
            let role_permission = freshapi::entities::role_permission::ActiveModel {
                id: Set(Uuid::new_v4()),
                role_id: Set(super_admin_role_id),
                permission_id: Set(*permission_id),
                created_at: Set(Utc::now().into()),
            };
            role_permission.insert(db).await?;
        }
        println!("✅ Assigned all permissions to super_admin role");

        // Admin gets user management and admin permissions
        let admin_permissions = ["read", "write", "admin", "user_management", "invite_users"];
        for (action, permission_id) in &permission_ids {
            if admin_permissions.contains(&action.as_ref()) {
                let role_permission = freshapi::entities::role_permission::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    role_id: Set(admin_role_id),
                    permission_id: Set(*permission_id),
                    created_at: Set(Utc::now().into()),
                };
                role_permission.insert(db).await?;
            }
        }
        println!("✅ Assigned admin permissions to admin role");

        // User gets basic permissions
        let user_permissions = ["read", "write"];
        for (action, permission_id) in &permission_ids {
            if user_permissions.contains(&action.as_ref()) {
                let role_permission = freshapi::entities::role_permission::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    role_id: Set(user_role_id),
                    permission_id: Set(*permission_id),
                    created_at: Set(Utc::now().into()),
                };
                role_permission.insert(db).await?;
            }
        }
        println!("✅ Assigned basic permissions to user role");

        // Set default role for existing admin user (if exists)
        if let Ok(admin_email) = std::env::var("ADMIN_EMAIL") {
            if let Some(admin_user) = freshapi::entities::user::Entity::find()
                .filter(freshapi::entities::user::Column::Email.eq(&admin_email))
                .one(db)
                .await?
            {
                let mut admin_user_active: freshapi::entities::user::ActiveModel = admin_user.into();
                admin_user_active.role_id = Set(Some(super_admin_role_id));
                admin_user_active.updated_at = Set(Utc::now().into());
                admin_user_active.update(db).await?;
                println!("✅ Assigned super_admin role to existing admin user: {}", admin_email);
            }
        }

        println!("🎉 RBAC seeding completed successfully!");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Clear role assignments from users
        use sea_orm::{UpdateMany, Set as SeaSet};
        let _result = freshapi::entities::user::Entity::update_many()
            .col_expr(freshapi::entities::user::Column::RoleId, Expr::value(Value::from(Option::<Uuid>::None)))
            .exec(db)
            .await?;

        // Delete role permissions
        freshapi::entities::role_permission::Entity::delete_many().exec(db).await?;
        
        // Delete permissions
        freshapi::entities::permission::Entity::delete_many().exec(db).await?;
        
        // Delete roles
        freshapi::entities::role::Entity::delete_many().exec(db).await?;
        
        // Delete resources
        freshapi::entities::resource::Entity::delete_many().exec(db).await?;

        println!("🗑️  RBAC data removed");

        Ok(())
    }
}