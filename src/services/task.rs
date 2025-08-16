use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;
use chrono::{DateTime, Utc, Datelike, Weekday, Duration};

use crate::entities::{prelude::*, task, project};
use crate::services::{ProjectService, ActivityService};
// EntityType imported when needed
use crate::graphql::types::{TaskStatus, TaskPriority, RecurrenceType};

#[derive(Clone)]
pub struct TaskService {
    db: DatabaseConnection,
    project_service: ProjectService,
    activity_service: ActivityService,
}

impl TaskService {
    pub fn new(db: DatabaseConnection, project_service: ProjectService, activity_service: ActivityService) -> Self {
        Self { db, project_service, activity_service }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Check if user can view/access a specific task
    pub async fn can_user_access_task(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id)
            .one(&self.db)
            .await?
            .ok_or("Task not found")?;

        // Check if user is a member of the project containing the task
        let user_role = self
            .project_service
            .get_user_project_role(task.project_id, user_id)
            .await?;

        // User can access task if they are a project member (any role including viewer)
        Ok(user_role.is_some())
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
        recurrence_type: Option<RecurrenceType>,
        recurrence_day: Option<i32>,
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

        let recurrence = recurrence_type.unwrap_or(RecurrenceType::None);
        let is_recurring = recurrence != RecurrenceType::None;
        let next_due_date = if is_recurring && due_date.is_some() {
            Some(self.calculate_next_due_date(due_date.unwrap(), &recurrence, recurrence_day)?)
        } else {
            None
        };

        let task_id = Uuid::new_v4();
        let new_task = task::ActiveModel {
            id: Set(task_id),
            name: Set(name.to_string()),
            description: Set(description),
            project_id: Set(project_id),
            assignee_id: Set(assignee_id),
            creator_id: Set(creator_id),
            status: Set(TaskStatus::Todo),
            priority: Set(priority.unwrap_or(TaskPriority::Medium)),
            recurrence_type: Set(recurrence),
            recurrence_day: Set(recurrence_day),
            is_recurring: Set(is_recurring),
            parent_task_id: Set(None),
            due_date: Set(due_date.map(|dt| dt.into())),
            next_due_date: Set(next_due_date.map(|dt| dt.into())),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let task = new_task.insert(&self.db).await?;

        // Log task creation activity
        self.activity_service
            .log_task_creation(task.id, creator_id, name, false, None)
            .await?;

        // Log assignment activity if task was assigned during creation
        if let Some(assignee_id) = assignee_id {
            self.activity_service
                .log_task_assignment(task.id, creator_id, None, Some(assignee_id))
                .await?;
        }

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
            query = query.filter(task::Column::Status.eq(status));
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
            query = query.filter(task::Column::Status.eq(status));
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
        recurrence_type: Option<RecurrenceType>,
        recurrence_day: Option<Option<i32>>,
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

        if let Some(ref name) = name {
            task_active.name = Set(name.clone());
        }

        if let Some(ref description) = description {
            task_active.description = Set(description.clone());
        }

        if let Some(status) = status {
            task_active.status = Set(status);
        }

        if let Some(priority) = priority {
            task_active.priority = Set(priority);
        }

        if let Some(recurrence_type) = recurrence_type {
            task_active.recurrence_type = Set(recurrence_type);
            task_active.is_recurring = Set(recurrence_type != RecurrenceType::None);
        }

        if let Some(recurrence_day) = recurrence_day {
            task_active.recurrence_day = Set(recurrence_day);
        }

        if let Some(due_date) = due_date {
            task_active.due_date = Set(due_date.map(|dt| dt.into()));
        }

        task_active.updated_at = Set(Utc::now().into());

        let updated_task = task_active.update(&self.db).await?;

        // Log task update activity
        let field_changes = serde_json::json!({
            "name": name,
            "description": description,
            "status": status.as_ref(),
            "priority": priority.as_ref(),
            "recurrence_type": recurrence_type.as_ref(),
            "recurrence_day": recurrence_day,
            "due_date": due_date
        });

        self.activity_service
            .log_task_update(task_id, user_id, field_changes)
            .await?;

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

        // Store old assignee for activity logging
        let old_assignee_id = task.assignee_id;

        let mut task_active: task::ActiveModel = task.into();
        task_active.assignee_id = Set(assignee_id);
        task_active.updated_at = Set(Utc::now().into());

        let updated_task = task_active.update(&self.db).await?;

        // Log assignment activity
        self.activity_service
            .log_task_assignment(task_id, assigner_id, old_assignee_id, assignee_id)
            .await?;

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
            .filter(|t| t.status == TaskStatus::Todo)
            .count() as u32;
        let in_progress = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count() as u32;
        let completed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count() as u32;
        let cancelled = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Cancelled)
            .count() as u32;

        // Count overdue tasks
        let now = Utc::now();
        let overdue = tasks
            .iter()
            .filter(|t| {
                if let Some(due_date) = &t.due_date {
                    let due_utc: DateTime<Utc> = due_date.clone().into();
                    due_utc < now && t.status != TaskStatus::Completed
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

    /// Calculate next due date based on recurrence pattern
    fn calculate_next_due_date(
        &self,
        current_due: DateTime<Utc>,
        recurrence: &RecurrenceType,
        recurrence_day: Option<i32>,
    ) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        match recurrence {
            RecurrenceType::None => Ok(current_due),
            RecurrenceType::Daily => Ok(current_due + Duration::days(1)),
            RecurrenceType::Weekdays => {
                let mut next = current_due + Duration::days(1);
                // Skip weekends
                while next.weekday() == Weekday::Sat || next.weekday() == Weekday::Sun {
                    next = next + Duration::days(1);
                }
                Ok(next)
            },
            RecurrenceType::Weekly => Ok(current_due + Duration::weeks(1)),
            RecurrenceType::Monthly => {
                if let Some(day) = recurrence_day {
                    // Use the specified day for monthly recurrence
                    let mut next_month = if current_due.month() == 12 {
                        current_due.with_year(current_due.year() + 1).unwrap().with_month(1).unwrap()
                    } else {
                        current_due.with_month(current_due.month() + 1).unwrap()
                    };
                    
                    // Handle end-of-month cases (e.g., Jan 31 -> Feb 28)
                    let target_day = std::cmp::min(day as u32, days_in_month(next_month.year(), next_month.month()));
                    next_month = next_month.with_day(target_day).unwrap();
                    Ok(next_month)
                } else {
                    // Use current day but move to next month
                    let current_day = current_due.day();
                    let mut next_month = if current_due.month() == 12 {
                        current_due.with_year(current_due.year() + 1).unwrap().with_month(1).unwrap()
                    } else {
                        current_due.with_month(current_due.month() + 1).unwrap()
                    };
                    
                    // Handle end-of-month cases
                    let target_day = std::cmp::min(current_day, days_in_month(next_month.year(), next_month.month()));
                    next_month = next_month.with_day(target_day).unwrap();
                    Ok(next_month)
                }
            },
        }
    }

    /// Complete a task and create recurring instance if needed
    /// Returns (completed_task, next_instance)
    pub async fn complete_task_with_recurrence(
        &self,
        task_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(task::Model, Option<task::Model>), Box<dyn std::error::Error>> {
        let task = Task::find_by_id(task_id)
            .one(&self.db)
            .await?
            .ok_or("Task not found")?;

        // Check permissions
        let user_role = self
            .project_service
            .get_user_project_role(task.project_id, actor_id)
            .await?;

        let can_complete = match user_role {
            Some(role) if role.can_manage_tasks() => true,
            _ => task.creator_id == actor_id || task.assignee_id == Some(actor_id),
        };

        if !can_complete {
            return Err("Insufficient permissions to complete this task".into());
        }

        // Update task status to completed
        let mut task_active: task::ActiveModel = task.clone().into();
        task_active.status = Set(TaskStatus::Completed);
        task_active.updated_at = Set(Utc::now().into());

        let completed_task = task_active.update(&self.db).await?;

        // Create next recurring instance if needed
        let next_instance = if task.is_recurring {
            let next_due = if let Some(current_due) = &task.due_date {
                let due_utc: DateTime<Utc> = current_due.clone().into();
                Some(self.calculate_next_due_date(
                    due_utc,
                    &task.recurrence_type,
                    task.recurrence_day,
                )?)
            } else {
                None
            };

            let next_task = task::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(task.name.clone()),
                description: Set(task.description.clone()),
                project_id: Set(task.project_id),
                assignee_id: Set(task.assignee_id),
                creator_id: Set(task.creator_id),
                status: Set(TaskStatus::Todo),
                priority: Set(task.priority.clone()),
                recurrence_type: Set(task.recurrence_type.clone()),
                recurrence_day: Set(task.recurrence_day),
                is_recurring: Set(true),
                parent_task_id: Set(Some(task_id)),
                due_date: Set(next_due.map(|dt| dt.into())),
                next_due_date: Set(None), // Will be calculated when this task is completed
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };

            let created_instance = next_task.insert(&self.db).await?;

            // Log recurring instance creation
            self.activity_service
                .log_task_creation(
                    created_instance.id,
                    task.creator_id,
                    &task.name,
                    true,
                    Some(task_id),
                )
                .await?;

            Some(created_instance)
        } else {
            None
        };

        // Log task completion activity
        self.activity_service
            .log_task_completion(task_id, actor_id, next_instance.as_ref().map(|t| t.id))
            .await?;

        Ok((completed_task, next_instance))
    }
}

// Helper function to get days in a month
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        },
        _ => 30,
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