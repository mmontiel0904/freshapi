use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth::{AuthenticatedUser, JwtService};

pub async fn auth_middleware(
    State(jwt_service): State<JwtService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = jwt_service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = AuthenticatedUser::from(claims);
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}