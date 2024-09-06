use crate::{
    handlers::{auth_handlers,
              comment_handlers,
              post_handlers,
              profile_handlers,
              user_handlers},
    session_auth::auth,
    AppState,
};
use axum::{
    http::StatusCode,
    middleware, response::IntoResponse, routing::{delete, get, post}, Router
};
use std::sync::Arc;

async fn handle_invalid_path() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "The requested resource was not found").into_response()
}

pub fn create_router(app_state: Arc<AppState>) -> Router {
    // Define the protected routes
    let protected_routes = Router::new()
        .route("/posts", post(post_handlers::create_post))
        .route("/posts/", post(post_handlers::create_post))

        .route("/posts/:post_id", delete(post_handlers::delete_post))
        .route("/posts/:post_id/react", post(post_handlers::react_to_post))
        .route("/auth/logout", post(auth_handlers::logout_handler))
        .route("/auth/status", post(auth_handlers::status_handler))
        .route("/posts/:post_id/comments", post(comment_handlers::create_comment_handler));

    // Define the unprotected routes
    let unprotected_routes = Router::new()
        .route("/user/:username", get(profile_handlers::get_profile))
        .route("/users", get(user_handlers::get_all_users))
        .route("/users/", get(user_handlers::get_all_users))

        .route("/posts", get(post_handlers::get_all_posts))
        .route("/posts/", get(post_handlers::get_all_posts))

        .route("/auth/login", post(auth_handlers::login_handler))
        .route("/auth/register", post(auth_handlers::register_handler))
        .route("/posts/:post_id/comments", get(comment_handlers::get_comments_handler))
        .fallback(handle_invalid_path)
        .route("/posts/:post_id", get(post_handlers::get_post));


    // Apply the middleware layer to protected routes
    let protected_routes_with_auth =
        protected_routes.layer(middleware::from_fn_with_state(app_state.clone(), auth));

    Router::new()
        .merge(protected_routes_with_auth)
        .merge(unprotected_routes)
        .with_state(app_state)
}
