# Task System Frontend Integration Guide

This guide shows how to integrate the new **Task Tracking System** with your TypeScript/Vue.js frontend. This complements the main [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) document by focusing specifically on project and task management features.

## 🎯 Task System Overview

The task system provides comprehensive project and task management with:
- **Project Management**: Create, manage, and organize projects with team members
- **Task Tracking**: Full task lifecycle with status, priority, and assignment management
- **Role-Based Access**: Project-level permissions with hierarchical roles
- **RBAC Integration**: Seamless integration with the existing permission system

## 📋 Schema Reference

### Key GraphQL Types

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
  tasks(status: String, assigneeId: UUID, limit: Int, offset: Int): [Task!]!
}

type Task {
  id: UUID!
  name: String!
  description: String
  projectId: UUID!
  assigneeId: UUID
  creatorId: UUID!
  status: String!        # "todo", "in_progress", "completed", "cancelled"
  priority: String!      # "low", "medium", "high", "urgent"
  dueDate: DateTime
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Related data (auto-resolved)
  project: Project
  assignee: User
  creator: User!
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

  // Task status and priority constants
  const TASK_STATUS = {
    TODO: 'todo',
    IN_PROGRESS: 'in_progress',
    COMPLETED: 'completed',
    CANCELLED: 'cancelled'
  } as const

  const TASK_PRIORITY = {
    LOW: 'low',
    MEDIUM: 'medium',
    HIGH: 'high',
    URGENT: 'urgent'
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

1. **Start with Core Features**:
   - Implement project listing and creation
   - Build basic task management (CRUD operations)
   - Add permission-based UI controls

2. **Enhance User Experience**:
   - Add drag-and-drop for task status changes
   - Implement task filtering and search
   - Create dashboard with task statistics

3. **Advanced Features**:
   - Add due date notifications
   - Implement task assignment workflows
   - Create reporting and analytics views

4. **Performance Optimization**:
   - Implement proper caching strategies
   - Add optimistic UI updates
   - Consider virtual scrolling for large task lists

5. **Mobile Responsiveness**:
   - Optimize task boards for mobile devices
   - Add touch-friendly interactions
   - Consider progressive web app features

The task system is designed to scale with your application needs while maintaining the security and performance patterns established in the main [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) guide. All permission checks work seamlessly with the existing RBAC system, ensuring consistent security across your entire application.