use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_uuid(User::Id))
                    .col(string_uniq(User::Email))
                    .col(string(User::PasswordHash))
                    .col(string_null(User::FirstName))
                    .col(string_null(User::LastName))
                    .col(boolean(User::IsEmailVerified).default(false))
                    .col(string_null(User::EmailVerificationToken))
                    .col(timestamp_with_time_zone_null(User::EmailVerificationExpiresAt))
                    .col(string_null(User::PasswordResetToken))
                    .col(timestamp_with_time_zone_null(User::PasswordResetExpiresAt))
                    .col(timestamp_with_time_zone(User::CreatedAt))
                    .col(timestamp_with_time_zone(User::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    PasswordHash,
    FirstName,
    LastName,
    IsEmailVerified,
    EmailVerificationToken,
    EmailVerificationExpiresAt,
    PasswordResetToken,
    PasswordResetExpiresAt,
    CreatedAt,
    UpdatedAt,
}
