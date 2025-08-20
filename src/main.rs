use std::env;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Request, State},
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod entities;
mod graphql;
mod services;


use auth::{AuthenticatedUser, JwtService, PermissionService};
use graphql::{create_schema, ApiSchema, DataLoaderContext};
use services::{EmailService, InvitationService, UserService, ProjectService, TaskService, ActivityService, ContextService, EmailContextService};

#[derive(Clone)]
struct AppState {
    schema: ApiSchema,
    db: DatabaseConnection,
    jwt_service: JwtService,
    permission_service: PermissionService,
    dataloader_context: DataLoaderContext,
    user_service: UserService,
    email_service: EmailService,
    invitation_service: InvitationService,
    project_service: ProjectService,
    task_service: TaskService,
    activity_service: ActivityService,
    context_service: ContextService,
    email_context_service: EmailContextService,
    frontend_url: String,
}

async fn optional_auth_middleware(
    State(jwt_service): State<JwtService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        // Try to authenticate
        match jwt_service.verify_token(token) {
            Ok(claims) => {
                let user = AuthenticatedUser::from(claims);
                request.extensions_mut().insert(Some(user));
            }
            Err(_) => {
                // Invalid token - continue without auth
                request.extensions_mut().insert(None::<AuthenticatedUser>);
            }
        }
    } else {
        // No auth header - continue without auth
        request.extensions_mut().insert(None::<AuthenticatedUser>);
    }

    Ok(next.run(request).await)
}

async fn graphql_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Option<AuthenticatedUser>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();
    
    if let Some(user) = user {
        request = request.data(user);
    }
    
    request = request
        .data(state.db.clone())
        .data(state.permission_service.clone())
        .data(state.dataloader_context.clone())
        .data(state.user_service.clone())
        .data(state.email_service.clone())
        .data(state.invitation_service.clone())
        .data(state.project_service.clone())
        .data(state.task_service.clone())
        .data(state.activity_service.clone())
        .data(state.context_service.clone())
        .data(state.email_context_service.clone())
        .data(state.frontend_url.clone());
    
    state.schema.execute(request).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>FreshAPI GraphQL Playground</title>
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
    </head>
    <body>
        <div id="root"></div>
        <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
        <script>
            GraphQLPlayground.init(document.getElementById('root'), {
                endpoint: '/graphql'
            })
        </script>
    </body>
    </html>
    "#)
}

async fn health() -> impl IntoResponse {
    "OK"
}

