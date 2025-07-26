# Changelog

All notable changes to FreshAPI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup with Rust and GraphQL stack
- PostgreSQL database configuration with Docker Compose
- Environment variable configuration (.env)
- SeaORM integration with PostgreSQL features
- JWT authentication foundation
- Email service integration (Resend)
- CORS configuration for frontend development
- Adminer database management interface
- Comprehensive project documentation (README.md, CLAUDE.md)
- Documentation rules and change tracking system

### Infrastructure
- Docker Compose setup for local PostgreSQL development
- Environment configuration for development and production
- Database connection and migration framework
- Logging and tracing infrastructure

### Documentation
- Created README.md with complete setup and usage instructions
- Updated CLAUDE.md with development guidelines and documentation rules
- Established CHANGELOG.md for tracking all project changes
- Documented GraphQL introspection for TypeScript/Vue.js integration

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