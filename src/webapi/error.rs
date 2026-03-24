use crate::business;

impl axum::response::IntoResponse for business::error::Error
{
  /// Turn any error into an HTTP internal server error to be sent to the
  /// client.
  fn into_response(self) -> axum::response::Response
  {
    // TODO: log the error.
    return (
      axum::http::StatusCode::INTERNAL_SERVER_ERROR,
      String::from("Internal server error")
    )
      .into_response();
  }
}

impl From<serde_json::Error> for business::error::Error
{
  fn from(_e: serde_json::Error) -> business::error::Error
  {
    // TODO: Better error.
    return business::error::Error::InvalidParameter;
  }
}
