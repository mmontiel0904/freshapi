use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{prelude::*, project, project_member, user};

#[derive(Clone)]
pub struct ProjectService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub enum ProjectRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl ProjectRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectRole::Owner => "owner",
            ProjectRole::Admin => "admin", 
            ProjectRole::Member => "member",
            ProjectRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(ProjectRole::Owner),
            "admin" => Some(ProjectRole::Admin),
            "member" => Some(ProjectRole::Member),
            "viewer" => Some(ProjectRole::Viewer),
            _ => None,
        }
    }

    pub fn can_manage_project(&self) -> bool {
        matches!(self, ProjectRole::Owner | ProjectRole::Admin)
    }

    pub fn can_invite_users(&self) -> bool {
        matches!(self, ProjectRole::Owner | ProjectRole::Admin)
    }

    pub fn can_manage_tasks(&self) -> bool {
        !matches!(self, ProjectRole::Viewer)
    }
}

impl ProjectService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Create a new project
    pub async fn create_project(
        &self,
        owner_id: Uuid,
        name: &str,
        description: Option<String>,
    ) -> Result<project::Model, Box<dyn std::error::Error>> {
        let tx = self.db.begin().await?;

        // Check if user already has a project with this name
        if let Some(_) = Project::find()
            .filter(project::Column::OwnerId.eq(owner_id))
            .filter(project::Column::Name.eq(name))
            .filter(project::Column::IsActive.eq(true))
            .one(&tx)
            .await?
        {
            return Err("Project with this name already exists".into());
        }

        let project_id = Uuid::new_v4();

        // Create project
        let new_project = project::ActiveModel {
            id: Set(project_id),
            name: Set(name.to_string()),
            description: Set(description),
            owner_id: Set(owner_id),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let project = new_project.insert(&tx).await?;

        // Add owner as project member with owner role
        let owner_member = project_member::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            user_id: Set(owner_id),
            role: Set(ProjectRole::Owner.as_str().to_string()),
            joined_at: Set(Utc::now().into()),
        };

        owner_member.insert(&tx).await?;

        tx.commit().await?;

        Ok(project)
    }

    /// Get project by ID with permission check
    pub async fn get_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<project::Model>, Box<dyn std::error::Error>> {
        // Check if user is a member of the project
        let membership = self.get_user_project_role(project_id, user_id).await?;
        if membership.is_none() {
            return Ok(None); // User is not a member
        }

        let project = Project::find_by_id(project_id)
            .filter(project::Column::IsActive.eq(true))
            .one(&self.db)
            .await?;

        Ok(project)
    }

    /// Get projects for a user (as member or owner)
    pub async fn get_user_projects(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<project::Model>, Box<dyn std::error::Error>> {
        let mut query = Project::find()
            .inner_join(ProjectMember)
            .filter(project_member::Column::UserId.eq(user_id))
            .filter(project::Column::IsActive.eq(true))
            .order_by_desc(project::Column::UpdatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let projects = query.all(&self.db).await?;
        Ok(projects)
    }

    /// Update project
    pub async fn update_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> Result<project::Model, Box<dyn std::error::Error>> {
        // Check if user can manage project
        let role = self.get_user_project_role(project_id, user_id).await?;
        match role {
            Some(r) if r.can_manage_project() => {},
            _ => return Err("Insufficient permissions to update project".into()),
        }

        let project = Project::find_by_id(project_id)
            .filter(project::Column::IsActive.eq(true))
            .one(&self.db)
            .await?
            .ok_or("Project not found")?;

        let mut project_active: project::ActiveModel = project.into();

        if let Some(name) = name {
            // Check for name conflicts
            if let Some(_) = Project::find()
                .filter(project::Column::OwnerId.eq(*project_active.owner_id.as_ref()))
                .filter(project::Column::Name.eq(&name))
                .filter(project::Column::Id.ne(project_id))
                .filter(project::Column::IsActive.eq(true))
                .one(&self.db)
                .await?
            {
                return Err("Project with this name already exists".into());
            }
            project_active.name = Set(name);
        }

        if let Some(description) = description {
            project_active.description = Set(description);
        }

        project_active.updated_at = Set(Utc::now().into());

        let updated_project = project_active.update(&self.db).await?;
        Ok(updated_project)
    }

    /// Soft delete project
    pub async fn delete_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Only project owner can delete
        let role = self.get_user_project_role(project_id, user_id).await?;
        match role {
            Some(ProjectRole::Owner) => {},
            _ => return Err("Only project owner can delete project".into()),
        }

        let project = Project::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or("Project not found")?;

        let mut project_active: project::ActiveModel = project.into();
        project_active.is_active = Set(false);
        project_active.updated_at = Set(Utc::now().into());

        project_active.update(&self.db).await?;

        Ok(())
    }

    /// Add user to project
    pub async fn add_project_member(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        new_member_id: Uuid,
        role: ProjectRole,
    ) -> Result<project_member::Model, Box<dyn std::error::Error>> {
        // Check if requester can invite users
        let requester_role = self.get_user_project_role(project_id, user_id).await?;
        match requester_role {
            Some(r) if r.can_invite_users() => {},
            _ => return Err("Insufficient permissions to add members".into()),
        }

        // Check if user is already a member
        if let Some(_) = ProjectMember::find()
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::UserId.eq(new_member_id))
            .one(&self.db)
            .await?
        {
            return Err("User is already a member of this project".into());
        }

        // Verify the new member user exists
        let _user = User::find_by_id(new_member_id)
            .one(&self.db)
            .await?
            .ok_or("User not found")?;

        // Verify project exists and is active
        let _project = Project::find_by_id(project_id)
            .filter(project::Column::IsActive.eq(true))
            .one(&self.db)
            .await?
            .ok_or("Project not found")?;

        let new_member = project_member::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            user_id: Set(new_member_id),
            role: Set(role.as_str().to_string()),
            joined_at: Set(Utc::now().into()),
        };

        let member = new_member.insert(&self.db).await?;
        Ok(member)
    }

    /// Update project member role
    pub async fn update_member_role(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        target_user_id: Uuid,
        new_role: ProjectRole,
    ) -> Result<project_member::Model, Box<dyn std::error::Error>> {
        // Check if requester can manage project
        let requester_role = self.get_user_project_role(project_id, user_id).await?;
        match requester_role {
            Some(r) if r.can_manage_project() => {},
            _ => return Err("Insufficient permissions to update member roles".into()),
        }

        // Cannot change owner role
        let target_role = self.get_user_project_role(project_id, target_user_id).await?;
        if let Some(ProjectRole::Owner) = target_role {
            return Err("Cannot change project owner role".into());
        }

        let member = ProjectMember::find()
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::UserId.eq(target_user_id))
            .one(&self.db)
            .await?
            .ok_or("User is not a member of this project")?;

        let mut member_active: project_member::ActiveModel = member.into();
        member_active.role = Set(new_role.as_str().to_string());

        let updated_member = member_active.update(&self.db).await?;
        Ok(updated_member)
    }

    /// Remove user from project
    pub async fn remove_project_member(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if requester can manage project
        let requester_role = self.get_user_project_role(project_id, user_id).await?;
        match requester_role {
            Some(r) if r.can_manage_project() => {},
            _ => return Err("Insufficient permissions to remove members".into()),
        }

        // Cannot remove project owner
        let target_role = self.get_user_project_role(project_id, target_user_id).await?;
        if let Some(ProjectRole::Owner) = target_role {
            return Err("Cannot remove project owner".into());
        }

        let member = ProjectMember::find()
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::UserId.eq(target_user_id))
            .one(&self.db)
            .await?
            .ok_or("User is not a member of this project")?;

        ProjectMember::delete_by_id(member.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Get project members
    pub async fn get_project_members(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<(project_member::Model, user::Model)>, Box<dyn std::error::Error>> {
        // Check if user is a member
        let role = self.get_user_project_role(project_id, user_id).await?;
        if role.is_none() {
            return Err("Access denied".into());
        }

        let members = ProjectMember::find()
            .filter(project_member::Column::ProjectId.eq(project_id))
            .find_also_related(User)
            .all(&self.db)
            .await?;

        let result = members
            .into_iter()
            .filter_map(|(member, user_opt)| user_opt.map(|user| (member, user)))
            .collect();

        Ok(result)
    }

    /// Get user's role in project
    pub async fn get_user_project_role(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<ProjectRole>, Box<dyn std::error::Error>> {
        let member = ProjectMember::find()
            .filter(project_member::Column::ProjectId.eq(project_id))
            .filter(project_member::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        match member {
            Some(m) => Ok(ProjectRole::from_str(&m.role)),
            None => Ok(None),
        }
    }

    /// Check if user can access project
    pub async fn can_user_access_project(
        &self,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let role = self.get_user_project_role(project_id, user_id).await?;
        Ok(role.is_some())
    }
}