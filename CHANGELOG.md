# Changelog

All notable changes to FreshAPI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete user management system with registration, login, and profile management
- PostgreSQL database with user table and migration system
- JWT authentication with secure token generation and verification
- GraphQL API with mutations (register, login, verifyEmail) and queries (me, health)
- Email service integration with verification email flow (console logging for development)
- Optional authentication middleware supporting both public and authenticated endpoints
- GraphQL Playground for API testing and development
- Health check endpoint for monitoring
- Comprehensive .gitignore for Rust projects
- SeaORM entity generation from database schema
- Password hashing with bcrypt
- CORS configuration for frontend development (Vue.js/TypeScript ready)
- Environment-based configuration with sensible defaults
- Admin user seeding system with environment variable configuration
- Database migration for secure admin user creation with bcrypt password hashing
- Admin user seeding with configurable credentials via ADMIN_EMAIL, ADMIN_PASSWORD, ADMIN_FIRST_NAME, ADMIN_LAST_NAME environment variables
- Railway deployment configuration with railway.toml and nixpacks.toml
- Environment-controlled GraphQL schema endpoints for development (/schema.graphql, /schema.json)
- Zero-maintenance GraphQL introspection system for automatic frontend type generation
- Production security with automatic schema endpoint blocking and introspection disabling
- **Comprehensive Role-Based Access Control (RBAC) system**
- **Multi-layered permission system with Users → Roles → Permissions → Resources architecture**
- **Hierarchical role system with configurable permission levels (super_admin=100, admin=50, user=10)**
- **Resource-based permissions for multi-application scalability**
- **GraphQL authorization guards with field-level permission checking**
- **Permission service with role inheritance and direct user permission overrides**
- **Admin user management mutations (assignRole, removeUserRole)**
- **Admin-only GraphQL queries (allUsers, allRoles, userPermissions)**
- **Database entities for roles, permissions, resources, and junction tables**
- **Automatic RBAC data seeding with default roles and permissions**
- **User invitation system with role-based access control**
- **Complete RBAC CRUD operations with 22 new GraphQL endpoints**
- **Enterprise-grade role and permission management system**
- **Full CRUD for roles, permissions, and resources with SeaORM optimization**
- **Advanced permission assignment system (role-based and direct user permissions)**
- **Comprehensive GraphQL input/output types for RBAC management**
- **Production-ready Vue.js frontend integration examples for RBAC**
- **Complete GraphQL testing guide with all RBAC operations**
- **Frontend RBAC management components with TypeScript support**

### Infrastructure
- Docker Compose setup with PostgreSQL 16 and Adminer
- Database migrations with SeaORM CLI integration
- Tokio async runtime with full feature set
- Tower HTTP middleware for CORS and authentication
- Structured logging with tracing and tracing-subscriber
- Error handling with thiserror and anyhow

### GraphQL Schema
- User type with profile information and verification status
- Authentication payload with user data and JWT token
- Input types for registration and login operations
- Message response type for operation confirmations
- Built-in introspection support for TypeScript code generation
- **22 new RBAC endpoints: 8 queries + 14 mutations for complete role/permission management**
- **Optimized GraphQL types with SeaORM ComplexObject implementations**
- **Advanced input validation for role hierarchies and permission assignments**

### Security
- JWT-based authentication with configurable expiration
- Password hashing with bcrypt (cost factor 12)
- Email verification token system with expiration
- Secure environment variable management
- Optional authentication for public endpoints
- Admin user seeding with secure password hashing and duplicate prevention
- Production-safe admin seeding that skips creation when environment variables are not set
- Reversible admin user migration with proper cleanup in rollback scenarios
- **Enterprise-grade Role-Based Access Control (RBAC) with permission hierarchies**
- **GraphQL field-level authorization with role and permission checking**
- **Resource-based permission model for multi-tenant application support**
- **Secure admin user management with role hierarchy enforcement**
- **Permission inheritance system with user-specific override capabilities**
- **Comprehensive RBAC validation preventing privilege escalation**
- **Admin-only access controls for all role and permission management operations**

### Documentation
- Complete README.md with setup instructions and API documentation
- Updated CLAUDE.md with development guidelines and documentation rules
- Updated .gitignore with comprehensive exclusions
- GraphQL introspection documentation for frontend integration
- FRONTEND_INTEGRATION.md with complete TypeScript/Vue.js setup guide
- Railway deployment configuration and environment variable documentation
- API schema synchronization guide for zero-maintenance frontend integration
- **RBAC system architecture documentation with multi-app scalability guidance**
- **Frontend testing guide for role-based access control features**
- **Admin user management and permission system documentation**
- **Updated CLAUDE.md with comprehensive RBAC system overview and security features**
- **Complete GraphQL testing guide with all 22 RBAC operations and examples**
- **Production-ready Vue.js RBAC management interface documentation**
- **Frontend integration examples with TypeScript and GraphQL codegen**

---

## How to Update This File

When making changes to the project:

1. **Add entries under `[Unreleased]` section**
2. **Use these categories**: Added, Changed, Deprecated, Removed, Fixed, Security
3. **Write clear, descriptive entries** that explain the impact
4. **Update immediately after making changes**, not in batch
5. **Create new version sections** when releasing

### Example Entry Format:
```markdown
### Added
- New user registration endpoint with email verification
- Password strength validation middleware

### Changed  
- Updated JWT token expiration from 1 hour to 24 hours
- Improved error messages for authentication failures

### Fixed
- Fixed database connection pool timeout issues
```