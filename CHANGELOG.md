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

### Security
- JWT-based authentication with configurable expiration
- Password hashing with bcrypt (cost factor 12)
- Email verification token system with expiration
- Secure environment variable management
- Optional authentication for public endpoints
- Admin user seeding with secure password hashing and duplicate prevention
- Production-safe admin seeding that skips creation when environment variables are not set
- Reversible admin user migration with proper cleanup in rollback scenarios

### Documentation
- Complete README.md with setup instructions and API documentation
- Updated CLAUDE.md with development guidelines and documentation rules
- Updated .gitignore with comprehensive exclusions
- GraphQL introspection documentation for frontend integration

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