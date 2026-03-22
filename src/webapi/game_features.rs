use crate::business;

// TODO: It's already in leads.rs. Factorize.
fn internal_error<E>(_error: E) -> (axum::http::StatusCode, String)
{
  // TODO: log the error.
  return (
    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    String::from("Internal server error")
  );
}

#[derive(Clone)]
pub struct ServiceState
{
  leaders: std::sync::Arc<business::leads::Leaders>,
  game_features: std::sync::Arc<business::game_features::GameFeatures>
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
) -> Result<(), (axum::http::StatusCode, String)>
{
  // TODO: check the permissions.
  let leaders: &business::leads::Leaders = &state_handle.0.leaders;
  let game_features: &business::game_features::GameFeatures =
    &state_handle.0.game_features;

  game_features
    .update(&request.id, request.cost_in_coins)
    .await
    .map_err(internal_error)?;

  return Ok(());
}

/// List all game features and their prices.
async fn list(
  state_handle: axum::extract::State<ServiceState>
) -> Result<String, (axum::http::StatusCode, String)>
{
  let game_features: &business::game_features::GameFeatures =
    &state_handle.0.game_features;

  let feature_list: std::collections::HashMap<String, i32> =
    game_features.list().await.map_err(internal_error)?;

  return Ok(
    serde_json::to_string(
      &serde_json::to_value(feature_list).map_err(internal_error)?
    )
    .map_err(internal_error)?
  );
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
    .route("/list", axum::routing::get(list))
    .with_state(state);
}
