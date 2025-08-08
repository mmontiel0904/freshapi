use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        println!("🔄 Creating task recurrence and activity system...");

        // Create PostgreSQL enum types for type safety
        manager
            .create_type(
                Type::create()
                    .as_enum(RecurrenceType::Table)
                    .values([
                        RecurrenceType::None,
                        RecurrenceType::Daily,
                        RecurrenceType::Weekdays,
                        RecurrenceType::Weekly,
                        RecurrenceType::Monthly,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TaskStatus::Table)
                    .values([
                        TaskStatus::Todo,
                        TaskStatus::InProgress,
                        TaskStatus::Completed,
                        TaskStatus::Cancelled,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TaskPriority::Table)
                    .values([
                        TaskPriority::Low,
                        TaskPriority::Medium,
                        TaskPriority::High,
                        TaskPriority::Urgent,
                    ])
                    .to_owned(),
            )
            .await?;

        println!("✅ Created PostgreSQL enum types");

        // Add recurrence fields to task table
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .add_column(
                        ColumnDef::new(Task::RecurrenceType)
                            .custom(RecurrenceType::Table)
                            .default(Expr::value("none"))
                            .not_null(),
                    )
                    .add_column(ColumnDef::new(Task::RecurrenceDay).integer())
                    .add_column(
                        ColumnDef::new(Task::IsRecurring)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .add_column(
                        ColumnDef::new(Task::ParentTaskId)
                            .uuid()
                    )
                    .add_column(
                        ColumnDef::new(Task::NextDueDate)
                            .timestamp_with_time_zone()
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for parent_task_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-task-parent")
                    .from(Task::Table, Task::ParentTaskId)
                    .to(Task::Table, Task::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        // Add indexes for recurrence queries
        manager
            .create_index(
                Index::create()
                    .name("idx_task_recurrence")
                    .table(Task::Table)
                    .col(Task::IsRecurring)
                    .col(Task::NextDueDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_parent")
                    .table(Task::Table)
                    .col(Task::ParentTaskId)
                    .to_owned(),
            )
            .await?;

        println!("✅ Added recurrence fields to task table");

        // Update existing task status and priority columns to use enums
        // First drop default constraints, then alter column types, then add new defaults
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE task ALTER COLUMN status DROP DEFAULT")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE task ALTER COLUMN priority DROP DEFAULT")
            .await?;

        // Use ALTER COLUMN with USING clause to handle type conversion
        let alter_status_sql = r#"
            ALTER TABLE task 
            ALTER COLUMN status TYPE task_status 
            USING (
                CASE 
                    WHEN LOWER(REPLACE(status, ' ', '_')) = 'todo' THEN 'todo'::task_status
                    WHEN LOWER(REPLACE(status, ' ', '_')) = 'in_progress' THEN 'in_progress'::task_status
                    WHEN LOWER(REPLACE(status, ' ', '_')) = 'completed' THEN 'completed'::task_status
                    WHEN LOWER(REPLACE(status, ' ', '_')) = 'cancelled' THEN 'cancelled'::task_status
                    ELSE 'todo'::task_status
                END
            )
        "#;
        manager
            .get_connection()
            .execute_unprepared(alter_status_sql)
            .await?;

        let alter_priority_sql = r#"
            ALTER TABLE task 
            ALTER COLUMN priority TYPE task_priority 
            USING (
                CASE 
                    WHEN LOWER(priority) = 'low' THEN 'low'::task_priority
                    WHEN LOWER(priority) = 'medium' THEN 'medium'::task_priority
                    WHEN LOWER(priority) = 'high' THEN 'high'::task_priority
                    WHEN LOWER(priority) = 'urgent' THEN 'urgent'::task_priority
                    ELSE 'medium'::task_priority
                END
            )
        "#;
        manager
            .get_connection()
            .execute_unprepared(alter_priority_sql)
            .await?;

        // Add back default values as enum types
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE task ALTER COLUMN status SET DEFAULT 'todo'::task_status")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE task ALTER COLUMN priority SET DEFAULT 'medium'::task_priority")
            .await?;

        println!("✅ Updated task status and priority to use enum types");

        // Create generic activities table
        manager
            .create_table(
                Table::create()
                    .table(Activity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Activity::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Activity::EntityType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Activity::EntityId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Activity::ActorId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Activity::ActionType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Activity::Description).text())
                    .col(ColumnDef::new(Activity::Metadata).json_binary())
                    .col(ColumnDef::new(Activity::Changes).json_binary())
                    .col(
                        ColumnDef::new(Activity::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key for actor_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-activity-actor")
                    .from(Activity::Table, Activity::ActorId)
                    .to(Alias::new("user"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add indexes for efficient activity queries
        manager
            .create_index(
                Index::create()
                    .name("idx_activity_entity")
                    .table(Activity::Table)
                    .col(Activity::EntityType)
                    .col(Activity::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_actor")
                    .table(Activity::Table)
                    .col(Activity::ActorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_created_at")
                    .table(Activity::Table)
                    .col((Activity::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_entity_created_at")
                    .table(Activity::Table)
                    .col(Activity::EntityType)
                    .col(Activity::EntityId)
                    .col((Activity::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        println!("✅ Created generic activities table");

        // Create activity comments table for rich comment support
        manager
            .create_table(
                Table::create()
                    .table(ActivityComment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ActivityComment::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ActivityComment::ActivityId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ActivityComment::Content)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ActivityComment::Mentions).json_binary())
                    .col(ColumnDef::new(ActivityComment::Attachments).json_binary())
                    .col(
                        ColumnDef::new(ActivityComment::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key for activity_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-activity-comment")
                    .from(ActivityComment::Table, ActivityComment::ActivityId)
                    .to(Activity::Table, Activity::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_comment_activity")
                    .table(ActivityComment::Table)
                    .col(ActivityComment::ActivityId)
                    .to_owned(),
            )
            .await?;

        println!("✅ Created activity comments table");
        println!("🎉 Task recurrence and activity system migration completed successfully!");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        println!("🗑 Dropping task recurrence and activity system...");

        // Drop activity comments table
        manager
            .drop_table(Table::drop().table(ActivityComment::Table).to_owned())
            .await?;

        // Drop activities table
        manager
            .drop_table(Table::drop().table(Activity::Table).to_owned())
            .await?;

        // Remove recurrence fields from task table
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .drop_column(Task::RecurrenceType)
                    .drop_column(Task::RecurrenceDay)
                    .drop_column(Task::IsRecurring)
                    .drop_column(Task::ParentTaskId)
                    .drop_column(Task::NextDueDate)
                    .to_owned(),
            )
            .await?;

        // Revert task columns to string types
        manager
            .alter_table(
                Table::alter()
                    .table(Task::Table)
                    .modify_column(
                        ColumnDef::new(Task::Status)
                            .string()
                            .default(Expr::value("todo"))
                            .not_null(),
                    )
                    .modify_column(
                        ColumnDef::new(Task::Priority)
                            .string()
                            .default(Expr::value("medium"))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Drop enum types
        manager
            .drop_type(Type::drop().name(TaskPriority::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(TaskStatus::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(RecurrenceType::Table).to_owned())
            .await?;

        println!("✅ Dropped all task recurrence and activity tables");

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    Status,
    Priority,
    RecurrenceType,
    RecurrenceDay,
    IsRecurring,
    ParentTaskId,
    NextDueDate,
}

#[derive(DeriveIden)]
enum RecurrenceType {
    Table,
    None,
    Daily,
    Weekdays,
    Weekly,
    Monthly,
}

#[derive(DeriveIden)]
enum TaskStatus {
    Table,
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(DeriveIden)]
enum TaskPriority {
    Table,
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(DeriveIden)]
enum Activity {
    Table,
    Id,
    EntityType,
    EntityId,
    ActorId,
    ActionType,
    Description,
    Metadata,
    Changes,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ActivityComment {
    Table,
    Id,
    ActivityId,
    Content,
    Mentions,
    Attachments,
    CreatedAt,
}