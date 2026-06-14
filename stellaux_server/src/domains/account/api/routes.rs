use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    common::{app_state::AppState, auth::AuthUser, dto::Pagination, error::AppResult},
    domains::account::dto::{ChangePasswordRequest, UpdateProfileRequest, UpsertAddress},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).patch(update_me))
        .route("/me/password", post(change_password))
        .route("/orders", get(list_orders))
        .route("/orders/{order_id}", get(get_order))
        .route("/addresses", get(list_addresses).post(create_address))
        .route(
            "/addresses/{address_id}",
            axum::routing::patch(update_address).delete(delete_address),
        )
}

async fn get_me(user: AuthUser) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "user_id": user.0.sub,
        "role": user.0.role,
        "_todo": "join profiles for display_name, email, avatar_url"
    })))
}

async fn update_me(
    _user: AuthUser,
    State(_state): State<AppState>,
    Json(_body): Json<UpdateProfileRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        json!({ "_todo": "update profiles row + auth.users email if changed" }),
    ))
}

async fn change_password(
    _user: AuthUser,
    State(_state): State<AppState>,
    Json(_body): Json<ChangePasswordRequest>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "_todo": "verify current via argon2, hash new, update profiles.password_hash"
    })))
}

async fn list_orders(
    _user: AuthUser,
    State(_state): State<AppState>,
    Query(_page): Query<Pagination>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "items": [],
        "total": 0,
        "_todo": "select orders where user_id = claims.sub"
    })))
}

async fn get_order(
    _user: AuthUser,
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "order_id": order_id,
        "_todo": "select order + order_items + tracking"
    })))
}

async fn list_addresses(_user: AuthUser, State(_state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "items": [] })))
}

async fn create_address(
    _user: AuthUser,
    State(_state): State<AppState>,
    Json(_body): Json<UpsertAddress>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "_todo": "insert addresses row" })))
}

async fn update_address(
    _user: AuthUser,
    State(_state): State<AppState>,
    Path(address_id): Path<Uuid>,
    Json(_body): Json<UpsertAddress>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "address_id": address_id })))
}

async fn delete_address(
    _user: AuthUser,
    State(_state): State<AppState>,
    Path(address_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({ "deleted": address_id })))
}
