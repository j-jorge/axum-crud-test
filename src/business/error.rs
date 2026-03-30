// SPDX-License-Identifier: AGPL-3.0-only
use thiserror::Error;

// An error type for our stuff, saving client code from handling the
// many error types from the sub systems.
#[derive(Debug, Error)]
pub enum Error {
  #[error("database error")]
  DataBase(#[from] tokio_postgres::Error),
  #[error("pool error")]
  Pool(#[from] deadpool_postgres::PoolError),
  #[error("invalid parameter")]
  InvalidParameter,
}
