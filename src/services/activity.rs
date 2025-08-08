use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, PaginatorTrait, Set,
};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::{prelude::*, activity};

#[derive(Clone)]
pub struct ActivityService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Task,
    Project,
    User,
    Settings,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Task => "task",
            EntityType::Project => "project",
            EntityType::User => "user",
            EntityType::Settings => "settings",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "task" => Some(EntityType::Task),
            "project" => Some(EntityType::Project),
            "user" => Some(EntityType::User),
            "settings" => Some(EntityType::Settings),
            _ => None,
        }
    }
}

impl ActivityService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Log a generic activity for any entity
    pub async fn log_activity(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
        actor_id: Uuid,
        action_type: &str,
        description: Option<String>,
        metadata: Option<Value>,
        changes: Option<Value>,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        let new_activity = activity::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set(entity_type.as_str().to_string()),
            entity_id: Set(entity_id),
            actor_id: Set(actor_id),
            action_type: Set(action_type.to_string()),
            description: Set(description),
            metadata: Set(metadata),
            changes: Set(changes),
            created_at: Set(Utc::now().into()),
        };

        let activity = new_activity.insert(&self.db).await?;
        Ok(activity)
    }

    /// Get activities for any entity type
    pub async fn get_entity_activities(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<activity::Model>, Box<dyn std::error::Error>> {
        let mut query = Activity::find()
            .filter(activity::Column::EntityType.eq(entity_type.as_str()))
            .filter(activity::Column::EntityId.eq(entity_id))
            .order_by_desc(activity::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let activities = query.all(&self.db).await?;
        Ok(activities)
    }

    /// Add a comment to any entity
    pub async fn add_comment(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
        actor_id: Uuid,
        content: &str,
        mentions: Option<Vec<Uuid>>,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        let mentions_json = mentions.map(|m| serde_json::to_value(m).unwrap());
        
        self.log_activity(
            entity_type,
            entity_id,
            actor_id,
            "commented",
            Some(format!("Added comment: {}", content.chars().take(100).collect::<String>())),
            Some(serde_json::json!({
                "comment_content": content,
                "mentions": mentions_json
            })),
            None,
        ).await
    }

    /// Log task status change
    pub async fn log_task_status_change(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
        old_status: &str,
        new_status: &str,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        self.log_activity(
            EntityType::Task,
            task_id,
            actor_id,
            "status_changed",
            Some(format!("Changed status from {} to {}", old_status, new_status)),
            None,
            Some(serde_json::json!({
                "field": "status",
                "old_value": old_status,
                "new_value": new_status
            })),
        ).await
    }

    /// Log task assignment change
    pub async fn log_task_assignment(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
        old_assignee_id: Option<Uuid>,
        new_assignee_id: Option<Uuid>,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        let description = match (old_assignee_id, new_assignee_id) {
            (None, Some(_)) => "Task assigned".to_string(),
            (Some(_), None) => "Task unassigned".to_string(),
            (Some(_), Some(_)) => "Task reassigned".to_string(),
            (None, None) => "Assignment unchanged".to_string(),
        };

        self.log_activity(
            EntityType::Task,
            task_id,
            actor_id,
            "assignment_changed",
            Some(description),
            None,
            Some(serde_json::json!({
                "field": "assignee_id",
                "old_value": old_assignee_id,
                "new_value": new_assignee_id
            })),
        ).await
    }

    /// Log task creation
    pub async fn log_task_creation(
        &self,
        task_id: Uuid,
        creator_id: Uuid,
        task_name: &str,
        is_recurring_instance: bool,
        parent_task_id: Option<Uuid>,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        let (action_type, description) = if is_recurring_instance {
            ("recurring_instance_created", format!("Created recurring instance: {}", task_name))
        } else {
            ("created", format!("Created task: {}", task_name))
        };

        self.log_activity(
            EntityType::Task,
            task_id,
            creator_id,
            action_type,
            Some(description),
            Some(serde_json::json!({
                "task_name": task_name,
                "is_recurring_instance": is_recurring_instance,
                "parent_task_id": parent_task_id
            })),
            None,
        ).await
    }

    /// Log task update
    pub async fn log_task_update(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
        field_changes: Value,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        self.log_activity(
            EntityType::Task,
            task_id,
            actor_id,
            "updated",
            Some("Task updated".to_string()),
            None,
            Some(field_changes),
        ).await
    }

    /// Log task completion
    pub async fn log_task_completion(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
        created_next_instance: Option<Uuid>,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        let description = if created_next_instance.is_some() {
            "Task completed and next recurring instance created".to_string()
        } else {
            "Task completed".to_string()
        };

        self.log_activity(
            EntityType::Task,
            task_id,
            actor_id,
            "completed",
            Some(description),
            Some(serde_json::json!({
                "next_instance_id": created_next_instance
            })),
            Some(serde_json::json!({
                "field": "status",
                "old_value": "in_progress",
                "new_value": "completed"
            })),
        ).await
    }

    /// Log task deletion
    pub async fn log_task_deletion(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
        task_name: &str,
    ) -> Result<activity::Model, Box<dyn std::error::Error>> {
        self.log_activity(
            EntityType::Task,
            task_id,
            actor_id,
            "deleted",
            Some(format!("Deleted task: {}", task_name)),
            Some(serde_json::json!({
                "task_name": task_name
            })),
            None,
        ).await
    }

    /// Get activity count for an entity
    pub async fn get_activity_count(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let count = Activity::find()
            .filter(activity::Column::EntityType.eq(entity_type.as_str()))
            .filter(activity::Column::EntityId.eq(entity_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// Get recent activities by actor
    pub async fn get_user_recent_activities(
        &self,
        actor_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<activity::Model>, Box<dyn std::error::Error>> {
        let mut query = Activity::find()
            .filter(activity::Column::ActorId.eq(actor_id))
            .order_by_desc(activity::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let activities = query.all(&self.db).await?;
        Ok(activities)
    }
}