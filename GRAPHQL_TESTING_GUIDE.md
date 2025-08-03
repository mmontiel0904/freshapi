# FreshAPI GraphQL Testing Guide

This guide provides comprehensive examples for testing all available GraphQL endpoints in the playground at `http://localhost:8080/playground`.

## 🔗 GraphQL Endpoint
- **URL**: `http://localhost:8080/graphql`
- **Playground**: `http://localhost:8080/playground`

## 📋 Table of Contents
1. [Public Endpoints](#public-endpoints)
2. [Authentication Required](#authentication-required)
3. [Admin-Only Endpoints](#admin-only-endpoints)
4. [Permission-Based Endpoints](#permission-based-endpoints)
5. [Headers Configuration](#headers-configuration)
6. [Common Variables](#common-variables)

---

## 🌐 Public Endpoints
*No authentication required*

### 🔍 Health Check
```graphql
query HealthCheck {
  health
}
```

### 🔑 Login
```graphql
mutation Login($input: LoginInput!) {
  login(input: $input) {
    user {
      id
      email
      firstName
      lastName
      isEmailVerified
      role {
        id
        name
        description
        level
      }
      permissions
      createdAt
      updatedAt
    }
    accessToken
    refreshToken
  }
}
```

**Variables:**
```json
{
  "input": {
    "email": "admin@example.com",
    "password": "admin123"
  }
}
```

### 🎟️ Accept Invitation
```graphql
mutation AcceptInvitation($input: AcceptInvitationInput!) {
  acceptInvitation(input: $input) {
    user {
      id
      email
      firstName
      lastName
      isEmailVerified
      role {
        name
        level
      }
    }
    accessToken
    refreshToken
  }
}
```

**Variables:**
```json
{
  "input": {
    "invitationToken": "invitation-token-here",
    "password": "newpassword123",
    "firstName": "John",
    "lastName": "Doe"
  }
}
```

### 🔄 Refresh Token
```graphql
mutation RefreshToken($input: RefreshTokenInput!) {
  refreshToken(input: $input) {
    user {
      id
      email
      firstName
      lastName
    }
    accessToken
    refreshToken
  }
}
```

**Variables:**
```json
{
  "input": {
    "refreshToken": "your-refresh-token-here"
  }
}
```

### 🔐 Request Password Reset
```graphql
mutation RequestPasswordReset($input: RequestPasswordResetInput!) {
  requestPasswordReset(input: $input) {
    message
  }
}
```

**Variables:**
```json
{
  "input": {
    "email": "user@example.com"
  }
}
```

### 🔑 Reset Password
```graphql
mutation ResetPassword($input: ResetPasswordInput!) {
  resetPassword(input: $input) {
    message
  }
}
```

**Variables:**
```json
{
  "input": {
    "token": "password-reset-token-here",
    "newPassword": "newpassword123"
  }
}
```

### ✅ Verify Email
```graphql
mutation VerifyEmail($token: String!) {
  verifyEmail(token: $token) {
    message
  }
}
```

**Variables:**
```json
{
  "token": "email-verification-token-here"
}
```

### 🚫 Register (Disabled)
```graphql
mutation Register($input: RegisterInput!) {
  register(input: $input) {
    id
    email
  }
}
```
*Note: This will return an error as public registration is disabled. Use invitation flow instead.*

---

## 🔐 Authentication Required
*Requires Authorization header with Bearer token*

### 👤 Get Current User
```graphql
query Me {
  me {
    id
    email
    firstName
    lastName
    isEmailVerified
    role {
      id
      name
      description
      level
      isActive
    }
    permissions
    createdAt
    updatedAt
  }
}
```

### 🚪 Logout
```graphql
mutation Logout {
  logout {
    message
  }
}
```

### 🔑 Change Password
```graphql
mutation ChangePassword($input: ChangePasswordInput!) {
  changePassword(input: $input) {
    message
  }
}
```

**Variables:**
```json
{
  "input": {
    "currentPassword": "currentpassword123",
    "newPassword": "newpassword123"
  }
}
```

---

## 🛠️ Permission-Based Endpoints
*Requires specific permissions*

### 📨 My Invitations
*Requires: `invite_users` permission*

```graphql
query MyInvitations {
  myInvitations {
    id
    email
    inviterUserId
    expiresAt
    isUsed
    usedAt
    role {
      id
      name
      level
    }
    createdAt
  }
}
```

### 📧 Invite User
*Requires: `invite_users` permission*

```graphql
mutation InviteUser($input: InviteUserInput!) {
  inviteUser(input: $input) {
    id
    email
    inviterUserId
    expiresAt
    isUsed
    createdAt
  }
}
```

**Variables:**
```json
{
  "input": {
    "email": "newuser@example.com"
  }
}
```

---

## 👑 Admin-Only Endpoints
*Requires admin permissions or user_management permissions*

### 👥 Get All Users
*Requires: admin permissions*

```graphql
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
      description
      level
      isActive
    }
    permissions
    createdAt
    updatedAt
  }
}
```

### 🏷️ Get All Roles
*Requires: admin permissions*

```graphql
query AllRoles {
  allRoles {
    id
    name
    description
    level
    isActive
    createdAt
    updatedAt
  }
}
```

### 👤 Get User by ID
*Requires: admin permissions*

```graphql
query UserById($userId: UUID!) {
  userById(userId: $userId) {
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
    updatedAt
  }
}
```

**Variables:**
```json
{
  "userId": "user-uuid-here"
}
```

### 🔍 Users by Role
*Requires: admin permissions*

```graphql
query UsersByRole($roleName: String!) {
  usersByRole(roleName: $roleName) {
    id
    email
    firstName
    lastName
    role {
      name
      level
    }
    permissions
    createdAt
  }
}
```

**Variables:**
```json
{
  "roleName": "admin"
}
```

### 🔐 User Permissions
*Requires: admin permissions*

```graphql
query UserPermissions($userId: UUID!) {
  userPermissions(userId: $userId)
}
```

**Variables:**
```json
{
  "userId": "user-uuid-here"
}
```

### 📧 Invite User with Role
*Requires: user_management permissions*

```graphql
mutation InviteUserWithRole($input: InviteUserWithRoleInput!) {
  inviteUserWithRole(input: $input) {
    id
    email
    inviterUserId
    expiresAt
    role {
      id
      name
      level
    }
    createdAt
  }
}
```

**Variables:**
```json
{
  "input": {
    "email": "newadmin@example.com",
    "roleId": "role-uuid-here"
  }
}
```

### 🏷️ Assign Role
*Requires: user_management permissions*

```graphql
mutation AssignRole($input: AssignRoleInput!) {
  assignRole(input: $input) {
    id
    email
    firstName
    lastName
    role {
      id
      name
      level
    }
    updatedAt
  }
}
```

**Variables:**
```json
{
  "input": {
    "userId": "user-uuid-here",
    "roleId": "role-uuid-here"
  }
}
```

### ❌ Remove User Role
*Requires: user_management permissions*

```graphql
mutation RemoveUserRole($userId: UUID!) {
  removeUserRole(userId: $userId) {
    id
    email
    firstName
    lastName
    role {
      name
    }
    updatedAt
  }
}
```

**Variables:**
```json
{
  "userId": "user-uuid-here"
}
```

### 🔐 Admin Reset User Password
*Requires: user_management permissions*

```graphql
mutation AdminResetUserPassword($input: AdminResetUserPasswordInput!) {
  adminResetUserPassword(input: $input) {
    message
  }
}
```

**Variables:**
```json
{
  "input": {
    "userId": "user-uuid-here"
  }
}
```

---

## 🔧 Headers Configuration

### For Authentication Required Endpoints
Add this header in the GraphQL Playground:

```json
{
  "Authorization": "Bearer your-access-token-here"
}
```

### How to Get Access Token
1. Use the Login mutation first
2. Copy the `accessToken` from the response
3. Add it to the Authorization header for subsequent requests

---

## 📝 Common Variables

### UUIDs (Replace with actual values)
```json
{
  "userId": "550e8400-e29b-41d4-a716-446655440000",
  "roleId": "550e8400-e29b-41d4-a716-446655440001",
  "invitationToken": "abc123def456ghi789",
  "verificationToken": "email-verification-token-here",
  "resetToken": "password-reset-token-here"
}
```

### Example Admin Login
```json
{
  "input": {
    "email": "admin@example.com",
    "password": "admin123"
  }
}
```

---

## 🚀 Quick Start Testing Flow

### 1. Health Check
```graphql
query { health }
```

### 2. Admin Login
```graphql
mutation Login($input: LoginInput!) {
  login(input: $input) {
    user { id email role { name } }
    accessToken
  }
}
```
Variables: `{ "input": { "email": "admin@example.com", "password": "admin123" } }`

### 3. Get Current User (with token)
```graphql
query { me { id email role { name } permissions } }
```

### 4. List All Users (admin only)
```graphql
query { allUsers { id email role { name } permissions } }
```

### 5. Invite New User
```graphql
mutation InviteUser($input: InviteUserInput!) {
  inviteUser(input: $input) {
    id email expiresAt
  }
}
```
Variables: `{ "input": { "email": "newuser@example.com" } }`

---

## 🛡️ Permission Levels

| Role | Level | Permissions |
|------|-------|-------------|
| super_admin | 100 | All permissions |
| admin | 50 | Most admin operations |
| user | 10 | Basic user operations |

## 📚 Error Handling

Common error responses:
- `Authentication failed` - Invalid credentials
- `User not found` - Invalid user ID
- `Permission denied` - Insufficient permissions
- `Invalid invitation` - Expired or used invitation token
- `Email verification failed` - Invalid verification token

---

## 🔗 Useful Links

- **GraphQL Playground**: http://localhost:8080/playground
- **Schema SDL** (dev only): http://localhost:8080/schema.graphql
- **Schema JSON** (dev only): http://localhost:8080/schema.json
- **Health Check**: http://localhost:8080/health

---

*Happy testing! 🚀*
