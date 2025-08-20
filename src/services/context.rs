use sea_orm::*;
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

use crate::entities::{
    context_type, project_context, project_context_category,
    prelude::*
};
use crate::graphql::types::{
    CreateContextCategoryInput, UpdateContextCategoryInput,
    ContextFilters, ContextConnection
};

#[derive(Clone)]
pub struct ContextService {
    db: DatabaseConnection,
}

impl ContextService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    // Context Type Management
    pub async fn get_context_types(&self, active_only: bool) -> Result<Vec<context_type::Model>> {
        let mut query = ContextType::find();
        
        if active_only {
            query = query.filter(context_type::Column::IsActive.eq(true));
        }
        
        query.all(&self.db).await.map_err(Into::into)
    }

    pub async fn get_context_type_by_name(&self, name: &str) -> Result<Option<context_type::Model>> {
        ContextType::find()
            .filter(context_type::Column::Name.eq(name))
            .filter(context_type::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    // Context Category Management
    pub async fn create_context_category(
        &self,
        input: CreateContextCategoryInput,
        created_by: Uuid,
    ) -> Result<project_context_category::Model> {
        // Get context type by name
        let context_type = self.get_context_type_by_name(&input.context_type_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Context type '{}' not found", input.context_type_name))?;

        // Check for duplicate category name within project and context type
        let existing = ProjectContextCategory::find()
            .filter(project_context_category::Column::ProjectId.eq(input.project_id))
            .filter(project_context_category::Column::ContextTypeId.eq(context_type.id))
            .filter(project_context_category::Column::Name.eq(&input.name))
            .filter(project_context_category::Column::IsActive.eq(true))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(anyhow::anyhow!(
                "Category '{}' already exists for this project and context type", 
                input.name
            ));
        }

        let category = project_context_category::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(input.project_id),
            context_type_id: Set(context_type.id),
            name: Set(input.name),
            color: Set(input.color.unwrap_or_else(|| "#6366f1".to_string())),
            description: Set(input.description),
            is_active: Set(true),
            created_by: Set(created_by),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        };

        category.insert(&self.db).await.map_err(Into::into)
    }

    pub async fn update_context_category(
        &self,
        input: UpdateContextCategoryInput,
    ) -> Result<project_context_category::Model> {
        let category = ProjectContextCategory::find_by_id(input.category_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category not found"))?;

        let mut category: project_context_category::ActiveModel = category.into();

        if let Some(name) = input.name {
            // Check for duplicate name if changing
            let existing = ProjectContextCategory::find()
                .filter(project_context_category::Column::ProjectId.eq(*category.project_id.as_ref()))
                .filter(project_context_category::Column::ContextTypeId.eq(*category.context_type_id.as_ref()))
                .filter(project_context_category::Column::Name.eq(&name))
                .filter(project_context_category::Column::Id.ne(input.category_id))
                .filter(project_context_category::Column::IsActive.eq(true))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(anyhow::anyhow!("Category name '{}' already exists", name));
            }

            category.name = Set(name);
        }

        if let Some(color) = input.color {
            category.color = Set(color);
        }

        if let Some(description) = input.description {
            category.description = Set(description);
        }

        if let Some(is_active) = input.is_active {
            category.is_active = Set(is_active);
        }

        category.updated_at = Set(Utc::now().naive_utc());

        category.update(&self.db).await.map_err(Into::into)
    }

    pub async fn delete_context_category(&self, category_id: Uuid) -> Result<()> {
        // Soft delete by marking as inactive
        let category = ProjectContextCategory::find_by_id(category_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category not found"))?;

        let mut category: project_context_category::ActiveModel = category.into();
        category.is_active = Set(false);
        category.updated_at = Set(Utc::now().naive_utc());

        category.update(&self.db).await?;
        Ok(())
    }

    pub async fn get_project_categories(
        &self,
        project_id: Uuid,
        context_type_name: Option<String>,
    ) -> Result<Vec<project_context_category::Model>> {
        let mut query = ProjectContextCategory::find()
            .filter(project_context_category::Column::ProjectId.eq(project_id))
            .filter(project_context_category::Column::IsActive.eq(true));

        if let Some(type_name) = context_type_name {
            query = query
                .join(JoinType::InnerJoin, project_context_category::Relation::ContextType.def())
                .filter(context_type::Column::Name.eq(type_name));
        }

        query
            .order_by_asc(project_context_category::Column::Name)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // Project Context Management
    pub async fn get_project_contexts(
        &self,
        project_id: Uuid,
        filters: Option<ContextFilters>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<ContextConnection> {
        let mut query = ProjectContext::find()
            .filter(project_context::Column::ProjectId.eq(project_id));

        // Apply filters
        if let Some(filters) = filters {
            if let Some(type_name) = filters.context_type_name {
                query = query
                    .join(JoinType::InnerJoin, project_context::Relation::ContextType.def())
                    .filter(context_type::Column::Name.eq(type_name));
            }

            if let Some(category_id) = filters.category_id {
                query = query.filter(project_context::Column::CategoryId.eq(category_id));
            }

            if let Some(is_archived) = filters.is_archived {
                query = query.filter(project_context::Column::IsArchived.eq(is_archived));
            }

            if let Some(created_after) = filters.created_after {
                query = query.filter(project_context::Column::CreatedAt.gte(created_after.naive_utc()));
            }

            if let Some(created_before) = filters.created_before {
                query = query.filter(project_context::Column::CreatedAt.lte(created_before.naive_utc()));
            }

            if let Some(tags) = filters.tags {
                if !tags.is_empty() {
                    // PostgreSQL array contains operator
                    let tag_conditions = tags.into_iter()
                        .map(|tag| project_context::Column::Tags.contains(&tag))
                        .collect::<Vec<_>>();
                    
                    // Use OR condition for multiple tags
                    if !tag_conditions.is_empty() {
                        let mut condition = tag_conditions[0].clone();
                        for tag_condition in tag_conditions.into_iter().skip(1) {
                            condition = condition.or(tag_condition);
                        }
                        query = query.filter(condition);
                    }
                }
            }
        }

        // Get total count
        let total_count = query.clone().count(&self.db).await? as u32;

        // Apply pagination
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let contexts = query
            .order_by_desc(project_context::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(ContextConnection {
            edges: contexts.into_iter().map(Into::into).collect(),
            total_count,
        })
    }

    pub async fn get_context_by_id(&self, context_id: Uuid) -> Result<Option<project_context::Model>> {
        ProjectContext::find_by_id(context_id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn archive_context(&self, context_id: Uuid) -> Result<project_context::Model> {
        let context = ProjectContext::find_by_id(context_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        let mut context: project_context::ActiveModel = context.into();
        context.is_archived = Set(true);
        context.updated_at = Set(Utc::now().naive_utc());

        context.update(&self.db).await.map_err(Into::into)
    }

    pub async fn restore_context(&self, context_id: Uuid) -> Result<project_context::Model> {
        let context = ProjectContext::find_by_id(context_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Context not found"))?;

        let mut context: project_context::ActiveModel = context.into();
        context.is_archived = Set(false);
        context.updated_at = Set(Utc::now().naive_utc());

        context.update(&self.db).await.map_err(Into::into)
    }

    // Statistics and Analytics
    pub async fn get_project_context_stats(&self, project_id: Uuid) -> Result<ContextStats> {
        let total = ProjectContext::find()
            .filter(project_context::Column::ProjectId.eq(project_id))
            .count(&self.db)
            .await? as u32;

        let archived = ProjectContext::find()
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(project_context::Column::IsArchived.eq(true))
            .count(&self.db)
            .await? as u32;

        let active = total - archived;

        // Get counts by context type
        let type_counts = ProjectContext::find()
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(project_context::Column::IsArchived.eq(false))
            .join(JoinType::InnerJoin, project_context::Relation::ContextType.def())
            .group_by(context_type::Column::Name)
            .column_as(project_context::Column::Id.count(), "count")
            .column(context_type::Column::Name)
            .into_tuple::<(i64, String)>()
            .all(&self.db)
            .await?;

        let mut by_type = std::collections::HashMap::new();
        for (count, type_name) in type_counts {
            by_type.insert(type_name, count as u32);
        }

        Ok(ContextStats {
            total,
            active,
            archived,
            by_type,
        })
    }
}

// Statistics type
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total: u32,
    pub active: u32,
    pub archived: u32,
    pub by_type: std::collections::HashMap<String, u32>,
}