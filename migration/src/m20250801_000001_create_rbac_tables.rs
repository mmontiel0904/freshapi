use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create roles table
        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(pk_uuid(Role::Id))
                    .col(string(Role::Name).unique_key())
                    .col(string_null(Role::Description))
                    .col(integer(Role::Level).default(0)) // Higher level = more permissions
                    .col(boolean(Role::IsActive).default(true))
                    .col(timestamp_with_time_zone(Role::CreatedAt))
                    .col(timestamp_with_time_zone(Role::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create resources table (for multi-app support)
        manager
            .create_table(
                Table::create()
                    .table(Resource::Table)
                    .if_not_exists()
                    .col(pk_uuid(Resource::Id))
                    .col(string(Resource::Name).unique_key()) // e.g., "freshapi", "app2"
                    .col(string_null(Resource::Description))
                    .col(boolean(Resource::IsActive).default(true))
                    .col(timestamp_with_time_zone(Resource::CreatedAt))
                    .col(timestamp_with_time_zone(Resource::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create permissions table
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(pk_uuid(Permission::Id))
                    .col(string(Permission::Action)) // e.g., "read", "write", "admin"
                    .col(uuid(Permission::ResourceId))
                    .col(string_null(Permission::Description))
                    .col(boolean(Permission::IsActive).default(true))
                    .col(timestamp_with_time_zone(Permission::CreatedAt))
                    .col(timestamp_with_time_zone(Permission::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-permission-resource")
                            .from(Permission::Table, Permission::ResourceId)
                            .to(Resource::Table, Resource::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-permission-action-resource")
                            .col(Permission::Action)
                            .col(Permission::ResourceId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create role_permissions junction table
        manager
            .create_table(
                Table::create()
                    .table(RolePermission::Table)
                    .if_not_exists()
                    .col(pk_uuid(RolePermission::Id))
                    .col(uuid(RolePermission::RoleId))
                    .col(uuid(RolePermission::PermissionId))
                    .col(timestamp_with_time_zone(RolePermission::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role-permission-role")
                            .from(RolePermission::Table, RolePermission::RoleId)
                            .to(Role::Table, Role::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-role-permission-permission")
                            .from(RolePermission::Table, RolePermission::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-role-permission-unique")
                            .col(RolePermission::RoleId)
                            .col(RolePermission::PermissionId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add role_id to users table
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(uuid_null(User::RoleId))
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint separately
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-user-role")
                    .from(User::Table, User::RoleId)
                    .to(Role::Table, Role::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Create user_permissions table for granular permissions
        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(pk_uuid(UserPermission::Id))
                    .col(uuid(UserPermission::UserId))
                    .col(uuid(UserPermission::PermissionId))
                    .col(boolean(UserPermission::IsGranted).default(true)) // true = grant, false = deny
                    .col(timestamp_with_time_zone(UserPermission::CreatedAt))
                    .col(timestamp_with_time_zone(UserPermission::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-permission-user")
                            .from(UserPermission::Table, UserPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-permission-permission")
                            .from(UserPermission::Table, UserPermission::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-user-permission-unique")
                            .col(UserPermission::UserId)
                            .col(UserPermission::PermissionId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign key constraints
        manager
            .drop_table(Table::drop().table(UserPermission::Table).to_owned())
            .await?;
        
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_foreign_key(Alias::new("fk-user-role"))
                    .drop_column(User::RoleId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Resource::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
    Name,
    Description,
    Level,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Resource {
    Table,
    Id,
    Name,
    Description,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    Action,
    ResourceId,
    Description,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RolePermission {
    Table,
    Id,
    RoleId,
    PermissionId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserPermission {
    Table,
    Id,
    UserId,
    PermissionId,
    IsGranted,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    RoleId,
}