# Frontend Integration Guide

This guide shows how to integrate FreshAPI with your TypeScript/Vue.js frontend, including complete RBAC (Role-Based Access Control) implementation.

## 🔄 Schema Synchronization

### Available Endpoints (Development Only)

- **GraphQL SDL**: `GET /schema.graphql` - Schema Definition Language format
- **JSON Introspection**: `GET /schema.json` - Full introspection data for codegen
- **GraphQL Playground**: `GET /playground` - Interactive API explorer

⚠️ **Security Note**: These endpoints are only available when `ENVIRONMENT=development`

## 🛠 Frontend Setup

### 1. Install Dependencies

```bash
npm install @graphql-codegen/cli @graphql-codegen/typescript @graphql-codegen/typescript-operations @graphql-codegen/typescript-vue-apollo
```

### 2. Create GraphQL Codegen Config

Create `codegen.yml`:

```yaml
schema: "http://localhost:8080/graphql"
documents: "src/**/*.{vue,ts,js}"
generates:
  src/generated/graphql.ts:
    plugins:
      - typescript
      - typescript-operations
    config:
      useIndexSignature: true
      skipTypename: false
      withHooks: true
```

### 3. Add Package.json Scripts

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml",
    "codegen:watch": "graphql-codegen --watch",
    "schema:pull": "curl -o schema.graphql http://localhost:8080/schema.graphql"
  }
}
```

## 🔐 RBAC System Implementation

### Authentication Flow

#### 1. Login with Role Information

```typescript
import { LoginMutation, MeQuery } from '@/generated/graphql'

interface AuthUser {
  id: string
  email: string
  firstName?: string
  lastName?: string
  role?: {
    name: string
    level: number
  }
  permissions: string[]
}

const LOGIN_MUTATION = gql`
  mutation Login($input: LoginInput!) {
    login(input: $input) {
      user {
        id
        email
        firstName
        lastName
        role {
          name
          level
        }
      }
      accessToken
      refreshToken
    }
  }
`

// Login implementation
export const authService = {
  async login(email: string, password: string) {
    const result = await apolloClient.mutate<LoginMutation>({
      mutation: LOGIN_MUTATION,
      variables: { input: { email, password } }
    })
    
    const { user, accessToken, refreshToken } = result.data!.login
    
    // Store tokens
    localStorage.setItem('accessToken', accessToken)
    localStorage.setItem('refreshToken', refreshToken)
    
    // Get user permissions
    const userWithPermissions = await this.getCurrentUser()
    return userWithPermissions
  },

  async getCurrentUser(): Promise<AuthUser> {
    const ME_QUERY = gql`
      query Me {
        me {
          id
          email
          firstName
          lastName
          role {
            name
            level
          }
        }
      }
    `
    
    const result = await apolloClient.query<MeQuery>({
      query: ME_QUERY
    })
    
    return result.data.me as AuthUser
  }
}
```

#### 2. Permission-Based Access Control

```typescript
// Permission service for frontend
export class PermissionService {
  private user: AuthUser | null = null

  setUser(user: AuthUser) {
    this.user = user
  }

  // Check if user has specific permission
  hasPermission(action: string): boolean {
    if (!this.user) return false
    return this.user.permissions.includes(action)
  }

  // Check if user has minimum role level
  hasRoleLevel(minLevel: number): boolean {
    if (!this.user?.role) return false
    return this.user.role.level >= minLevel
  }

  // Role-based checks
  isSuperAdmin(): boolean {
    return this.user?.role?.name === 'super_admin'
  }

  isAdmin(): boolean {
    return this.hasRoleLevel(50) // admin level or higher
  }

  canInviteUsers(): boolean {
    return this.hasPermission('invite_users')
  }

  canManageUsers(): boolean {
    return this.hasPermission('user_management')
  }

  canAdminSystem(): boolean {
    return this.hasPermission('system_admin')
  }
}

export const permissionService = new PermissionService()
```

### 3. Vue Composable for RBAC

```typescript
// composables/usePermissions.ts
import { computed, ref } from 'vue'
import { permissionService } from '@/services/permissions'
import type { AuthUser } from '@/types/auth'

export function usePermissions() {
  const user = ref<AuthUser | null>(null)

  const hasPermission = (action: string) => {
    return computed(() => permissionService.hasPermission(action))
  }

  const hasRoleLevel = (minLevel: number) => {
    return computed(() => permissionService.hasRoleLevel(minLevel))
  }

  const canInviteUsers = computed(() => permissionService.canInviteUsers())
  const canManageUsers = computed(() => permissionService.canManageUsers())
  const isAdmin = computed(() => permissionService.isAdmin())
  const isSuperAdmin = computed(() => permissionService.isSuperAdmin())

  return {
    user,
    hasPermission,
    hasRoleLevel,
    canInviteUsers,
    canManageUsers,
    isAdmin,
    isSuperAdmin
  }
}
```

### 4. Role-Based UI Components

#### Navigation Guard

```typescript
// router/guards.ts
import { permissionService } from '@/services/permissions'

