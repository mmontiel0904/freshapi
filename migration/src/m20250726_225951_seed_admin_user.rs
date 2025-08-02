use sea_orm_migration::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use uuid::Uuid;
use std::env;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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

        // Get database connection
        let db = manager.get_connection();

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
            role_id: Set(None), // Will be set by RBAC seeding
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        admin_user.insert(db).await?;
        println!("✅ Admin user created successfully: {}", admin_email);

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Get admin email from environment
        let admin_email = match env::var("ADMIN_EMAIL") {
            Ok(email) => email,
            Err(_) => {
                println!("⚠️  ADMIN_EMAIL not set, cannot remove admin user");
                return Ok(());
            }
        };

        let db = manager.get_connection();

        // Remove admin user
        let result = freshapi::entities::user::Entity::delete_many()
            .filter(freshapi::entities::user::Column::Email.eq(&admin_email))
            .exec(db)
            .await?;

        if result.rows_affected > 0 {
            println!("🗑️  Admin user removed: {}", admin_email);
        } else {
            println!("⚠️  Admin user not found: {}", admin_email);
        }

        Ok(())
    }
}
