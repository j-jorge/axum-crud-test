// An error type for our stuff, saving client code from handling the
// many error types from the sub systems.
#[derive(Debug)]
pub enum Error {
  // I saw a #[from] on the web, it seems to implement From<>
  // automatically, but I could not have it working here (something
  // like it being not available).
  DataBase(tokio_postgres::Error),
  Pool(deadpool_postgres::PoolError),
  InvalidParameter,
}

impl From<tokio_postgres::Error> for Error {
  fn from(e: tokio_postgres::Error) -> Error {
    return Error::DataBase(e);
  }
}

impl From<deadpool_postgres::PoolError> for Error {
  fn from(e: deadpool_postgres::PoolError) -> Error {
    return Error::Pool(e);
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Error::DataBase(e) => return write!(f, "DataBase: {}", e),
      Error::Pool(e) => return write!(f, "Pool: {}", e),
      Error::InvalidParameter => return write!(f, "InvalidParameter"),
    }
  }
}
