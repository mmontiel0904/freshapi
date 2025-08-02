use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::{prelude::*, user};
use crate::auth::JwtService;

#[derive(Clone)]
pub struct UserService {
    db: DatabaseConnection,
    jwt_service: JwtService,
}

impl UserService {
    pub fn new(db: DatabaseConnection, jwt_service: JwtService) -> Self {
        Self { db, jwt_service }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn register_user_with_invitation(
        &self,
        email: &str,
        password: &str,
        first_name: Option<String>,
        last_name: Option<String>,
        invitation_token: &str,
    ) -> Result<(user::Model, String, String), Box<dyn std::error::Error>> {
        use crate::entities::{prelude::*, invitation};
        use sea_orm::TransactionTrait;
        
        // Use transaction to ensure atomicity - OPTIMIZED
        let tx = self.db.begin().await?;
        
        // Check if user already exists
        if let Some(_) = User::find()
            .filter(user::Column::Email.eq(email))
            .one(&tx)
            .await?
        {
            return Err("User with this email already exists".into());
        }

        // Get invitation to extract role_id if present
        let invitation = Invitation::find()
            .filter(invitation::Column::Token.eq(invitation_token))
            .one(&tx)
            .await?
            .ok_or("Invalid invitation token")?;

        // Validate invitation hasn't been used and hasn't expired
        if invitation.is_used {
            return Err("Invitation has already been used".into());
        }
        
        if invitation.expires_at < Utc::now() {
            return Err("Invitation has expired".into());
        }

        // Hash password
        let password_hash = hash(password, DEFAULT_COST)?;

        // Create user (skip email verification since invitation validates email)
        let new_user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash),
            first_name: Set(first_name),
            last_name: Set(last_name),
            is_email_verified: Set(true), // Auto-verify since invitation was to this email
            email_verification_token: Set(None),
            email_verification_expires_at: Set(None),
            password_reset_token: Set(None),
            password_reset_expires_at: Set(None),
            refresh_token: Set(None),
            refresh_token_expires_at: Set(None),
            invitation_token: Set(Some(invitation_token.to_string())),
            role_id: Set(invitation.role_id), // Assign role from invitation
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let user = new_user.insert(&tx).await?;

        // Mark invitation as used - within same transaction
        let mut invitation_active: invitation::ActiveModel = invitation.into();
        invitation_active.is_used = Set(true);
        invitation_active.used_at = Set(Some(Utc::now().into()));
        invitation_active.updated_at = Set(Utc::now().into());
        invitation_active.update(&tx).await?;

        // Generate tokens for immediate login
        let access_token = self.jwt_service.generate_access_token(user.id, &user.email)?;
        let refresh_token = self.jwt_service.generate_refresh_token();
        let refresh_expires = self.jwt_service.get_refresh_token_expiration();

        // Store refresh token
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.refresh_token = Set(Some(refresh_token.clone()));
        user_active.refresh_token_expires_at = Set(Some(refresh_expires.into()));
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&tx).await?;

        // Commit transaction - all operations succeed or fail together
        tx.commit().await?;