// Webhook endpoint for n8n email ingestion
async fn ingest_email_webhook(
    State(state): State<AppState>,
    Json(payload): Json<crate::graphql::types::EmailIngestInput>,
) -> Response {
    // Log the incoming webhook request
    info!("📧 Received email ingestion webhook for project {}", payload.project_id);
    
    match state.email_context_service.ingest_email(payload).await {
        Ok(email_context) => {
            info!("✅ Successfully ingested email context {}", email_context.id);
            (StatusCode::CREATED, Json(serde_json::json!({
                "success": true,
                "email_id": email_context.id,
                "message": "Email context ingested successfully"
            }))).into_response()
        }
        Err(e) => {
            warn!("❌ Failed to ingest email: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))).into_response()
        }
    }
}

async fn graphql_schema(State(state): State<AppState>) -> impl IntoResponse {
    // Only expose schema in development environment
    let environment = env::var("RAILWAY_ENVIRONMENT_NAME")
        .or_else(|_| env::var("ENVIRONMENT"))
        .unwrap_or_else(|_| "production".to_string());
    
    if environment != "development" {
        return (StatusCode::NOT_FOUND, "Schema not available in production").into_response();
    }
    
    info!("🔧 Schema endpoint accessed in development mode");
    
    // Auto-generated SDL from your actual schema - no hardcoding!
    let sdl = state.schema.sdl();
    
    (
        [(axum::http::header::CONTENT_TYPE, "application/graphql")],
        sdl
    ).into_response()
}

async fn graphql_introspection(State(state): State<AppState>) -> impl IntoResponse {
    // Only allow introspection in development
    let environment = env::var("RAILWAY_ENVIRONMENT_NAME")
        .or_else(|_| env::var("ENVIRONMENT"))
        .unwrap_or_else(|_| "production".to_string());
    
    if environment != "development" {
        return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({
            "error": "Introspection disabled in production"
        }))).into_response();
    }
    
    info!("🔍 GraphQL introspection accessed in development mode");
    
    // Standard GraphQL introspection query - zero maintenance required!
    let introspection_query = r#"
        query IntrospectionQuery {
            __schema {
                queryType { name }
                mutationType { name }
                subscriptionType { name }
                types {
                    kind
                    name
                    description
                    fields(includeDeprecated: true) {
                        name
                        description
                        type {
                            kind
                            name
                            ofType {
                                kind
                                name
                                ofType {
                                    kind
                                    name
                                }
                            }
                        }
                        isDeprecated
                        deprecationReason
                    }
                    inputFields {
                        name
                        description
                        type {
                            kind
                            name
                            ofType {
                                kind
                                name
                            }
                        }
                        defaultValue
                    }
                    interfaces {
                        kind
                        name
                    }
                    enumValues(includeDeprecated: true) {
                        name
                        description
                        isDeprecated
                        deprecationReason
                    }
                    possibleTypes {
                        kind
                        name
                    }
                }
                directives {
                    name
                    description
                    locations
                    args {
                        name
                        description
                        type {
                            kind
                            name
                        }
                        defaultValue
                    }
                }
            }
        }
    "#;

    let request = async_graphql::Request::new(introspection_query);
    let response = state.schema.execute(request).await;
    
    axum::Json(response).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "freshapi=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get configuration from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("JWT_SECRET not set, using default (not secure for production)");
            "default-secret-change-in-production".to_string()
        });
    let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse::<i64>()
        .unwrap_or(24);
    let resend_api_key = env::var("RESEND_API_KEY")
        .unwrap_or_else(|_| {
            warn!("RESEND_API_KEY not set, email functionality will not work");
            "dummy-key".to_string()
        });
    let from_email = env::var("FROM_EMAIL")
        .unwrap_or_else(|_| "noreply@freshapi.dev".to_string());
    let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());
    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    
    // Check environment - Railway uses RAILWAY_ENVIRONMENT_NAME, fallback to ENVIRONMENT
    let environment = env::var("RAILWAY_ENVIRONMENT_NAME")
        .or_else(|_| env::var("ENVIRONMENT"))
        .unwrap_or_else(|_| "development".to_string());
    
    info!("🚀 Starting FreshAPI in {} environment", environment);

    // Connect to database
    info!("Connecting to database...");
    let db = Database::connect(&database_url).await?;
    info!("Database connected successfully");


    // Initialize services
    let jwt_service = JwtService::new(&jwt_secret, jwt_expiration_hours, 30); // 30 days for refresh tokens
    let permission_service = PermissionService::new(db.clone());
    let email_service = EmailService::new(&resend_api_key, from_email);
    let user_service = UserService::new(db.clone(), jwt_service.clone());
    let invitation_service = InvitationService::new(db.clone(), email_service.clone());
    let project_service = ProjectService::new(db.clone());
    let activity_service = ActivityService::new(db.clone());
    let task_service = TaskService::new(db.clone(), project_service.clone(), activity_service.clone());
    let context_service = ContextService::new(db.clone());
    let email_context_service = EmailContextService::new(db.clone());

    // Create GraphQL schema
    let schema = create_schema();

    // Create DataLoader context for optimized permission loading
    let dataloader_context = DataLoaderContext::new(db.clone());

    // Application state
    let app_state = AppState {
        schema,
        db,
        jwt_service: jwt_service.clone(),
        permission_service,
        dataloader_context,
        user_service,
        email_service,
        invitation_service,
        project_service,
        task_service,
        activity_service,
        context_service,
        email_context_service,
        frontend_url,
    };

    // Setup CORS
    let cors = if cors_origins.trim() == "*" {
        // Allow any origin (DANGEROUS - only for development!)
        warn!("🚨 CORS set to accept ANY origin (*) - only use in development!");
        CorsLayer::permissive()
    } else {
        // Parse specific origins
        let origins: Vec<HeaderValue> = cors_origins
            .split(',')
            .filter_map(|origin| origin.trim().parse().ok())
            .collect();
        
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT, Method::PATCH, Method::DELETE])
            .allow_headers([
                CONTENT_TYPE,
                AUTHORIZATION,
                HeaderName::from_static("x-apollo-tracing"),
                HeaderName::from_static("apollo-require-preflight"),
                HeaderName::from_static("x-requested-with"),
            ])
            .allow_credentials(true)
    };

    // Create router
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .route("/health", get(health))
        .route("/schema.graphql", get(graphql_schema))
        .route("/schema.json", get(graphql_introspection))
        .route("/webhooks/email/ingest", post(ingest_email_webhook))
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            jwt_service,
            optional_auth_middleware,
        ))
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Server starting on http://{}", addr);
    info!("📊 GraphQL Playground available at http://{}/playground", addr);
    info!("🏥 Health check available at http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
