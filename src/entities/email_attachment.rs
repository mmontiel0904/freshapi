use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_attachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub email_context_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    #[sea_orm(nullable)]
    pub file_size: Option<i64>,
    pub content_type: Option<String>,
    pub file_hash: Option<String>,
    pub storage_path: String,
    pub extracted_text: Option<String>,
    pub is_processed: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::email_context::Entity",
        from = "Column::EmailContextId",
        to = "super::email_context::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    EmailContext,
}

impl Related<super::email_context::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailContext.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}