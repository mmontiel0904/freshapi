use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "context_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    pub schema_version: i32,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project_context_category::Entity")]
    ProjectContextCategories,
    #[sea_orm(has_many = "super::project_context::Entity")]
    ProjectContexts,
}

impl Related<super::project_context_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectContextCategories.def()
    }
}

impl Related<super::project_context::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProjectContexts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}