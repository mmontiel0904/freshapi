use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_context_categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub project_id: Uuid,
    pub context_type_id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_by: Uuid,
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
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CreatedBy,
    #[sea_orm(has_many = "super::project_context::Entity")]
    ProjectContexts,
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

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl Related<super::project_context::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectContexts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}