        Ok((updated_user, access_token, refresh_token))
    }

    pub async fn register_user(
        &self,
        _email: &str,
        _password: &str,
        _first_name: Option<String>,
        _last_name: Option<String>,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        return Err("Public registration is disabled. Use invitation-based registration.".into());
    }

    pub async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(user::Model, String, String), Box<dyn std::error::Error>> {
        // Find user by email
        let user = User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await?
            .ok_or("Invalid credentials")?;

        // Verify password
        if !verify(password, &user.password_hash)? {
            return Err("Invalid credentials".into());
        }

        // Generate tokens
        let access_token = self.jwt_service.generate_access_token(user.id, &user.email)?;
        let refresh_token = self.jwt_service.generate_refresh_token();
        let refresh_expires = self.jwt_service.get_refresh_token_expiration();

        // Store refresh token in database
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.refresh_token = Set(Some(refresh_token.clone()));
        user_active.refresh_token_expires_at = Set(Some(refresh_expires.into()));
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&self.db).await?;

        Ok((updated_user, access_token, refresh_token))
    }

    pub async fn verify_email(
        &self,
        token: &str,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        let user = User::find()
            .filter(user::Column::EmailVerificationToken.eq(token))
            .one(&self.db)
            .await?
            .ok_or("Invalid verification token")?;

        // Check if token has expired
        if let Some(expires_at) = user.email_verification_expires_at {
            if Utc::now() > expires_at {
                return Err("Verification token has expired".into());
            }
        }

        // Update user
        let mut user_active: user::ActiveModel = user.into();
        user_active.is_email_verified = Set(true);
        user_active.email_verification_token = Set(None);
        user_active.email_verification_expires_at = Set(None);
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&self.db).await?;
        Ok(updated_user)
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(user::Model, String, String), Box<dyn std::error::Error>> {
        // Find user by refresh token
        let user = User::find()
            .filter(user::Column::RefreshToken.eq(refresh_token))
            .one(&self.db)
            .await?
            .ok_or("Invalid refresh token")?;

        // Check if refresh token has expired
        if let Some(expires_at) = user.refresh_token_expires_at {
            if Utc::now() > expires_at {
                return Err("Refresh token has expired".into());
            }
        } else {
            return Err("Invalid refresh token".into());
        }

        // Generate new tokens
        let new_access_token = self.jwt_service.generate_access_token(user.id, &user.email)?;
        let new_refresh_token = self.jwt_service.generate_refresh_token();
        let new_refresh_expires = self.jwt_service.get_refresh_token_expiration();

        // Update refresh token in database
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.refresh_token = Set(Some(new_refresh_token.clone()));
        user_active.refresh_token_expires_at = Set(Some(new_refresh_expires.into()));
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&self.db).await?;

        Ok((updated_user, new_access_token, new_refresh_token))
    }

    pub async fn revoke_refresh_token(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let user = User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        let mut user_active: user::ActiveModel = user.into();
        user_active.refresh_token = Set(None);
        user_active.refresh_token_expires_at = Set(None);
        user_active.updated_at = Set(Utc::now().into());

        user_active.update(&self.db).await?;
        Ok(())
    }

    pub async fn request_password_reset(
        &self,
        email: &str,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        // Find user by email
        let user = User::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        // Generate password reset token
        let reset_token = Uuid::new_v4().to_string();
        let reset_expires_at = Utc::now() + Duration::hours(1); // 1 hour to reset

        // Update user with reset token
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.password_reset_token = Set(Some(reset_token));
        user_active.password_reset_expires_at = Set(Some(reset_expires_at.into()));
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&self.db).await?;
        Ok(updated_user)
    }

    pub async fn reset_password(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        // Find user by reset token
        let user = User::find()
            .filter(user::Column::PasswordResetToken.eq(token))
            .one(&self.db)
            .await?
            .ok_or("Invalid reset token")?;

        // Check if reset token has expired
        if let Some(expires_at) = user.password_reset_expires_at {
            if Utc::now() > expires_at {
                return Err("Reset token has expired".into());
            }
        } else {
            return Err("Invalid reset token".into());
        }

        // Hash new password
        let password_hash = hash(new_password, DEFAULT_COST)?;

        // Update password and clear reset token
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.password_hash = Set(password_hash);
        user_active.password_reset_token = Set(None);
        user_active.password_reset_expires_at = Set(None);
        user_active.updated_at = Set(Utc::now().into());

        // Also revoke all refresh tokens for security
        user_active.refresh_token = Set(None);
        user_active.refresh_token_expires_at = Set(None);

        let updated_user = user_active.update(&self.db).await?;
        Ok(updated_user)
    }

    pub async fn find_user_by_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<user::Model>, Box<dyn std::error::Error>> {
        let user = User::find_by_id(user_id).one(&self.db).await?;
        Ok(user)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        // Find user by ID
        let user = User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        // Verify current password
        if !verify(current_password, &user.password_hash)? {
            return Err("Current password is incorrect".into());
        }

        // Hash new password
        let new_password_hash = hash(new_password, DEFAULT_COST)?;

        // Update password
        let mut user_active: user::ActiveModel = user.clone().into();
        user_active.password_hash = Set(new_password_hash);
        user_active.updated_at = Set(Utc::now().into());

        // Revoke all refresh tokens for security
        user_active.refresh_token = Set(None);
        user_active.refresh_token_expires_at = Set(None);

        let updated_user = user_active.update(&self.db).await?;
        Ok(updated_user)
    }

    pub async fn admin_reset_user_password(
        &self,
        user_id: Uuid,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        // Find user by ID
        let user = User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        // Generate a random password
        let new_password = Uuid::new_v4().to_string();
        let password_hash = hash(&new_password, DEFAULT_COST)?;

        // Update user's password
        let mut user_active: user::ActiveModel = user.into();
        user_active.password_hash = Set(password_hash);
        user_active.updated_at = Set(Utc::now().into());

        let updated_user = user_active.update(&self.db).await?;

        println!("🔄 Admin reset password for user: {} | New password: {}", updated_user.email, new_password);

        Ok(updated_user)
    }

    pub async fn get_user_permissions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        use crate::entities::{permission, resource, role_permission, user_permission};
        use sea_orm::{JoinType, QuerySelect, RelationTrait};

        let mut all_permissions = Vec::new();

        // Get permissions through role
        if let Ok(Some(user)) = User::find_by_id(user_id).one(&self.db).await {
            if let Some(role_id) = user.role_id {
                let role_perms = role_permission::Entity::find()
                    .filter(role_permission::Column::RoleId.eq(role_id))
                    .join(JoinType::InnerJoin, role_permission::Relation::Permission.def())
                    .join(JoinType::InnerJoin, permission::Relation::Resource.def())
                    .column_as(permission::Column::Action, "action")
                    .column_as(resource::Column::Name, "resource_name")
                    .into_tuple::<(String, String)>()
                    .all(&self.db)
                    .await?;

                for (action, resource_name) in role_perms {
                    all_permissions.push(format!("{}:{}", action, resource_name));
                }
            }

            // Get direct user permissions
            let user_perms = user_permission::Entity::find()
                .filter(user_permission::Column::UserId.eq(user_id))
                .join(JoinType::InnerJoin, user_permission::Relation::Permission.def())
                .join(JoinType::InnerJoin, permission::Relation::Resource.def())
                .column_as(permission::Column::Action, "action")
                .column_as(resource::Column::Name, "resource_name")
                .into_tuple::<(String, String)>()
                .all(&self.db)
                .await?;

            for (action, resource_name) in user_perms {
                all_permissions.push(format!("{}:{}", action, resource_name));
            }
        }

        // Deduplicate permissions
        all_permissions.sort();
        all_permissions.dedup();

        Ok(all_permissions)
    }

    pub async fn get_user_role(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::entities::role::Model>, Box<dyn std::error::Error>> {
        let user = User::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        if let Some(role_id) = user.role_id {
            let role = crate::entities::role::Entity::find_by_id(role_id)
                .one(&self.db)
                .await?;
            Ok(role)
        } else {
            Ok(None)
        }
    }
}