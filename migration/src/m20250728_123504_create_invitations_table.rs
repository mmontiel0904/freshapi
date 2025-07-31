use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invitation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Invitation::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(Invitation::Email)
                            .string()
                            .not_null()
                            .unique_key()
                    )
                    .col(
                        ColumnDef::new(Invitation::InviterUserId)
                            .uuid()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Invitation::Token)
                            .string()
                            .not_null()
                            .unique_key()
                    )
                    .col(
                        ColumnDef::new(Invitation::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Invitation::IsUsed)
                            .boolean()
                            .not_null()
                            .default(false)
                    )
                    .col(
                        ColumnDef::new(Invitation::UsedAt)
                            .timestamp_with_time_zone()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Invitation::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Invitation::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invitation_inviter_user")
                            .from(Invitation::Table, Invitation::InviterUserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invitation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    Id,
    Email,
    InviterUserId,
    Token,
    ExpiresAt,
    IsUsed,
    UsedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
