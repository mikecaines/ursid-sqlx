use std::fmt::Formatter;

use crate::error::ExecuteError;

#[derive(Debug)]
pub struct CrudError {
	kind: CrudErrorKind,
}

impl CrudError {
	pub fn kind(&self) -> &CrudErrorKind {
		&self.kind
	}

	#[allow(unused)]
	pub(crate) fn new(kind: CrudErrorKind) -> Self {
		Self { kind }
	}
}

impl std::error::Error for CrudError {}

impl std::fmt::Display for CrudError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "CRUD error: {}", &self.kind)
	}
}

#[derive(Debug)]
pub enum CrudErrorKind {
	MultipleRowsWouldBeUpdated,
	MissingKeyColumns,
}

impl std::fmt::Display for CrudErrorKind {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			CrudErrorKind::MultipleRowsWouldBeUpdated => {
				write!(f, "Multiple rows would be updated")
			}
			CrudErrorKind::MissingKeyColumns => write!(f, "At least one key column is required"),
		}
	}
}

impl From<CrudError> for ExecuteError {
	fn from(e: CrudError) -> Self {
		ExecuteError::new(e)
	}
}
