use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use crate::graphql::types::AccountingProcess;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_contexts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    
    // Email Headers
    pub from_email: String,
    pub from_name: Option<String>,
    pub to_emails: Vec<String>,
    pub cc_emails: Option<Vec<String>>,
    pub bcc_emails: Option<Vec<String>>,
    pub reply_to: Option<String>,
    
    // Content
    pub subject: String,
    pub message_preview: Option<String>,
    pub full_message: String,
    pub message_html: Option<String>,
    
    // Business Context
    pub accounting_process: AccountingProcess,
    pub ai_summary: Option<String>,
    pub confidence_score: Option<Decimal>,
    pub extracted_entities: Option<serde_json::Value>,
    
    // Email Metadata
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub message_date: Option<DateTime>,
    pub received_date: DateTime,
    
    // Attachments
    pub has_attachments: bool,
    pub attachment_count: i32,
    
    // Processing Status
    pub processing_status: String,
    pub processing_notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project_context::Entity",
        from = "Column::Id",
        to = "super::project_context::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ProjectContext,
    #[sea_orm(has_many = "super::email_attachment::Entity")]
    EmailAttachments,
}

impl Related<super::project_context::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectContext.def()
    }
}

impl Related<super::email_attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailAttachments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}