use crate::{
    handlers::{
        comment, create_post, delete_post, get_all_posts, get_all_users, get_comments, get_post, get_profile, is_logged_in, login_user_handler, logout_handler, react_to_post, register_user_handler
    },
    session_auth::auth,
    AppState,
};
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    // Define the protected routes
    let protected_routes = Router::new()
        .route("/posts", post(create_post))
        .route("/posts/:post_id", delete(delete_post))
        .route("/posts/:post_id/react", post(react_to_post))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/status", post(is_logged_in))
        .route("/posts/:post_id/comments", post(comment));

    // Define the unprotected routes
    let unprotected_routes = Router::new()
        .route("/user/:username", get(get_profile))
        .route("/users", get(get_all_users))
        .route("/posts", get(get_all_posts))
        .route("/auth/login", post(login_user_handler))
        .route("/auth/register", post(register_user_handler))
        .route("/posts/:post_id/comments", get(get_comments))
        .route("/posts/:post_id", get(get_post));


    // Apply the middleware layer to protected routes
    let protected_routes_with_auth =
        protected_routes.layer(middleware::from_fn_with_state(app_state.clone(), auth));

    Router::new()
        .merge(protected_routes_with_auth)
        .merge(unprotected_routes)
        .with_state(app_state)
}
