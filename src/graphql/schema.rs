use async_graphql::{EmptySubscription, Schema};

use crate::graphql::{MutationRoot, QueryRoot};

pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> ApiSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}