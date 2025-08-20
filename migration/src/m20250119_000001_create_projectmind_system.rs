use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create accounting_process_enum
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("accounting_process_enum"))
                    .values([
                        Alias::new("AP"),
                        Alias::new("AR"),
                        Alias::new("BR"),
                        Alias::new("Reporting"),
                        Alias::new("General"),
                        Alias::new("Tax"),
                        Alias::new("Payroll"),
                        Alias::new("Audit"),
                    ])
                    .to_owned(),
            )
            .await?;

        // Create context_types table
        manager
            .create_table(
                Table::create()
                    .table(ContextType::Table)
                    .if_not_exists()
                    .col(pk_uuid(ContextType::Id))
                    .col(string_len(ContextType::Name, 50).unique_key())
                    .col(text_null(ContextType::Description))
                    .col(integer(ContextType::SchemaVersion).default(1))
                    .col(boolean(ContextType::IsActive).default(true))
                    .col(timestamp_with_time_zone(ContextType::CreatedAt))
                    .col(timestamp_with_time_zone(ContextType::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // Create project_context_categories table
        manager
            .create_table(
                Table::create()
                    .table(ProjectContextCategory::Table)
                    .if_not_exists()
                    .col(pk_uuid(ProjectContextCategory::Id))
                    .col(uuid(ProjectContextCategory::ProjectId))
                    .col(uuid(ProjectContextCategory::ContextTypeId))
                    .col(string_len(ProjectContextCategory::Name, 100))
                    .col(string_len(ProjectContextCategory::Color, 7).default("#6366f1"))
                    .col(text_null(ProjectContextCategory::Description))
                    .col(boolean(ProjectContextCategory::IsActive).default(true))
                    .col(uuid(ProjectContextCategory::CreatedBy))
                    .col(timestamp_with_time_zone(ProjectContextCategory::CreatedAt))
                    .col(timestamp_with_time_zone(ProjectContextCategory::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_category_project")
                            .from(ProjectContextCategory::Table, ProjectContextCategory::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_category_context_type")
                            .from(ProjectContextCategory::Table, ProjectContextCategory::ContextTypeId)
                            .to(ContextType::Table, ContextType::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_category_created_by")
                            .from(ProjectContextCategory::Table, ProjectContextCategory::CreatedBy)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint for project_context_categories
        manager
            .create_index(
                Index::create()
                    .name("idx_project_context_categories_unique")
                    .table(ProjectContextCategory::Table)
                    .col(ProjectContextCategory::ProjectId)
                    .col(ProjectContextCategory::ContextTypeId)
                    .col(ProjectContextCategory::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create project_contexts table
        manager
            .create_table(
                Table::create()
                    .table(ProjectContext::Table)
                    .if_not_exists()
                    .col(pk_uuid(ProjectContext::Id))
                    .col(uuid(ProjectContext::ProjectId))
                    .col(uuid(ProjectContext::ContextTypeId))
                    .col(uuid_null(ProjectContext::CategoryId))
                    .col(string_len(ProjectContext::Title, 255))
                    .col(text_null(ProjectContext::Description))
                    .col(ColumnDef::new(ProjectContext::Tags).array(ColumnType::Text).default(Expr::val("{}")).not_null())
                    .col(json_binary_null(ProjectContext::Metadata))
                    .col(boolean(ProjectContext::IsArchived).default(false))
                    .col(uuid_null(ProjectContext::CreatedBy))
                    .col(timestamp_with_time_zone(ProjectContext::CreatedAt))
                    .col(timestamp_with_time_zone(ProjectContext::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_project")
                            .from(ProjectContext::Table, ProjectContext::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_context_type")
                            .from(ProjectContext::Table, ProjectContext::ContextTypeId)
                            .to(ContextType::Table, ContextType::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_category")
                            .from(ProjectContext::Table, ProjectContext::CategoryId)
                            .to(ProjectContextCategory::Table, ProjectContextCategory::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_context_created_by")
                            .from(ProjectContext::Table, ProjectContext::CreatedBy)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Create email_contexts table
        manager
            .create_table(
                Table::create()
                    .table(EmailContext::Table)
                    .if_not_exists()
                    .col(pk_uuid(EmailContext::Id))
                    .col(string_len(EmailContext::FromEmail, 255))
                    .col(string_len_null(EmailContext::FromName, 255))
                    .col(ColumnDef::new(EmailContext::ToEmails).array(ColumnType::Text).not_null())
                    .col(ColumnDef::new(EmailContext::CcEmails).array(ColumnType::Text).default(Expr::val("{}")))
                    .col(ColumnDef::new(EmailContext::BccEmails).array(ColumnType::Text).default(Expr::val("{}")))
                    .col(string_len_null(EmailContext::ReplyTo, 255))
                    .col(string_len(EmailContext::Subject, 500))
                    .col(text_null(EmailContext::MessagePreview))
                    .col(text(EmailContext::FullMessage))
                    .col(text_null(EmailContext::MessageHtml))
                    .col(
                        ColumnDef::new(EmailContext::AccountingProcess)
                            .custom(Alias::new("accounting_process_enum"))
                            .not_null(),
                    )
                    .col(text_null(EmailContext::AiSummary))
                    .col(decimal_len_null(EmailContext::ConfidenceScore, 5, 4))
                    .col(json_binary_null(EmailContext::ExtractedEntities))
                    .col(string_len_null(EmailContext::MessageId, 255))
                    .col(string_len_null(EmailContext::ThreadId, 255))
                    .col(string_len_null(EmailContext::InReplyTo, 255))
                    .col(timestamp_with_time_zone_null(EmailContext::MessageDate))
                    .col(timestamp_with_time_zone(EmailContext::ReceivedDate))
                    .col(boolean(EmailContext::HasAttachments).default(false))
                    .col(integer(EmailContext::AttachmentCount).default(0))
                    .col(string_len(EmailContext::ProcessingStatus, 20).default("completed"))
                    .col(text_null(EmailContext::ProcessingNotes))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_context_project_context")
                            .from(EmailContext::Table, EmailContext::Id)
                            .to(ProjectContext::Table, ProjectContext::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // TODO: Add check constraints for email_contexts
        // These will be handled at the application level for now

        // Create email_attachments table
        manager
            .create_table(
                Table::create()
                    .table(EmailAttachment::Table)
                    .if_not_exists()
                    .col(pk_uuid(EmailAttachment::Id))
                    .col(uuid(EmailAttachment::EmailContextId))
                    .col(string_len(EmailAttachment::Filename, 255))
                    .col(string_len(EmailAttachment::OriginalFilename, 255))
                    .col(big_integer_null(EmailAttachment::FileSize))
                    .col(string_len_null(EmailAttachment::ContentType, 100))
                    .col(string_len_null(EmailAttachment::FileHash, 64))
                    .col(text(EmailAttachment::StoragePath))
                    .col(text_null(EmailAttachment::ExtractedText))
                    .col(boolean(EmailAttachment::IsProcessed).default(false))
                    .col(timestamp_with_time_zone(EmailAttachment::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_attachment_email_context")
                            .from(EmailAttachment::Table, EmailAttachment::EmailContextId)
                            .to(EmailContext::Table, EmailContext::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create performance indexes
        self.create_indexes(manager).await?;

        // Seed initial data
        self.seed_initial_data(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(EmailAttachment::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EmailContext::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectContext::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectContextCategory::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ContextType::Table).to_owned())
            .await?;

        // Drop enum type
        manager
            .drop_type(Type::drop().name(Alias::new("accounting_process_enum")).to_owned())
            .await?;

        Ok(())
    }
}

impl Migration {
    async fn create_indexes(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        // Core context indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_project_contexts_project_type")
                    .table(ProjectContext::Table)
                    .col(ProjectContext::ProjectId)
                    .col(ProjectContext::ContextTypeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_contexts_category")
                    .table(ProjectContext::Table)
                    .col(ProjectContext::CategoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_contexts_created_at")
                    .table(ProjectContext::Table)
                    .col(ProjectContext::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_contexts_archived")
                    .table(ProjectContext::Table)
                    .col(ProjectContext::IsArchived)
                    .col(ProjectContext::ProjectId)
                    .to_owned(),
            )
            .await?;

        // Email context indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_email_contexts_from")
                    .table(EmailContext::Table)
                    .col(EmailContext::FromEmail)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_contexts_process")
                    .table(EmailContext::Table)
                    .col(EmailContext::AccountingProcess)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_contexts_message_date")
                    .table(EmailContext::Table)
                    .col(EmailContext::MessageDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_email_contexts_status")
                    .table(EmailContext::Table)
                    .col(EmailContext::ProcessingStatus)
                    .to_owned(),
            )
            .await?;

        // Category management indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_context_categories_project_type")
                    .table(ProjectContextCategory::Table)
                    .col(ProjectContextCategory::ProjectId)
                    .col(ProjectContextCategory::ContextTypeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_context_categories_active")
                    .table(ProjectContextCategory::Table)
                    .col(ProjectContextCategory::IsActive)
                    .col(ProjectContextCategory::ProjectId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn seed_initial_data(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        // Insert default context types
        let insert_context_types = Query::insert()
            .into_table(ContextType::Table)
            .columns([
                ContextType::Id,
                ContextType::Name,
                ContextType::Description,
                ContextType::SchemaVersion,
                ContextType::IsActive,
                ContextType::CreatedAt,
                ContextType::UpdatedAt,
            ])
            .values_panic([
                uuid::Uuid::new_v4().into(),
                "email".into(),
                "Email context for communication tracking".into(),
                1.into(),
                true.into(),
                chrono::Utc::now().into(),
                chrono::Utc::now().into(),
            ])
            .values_panic([
                uuid::Uuid::new_v4().into(),
                "document".into(),
                "Document context for file-based information".into(),
                1.into(),
                true.into(),
                chrono::Utc::now().into(),
                chrono::Utc::now().into(),
            ])
            .values_panic([
                uuid::Uuid::new_v4().into(),
                "meeting".into(),
                "Meeting context for discussions and decisions".into(),
                1.into(),
                false.into(), // Not yet implemented
                chrono::Utc::now().into(),
                chrono::Utc::now().into(),
            ])
            .to_owned();

        manager.exec_stmt(insert_context_types).await?;

        Ok(())
    }
}

// Define table enums for schema management
#[derive(DeriveIden)]
enum ContextType {
    Table,
    Id,
    Name,
    Description,
    SchemaVersion,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProjectContextCategory {
    Table,
    Id,
    ProjectId,
    ContextTypeId,
    Name,
    Color,
    Description,
    IsActive,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProjectContext {
    Table,
    Id,
    ProjectId,
    ContextTypeId,
    CategoryId,
    Title,
    Description,
    Tags,
    Metadata,
    IsArchived,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum EmailContext {
    Table,
    Id,
    FromEmail,
    FromName,
    ToEmails,
    CcEmails,
    BccEmails,
    ReplyTo,
    Subject,
    MessagePreview,
    FullMessage,
    MessageHtml,
    AccountingProcess,
    AiSummary,
    ConfidenceScore,
    ExtractedEntities,
    MessageId,
    ThreadId,
    InReplyTo,
    MessageDate,
    ReceivedDate,
    HasAttachments,
    AttachmentCount,
    ProcessingStatus,
    ProcessingNotes,
}

#[derive(DeriveIden)]
enum EmailAttachment {
    Table,
    Id,
    EmailContextId,
    Filename,
    OriginalFilename,
    FileSize,
    ContentType,
    FileHash,
    StoragePath,
    ExtractedText,
    IsProcessed,
    CreatedAt,
}

// Reference existing tables for foreign keys
#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}