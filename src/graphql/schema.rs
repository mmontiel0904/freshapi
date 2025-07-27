use async_graphql::{EmptySubscription, Schema};
use std::env;

use crate::graphql::{MutationRoot, QueryRoot};

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> ApiSchema {
    let mut schema_builder = Schema::build(QueryRoot, MutationRoot, EmptySubscription);
    
    // Disable introspection in production for security
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    if environment != "development" {
        schema_builder = schema_builder.disable_introspection();
    }
    
    schema_builder.finish()
}