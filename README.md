# FreshAPI

A modern GraphQL API with enterprise-grade Role-Based Access Control (RBAC), designed to power multi-application projects with comprehensive user management and security features.

## Overview

FreshAPI provides a robust, scalable foundation for building multi-application projects with comprehensive security and user management. Built with Rust, GraphQL, and PostgreSQL, it offers:

- **Role-Based Access Control (RBAC)**: Multi-layered permission system with hierarchical roles
- **User Management**: Complete user lifecycle with invitation-based registration
- **GraphQL API**: Modern, type-safe API with field-level authorization
- **JWT Authentication**: Secure token-based authentication with refresh tokens  
- **Multi-App Architecture**: Resource-based permissions for scaling across applications
- **Admin Management**: Comprehensive user and role management capabilities
- **Email Integration**: User verification and notifications via Resend
- **Frontend Ready**: Designed for TypeScript/Vue.js with automatic type generation
- **Docker Support**: Containerized PostgreSQL for local development

## Tech Stack

- **Backend**: Rust with Axum web framework
- **GraphQL**: async-graphql for schema definition and resolvers
- **Database**: PostgreSQL with SeaORM and comprehensive migrations
- **Authentication**: JWT tokens with bcrypt password hashing
- **Authorization**: Role-Based Access Control (RBAC) with hierarchical permissions
- **Email**: Resend service integration
- **Frontend Integration**: GraphQL introspection for TypeScript generation

## Quick Start

### Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- Git

### Local Development Setup

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd freshapi
   ```

2. **Start PostgreSQL**:
   ```bash
   docker-compose up -d
   ```

3. **Configure environment**:
   - Copy `.env` and update values as needed
   - Set your `RESEND_API_KEY` for email functionality
   - Update `JWT_SECRET` for production use

4. **Run migrations**:
   ```bash
   cargo run --bin migration
   ```

5. **Start the API**:
   ```bash
   cargo run
   ```

The API will be available at `http://localhost:8080/graphql` with GraphQL Playground enabled in development.

### Database Management

- **PostgreSQL**: Running on `localhost:5432`
- **Adminer**: Web interface at `http://localhost:8081`
- **Credentials**: See `.env` file

## API Documentation

### GraphQL Introspection

The API supports full GraphQL introspection for automatic documentation and TypeScript generation:

```bash
# Generate schema for frontend
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "query IntrospectionQuery { __schema { types { name } } }"}' \
  > schema.json
```

### Frontend Integration

For TypeScript/Vue.js projects:

1. Use GraphQL Code Generator with the introspection endpoint
2. Configure CORS origins in `.env` (`CORS_ALLOWED_ORIGINS`)
3. Generate TypeScript types from the GraphQL schema

## Development

### Available Commands

```bash
# Development
cargo run              # Start the server
cargo test             # Run tests
cargo check            # Type check without building
cargo clippy           # Linting
cargo fmt              # Code formatting

# Database
docker-compose up -d   # Start PostgreSQL
docker-compose down    # Stop services
cargo run --bin migration  # Run database migrations

# Production
cargo build --release  # Optimized build
```

### Project Structure

```
src/
├── main.rs           # Application entry point
├── entities/         # Database models (users, roles, permissions)
├── graphql/          # GraphQL schema and resolvers
├── auth/             # Authentication & authorization (RBAC)
├── services/         # Business logic services
└── migration/        # Database migrations
```

## RBAC System Architecture

### Permission Model

FreshAPI implements a sophisticated **Users → Roles → Permissions → Resources** architecture:

- **Resources**: Applications/modules (e.g., `freshapi`, future apps)
- **Permissions**: Actions on resources (`read`, `write`, `admin`, `user_management`, `system_admin`)
- **Roles**: Permission collections with hierarchy levels (`super_admin=100`, `admin=50`, `user=10`)
- **Users**: Assigned to roles + optional direct permission overrides

### Default Roles

