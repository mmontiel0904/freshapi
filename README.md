# FreshAPI

A modern GraphQL API for user management and authentication, designed to power personal tools and experimental projects.

## Overview

FreshAPI provides a robust, scalable foundation for building personal projects with user management capabilities. Built with Rust, GraphQL, and PostgreSQL, it offers:

- **User Management**: Complete user lifecycle (registration, authentication, profile management)
- **GraphQL API**: Modern, type-safe API with introspection support
- **JWT Authentication**: Secure token-based authentication
- **Email Integration**: User verification and notifications via Resend
- **Frontend Ready**: Designed for TypeScript/Vue.js integration
- **Docker Support**: Containerized PostgreSQL for local development

## Tech Stack

- **Backend**: Rust with Axum web framework
- **GraphQL**: async-graphql for schema definition and resolvers
- **Database**: PostgreSQL with SeaORM
- **Authentication**: JWT tokens with bcrypt password hashing
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
├── models/           # Database models
├── graphql/          # GraphQL schema and resolvers
├── auth/             # Authentication logic
├── email/            # Email service integration
└── migration/        # Database migrations
```

## User Management Features

### Core Functionality

- **User Registration**: Email-based registration with verification
- **Authentication**: JWT-based login/logout
- **Profile Management**: User profile CRUD operations
- **Password Management**: Secure password reset flow
- **Email Verification**: Account activation and notifications

### API Endpoints

GraphQL endpoint: `POST /graphql`

Key mutations and queries:
- `registerUser`: Create new user account
- `loginUser`: Authenticate and receive JWT
- `updateProfile`: Modify user information
- `resetPassword`: Initiate password reset
- `verifyEmail`: Confirm email address

## Configuration

### Environment Variables

See `.env` file for all configuration options:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT signing
- `RESEND_API_KEY`: Email service API key
- `CORS_ALLOWED_ORIGINS`: Frontend origins for CORS

### Security Considerations

- Change default JWT secret in production
- Use secure PostgreSQL credentials
- Configure proper CORS origins
- Enable HTTPS in production deployment

## Contributing

### Documentation Rules

- **Always read existing documentation before editing**
- **Update CHANGELOG.md for each commit**
- **Maintain API documentation with code changes**
- **Update README.md when adding features**

### Commit Log

All changes are tracked in `CHANGELOG.md` to monitor repository evolution and maintain clear project history.

## License

[Add your license here]

## Support

This is a personal project for experimental and learning purposes. For issues or questions, please use the GitHub issue tracker.