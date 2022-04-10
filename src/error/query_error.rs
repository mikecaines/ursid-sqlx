use crate::error::ExecuteError;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct QueryError {}

impl QueryError {
	pub(crate) fn new() -> Self {
		Self {}
	}
}

impl std::error::Error for QueryError {}

impl std::fmt::Display for QueryError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"FrozenSql has already been consumed, and cannot be used to build another query"
		)
	}
}

impl From<QueryError> for ExecuteError {
	fn from(e: QueryError) -> Self {
		ExecuteError::new(e)
	}
}
