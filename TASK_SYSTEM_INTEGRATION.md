# Task System Frontend Integration Guide

This guide shows how to integrate the new **Task Tracking System** with **Recurring Tasks** and **Activity Logging** features with your TypeScript/Vue.js frontend. This complements the main [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) document by focusing specifically on project and task management features.

## 🎯 Task System Overview

The task system provides comprehensive project and task management with:
- **Project Management**: Create, manage, and organize projects with team members
- **Task Tracking**: Full task lifecycle with status, priority, and assignment management
- **Recurring Tasks**: Automated task creation with flexible recurrence patterns (daily, weekdays, weekly, monthly)
- **Activity Logging**: Complete audit trail and commenting system for all entities
- **Role-Based Access**: Project-level permissions with hierarchical roles
- **RBAC Integration**: Seamless integration with the existing permission system
- **Type-Safe Enums**: GraphQL introspection-enabled enums for better frontend DX

## 📋 Schema Reference

### Key GraphQL Types

#### Type-Safe Enums (GraphQL Introspection Enabled)

```graphql
enum TaskStatus {
  TODO
  IN_PROGRESS
  COMPLETED
  CANCELLED
}

enum TaskPriority {
  LOW
  MEDIUM
  HIGH
  URGENT
}

enum RecurrenceType {
  NONE
  DAILY       # Every day
  WEEKDAYS    # Monday-Friday only
  WEEKLY      # Same day each week  
  MONTHLY     # Same day of month
}

enum EntityType {
  TASK
  PROJECT
  USER
  SETTINGS
}
```

#### Core Types

```graphql
type Project {
  id: UUID!
  name: String!
  description: String
  ownerId: UUID!
  isActive: Boolean!
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Related data (auto-resolved)
  owner: User
  members: [ProjectMember!]!
  tasks(status: TaskStatus, assigneeId: UUID, limit: Int, offset: Int): [Task!]!
}

type Task {
  id: UUID!
  name: String!
  description: String
  projectId: UUID!
  assigneeId: UUID
  creatorId: UUID!
  status: TaskStatus!
  priority: TaskPriority!
  
  # Recurring task fields
  recurrenceType: RecurrenceType!
  recurrenceDay: Int              # For monthly: day of month (1-31), for weekly: day of week (1-7)
  isRecurring: Boolean!
  parentTaskId: UUID              # Links to original recurring task
  nextDueDate: DateTime           # When to create next instance
  
  dueDate: DateTime
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Related data (auto-resolved)
  project: Project
  assignee: User
  creator: User!
  parentTask: Task                # Original recurring task
  recurringInstances(limit: Int): [Task!]!  # Child recurring instances
  activities(limit: Int, offset: Int): [Activity!]!
  activityCount: Int!
}

type Activity {
  id: UUID!
  entityType: String!             # "task", "project", "user", "settings"
  entityId: UUID!
  actorId: UUID!
  actionType: String!             # "created", "updated", "completed", "commented", etc.
  description: String
  createdAt: DateTime!
  
  # Related data (auto-resolved)
  actor: User
  metadataJson: String            # JSON string of metadata
  changesJson: String             # JSON string of field changes
}

type ProjectMember {
  id: UUID!
  projectId: UUID!
  userId: UUID!
  role: String!          # "owner", "admin", "member", "viewer"
  joinedAt: DateTime!
  user: User!
}

type TaskStats {
  total: Int!
  todo: Int!
  inProgress: Int!
  completed: Int!
  cancelled: Int!
  overdue: Int!
}
```

#### Input Types

```graphql
input CreateTaskInput {
  projectId: UUID!
  name: String!
  description: String
  assigneeId: UUID
  priority: TaskPriority          # Default: MEDIUM
  recurrenceType: RecurrenceType  # Default: NONE
  recurrenceDay: Int              # Required for MONTHLY (1-31) or WEEKLY (1-7)
  dueDate: DateTime
}

input UpdateTaskInput {
  taskId: UUID!
  name: String
  description: String
  status: TaskStatus
  priority: TaskPriority
  recurrenceType: RecurrenceType
  recurrenceDay: Int
  dueDate: DateTime
}

input AddCommentInput {
  entityType: EntityType!         # TASK, PROJECT, USER, SETTINGS
  entityId: UUID!
  content: String!
  mentions: [UUID!]               # User IDs to mention
}
```

#### New Queries and Mutations

```graphql
# Recurring task mutations
mutation CompleteTaskWithRecurrence($taskId: UUID!) {
  completeTaskWithRecurrence(taskId: $taskId) {
    originalTask: Task
    nextInstance: Task           # Null if not recurring
  }
}

# Activity system
query GetActivities($entityType: EntityType!, $entityId: UUID!, $limit: Int, $offset: Int) {
  activities(entityType: $entityType, entityId: $entityId, limit: $limit, offset: $offset) {
    id
    actionType
    description
    createdAt
    actor {
      id
      email
      firstName
      lastName
    }
    metadataJson
    changesJson
  }
}

mutation AddComment($input: AddCommentInput!) {
  addComment(input: $input) {
    id
    description
    createdAt
    actor {
      id
      email
      firstName
      lastName
    }
  }
}

# Enhanced task queries with new fields
query GetTask($taskId: UUID!) {
  task(taskId: $taskId) {
    id
    name
    description
    status
    priority
    recurrenceType
    recurrenceDay
    isRecurring
    parentTaskId
    dueDate
    nextDueDate
    
    parentTask {
      id
      name
    }
    
    recurringInstances(limit: 10) {
      id
      name
      status
      dueDate
      createdAt
    }
    
    activities(limit: 20) {
      id
      actionType
      description
      createdAt
      actor {
        id
        email
        firstName
        lastName
      }
    }
    
    activityCount
  }
}
```

