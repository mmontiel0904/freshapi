use sea_orm_migration::prelude::*;
use sea_orm::{ConnectionTrait, EntityTrait, ColumnTrait, QueryFilter, DbErr, ActiveModelTrait, Set};
use chrono::Utc;
use uuid::Uuid;

/// Assigns all permissions for a resource to super_admin and admin roles
/// This ensures that when new resources are added, admin users automatically get access
pub async fn assign_resource_permissions_to_admin_roles(
    db: &impl ConnectionTrait,
    resource_id: Uuid,
    admin_permission_actions: &[&str], // Specific permissions for admin role (if empty, gets all)
) -> Result<(), DbErr> {
    // Get all permissions for this resource
    let permissions = freshapi::entities::permission::Entity::find()
        .filter(freshapi::entities::permission::Column::ResourceId.eq(resource_id))
        .all(db)
        .await?;

    // Get super_admin and admin roles
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

    // Assign ALL permissions to super_admin
    for permission in &permissions {
        // Check if assignment already exists
        let existing = freshapi::entities::role_permission::Entity::find()
            .filter(freshapi::entities::role_permission::Column::RoleId.eq(super_admin_role.id))
            .filter(freshapi::entities::role_permission::Column::PermissionId.eq(permission.id))
            .one(db)
            .await?;

        if existing.is_none() {
            let role_permission = freshapi::entities::role_permission::ActiveModel {
                id: Set(Uuid::new_v4()),
                role_id: Set(super_admin_role.id),
                permission_id: Set(permission.id),
                created_at: Set(Utc::now().into()),
            };
            role_permission.insert(db).await?;
            println!("✅ Assigned {} to super_admin", permission.action);
        }
    }

    // Assign specified permissions to admin (or all if none specified)
    for permission in &permissions {
        let should_assign = if admin_permission_actions.is_empty() {
            true // Assign all permissions if no specific list provided
        } else {
            admin_permission_actions.contains(&permission.action.as_str())
        };

        if should_assign {
            // Check if assignment already exists
            let existing = freshapi::entities::role_permission::Entity::find()
                .filter(freshapi::entities::role_permission::Column::RoleId.eq(admin_role.id))
                .filter(freshapi::entities::role_permission::Column::PermissionId.eq(permission.id))
                .one(db)
                .await?;

            if existing.is_none() {
                let role_permission = freshapi::entities::role_permission::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    role_id: Set(admin_role.id),
                    permission_id: Set(permission.id),
                    created_at: Set(Utc::now().into()),
                };
                role_permission.insert(db).await?;
                println!("✅ Assigned {} to admin", permission.action);
            }
        }
    }

    Ok(())
}

/// Creates a resource and its permissions, then assigns them to admin roles
pub async fn create_resource_with_admin_permissions(
    db: &impl ConnectionTrait,
    resource_name: &str,
    resource_description: &str,
    permissions: &[(&str, &str)], // (action, description) pairs
    admin_permission_actions: &[&str], // Which permissions admin role should get
) -> Result<Uuid, DbErr> {
    // Create resource
    let resource_id = Uuid::new_v4();
    let resource = freshapi::entities::resource::ActiveModel {
        id: Set(resource_id),
        name: Set(String::from(resource_name)),
        description: Set(Some(String::from(resource_description))),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };
    resource.insert(db).await?;
    println!("✅ Created resource: {}", resource_name);

    // Create permissions
    for (action, description) in permissions {
        let permission_id = Uuid::new_v4();
        let permission = freshapi::entities::permission::ActiveModel {
            id: Set(permission_id),
            action: Set(String::from(*action)),
            resource_id: Set(resource_id),
            description: Set(Some(String::from(*description))),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        permission.insert(db).await?;
        println!("✅ Created permission: {}", action);
    }

    // Assign permissions to admin roles
    assign_resource_permissions_to_admin_roles(db, resource_id, admin_permission_actions).await?;

    Ok(resource_id)
}