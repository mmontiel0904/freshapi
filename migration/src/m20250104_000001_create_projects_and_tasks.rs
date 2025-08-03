use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        println!("🏗 Creating projects and tasks tables...");

        // Create projects table
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::Name).string().not_null())
                    .col(ColumnDef::new(Project::Description).text())
                    .col(
                        ColumnDef::new(Project::OwnerId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Project::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Project::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Project::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_owner")
                            .from(Project::Table, Project::OwnerId)
                            .to(Alias::new("user"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_project_owner_name")
                            .col(Project::OwnerId)
                            .col(Project::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        println!("✅ Created projects table");

        // Create project_members table (many-to-many)
        manager
            .create_table(
                Table::create()
                    .table(ProjectMember::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectMember::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectMember::ProjectId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectMember::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectMember::Role)
                            .string()
                            .not_null()
                            .default(Expr::value("member")),
                    )
                    .col(
                        ColumnDef::new(ProjectMember::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_member_project")
                            .from(ProjectMember::Table, ProjectMember::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_member_user")
                            .from(ProjectMember::Table, ProjectMember::UserId)
                            .to(Alias::new("user"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_project_user_unique")
                            .col(ProjectMember::ProjectId)
                            .col(ProjectMember::UserId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        println!("✅ Created project_members table");

        // Create tasks table
        let mut task_table = Table::create();
        task_table
            .table(Task::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Task::Id)
                    .uuid()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Task::Name).string().not_null())
            .col(ColumnDef::new(Task::Description).text())
            .col(
                ColumnDef::new(Task::ProjectId)
                    .uuid()
                    .not_null(),
            )
            .col(ColumnDef::new(Task::AssigneeId).uuid())
            .col(
                ColumnDef::new(Task::CreatorId)
                    .uuid()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Task::Status)
                    .string()
                    .not_null()
                    .default(Expr::value("todo")),
            )
            .col(
                ColumnDef::new(Task::Priority)
                    .string()
                    .not_null()
                    .default(Expr::value("medium")),
            )
            .col(ColumnDef::new(Task::DueDate).timestamp_with_time_zone())
            .col(
                ColumnDef::new(Task::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(
                ColumnDef::new(Task::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp()),
            );

        manager.create_table(task_table.to_owned()).await?;

        // Add foreign keys
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-task-project")
                    .from(Task::Table, Task::ProjectId)
                    .to(Project::Table, Project::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-task-assignee")
                    .from(Task::Table, Task::AssigneeId)
                    .to(Alias::new("user"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-task-creator")
                    .from(Task::Table, Task::CreatorId)
                    .to(Alias::new("user"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_task_project")
                    .table(Task::Table)
                    .col(Task::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_assignee")
                    .table(Task::Table)
                    .col(Task::AssigneeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_status")
                    .table(Task::Table)
                    .col(Task::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_due_date")
                    .table(Task::Table)
                    .col(Task::DueDate)
                    .to_owned(),
            )
            .await?;

        println!("✅ Created tasks table");
        println!("🎉 Projects and tasks tables created successfully!");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        println!("🗑 Dropping projects and tasks tables...");

        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectMember::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await?;

        println!("✅ Dropped all project and task tables");

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    Name,
    Description,
    OwnerId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProjectMember {
    Table,
    Id,
    ProjectId,
    UserId,
    Role,
    JoinedAt,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    Name,
    Description,
    ProjectId,
    AssigneeId,
    CreatorId,
    Status,
    Priority,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

