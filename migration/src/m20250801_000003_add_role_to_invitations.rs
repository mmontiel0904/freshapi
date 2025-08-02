use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add role_id column to invitations table
        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .add_column(uuid_null(Invitation::RoleId))
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint to role
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-invitation-role")
                    .from(Invitation::Table, Invitation::RoleId)
                    .to(Role::Table, Role::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key constraint
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-invitation-role")
                    .table(Invitation::Table)
                    .to_owned(),
            )
            .await?;

        // Drop column
        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .drop_column(Invitation::RoleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    RoleId,
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
}