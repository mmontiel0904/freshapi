use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add context_id column to task table
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(ColumnDef::new(Task::ContextId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint to project_context
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_task_context")
                            .from_tbl(Task::Table)
                            .from_col(Task::ContextId)
                            .to_tbl(ProjectContext::Table)
                            .to_col(ProjectContext::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::NoAction)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key constraint
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_foreign_key(Alias::new("fk_task_context"))
                    .to_owned(),
            )
            .await?;

        // Drop context_id column
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::ContextId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    ContextId,
}

#[derive(DeriveIden)]
enum ProjectContext {
    Table,
    Id,
}