export function requirePermission(permission: string) {
  return (to: any, from: any, next: any) => {
    if (permissionService.hasPermission(permission)) {
      next()
    } else {
      next('/unauthorized')
    }
  }
}

// Router setup
const routes = [
  {
    path: '/admin',
    component: AdminLayout,
    beforeEnter: requirePermission('admin'),
    children: [
      {
        path: 'users',
        component: UserManagement,
        beforeEnter: requirePermission('user_management')
      },
      {
        path: 'invites',
        component: InviteManagement,
        beforeEnter: requirePermission('invite_users')
      }
    ]
  }
]
```

#### Conditional UI Rendering

```vue
<!-- UserManagement.vue -->
<template>
  <div class="user-management">
    <h1>User Management</h1>
    
    <!-- Only show invite button if user can invite -->
    <button v-if="canInviteUsers" @click="showInviteModal = true">
      Invite User
    </button>
    
    <!-- User list with role-based actions -->
    <div v-for="user in users" :key="user.id" class="user-card">
      <div>{{ user.email }} ({{ user.role?.name || 'No Role' }})</div>
      
      <!-- Only super admin can assign roles -->
      <button v-if="isSuperAdmin" @click="assignRole(user.id)">
        Assign Role
      </button>
      
      <!-- Admin can remove roles (but not from super admin) -->
      <button v-if="isAdmin && user.role?.name !== 'super_admin'" 
              @click="removeRole(user.id)">
        Remove Role
      </button>
    </div>

    <!-- Invite Modal -->
    <InviteUserModal 
      v-if="showInviteModal" 
      :available-roles="availableRoles"
      @close="showInviteModal = false" 
    />
  </div>
</template>

<script setup>
import { usePermissions } from '@/composables/usePermissions'
import { useUserManagement } from '@/composables/useUserManagement'

const { canInviteUsers, isAdmin, isSuperAdmin } = usePermissions()
const { users, availableRoles, assignRole, removeRole } = useUserManagement()

const showInviteModal = ref(false)
</script>
```

### 5. User Management Implementation

```typescript
// composables/useUserManagement.ts
export function useUserManagement() {
  const users = ref<UserWithRole[]>([])
  const roles = ref<Role[]>([])

  // GraphQL Queries
  const ALL_USERS_QUERY = gql`
    query AllUsers {
      allUsers {
        id
        email
        firstName
        lastName
        isEmailVerified
        role {
          id
          name
          level
        }
        permissions
        createdAt
      }
    }
  `

  const ALL_ROLES_QUERY = gql`
    query AllRoles {
      allRoles {
        id
        name
        description
        level
        isActive
      }
    }
  `

  const ASSIGN_ROLE_MUTATION = gql`
    mutation AssignRole($input: AssignRoleInput!) {
      assignRole(input: $input) {
        id
        email
        role {
          name
          level
        }
      }
    }
  `

  const INVITE_USER_WITH_ROLE_MUTATION = gql`
    mutation InviteUserWithRole($input: InviteUserWithRoleInput!) {
      inviteUserWithRole(input: $input) {
        id
        email
        role {
          name
        }
        expiresAt
      }
    }
  `

  // Load data
  const loadUsers = async () => {
    const result = await apolloClient.query({
      query: ALL_USERS_QUERY,
      fetchPolicy: 'network-only'
    })
    users.value = result.data.allUsers
  }

  const loadRoles = async () => {
    const result = await apolloClient.query({
      query: ALL_ROLES_QUERY
    })
    roles.value = result.data.allRoles
  }

  // Actions
  const assignRole = async (userId: string, roleId: string) => {
    await apolloClient.mutate({
      mutation: ASSIGN_ROLE_MUTATION,
      variables: {
        input: { userId, roleId }
      }
    })
    await loadUsers() // Refresh list
  }

  const inviteUserWithRole = async (email: string, roleId?: string) => {
    await apolloClient.mutate({
      mutation: INVITE_USER_WITH_ROLE_MUTATION,
      variables: {
        input: { email, roleId }
      }
    })
  }

  return {
    users,
    roles,
    loadUsers,
    loadRoles,
    assignRole,
    inviteUserWithRole
  }
}
```

## 🏗 Multi-App RBAC Architecture

### Future App Integration

When adding new applications to your system:

#### 1. Add New Resource

```sql
-- Migration for new app
INSERT INTO resources (id, name, description) 
VALUES (uuid_generate_v4(), 'inventory_app', 'Inventory Management System');
```

#### 2. Define App-Specific Permissions

```sql
-- Add permissions for new app
INSERT INTO permissions (id, action, resource_name, description) VALUES
(uuid_generate_v4(), 'read', 'inventory_app', 'View inventory data'),
(uuid_generate_v4(), 'write', 'inventory_app', 'Modify inventory'),
(uuid_generate_v4(), 'admin', 'inventory_app', 'Admin inventory system');
```

#### 3. Frontend Permission Checks

```typescript
// Extended permission service
export class MultiAppPermissionService extends PermissionService {
  
