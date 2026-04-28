// SPDX-License-Identifier: AGPL-3.0-only
use crate::business;
use crate::webapi::auth;
use crate::webapi::json_helper;

#[derive(Clone)]
pub struct ServiceState {
  leaders: std::sync::Arc<business::leads::Leaders>,
  shop: std::sync::Arc<business::shop::Shop>,
}

/// Middleware to validate that the request comes from a leader.
async fn auth(
  state_handle: axum::extract::State<ServiceState>,
  request: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response<axum::body::Body> {
  return auth::validate_request(&state_handle.0.leaders, request, next).await;
}

#[derive(serde::Deserialize)]
struct ShopUpdateRequest {
  #[serde(rename = "product-id")]
  product_id: String,
  coins: i32,
}

/// Register a shop product and its reward, creating the item if it
/// does not exist. This requires an administrator.
async fn update(
  state_handle: axum::extract::State<ServiceState>,
  axum::response::Json(request): axum::response::Json<ShopUpdateRequest>,
) -> business::result::Result<()> {
  let shop: &business::shop::Shop = &state_handle.0.shop;

  shop.update(&request.product_id, request.coins).await?;

  return Ok(());
}

/// List all shop products.
async fn list(
  state_handle: axum::extract::State<ServiceState>,
) -> business::result::Result<String> {
  return Ok(serde_json::to_string(&json_helper::to_json(
    &state_handle.0.shop.list().await?,
  )?)?);
}

/// Configure all routes for this service.
pub fn route(
  leaders: std::sync::Arc<business::leads::Leaders>,
  shop: std::sync::Arc<business::shop::Shop>,
) -> axum::Router {
  let state = ServiceState { leaders, shop };

  return axum::Router::new()
    .route("/update", axum::routing::post(update))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
    .route("/list", axum::routing::get(list))
    .with_state(state);
}
