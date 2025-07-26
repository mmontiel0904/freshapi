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

The project currently contains only a basic main.rs with a "Hello, world!" placeholder, indicating it's ready for initial API development.