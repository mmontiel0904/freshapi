# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FreshAPI is a Rust-based GraphQL API service built with modern async frameworks. The project is currently in initial development stage with a basic Cargo project structure.

## Tech Stack

- **Framework**: Axum web framework with async-graphql for GraphQL support
- **Database**: SeaORM for database operations and migrations
- **Authentication**: JWT tokens with bcrypt for password hashing
- **Email**: Resend service integration for email functionality
- **Tracing**: Built-in logging and tracing with tracing-subscriber
- **Environment**: Environment variable management with dotenvy

## Development Commands

```bash
# Database setup (run first)
docker-compose up -d

# Build the project
cargo build

# Run the application
cargo run

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run with release optimizations
cargo run --release

# Database migrations
cargo run --bin migration

# Stop database
docker-compose down
```

## Architecture Notes

This is a fresh Rust project configured for building a GraphQL API with:

- Async/await support via tokio runtime
- GraphQL schema and resolvers through async-graphql
- Web server capabilities with Axum
- Database ORM with SeaORM and migration support
- Authentication infrastructure with JWT and bcrypt
- External service integrations (email via Resend, HTTP requests via reqwest)
- Structured error handling with anyhow and thiserror
- UUID support for identifiers
- JSON serialization with serde

## Documentation Rules

**CRITICAL**: Before editing any documentation file (README.md, CLAUDE.md, CHANGELOG.md), you MUST:

1. **Read the existing file completely** using the Read tool
2. **Understand the current content and structure**
3. **Make targeted updates** that preserve existing information
4. **Update CHANGELOG.md** with details about what documentation was changed

## Frontend Integration Notes

The API is designed with TypeScript/Vue.js frontend integration in mind:

- GraphQL introspection enabled for automatic type generation
- CORS configured for local frontend development (ports 3000, 5173)
- JWT authentication compatible with frontend token storage
- Structured error responses for proper frontend error handling

## Local Development Setup

The project includes Docker Compose for PostgreSQL:
- Database: `postgresql://freshapi_user:freshapi_password@localhost:5432/freshapi_db`
- Adminer web interface: `http://localhost:8081`
- All configuration in `.env` file

## Railway Deployment

The project is fully configured for Railway deployment:

### Required Environment Variables:
- `JWT_SECRET` - Secure production JWT secret (CRITICAL)
- `DATABASE_URL` - Railway provides automatically
- `ENVIRONMENT` - Set to "production" automatically

### Optional Environment Variables:
- `RESEND_API_KEY` - Email service integration
- `CORS_ALLOWED_ORIGINS` - Frontend domain CORS configuration
- Admin seeding variables (remove after initial setup)

### Schema Synchronization:
- Development: `/schema.graphql` and `/schema.json` endpoints available
- Production: Schema endpoints automatically disabled for security
- Frontend: Use GraphQL Codegen with development endpoints for TypeScript generation

See `FRONTEND_INTEGRATION.md` for complete frontend integration guide.