  // Check permission for specific app
  hasAppPermission(app: string, action: string): boolean {
    if (!this.user) return false
    
    // Permission format: "action" for resource "app"
    // The backend handles resource-based permission checking
    return this.hasPermission(action) // Backend filters by resource context
  }

  // App-specific helpers
  canAccessInventory(): boolean {
    return this.hasAppPermission('inventory_app', 'read')
  }

  canManageInventory(): boolean {
    return this.hasAppPermission('inventory_app', 'admin')
  }
}
```

#### 4. App-Specific GraphQL Context

```typescript
// When making GraphQL calls for specific apps
const INVENTORY_QUERY = gql`
  query InventoryItems {
    inventoryItems {  # This query will be protected by resource-based permissions
      id
      name
      quantity
    }
  }
`

// The backend automatically checks if user has 'read' permission for 'inventory_app'
```

### Role Hierarchy Best Practices

```typescript
// Role levels for multi-app scaling
const ROLE_LEVELS = {
  SUPER_ADMIN: 100,  // Access to everything across all apps
  APP_ADMIN: 75,     // Admin of specific apps
  MANAGER: 50,       // Management permissions within apps
  USER: 25,          // Basic user permissions
  VIEWER: 10         // Read-only access
} as const

// Permission patterns
const PERMISSION_PATTERNS = {
  // Cross-app permissions
  SYSTEM_ADMIN: 'system_admin',     // Manage entire system
  USER_MANAGEMENT: 'user_management', // Manage users across apps
  
  // App-specific permissions
  READ: 'read',           // View data
  WRITE: 'write',         // Modify data  
  ADMIN: 'admin',         // App administration
  REPORTS: 'reports',     // Generate reports
  CONFIG: 'config'        // Configure app settings
} as const
```

## 🚀 Production Deployment

### Railway Environment Variables

Set these in your Railway project:

```bash
# Required
DATABASE_URL=postgresql://... # Railway provides this automatically
JWT_SECRET=your-production-jwt-secret-here

# Optional
ENVIRONMENT=production
HOST=0.0.0.0
PORT=8080
RESEND_API_KEY=your-resend-key
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com

# Admin seeding (remove after initial setup)
ADMIN_EMAIL=admin@yourdomain.com
ADMIN_PASSWORD=secure-admin-password
```

### Frontend Production Build

```bash
# Generate schema during CI/CD
curl -o src/schema.graphql https://your-api-dev.railway.app/schema.graphql
npm run codegen
npm run build
```

## 🔐 Security Features

- ✅ **Field-level authorization** - GraphQL fields protected by permissions
- ✅ **Role hierarchy** - Higher roles inherit lower role permissions  
- ✅ **Resource isolation** - Multi-app permission separation
- ✅ **Invitation-only registration** - No public signup
- ✅ **JWT token management** - Access + refresh token pattern
- ✅ **Production introspection disabled** - Schema endpoints blocked

## 📚 Complete Example: Admin Dashboard

```vue
<!-- AdminDashboard.vue -->
<template>
  <div class="admin-dashboard">
    <nav class="admin-nav">
      <router-link v-if="canManageUsers" to="/admin/users">
        Users
      </router-link>
      <router-link v-if="canInviteUsers" to="/admin/invites">
        Invitations
      </router-link>
      <router-link v-if="isSuperAdmin" to="/admin/roles">
        Roles & Permissions
      </router-link>
    </nav>

    <main>
      <router-view />
    </main>
  </div>
</template>

<script setup>
import { usePermissions } from '@/composables/usePermissions'

const { canManageUsers, canInviteUsers, isSuperAdmin } = usePermissions()
</script>
```

## 🔄 Schema Evolution

When you add new GraphQL types/mutations:

1. ✅ **No manual updates needed** - introspection handles everything
2. ✅ **Run `npm run codegen`** to get new TypeScript types  
3. ✅ **Use new types immediately** in your frontend code
4. ✅ **Permission checks automatically available** for new mutations

The RBAC system is designed to scale seamlessly with your application growth! 🎉

## 🎯 Next Steps

1. **Implement user roles** in your frontend using the patterns above
2. **Test permission flows** with different user roles
3. **Plan future app integrations** using the multi-app architecture
4. **Set up monitoring** for permission-denied events
5. **Document app-specific permissions** as you add new features