- **super_admin** (Level 100): Full system access, all permissions
- **admin** (Level 50): User management, admin operations  
- **user** (Level 10): Basic read/write access

### Authorization Features

- **Hierarchical Permissions**: Higher role levels inherit lower permissions
- **Resource-Based**: Scale permissions across multiple applications
- **GraphQL Guards**: Field-level authorization checks
- **Direct Overrides**: Grant/deny specific user permissions
- **Admin Management**: Role assignment and user management

## User Management Features

### Core Functionality

- **Invitation-Based Registration**: Secure user onboarding system
- **Authentication**: JWT-based login/logout with refresh tokens
- **Role Management**: Comprehensive RBAC with admin controls
- **Profile Management**: User profile CRUD operations
- **Password Management**: Secure password reset flow
- **Email Verification**: Account activation and notifications

### GraphQL API

GraphQL endpoint: `POST /graphql`

#### Public Mutations
- `login`: Authenticate and receive JWT tokens
- `acceptInvitation`: Register via invitation token
- `refreshToken`: Get new access token
- `requestPasswordReset`: Initiate password reset
- `resetPassword`: Complete password reset

#### Authenticated Queries
- `me`: Get current user profile
- `myInvitations`: List user's sent invitations

#### Admin-Only Operations (require admin/user_management permissions)
- `allUsers`: List all users with roles and permissions
- `allRoles`: List available roles
- `assignRole`: Assign role to user
- `removeUserRole`: Remove user's role
- `inviteUser`: Send user invitation
- `userPermissions`: Check user's permissions

## Configuration

### Environment Variables

#### Required for Railway Deployment:
- `DATABASE_URL`: PostgreSQL connection string (Railway provides automatically)
- `JWT_SECRET`: Secure secret key for JWT signing (**CRITICAL - must set**)
- `ENVIRONMENT`: Set to "production" (Railway sets automatically)

#### Optional Configuration:
- `RESEND_API_KEY`: Email service API key for email functionality
- `CORS_ALLOWED_ORIGINS`: Frontend domains for CORS
  - Production: `https://your-app.com,https://www.your-app.com`
  - Development: `http://localhost:3000,http://localhost:5173`
  - **Any origin**: `*` (⚠️ **DANGEROUS** - development only!)
- `HOST`: Server host (default: `0.0.0.0`)
- `PORT`: Server port (default: `8080`)
- `JWT_EXPIRATION_HOURS`: Token expiration time (default: `24`)

#### Admin User Seeding (Initial Setup Only):
- `ADMIN_EMAIL`: Admin user email
- `ADMIN_PASSWORD`: Admin user password
- `ADMIN_FIRST_NAME`: Admin first name (optional)
- `ADMIN_LAST_NAME`: Admin last name (optional)

**Note**: Remove admin credentials from environment after initial setup for security.

### Railway Deployment

1. **Connect Repository**: Link your GitHub repo to Railway
2. **Set Environment Variables**: Configure required variables in Railway dashboard
3. **Deploy**: Railway automatically builds and deploys your API
4. **Database**: Railway provisions PostgreSQL and sets `DATABASE_URL`

### Security Considerations

- **CRITICAL**: Set secure `JWT_SECRET` in production (not the default)
- Schema introspection automatically disabled in production
- Admin credentials should be removed after initial setup
- Configure proper CORS origins for your frontend domains
- Railway automatically enables HTTPS

## Contributing

### Documentation Rules

- **Always read existing documentation before editing**
- **Update CHANGELOG.md for each commit**
- **Maintain API documentation with code changes**
- **Update README.md when adding features**

### Commit Log

All changes are tracked in `CHANGELOG.md` to monitor repository evolution and maintain clear project history.

## License

MIT License

Copyright (c) 2025 [Author] <marcosmontiel791@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Support

This is a personal project for experimental and learning purposes. For issues or questions, please use the GitHub issue tracker.