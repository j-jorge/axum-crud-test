use crate::business;
use crate::webapi::auth;

#[derive(Clone)]
pub struct ServiceState
{
  leaders: std::sync::Arc<business::leads::Leaders>,
  game_features: std::sync::Arc<business::game_features::GameFeatures>
}

/// Middleware to validate that the request comes from a leader.
async fn auth(
  state_handle: axum::extract::State<ServiceState>,
  request: axum::extract::Request,
  next: axum::middleware::Next
) -> axum::response::Response<axum::body::Body>
{
  return auth::validate_request(&state_handle.0.leaders, request, next).await;
}

#[derive(serde::Deserialize)]
struct GameFeaturesUpdateRequest
{
  id: String,
  cost_in_coins: i32
}

/// Set the price of a game feature, creating the item if it does not exist.
/// This requires an administrator.
async fn update(
  state_handle: axum::extract::State<ServiceState>,
  axum::response::Json(request): axum::response::Json<
    GameFeaturesUpdateRequest
  >
) -> business::result::Result<()>
{
  let game_features: &business::game_features::GameFeatures =
    &state_handle.0.game_features;

  game_features
    .update(&request.id, request.cost_in_coins)
    .await?;

  return Ok(());
}

/// List all game features and their prices.
async fn list(
  state_handle: axum::extract::State<ServiceState>
) -> business::result::Result<String>
{
  let game_features: &business::game_features::GameFeatures =
    &state_handle.0.game_features;

  let feature_list: std::collections::HashMap<String, i32> =
    game_features.list().await?;

  return Ok(serde_json::to_string(&feature_list)?);
}

/// Configure all routes for this service.
pub fn route(
  leaders: std::sync::Arc<business::leads::Leaders>,
  game_features: std::sync::Arc<business::game_features::GameFeatures>
) -> axum::Router
{
  let state = ServiceState {
    leaders: leaders,
    game_features: game_features
  };

  return axum::Router::new()
    .route("/update", axum::routing::post(update))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth))
    .route("/list", axum::routing::get(list))
    .with_state(state);
}
