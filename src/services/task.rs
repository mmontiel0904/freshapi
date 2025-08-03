use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::entities::{prelude::*, task, project};
use crate::services::ProjectService;

#[derive(Clone)]
pub struct TaskService {
    db: DatabaseConnection,
    project_service: ProjectService,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "todo" => Some(TaskStatus::Todo),
            "in_progress" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "low" => Some(TaskPriority::Low),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "urgent" => Some(TaskPriority::Urgent),
            _ => None,
        }
    }
}

impl TaskService {
    pub fn new(db: DatabaseConnection, project_service: ProjectService) -> Self {
        Self { db, project_service }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        project_id: Uuid,
        creator_id: Uuid,
        name: &str,
        description: Option<String>,
        assignee_id: Option<Uuid>,
        priority: Option<TaskPriority>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<task::Model, Box<dyn std::error::Error>> {
        // Check if user can create tasks in this project
        let creator_role = self
            .project_service
            .get_user_project_role(project_id, creator_id)
            .await?;

        match creator_role {
            Some(role) if role.can_manage_tasks() => {},
            _ => return Err("Insufficient permissions to create tasks in this project".into()),
        }

        // Verify project exists and is active
        let _project = Project::find_by_id(project_id)
            .filter(project::Column::IsActive.eq(true))
            .one(&self.db)
            .await?
            .ok_or("Project not found or inactive")?;

        // If assignee is specified, verify they are a project member
        if let Some(assignee_id) = assignee_id {
            let assignee_role = self
                .project_service
                .get_user_project_role(project_id, assignee_id)
                .await?;
            
            if assignee_role.is_none() {
                return Err("Assignee must be a project member".into());
            }
        }

        let new_task = task::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name.to_string()),
            description: Set(description),
            project_id: Set(project_id),
            assignee_id: Set(assignee_id),
            creator_id: Set(creator_id),
            status: Set(TaskStatus::Todo.as_str().to_string()),
            priority: Set(
                priority
                    .unwrap_or(TaskPriority::Medium)
                    .as_str()
                    .to_string()
            ),
            due_date: Set(due_date.map(|dt| dt.into())),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let task = new_task.insert(&self.db).await?;
        Ok(task)
    }

