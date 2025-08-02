pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250726_225951_seed_admin_user;
mod m20250728_121007_add_refresh_tokens_to_user;
mod m20250728_123504_create_invitations_table;
mod m20250728_123616_add_invitation_token_to_user;
mod m20250801_000001_create_rbac_tables;
mod m20250801_000002_seed_rbac_data;
mod m20250801_000003_add_role_to_invitations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250726_225951_seed_admin_user::Migration),
            Box::new(m20250728_121007_add_refresh_tokens_to_user::Migration),
            Box::new(m20250728_123504_create_invitations_table::Migration),
            Box::new(m20250728_123616_add_invitation_token_to_user::Migration),
            Box::new(m20250801_000001_create_rbac_tables::Migration),
            Box::new(m20250801_000002_seed_rbac_data::Migration),
            Box::new(m20250801_000003_add_role_to_invitations::Migration),
        ]
    }
}