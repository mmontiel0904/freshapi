use sea_orm_migration::prelude::*;
use sea_orm::{ActiveModelTrait, Set, EntityTrait, ColumnTrait, QueryFilter, ConnectionTrait, DbErr, Value};
use chrono::Utc;
use uuid::Uuid;
use std::env;
use bcrypt::{hash, DEFAULT_COST};

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

        // Seed admin user
        seed_admin_user(db, super_admin_role_id).await?;

        println!("🎉 RBAC seeding completed successfully!");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove admin user
        if let Ok(admin_email) = env::var("ADMIN_EMAIL") {
            let result = freshapi::entities::user::Entity::delete_many()
                .filter(freshapi::entities::user::Column::Email.eq(&admin_email))
                .exec(db)
                .await?;

            if result.rows_affected > 0 {
                println!("🗑️  Admin user removed: {}", admin_email);
            } else {
                println!("⚠️  Admin user not found: {}", admin_email);
            }
        }

        // Clear role assignments from users
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

async fn seed_admin_user(db: &impl ConnectionTrait, role_id: Uuid) -> Result<(), DbErr> {
    // Only seed admin user if environment variables are set
    let admin_email = match env::var("ADMIN_EMAIL") {
        Ok(email) => email,
        Err(_) => {
            println!("⚠️  ADMIN_EMAIL not set, skipping admin user creation");
            return Ok(());
        }
    };

    let admin_password = match env::var("ADMIN_PASSWORD") {
        Ok(password) => password,
        Err(_) => {
            println!("⚠️  ADMIN_PASSWORD not set, skipping admin user creation");
            return Ok(());
        }
    };

    let admin_first_name = env::var("ADMIN_FIRST_NAME").unwrap_or_else(|_| "Admin".to_string());
    let admin_last_name = env::var("ADMIN_LAST_NAME").unwrap_or_else(|_| "User".to_string());

    println!("🌱 Seeding admin user: {}", admin_email);

    // Check if admin user already exists
    let existing_user = freshapi::entities::user::Entity::find()
        .filter(freshapi::entities::user::Column::Email.eq(&admin_email))
        .one(db)
        .await?;

    if existing_user.is_some() {
        println!("✅ Admin user already exists, skipping creation");
        return Ok(());
    }

    // Hash the password
    let password_hash = hash(&admin_password, DEFAULT_COST)
        .map_err(|e| DbErr::Custom(format!("Failed to hash password: {}", e)))?;

    // Create admin user
    let admin_user = freshapi::entities::user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(admin_email.clone()),
        password_hash: Set(password_hash),
        first_name: Set(Some(admin_first_name)),
        last_name: Set(Some(admin_last_name)),
        is_email_verified: Set(true), // Admin user is pre-verified
        email_verification_token: Set(None),
        email_verification_expires_at: Set(None),
        password_reset_token: Set(None),
        password_reset_expires_at: Set(None),
        refresh_token: Set(None),
        refresh_token_expires_at: Set(None),
        invitation_token: Set(None), // Admin doesn't need invitation
        role_id: Set(Some(role_id)), // Assign role directly
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    admin_user.insert(db).await?;
    println!("✅ Admin user created successfully with super_admin role: {}", admin_email);

    Ok(())
}
