use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Make created_by nullable in project_context_category table
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectContextCategory::Table)
                    .modify_column(ColumnDef::new(ProjectContextCategory::CreatedBy).uuid().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert created_by to not null in project_context_category table
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectContextCategory::Table)
                    .modify_column(ColumnDef::new(ProjectContextCategory::CreatedBy).uuid().not_null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectContextCategory {
    Table,
    CreatedBy,
}