    /// Get task by ID with permission check
    pub async fn get_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<task::Model>, Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id).one(&self.db).await?;

        match task {
            Some(t) => {
                // Check if user can access the project
                let can_access = self
                    .project_service
                    .can_user_access_project(t.project_id, user_id)
                    .await?;

                if can_access {
                    Ok(Some(t))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get tasks for a project
    pub async fn get_project_tasks(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        status_filter: Option<TaskStatus>,
        assignee_filter: Option<Uuid>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<task::Model>, Box<dyn std::error::Error>> {
        // Check if user can access the project
        let can_access = self
            .project_service
            .can_user_access_project(project_id, user_id)
            .await?;

        if !can_access {
            return Err("Access denied to project".into());
        }

        let mut query = Task::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .order_by_desc(task::Column::UpdatedAt);

        if let Some(status) = status_filter {
            query = query.filter(task::Column::Status.eq(status.as_str()));
        }

        if let Some(assignee_id) = assignee_filter {
            query = query.filter(task::Column::AssigneeId.eq(assignee_id));
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let tasks = query.all(&self.db).await?;
        Ok(tasks)
    }

    /// Get tasks assigned to a user across all projects
    pub async fn get_user_assigned_tasks(
        &self,
        user_id: Uuid,
        status_filter: Option<TaskStatus>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<task::Model>, Box<dyn std::error::Error>> {
        let mut query = Task::find()
            .filter(task::Column::AssigneeId.eq(user_id))
            .order_by_desc(task::Column::UpdatedAt);

        if let Some(status) = status_filter {
            query = query.filter(task::Column::Status.eq(status.as_str()));
        }

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let tasks = query.all(&self.db).await?;
        Ok(tasks)
    }

    /// Update task
    pub async fn update_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        due_date: Option<Option<DateTime<Utc>>>,
    ) -> Result<task::Model, Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id)
            .one(&self.db)
            .await?
            .ok_or("Task not found")?;

        // Check permissions - user must be able to manage tasks in the project
        // OR be the task creator OR be the assignee
        let user_role = self
            .project_service
            .get_user_project_role(task.project_id, user_id)
            .await?;

        let can_edit = match user_role {
            Some(role) if role.can_manage_tasks() => true,
            _ => {
                // Allow task creator or assignee to edit
                task.creator_id == user_id || task.assignee_id == Some(user_id)
            }
        };

        if !can_edit {
            return Err("Insufficient permissions to update this task".into());
        }

        let mut task_active: task::ActiveModel = task.into();

        if let Some(name) = name {
            task_active.name = Set(name);
        }

        if let Some(description) = description {
            task_active.description = Set(description);
        }

        if let Some(status) = status {
            task_active.status = Set(status.as_str().to_string());
        }

        if let Some(priority) = priority {
            task_active.priority = Set(priority.as_str().to_string());
        }

        if let Some(due_date) = due_date {
            task_active.due_date = Set(due_date.map(|dt| dt.into()));
        }

        task_active.updated_at = Set(Utc::now().into());

        let updated_task = task_active.update(&self.db).await?;
        Ok(updated_task)
    }

    /// Assign task to user
    pub async fn assign_task(
        &self,
        task_id: Uuid,
        assigner_id: Uuid,
        assignee_id: Option<Uuid>,
    ) -> Result<task::Model, Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id)
            .one(&self.db)
            .await?
            .ok_or("Task not found")?;

        // Check if assigner can assign tasks in this project
        let assigner_role = self
            .project_service
            .get_user_project_role(task.project_id, assigner_id)
            .await?;

        match assigner_role {
            Some(role) if role.can_manage_tasks() => {},
            _ => return Err("Insufficient permissions to assign tasks".into()),
        }

        // If assignee is specified, verify they are a project member
        if let Some(assignee_id) = assignee_id {
            let assignee_role = self
                .project_service
                .get_user_project_role(task.project_id, assignee_id)
                .await?;
            
            if assignee_role.is_none() {
                return Err("Assignee must be a project member".into());
            }
        }

        let mut task_active: task::ActiveModel = task.into();
        task_active.assignee_id = Set(assignee_id);
        task_active.updated_at = Set(Utc::now().into());

        let updated_task = task_active.update(&self.db).await?;
        Ok(updated_task)
    }

    /// Delete task
    pub async fn delete_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id)
            .one(&self.db)
            .await?
            .ok_or("Task not found")?;

        // Check permissions - user must be able to manage tasks in the project
        // OR be the task creator
        let user_role = self
            .project_service
            .get_user_project_role(task.project_id, user_id)
            .await?;

        let can_delete = match user_role {
            Some(role) if role.can_manage_tasks() => true,
            _ => task.creator_id == user_id,
        };

        if !can_delete {
            return Err("Insufficient permissions to delete this task".into());
        }

        Task::delete_by_id(task_id).exec(&self.db).await?;
        Ok(())
    }

    /// Get task statistics for a project
    pub async fn get_project_task_stats(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskStats, Box<dyn std::error::Error>> {
        // Check if user can access the project
        let can_access = self
            .project_service
            .can_user_access_project(project_id, user_id)
            .await?;

        if !can_access {
            return Err("Access denied to project".into());
        }

        let tasks = Task::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;

        let total = tasks.len() as u32;
        let todo = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Todo.as_str())
            .count() as u32;
        let in_progress = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress.as_str())
            .count() as u32;
        let completed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed.as_str())
            .count() as u32;
        let cancelled = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Cancelled.as_str())
            .count() as u32;

        // Count overdue tasks
        let now = Utc::now();
        let overdue = tasks
            .iter()
            .filter(|t| {
                if let Some(due_date) = &t.due_date {
                    let due_utc: DateTime<Utc> = due_date.clone().into();
                    due_utc < now && t.status != TaskStatus::Completed.as_str()
                } else {
                    false
                }
            })
            .count() as u32;

        Ok(TaskStats {
            total,
            todo,
            in_progress,
            completed,
            cancelled,
            overdue,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TaskStats {
    pub total: u32,
    pub todo: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub cancelled: u32,
    pub overdue: u32,
}