## 🔐 Task System Permissions

Reference the main [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md#rbac-system-implementation) for the permission system setup, then add these task-specific permission checks:

### Permission Structure

```typescript
// Task system permissions (resource: "task_system")
const TASK_PERMISSIONS = {
  CREATE: 'create',           // Create projects and tasks
  READ: 'read',              // View projects and tasks
  WRITE: 'write',            // Edit projects and tasks
  ADMIN: 'admin',            // Delete projects/tasks, manage settings
  USER_MANAGEMENT: 'user_management'  // Invite/remove project members
} as const

// Permission levels by role
const ROLE_PERMISSIONS = {
  super_admin: ['create', 'read', 'write', 'admin', 'user_management'],
  admin: ['create', 'read', 'write', 'admin', 'user_management'],
  user: ['create', 'read', 'write']  // Cannot delete or manage users
} as const
```

### Task Permission Service

```typescript
// services/taskPermissions.ts
import { permissionService } from '@/services/permissions'

export class TaskPermissionService {
  // Task system permissions
  async canCreateProjects(): Promise<boolean> {
    return permissionService.hasPermission('task_system:create')
  }

  async canViewProjects(): Promise<boolean> {
    return permissionService.hasPermission('task_system:read')
  }

  async canEditProjects(): Promise<boolean> {
    return permissionService.hasPermission('task_system:write')
  }

  async canDeleteProjects(): Promise<boolean> {
    return permissionService.hasPermission('task_system:admin')
  }

  async canManageProjectMembers(): Promise<boolean> {
    return permissionService.hasPermission('task_system:user_management')
  }

  // Sync versions (require permissions to be loaded)
  canCreateProjectsSync(): boolean {
    return permissionService.hasPermissionSync('task_system:create')
  }

  canViewProjectsSync(): boolean {
    return permissionService.hasPermissionSync('task_system:read')
  }

  canEditProjectsSync(): boolean {
    return permissionService.hasPermissionSync('task_system:write')
  }

  canDeleteProjectsSync(): boolean {
    return permissionService.hasPermissionSync('task_system:admin')
  }

  canManageProjectMembersSync(): boolean {
    return permissionService.hasPermissionSync('task_system:user_management')
  }
}

export const taskPermissionService = new TaskPermissionService()
```

## 🏗 Vue Composables for Task Management

### Project Management Composable

```typescript
// composables/useProjects.ts
import { ref, computed } from 'vue'
import { useApolloClient } from '@vue/apollo-composable'
import type { Project, CreateProjectInput, UpdateProjectInput } from '@/generated/graphql'

export function useProjects() {
  const apolloClient = useApolloClient()
  const projects = ref<Project[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // GraphQL Queries and Mutations
  const MY_PROJECTS_QUERY = gql`
    query MyProjects($limit: Int, $offset: Int) {
      myProjects(limit: $limit, offset: $offset) {
        id
        name
        description
        ownerId
        isActive
        createdAt
        updatedAt
        owner {
          id
          email
          firstName
          lastName
        }
        members {
          id
          role
          joinedAt
          user {
            id
            email
            firstName
            lastName
          }
        }
      }
    }
  `

  const PROJECT_QUERY = gql`
    query Project($projectId: UUID!) {
      project(projectId: $projectId) {
        id
        name
        description
        ownerId
        isActive
        createdAt
        updatedAt
        owner {
          id
          email
          firstName
          lastName
        }
        members {
          id
          role
          joinedAt
          user {
            id
            email
            firstName
            lastName
          }
        }
        tasks(limit: 10) {
          id
          name
          status
          priority
          dueDate
          assignee {
            id
            email
            firstName
            lastName
          }
        }
      }
    }
  `

  const CREATE_PROJECT_MUTATION = gql`
    mutation CreateProject($input: CreateProjectInput!) {
      createProject(input: $input) {
        id
        name
        description
        ownerId
        createdAt
      }
    }
  `

  const UPDATE_PROJECT_MUTATION = gql`
    mutation UpdateProject($input: UpdateProjectInput!) {
      updateProject(input: $input) {
        id
        name
        description
        updatedAt
      }
    }
  `

  const DELETE_PROJECT_MUTATION = gql`
    mutation DeleteProject($projectId: UUID!) {
      deleteProject(projectId: $projectId) {
        message
      }
    }
  `

  // Actions
  const loadProjects = async (limit = 50, offset = 0) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.query({
        query: MY_PROJECTS_QUERY,
        variables: { limit, offset },
        fetchPolicy: 'cache-first'
      })
      
      projects.value = result.data.myProjects
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load projects'
      console.error('Failed to load projects:', err)
    } finally {
      loading.value = false
    }
  }

  const getProject = async (projectId: string) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.query({
        query: PROJECT_QUERY,
        variables: { projectId },
        fetchPolicy: 'cache-first'
      })
      
      return result.data.project
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load project'
      console.error('Failed to load project:', err)
      return null
    } finally {
      loading.value = false
    }
  }

  const createProject = async (input: CreateProjectInput) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: CREATE_PROJECT_MUTATION,
        variables: { input }
      })
      
      const newProject = result.data.createProject
      projects.value.unshift(newProject)
      return newProject
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create project'
      console.error('Failed to create project:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateProject = async (input: UpdateProjectInput) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: UPDATE_PROJECT_MUTATION,
        variables: { input }
      })
      
      const updatedProject = result.data.updateProject
      const index = projects.value.findIndex(p => p.id === updatedProject.id)
      if (index !== -1) {
        projects.value[index] = { ...projects.value[index], ...updatedProject }
      }
      return updatedProject
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update project'
      console.error('Failed to update project:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteProject = async (projectId: string) => {
    loading.value = true
    error.value = null
    
    try {
      await apolloClient.client.mutate({
        mutation: DELETE_PROJECT_MUTATION,
        variables: { projectId }
      })
      
      projects.value = projects.value.filter(p => p.id !== projectId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete project'
      console.error('Failed to delete project:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // Computed properties
  const activeProjects = computed(() => 
    projects.value.filter(p => p.isActive)
  )

  const projectCount = computed(() => projects.value.length)

  return {
    // State
    projects,
    loading,
    error,
    
    // Computed
    activeProjects,
    projectCount,
    
    // Actions
    loadProjects,
    getProject,
    createProject,
    updateProject,
    deleteProject
  }
}
```

### Task Management Composable

```typescript
// composables/useTasks.ts
import { ref, computed } from 'vue'
import { useApolloClient } from '@vue/apollo-composable'
import type { Task, CreateTaskInput, UpdateTaskInput, TaskStats } from '@/generated/graphql'

export function useTasks() {
  const apolloClient = useApolloClient()
  const tasks = ref<Task[]>([])
  const taskStats = ref<TaskStats | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Type-safe enum constants (matching GraphQL enums)
  const TASK_STATUS = {
    TODO: 'TODO',
    IN_PROGRESS: 'IN_PROGRESS',
    COMPLETED: 'COMPLETED',
    CANCELLED: 'CANCELLED'
  } as const

  const TASK_PRIORITY = {
    LOW: 'LOW',
    MEDIUM: 'MEDIUM',
    HIGH: 'HIGH',
    URGENT: 'URGENT'
  } as const

  const RECURRENCE_TYPE = {
    NONE: 'NONE',
    DAILY: 'DAILY',
    WEEKDAYS: 'WEEKDAYS', 
    WEEKLY: 'WEEKLY',
    MONTHLY: 'MONTHLY'
  } as const

  // GraphQL Queries and Mutations
  const PROJECT_TASKS_QUERY = gql`
    query ProjectTasks(
      $projectId: UUID!, 
      $status: String, 
      $assigneeId: UUID, 
      $limit: Int, 
      $offset: Int
    ) {
      projectTasks(
        projectId: $projectId, 
        status: $status, 
        assigneeId: $assigneeId, 
        limit: $limit, 
        offset: $offset
      ) {
        id
        name
        description
        projectId
        assigneeId
        creatorId
        status
        priority
        dueDate
        createdAt
        updatedAt
        assignee {
          id
          email
          firstName
          lastName
        }
        creator {
          id
          email
          firstName
          lastName
        }
      }
    }
  `

  const MY_ASSIGNED_TASKS_QUERY = gql`
    query MyAssignedTasks($status: String, $limit: Int, $offset: Int) {
      myAssignedTasks(status: $status, limit: $limit, offset: $offset) {
        id
        name
        description
        projectId
        status
        priority
        dueDate
        createdAt
        updatedAt
        project {
          id
          name
        }
        creator {
          id
          email
          firstName
          lastName
        }
      }
    }
  `

  const TASK_STATS_QUERY = gql`
    query ProjectTaskStats($projectId: UUID!) {
      projectTaskStats(projectId: $projectId) {
        total
        todo
        inProgress
        completed
        cancelled
        overdue
      }
    }
  `

  const CREATE_TASK_MUTATION = gql`
    mutation CreateTask($input: CreateTaskInput!) {
      createTask(input: $input) {
        id
        name
        description
        projectId
        assigneeId
        creatorId
        status
        priority
        dueDate
        createdAt
        assignee {
          id
          email
          firstName
          lastName
        }
      }
    }
  `

  const UPDATE_TASK_MUTATION = gql`
    mutation UpdateTask($input: UpdateTaskInput!) {
      updateTask(input: $input) {
        id
        name
        description
        status
        priority
        dueDate
        updatedAt
      }
    }
  `

  const ASSIGN_TASK_MUTATION = gql`
    mutation AssignTask($input: AssignTaskInput!) {
      assignTask(input: $input) {
        id
        assigneeId
        assignee {
          id
          email
          firstName
          lastName
        }
        updatedAt
      }
    }
  `

  const DELETE_TASK_MUTATION = gql`
    mutation DeleteTask($taskId: UUID!) {
      deleteTask(taskId: $taskId) {
        message
      }
    }
  `

  // Actions
  const loadProjectTasks = async (
    projectId: string, 
    filters: {
      status?: string
      assigneeId?: string
      limit?: number
      offset?: number
    } = {}
  ) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.query({
        query: PROJECT_TASKS_QUERY,
        variables: { projectId, ...filters },
        fetchPolicy: 'cache-first'
      })
      
      tasks.value = result.data.projectTasks
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load tasks'
      console.error('Failed to load tasks:', err)
    } finally {
      loading.value = false
    }
  }

  const loadMyAssignedTasks = async (
    filters: {
      status?: string
      limit?: number
      offset?: number
    } = {}
  ) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.query({
        query: MY_ASSIGNED_TASKS_QUERY,
        variables: filters,
        fetchPolicy: 'cache-first'
      })
      
      tasks.value = result.data.myAssignedTasks
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load assigned tasks'
      console.error('Failed to load assigned tasks:', err)
    } finally {
      loading.value = false
    }
  }

  const loadTaskStats = async (projectId: string) => {
    try {
      const result = await apolloClient.client.query({
        query: TASK_STATS_QUERY,
        variables: { projectId },
        fetchPolicy: 'cache-first'
      })
      
      taskStats.value = result.data.projectTaskStats
    } catch (err) {
      console.error('Failed to load task stats:', err)
    }
  }

  const createTask = async (input: CreateTaskInput) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: CREATE_TASK_MUTATION,
        variables: { input }
      })
      
      const newTask = result.data.createTask
      tasks.value.unshift(newTask)
      return newTask
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create task'
      console.error('Failed to create task:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateTask = async (input: UpdateTaskInput) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: UPDATE_TASK_MUTATION,
        variables: { input }
      })
      
      const updatedTask = result.data.updateTask
      const index = tasks.value.findIndex(t => t.id === updatedTask.id)
      if (index !== -1) {
        tasks.value[index] = { ...tasks.value[index], ...updatedTask }
      }
      return updatedTask
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update task'
      console.error('Failed to update task:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const assignTask = async (taskId: string, assigneeId: string | null) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: ASSIGN_TASK_MUTATION,
        variables: { input: { taskId, assigneeId } }
      })
      
      const updatedTask = result.data.assignTask
      const index = tasks.value.findIndex(t => t.id === updatedTask.id)
      if (index !== -1) {
        tasks.value[index] = { ...tasks.value[index], ...updatedTask }
      }
      return updatedTask
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to assign task'
      console.error('Failed to assign task:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteTask = async (taskId: string) => {
    loading.value = true
    error.value = null
    
    try {
      await apolloClient.client.mutate({
        mutation: DELETE_TASK_MUTATION,
        variables: { taskId }
      })
      
      tasks.value = tasks.value.filter(t => t.id !== taskId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete task'
      console.error('Failed to delete task:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // Computed properties
  const tasksByStatus = computed(() => {
    return {
      todo: tasks.value.filter(t => t.status === TASK_STATUS.TODO),
      inProgress: tasks.value.filter(t => t.status === TASK_STATUS.IN_PROGRESS),
      completed: tasks.value.filter(t => t.status === TASK_STATUS.COMPLETED),
      cancelled: tasks.value.filter(t => t.status === TASK_STATUS.CANCELLED)
    }
  })

  const overdueTasks = computed(() => {
    const now = new Date()
    return tasks.value.filter(t => 
      t.dueDate && 
      new Date(t.dueDate) < now && 
      t.status !== TASK_STATUS.COMPLETED
    )
  })

  const urgentTasks = computed(() => 
    tasks.value.filter(t => t.priority === TASK_PRIORITY.URGENT)
  )

  const taskCount = computed(() => tasks.value.length)

  return {
    // Constants
    TASK_STATUS,
    TASK_PRIORITY,
    
    // State
    tasks,
    taskStats,
    loading,
    error,
    
    // Computed
    tasksByStatus,
    overdueTasks,
    urgentTasks,
    taskCount,
    
    // Actions
    loadProjectTasks,
    loadMyAssignedTasks,
    loadTaskStats,
    createTask,
    updateTask,
    assignTask,
    deleteTask
  }
}
```

### Project Member Management Composable

```typescript
// composables/useProjectMembers.ts
import { ref } from 'vue'
import { useApolloClient } from '@vue/apollo-composable'
import type { ProjectMember, AddProjectMemberInput, UpdateMemberRoleInput } from '@/generated/graphql'

export function useProjectMembers() {
  const apolloClient = useApolloClient()
  const members = ref<ProjectMember[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Project role constants
  const PROJECT_ROLES = {
    OWNER: 'owner',
    ADMIN: 'admin',
    MEMBER: 'member',
    VIEWER: 'viewer'
  } as const

  // GraphQL Mutations
  const ADD_PROJECT_MEMBER_MUTATION = gql`
    mutation AddProjectMember($input: AddProjectMemberInput!) {
      addProjectMember(input: $input) {
        message
      }
    }
  `

  const UPDATE_MEMBER_ROLE_MUTATION = gql`
    mutation UpdateMemberRole($input: UpdateMemberRoleInput!) {
      updateMemberRole(input: $input) {
        message
      }
    }
  `

  const REMOVE_PROJECT_MEMBER_MUTATION = gql`
    mutation RemoveProjectMember($input: RemoveProjectMemberInput!) {
      removeProjectMember(input: $input) {
        message
      }
    }
  `

  // Actions
  const addProjectMember = async (input: AddProjectMemberInput) => {
    loading.value = true
    error.value = null
    
    try {
      await apolloClient.client.mutate({
        mutation: ADD_PROJECT_MEMBER_MUTATION,
        variables: { input }
      })
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to add project member'
      console.error('Failed to add project member:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateMemberRole = async (input: UpdateMemberRoleInput) => {
    loading.value = true
    error.value = null
    
    try {
      await apolloClient.client.mutate({
        mutation: UPDATE_MEMBER_ROLE_MUTATION,
        variables: { input }
      })
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update member role'
      console.error('Failed to update member role:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  const removeProjectMember = async (projectId: string, userId: string) => {
    loading.value = true
    error.value = null
    
    try {
      await apolloClient.client.mutate({
        mutation: REMOVE_PROJECT_MEMBER_MUTATION,
        variables: { input: { projectId, userId } }
      })
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to remove project member'
      console.error('Failed to remove project member:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    // Constants
    PROJECT_ROLES,
    
    // State
    members,
    loading,
    error,
    
    // Actions
    addProjectMember,
    updateMemberRole,
    removeProjectMember
  }
}
```

### Recurring Tasks Composable

```typescript
// composables/useRecurringTasks.ts
import { ref } from 'vue'
import { useApolloClient } from '@vue/apollo-composable'
import type { Task } from '@/generated/graphql'

export function useRecurringTasks() {
  const apolloClient = useApolloClient()
  const loading = ref(false)
  const error = ref<string | null>(null)

  const COMPLETE_TASK_WITH_RECURRENCE_MUTATION = gql`
    mutation CompleteTaskWithRecurrence($taskId: UUID!) {
      completeTaskWithRecurrence(taskId: $taskId) {
        originalTask {
          id
          status
          updatedAt
        }
        nextInstance {
          id
          name
          status
          dueDate
          parentTaskId
          createdAt
        }
      }
    }
  `

  const completeRecurringTask = async (taskId: string): Promise<{
    originalTask: Task
    nextInstance: Task | null
  }> => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: COMPLETE_TASK_WITH_RECURRENCE_MUTATION,
        variables: { taskId }
      })
      
      return result.data.completeTaskWithRecurrence
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to complete recurring task'
      console.error('Failed to complete recurring task:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // Helper to format recurrence descriptions for UI
  const formatRecurrenceDescription = (recurrenceType: string, recurrenceDay?: number): string => {
    switch (recurrenceType) {
      case 'DAILY':
        return 'Repeats daily'
      case 'WEEKDAYS':
        return 'Repeats on weekdays (Mon-Fri)'
      case 'WEEKLY':
        return 'Repeats weekly'
      case 'MONTHLY':
        if (recurrenceDay) {
          const suffix = getOrdinalSuffix(recurrenceDay)
          return `Repeats on the ${recurrenceDay}${suffix} of each month`
        }
        return 'Repeats monthly'
      case 'NONE':
      default:
        return 'Does not repeat'
    }
  }

  // Helper to get ordinal suffix (1st, 2nd, 3rd, etc.)
  const getOrdinalSuffix = (day: number): string => {
    if (day >= 11 && day <= 13) return 'th'
    switch (day % 10) {
      case 1: return 'st'
      case 2: return 'nd'
      case 3: return 'rd'
      default: return 'th'
    }
  }

  return {
    // State
    loading,
    error,
    
    // Actions
    completeRecurringTask,
    
    // Helpers
    formatRecurrenceDescription
  }
}
```

### Activity System Composable

```typescript
// composables/useActivities.ts
import { ref, computed } from 'vue'
import { useApolloClient } from '@vue/apollo-composable'
import type { Activity } from '@/generated/graphql'

export function useActivities() {
  const apolloClient = useApolloClient()
  const activities = ref<Activity[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const ENTITY_TYPE = {
    TASK: 'TASK',
    PROJECT: 'PROJECT',
    USER: 'USER',
    SETTINGS: 'SETTINGS'
  } as const

  const GET_ACTIVITIES_QUERY = gql`
    query GetActivities($entityType: EntityType!, $entityId: UUID!, $limit: Int, $offset: Int) {
      activities(entityType: $entityType, entityId: $entityId, limit: $limit, offset: $offset) {
        id
        actionType
        description
        createdAt
        actor {
          id
          email
          firstName
          lastName
        }
        metadataJson
        changesJson
      }
    }
  `

  const ADD_COMMENT_MUTATION = gql`
    mutation AddComment($input: AddCommentInput!) {
      addComment(input: $input) {
        id
        description
        createdAt
        actor {
          id
          email
          firstName
          lastName
        }
      }
    }
  `

  const loadActivities = async (
    entityType: keyof typeof ENTITY_TYPE,
    entityId: string,
    limit = 50,
    offset = 0
  ) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.query({
        query: GET_ACTIVITIES_QUERY,
        variables: {
          entityType: ENTITY_TYPE[entityType],
          entityId,
          limit,
          offset
        },
        fetchPolicy: 'cache-first'
      })
      
      activities.value = result.data.activities
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load activities'
      console.error('Failed to load activities:', err)
    } finally {
      loading.value = false
    }
  }

  const addComment = async (
    entityType: keyof typeof ENTITY_TYPE,
    entityId: string,
    content: string,
    mentions?: string[]
  ) => {
    loading.value = true
    error.value = null
    
    try {
      const result = await apolloClient.client.mutate({
        mutation: ADD_COMMENT_MUTATION,
        variables: {
          input: {
            entityType: ENTITY_TYPE[entityType],
            entityId,
            content,
            mentions
          }
        }
      })
      
      const newActivity = result.data.addComment
      activities.value.unshift(newActivity)
      return newActivity
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to add comment'
      console.error('Failed to add comment:', err)
      throw err
    } finally {
      loading.value = false
    }
  }

  // Computed properties
  const commentActivities = computed(() =>
    activities.value.filter(a => a.actionType === 'commented')
  )

  const systemActivities = computed(() =>
    activities.value.filter(a => a.actionType !== 'commented')
  )

  const activityCount = computed(() => activities.value.length)

  // Helper to format activity descriptions
  const formatActivityDescription = (activity: Activity): string => {
    if (activity.description) return activity.description
    
    // Fallback formatting based on action type
    switch (activity.actionType) {
      case 'created':
        return 'Item created'
      case 'updated':
        return 'Item updated'
      case 'completed':
        return 'Item completed'
      case 'assignment_changed':
        return 'Assignment changed'
      case 'status_changed':
        return 'Status changed'
      default:
        return activity.actionType.replace('_', ' ')
    }
  }

  // Helper to parse and format field changes
  const formatFieldChanges = (changesJson?: string): Array<{
    field: string
    oldValue: any
    newValue: any
  }> => {
    if (!changesJson) return []
    
    try {
      const changes = JSON.parse(changesJson)
      return Object.entries(changes).map(([field, change]: [string, any]) => ({
        field,
        oldValue: change.old_value,
        newValue: change.new_value
      }))
    } catch {
      return []
    }
  }

  return {
    // Constants
    ENTITY_TYPE,
    
    // State
    activities,
    loading,
    error,
    
    // Computed
    commentActivities,
    systemActivities,
    activityCount,
    
    // Actions
    loadActivities,
    addComment,
    
    // Helpers
    formatActivityDescription,
    formatFieldChanges
  }
}
```

## 🎨 Task System UI Components

### Project Dashboard Component

```vue
<!-- components/ProjectDashboard.vue -->
<template>
  <div class="project-dashboard">
    <!-- Header with permissions-based actions -->
    <div class="dashboard-header">
      <h1>Projects</h1>
      <button 
        v-if="canCreateProjectsSync" 
        @click="showCreateModal = true"
        class="btn btn-primary"
      >
        <PlusIcon class="w-4 h-4 mr-2" />
        New Project
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading">
      Loading projects...
    </div>

    <!-- Error state -->
    <div v-if="error" class="error">
      {{ error }}
    </div>

    <!-- Projects grid -->
    <div v-else class="projects-grid">
      <div 
        v-for="project in activeProjects" 
        :key="project.id"
        class="project-card"
        @click="$router.push(`/projects/${project.id}`)"
      >
        <div class="project-header">
          <h3>{{ project.name }}</h3>
          <span class="member-count">
            {{ project.members.length }} members
          </span>
        </div>
        
        <p v-if="project.description" class="project-description">
          {{ project.description }}
        </p>
        
        <div class="project-footer">
          <div class="project-owner">
            Owner: {{ formatUserName(project.owner) }}
          </div>
          <div class="project-date">
            Created {{ formatDate(project.createdAt) }}
          </div>
        </div>
      </div>
    </div>

    <!-- Create Project Modal -->
    <CreateProjectModal 
      v-if="showCreateModal"
      @close="showCreateModal = false"
      @created="handleProjectCreated"
    />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useProjects } from '@/composables/useProjects'
import { taskPermissionService } from '@/services/taskPermissions'
import CreateProjectModal from './CreateProjectModal.vue'
import { PlusIcon } from '@heroicons/vue/24/outline'

const { 
  projects, 
  activeProjects, 
  loading, 
  error, 
  loadProjects 
} = useProjects()

const showCreateModal = ref(false)
const canCreateProjectsSync = taskPermissionService.canCreateProjectsSync

// Load projects on mount
onMounted(() => {
  loadProjects()
})

const handleProjectCreated = (project) => {
  showCreateModal.value = false
  // Projects list will be updated automatically by the composable
}

const formatUserName = (user) => {
  if (!user) return 'Unknown'
  return [user.firstName, user.lastName].filter(Boolean).join(' ') || user.email
}

const formatDate = (date) => {
  return new Date(date).toLocaleDateString()
}
</script>

<style scoped>
.project-dashboard {
  @apply p-6;
}

.dashboard-header {
  @apply flex justify-between items-center mb-6;
}

.projects-grid {
  @apply grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6;
}

.project-card {
  @apply bg-white rounded-lg shadow-md p-6 cursor-pointer transition-shadow hover:shadow-lg;
}

.project-header {
  @apply flex justify-between items-start mb-3;
}

.project-header h3 {
  @apply text-lg font-semibold text-gray-900;
}

.member-count {
  @apply text-sm text-gray-500;
}

.project-description {
  @apply text-gray-600 mb-4 line-clamp-2;
}

.project-footer {
  @apply text-sm text-gray-500 space-y-1;
}

.loading, .error {
  @apply text-center py-8;
}

.error {
  @apply text-red-600;
}
</style>
```

### Task Board Component (Kanban Style)

```vue
<!-- components/TaskBoard.vue -->
<template>
  <div class="task-board">
    <!-- Board header -->
    <div class="board-header">
      <h2>{{ project?.name }} - Tasks</h2>
      <div class="board-actions">
        <TaskFilters 
          v-model:status="statusFilter"
          v-model:assignee="assigneeFilter"
          :members="project?.members || []"
        />
        <button 
          v-if="canCreateProjectsSync"
          @click="showCreateTaskModal = true"
          class="btn btn-primary"
        >
          <PlusIcon class="w-4 h-4 mr-2" />
          New Task
        </button>
      </div>
    </div>

    <!-- Task statistics -->
    <TaskStatsBar v-if="taskStats" :stats="taskStats" />

    <!-- Kanban columns -->
    <div class="kanban-board">
      <div 
        v-for="status in Object.values(TASK_STATUS)" 
        :key="status"
        class="kanban-column"
      >
        <div class="column-header">
          <h3>{{ formatStatusName(status) }}</h3>
          <span class="task-count">
            {{ tasksByStatus[status]?.length || 0 }}
          </span>
        </div>
        
        <div class="column-content">
          <TaskCard 
            v-for="task in tasksByStatus[status]" 
            :key="task.id"
            :task="task"
            :project-members="project?.members || []"
            @update="handleTaskUpdate"
            @delete="handleTaskDelete"
            @assign="handleTaskAssign"
          />
        </div>
      </div>
    </div>

    <!-- Create Task Modal -->
    <CreateTaskModal 
      v-if="showCreateTaskModal"
      :project-id="projectId"
      :project-members="project?.members || []"
      @close="showCreateTaskModal = false"
      @created="handleTaskCreated"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useTasks } from '@/composables/useTasks'
import { useProjects } from '@/composables/useProjects'
import { taskPermissionService } from '@/services/taskPermissions'
import TaskCard from './TaskCard.vue'
import TaskFilters from './TaskFilters.vue'
import TaskStatsBar from './TaskStatsBar.vue'
import CreateTaskModal from './CreateTaskModal.vue'
import { PlusIcon } from '@heroicons/vue/24/outline'

const route = useRoute()
const projectId = computed(() => route.params.id as string)

const { 
  TASK_STATUS,
  tasks,
  taskStats,
  tasksByStatus,
  loading,
  error,
  loadProjectTasks,
  loadTaskStats,
  updateTask,
  deleteTask,
  assignTask
} = useTasks()

const { getProject } = useProjects()

const project = ref(null)
const showCreateTaskModal = ref(false)
const statusFilter = ref('')
const assigneeFilter = ref('')
const canCreateProjectsSync = taskPermissionService.canCreateProjectsSync

// Load project and tasks
onMounted(async () => {
  if (projectId.value) {
    project.value = await getProject(projectId.value)
    await Promise.all([
      loadProjectTasks(projectId.value, {
        status: statusFilter.value || undefined,
        assigneeId: assigneeFilter.value || undefined
      }),
      loadTaskStats(projectId.value)
    ])
  }
})

// Reload tasks when filters change
watch([statusFilter, assigneeFilter], () => {
  if (projectId.value) {
    loadProjectTasks(projectId.value, {
      status: statusFilter.value || undefined,
      assigneeId: assigneeFilter.value || undefined
    })
  }
})

const handleTaskUpdate = async (taskId, updates) => {
  try {
    await updateTask({ taskId, ...updates })
    // Reload stats after status changes
    if (updates.status) {
      loadTaskStats(projectId.value)
    }
  } catch (err) {
    console.error('Failed to update task:', err)
  }
}

const handleTaskDelete = async (taskId) => {
  try {
    await deleteTask(taskId)
    loadTaskStats(projectId.value) // Refresh stats
  } catch (err) {
    console.error('Failed to delete task:', err)
  }
}

const handleTaskAssign = async (taskId, assigneeId) => {
  try {
    await assignTask(taskId, assigneeId)
  } catch (err) {
    console.error('Failed to assign task:', err)
  }
}

const handleTaskCreated = () => {
  showCreateTaskModal.value = false
  loadProjectTasks(projectId.value) // Refresh task list
  loadTaskStats(projectId.value) // Refresh stats
}

const formatStatusName = (status) => {
  const names = {
    todo: 'To Do',
    in_progress: 'In Progress',
    completed: 'Completed',
    cancelled: 'Cancelled'
  }
  return names[status] || status
}
</script>

<style scoped>
.task-board {
  @apply p-6;
}

.board-header {
  @apply flex justify-between items-center mb-6;
}

.board-actions {
  @apply flex items-center space-x-4;
}

.kanban-board {
  @apply grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6;
}

.kanban-column {
  @apply bg-gray-50 rounded-lg p-4;
}

.column-header {
  @apply flex justify-between items-center mb-4 pb-2 border-b border-gray-200;
}

.column-header h3 {
  @apply font-semibold text-gray-900;
}

.task-count {
  @apply bg-gray-200 text-gray-700 px-2 py-1 rounded-full text-xs;
}

.column-content {
  @apply space-y-3;
}
</style>
```

## 🔒 Permission-Based Access Control Examples

### Project Access Guard

```typescript
// guards/projectGuards.ts
import { taskPermissionService } from '@/services/taskPermissions'
import type { RouteLocationNormalized } from 'vue-router'

export async function canAccessProjects(
  to: RouteLocationNormalized, 
  from: RouteLocationNormalized, 
  next: Function
) {
  const canView = await taskPermissionService.canViewProjects()
  
  if (canView) {
    next()
  } else {
    next('/unauthorized')
  }
}

export async function canManageProject(
  to: RouteLocationNormalized, 
  from: RouteLocationNormalized, 
  next: Function
) {
  const canEdit = await taskPermissionService.canEditProjects()
  
  if (canEdit) {
    next()
  } else {
    next('/projects') // Redirect to view-only
  }
}

// Router configuration
const routes = [
  {
    path: '/projects',
    component: ProjectDashboard,
    beforeEnter: canAccessProjects
  },
  {
    path: '/projects/:id/settings',
    component: ProjectSettings,
    beforeEnter: canManageProject
  }
]
```

### Conditional UI Elements

```vue
<!-- ProjectActions.vue -->
<template>
  <div class="project-actions">
    <!-- Everyone with task_system:read can view -->
    <button @click="viewProject" class="btn btn-secondary">
      View Details
    </button>
    
    <!-- Only users with task_system:write can edit -->
    <button 
      v-if="canEditProjectsSync" 
      @click="editProject" 
      class="btn btn-primary"
    >
      Edit Project
    </button>
    
    <!-- Only users with task_system:user_management can manage members -->
    <button 
      v-if="canManageProjectMembersSync" 
      @click="manageMembers" 
      class="btn btn-secondary"
    >
      Manage Members
    </button>
    
    <!-- Only users with task_system:admin can delete -->
    <button 
      v-if="canDeleteProjectsSync" 
      @click="deleteProject" 
      class="btn btn-danger"
    >
      Delete Project
    </button>
  </div>
</template>

<script setup>
import { taskPermissionService } from '@/services/taskPermissions'

const canEditProjectsSync = taskPermissionService.canEditProjectsSync
const canManageProjectMembersSync = taskPermissionService.canManageProjectMembersSync
const canDeleteProjectsSync = taskPermissionService.canDeleteProjectsSync

// ... methods
</script>
```

## 🚀 Advanced Integration Patterns

### Real-time Task Updates (Optional)

If you add WebSocket support later, you can extend the composables:

```typescript
// composables/useRealtimeTasks.ts
import { useTasks } from './useTasks'
import { useWebSocket } from './useWebSocket'

export function useRealtimeTasks(projectId: string) {
  const { tasks, loadProjectTasks, ...taskMethods } = useTasks()
  const { subscribe } = useWebSocket()

  // Subscribe to real-time task updates
  onMounted(() => {
    subscribe(`project:${projectId}:tasks`, (update) => {
      switch (update.type) {
        case 'TASK_CREATED':
          tasks.value.unshift(update.task)
          break
        case 'TASK_UPDATED':
          const index = tasks.value.findIndex(t => t.id === update.task.id)
          if (index !== -1) {
            tasks.value[index] = update.task
          }
          break
        case 'TASK_DELETED':
          tasks.value = tasks.value.filter(t => t.id !== update.taskId)
          break
      }
    })
  })

  return {
    tasks,
    loadProjectTasks,
    ...taskMethods
  }
}
```

### Offline Support Pattern

```typescript
// composables/useOfflineTasks.ts
import { useTasks } from './useTasks'
import { useOfflineStorage } from './useOfflineStorage'

export function useOfflineTasks() {
  const { tasks, createTask, updateTask, ...taskMethods } = useTasks()
  const { isOnline, queueAction, syncPendingActions } = useOfflineStorage()

  const createTaskOffline = async (input: CreateTaskInput) => {
    if (isOnline.value) {
      return createTask(input)
    } else {
      // Queue for later sync
      const tempId = `temp_${Date.now()}`
      const tempTask = { ...input, id: tempId, status: 'pending_sync' }
      tasks.value.unshift(tempTask)
      queueAction('createTask', input)
      return tempTask
    }
  }

  const updateTaskOffline = async (input: UpdateTaskInput) => {
    if (isOnline.value) {
      return updateTask(input)
    } else {
      // Update locally and queue
      const index = tasks.value.findIndex(t => t.id === input.taskId)
      if (index !== -1) {
        tasks.value[index] = { ...tasks.value[index], ...input }
      }
      queueAction('updateTask', input)
    }
  }

  // Auto-sync when coming back online
  watch(isOnline, (online) => {
    if (online) {
      syncPendingActions()
    }
  })

  return {
    tasks,
    createTask: createTaskOffline,
    updateTask: updateTaskOffline,
    ...taskMethods
  }
}
```

## 📊 Analytics and Reporting

### Task Analytics Composable

```typescript
// composables/useTaskAnalytics.ts
import { computed } from 'vue'
import { useTasks } from './useTasks'

export function useTaskAnalytics() {
  const { tasks, taskStats } = useTasks()

  const completionRate = computed(() => {
    if (!taskStats.value || taskStats.value.total === 0) return 0
    return (taskStats.value.completed / taskStats.value.total) * 100
  })

  const productivityTrend = computed(() => {
    // Calculate tasks completed in the last 7 days
    const weekAgo = new Date()
    weekAgo.setDate(weekAgo.getDate() - 7)
    
    return tasks.value.filter(task => 
      task.status === 'completed' && 
      new Date(task.updatedAt) >= weekAgo
    ).length
  })

  const priorityDistribution = computed(() => {
    const distribution = { low: 0, medium: 0, high: 0, urgent: 0 }
    tasks.value.forEach(task => {
      distribution[task.priority]++
    })
    return distribution
  })

  return {
    completionRate,
    productivityTrend,
    priorityDistribution,
    taskStats
  }
}
```

## 🎯 Next Steps for Frontend Implementation

### Phase 1: Core Features
1. **Basic Task Management**:
   - Implement project listing and creation
   - Build CRUD operations for tasks
   - Add permission-based UI controls

2. **Type-Safe Implementation**:
   - Generate TypeScript types from GraphQL schema
   - Utilize enum introspection for dropdowns and validation
   - Implement proper error handling with typed responses

### Phase 2: Recurring Tasks
1. **Recurrence UI Components**:
   - Create recurrence pattern selector component
   - Add recurrence description display in task cards
   - Implement recurring task completion with next instance preview

2. **Recurring Task Management**:
   - Show parent-child relationship in task lists
   - Add filtering for recurring vs one-time tasks
   - Create recurring task template editing

### Phase 3: Activity & Comments System
1. **Activity Timeline**:
   - Build activity feed component with filtering
   - Implement real-time activity updates
   - Add user mentions and notifications

2. **Comments & Collaboration**:
   - Create comment input with rich text support
   - Add @mentions with user autocomplete
   - Implement activity-based notifications

### Phase 4: Advanced Features
1. **Enhanced User Experience**:
   - Add drag-and-drop for task status changes
   - Implement advanced filtering (status, assignee, recurrence, etc.)
   - Create comprehensive dashboard with statistics

2. **Analytics & Reporting**:
   - Task completion trends over time
   - Recurring task performance metrics
   - Activity-based productivity insights

### Phase 5: Performance & Scale
1. **Optimization**:
   - Implement proper caching strategies for activities
   - Add optimistic UI updates for comments
   - Consider virtual scrolling for activity feeds

2. **Real-time Features**:
   - WebSocket integration for live activity updates
   - Real-time collaboration indicators
   - Live task status synchronization

## 🔍 GraphQL Introspection for Type Generation

Use the enhanced enum system for automatic TypeScript generation:

```bash
# Generate types with proper enum support
npx graphql-codegen --config codegen.yml
```

**codegen.yml example:**
```yaml
generates:
  src/generated/graphql.ts:
    plugins:
      - typescript
      - typescript-operations
    config:
      enumsAsTypes: true          # Generate enums as union types
      maybeValue: T | null        # Handle nullable types properly
      skipTypename: false         # Include __typename for better caching
```

This will generate proper TypeScript types like:

```typescript
export type TaskStatus = 'TODO' | 'IN_PROGRESS' | 'COMPLETED' | 'CANCELLED'
export type RecurrenceType = 'NONE' | 'DAILY' | 'WEEKDAYS' | 'WEEKLY' | 'MONTHLY'
```

## 🚀 Key Benefits of the Enhanced System

1. **Type Safety**: Full type safety from database to frontend with GraphQL introspection
2. **Simplicity**: Simple enum-based recurrence patterns instead of complex cron expressions  
3. **Flexibility**: Generic activity system supports future entity types seamlessly
4. **Performance**: Efficient database queries with proper indexing and pagination
5. **Scalability**: Activity logging scales to millions of records with minimal impact
6. **User Experience**: Rich activity feeds and commenting system enhance collaboration

The enhanced task system maintains all existing security and performance patterns while adding powerful new capabilities. The type-safe enum system ensures consistency across your entire application stack, from database constraints to frontend validation.