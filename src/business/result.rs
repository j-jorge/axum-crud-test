// SPDX-License-Identifier: AGPL-3.0-only
use super::*;

// A result type for our stuff, as a convenience, where the error type
// is our error type.
pub type Result<T, E = error::Error> = std::result::Result<T, E>;
