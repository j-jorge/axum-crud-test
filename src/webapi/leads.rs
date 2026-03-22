use crate::business;

use axum::response::IntoResponse;

type StateHandle = std::sync::Arc<business::leads::Leaders>;

/// Turn any error into an HTTP internal server error to be sent to the client.
// TODO: extract that into another file as it will be used everywhere
// in the web API.
fn internal_error(
  _error: business::error::Error,
) -> (axum::http::StatusCode, String)
{
  return (
    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    String::from("Internal server error"),
  );
}

// TODO: The token validation function listed below should be in
// another file as they are going to be used by many web services.

fn extract_auth(
  headers: &axum::http::header::HeaderMap,
) -> Option<&axum::http::header::HeaderValue>
{
  return headers.get(axum::http::header::AUTHORIZATION);
}

/// Check that the token in the authorization header is an element of
/// the leader list.
///
/// When the parameter allow_init is true then the function will pass
/// if there is no configured administrator (i.e. when the application
/// is executed for the first time).
async fn valid_admin_internal(
  state_handle: &axum::extract::State<StateHandle>,
  auth_header: Option<&axum::http::header::HeaderValue>,
  allow_init: bool,
) -> business::result::Result<bool>
{
  if let Some(header) = auth_header
    && let Ok(token_str) = header.to_str()
  {
    let leaders: &business::leads::Leaders = &state_handle.0;
    return Ok(
      leaders.validate_token(&token_str).await?
        || (allow_init && leaders.is_in_initialization_state().await?),
    );
  }
  else
  {
    // TODO: a log (but also check that the information is not in the
    // default logs)
    println!("no header");
  }

  return Ok(false);
}

/// Check that the token in the authorization header is an element of
/// the leader list.
async fn valid_admin(
  state_handle: &axum::extract::State<StateHandle>,
  auth_header: Option<&axum::http::header::HeaderValue>,
) -> business::result::Result<bool>
{
  return valid_admin_internal(state_handle, auth_header, false).await;
}

/// Check that the token in the authorization header is an element of
/// the leader list or else that there is no configured leader.
async fn weak_valid_admin(
  state_handle: &axum::extract::State<StateHandle>,
  auth_header: Option<&axum::http::header::HeaderValue>,
) -> business::result::Result<bool>
{
  return valid_admin_internal(state_handle, auth_header, true).await;
}

/// Middleware to validate that the request comes from a leader.
async fn auth(
  state_handle: axum::extract::State<StateHandle>,
  request: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response<axum::body::Body>
{
  // I would have wanted to pass the request directly to
  // validate_admin but it does not work. See this discussion:
  //
  // middleware::from_fn fails if fn calls async fn with &Request argument.
  // https://github.com/tokio-rs/axum/discussions/2571
  //
  // The workaround of passing the request by value would not work
  // since I need the request for the call to next.run() below, so I
  // extract the authorization header here.
  let r: business::result::Result<bool> =
    valid_admin(&state_handle, extract_auth(&request.headers())).await;

  if r.is_err()
  {
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response();
  }

  if r.unwrap()
  {
    return next.run(request).await;
  }

  return (axum::http::StatusCode::UNAUTHORIZED).into_response();
}

/// Middleware to validate that the request comes from a leader **or
/// that no leader exists**.
async fn weak_auth(
  state_handle: axum::extract::State<StateHandle>,
  request: axum::extract::Request,
  next: axum::middleware::Next,
) -> axum::response::Response<axum::body::Body>
{
  let r: business::result::Result<bool> =
    weak_valid_admin(&state_handle, extract_auth(&request.headers())).await;

  if r.is_err()
  {
    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR).into_response();
  }

  if r.unwrap()
  {
    return next.run(request).await;
  }

  return (axum::http::StatusCode::UNAUTHORIZED).into_response();
}

/// Public API, returns the tokens of all leaders. User must provide a valid
/// leader token.
async fn list_leaders(
  state_handle: axum::extract::State<StateHandle>,
) -> Result<String, (axum::http::StatusCode, String)>
{
  let leaders: &business::leads::Leaders = &state_handle.0;
  let result: String =
    serde_json::to_string(&leaders.all_tokens().await.map_err(internal_error)?)
      .unwrap();

  return Ok(result);
}

/// Public API, creates a new leader token.
async fn create_leader(
  state_handle: axum::extract::State<StateHandle>,
) -> Result<String, (axum::http::StatusCode, String)>
{
  let leaders: &business::leads::Leaders = &state_handle.0;
  let result: String = serde_json::to_string(
    &leaders.create_token().await.map_err(internal_error)?,
  )
  .unwrap();

  return Ok(result);
}

/// Configure all routes for this service.
pub fn route(state: StateHandle) -> axum::Router
{
  // Routes that require an authorization token.
  let strong_auth_routes = axum::Router::new()
    .route("/list", axum::routing::get(list_leaders))
    .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth));

  // Routes that require an authorization token or being in the
  // initialization state (i.e. no configured administrator).
  let weak_auth_routes = axum::Router::new()
    .route("/create", axum::routing::post(create_leader))
    .route_layer(axum::middleware::from_fn_with_state(
      state.clone(),
      weak_auth,
    ));

  return axum::Router::new()
    .merge(strong_auth_routes)
    .merge(weak_auth_routes)
    .with_state(state);
}
