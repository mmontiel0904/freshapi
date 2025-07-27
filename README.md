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

#### Required for Railway Deployment:
- `DATABASE_URL`: PostgreSQL connection string (Railway provides automatically)
- `JWT_SECRET`: Secure secret key for JWT signing (**CRITICAL - must set**)
- `ENVIRONMENT`: Set to "production" (Railway sets automatically)

#### Optional Configuration:
- `RESEND_API_KEY`: Email service API key for email functionality
- `CORS_ALLOWED_ORIGINS`: Frontend domains for CORS (e.g., `https://your-app.com`)
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