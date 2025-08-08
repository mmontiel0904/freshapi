pub use sea_orm_migration::prelude::*;

mod rbac_helpers;
mod m20220101_000001_create_table;

mod m20250728_121007_add_refresh_tokens_to_user;
mod m20250728_123504_create_invitations_table;
mod m20250728_123616_add_invitation_token_to_user;
mod m20250801_000001_create_rbac_tables;
mod m20250801_000002_seed_rbac_data;
mod m20250801_000003_add_role_to_invitations;
mod m20250104_000001_create_projects_and_tasks;
mod m20250104_000002_seed_task_permissions;
mod m20250808_000001_add_task_recurrence_and_activities;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            
            Box::new(m20250728_121007_add_refresh_tokens_to_user::Migration),
            Box::new(m20250728_123504_create_invitations_table::Migration),
            Box::new(m20250728_123616_add_invitation_token_to_user::Migration),
            Box::new(m20250801_000001_create_rbac_tables::Migration),
            Box::new(m20250801_000002_seed_rbac_data::Migration),
            Box::new(m20250801_000003_add_role_to_invitations::Migration),
            Box::new(m20250104_000001_create_projects_and_tasks::Migration),
            Box::new(m20250104_000002_seed_task_permissions::Migration),
            Box::new(m20250808_000001_add_task_recurrence_and_activities::Migration),
        ]
    }
}