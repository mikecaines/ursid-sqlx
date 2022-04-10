use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct ExecuteError {
	cause: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl ExecuteError {
	pub(crate) fn new<E: std::error::Error + Send + Sync + 'static>(cause: E) -> Self {
		Self {
			cause: Box::new(cause),
		}
	}
}

impl std::error::Error for ExecuteError {
	fn cause(&self) -> Option<&dyn Error> {
		Some(self.cause.as_ref())
	}
}

impl std::fmt::Display for ExecuteError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.write_str("SQL execute error")
	}
}

impl From<sqlx::Error> for ExecuteError {
	fn from(e: sqlx::Error) -> Self {
		Self::new(e)
	}
}
