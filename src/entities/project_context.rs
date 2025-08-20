use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_contexts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub project_id: Uuid,
    pub context_type_id: Uuid,
    #[sea_orm(nullable)]
    pub category_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub is_archived: bool,
    #[sea_orm(nullable)]
    pub created_by: Option<Uuid>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::context_type::Entity",
        from = "Column::ContextTypeId",
        to = "super::context_type::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ContextType,
    #[sea_orm(
        belongs_to = "super::project_context_category::Entity",
        from = "Column::CategoryId",
        to = "super::project_context_category::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Category,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CreatedBy,
    #[sea_orm(has_one = "super::email_context::Entity")]
    EmailContext,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::context_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ContextType.def()
    }
}

impl Related<super::project_context_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::email_context::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailContext.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}