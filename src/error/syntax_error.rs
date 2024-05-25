use std::fmt::Formatter;

use crate::error::ExecuteError;

#[derive(Debug)]
pub struct SyntaxError {
	kind: SyntaxErrorKind,
	sql: String,
}

impl SyntaxError {
	pub fn kind(&self) -> &SyntaxErrorKind {
		&self.kind
	}

	pub fn sql(&self) -> &str {
		self.sql.as_str()
	}

	#[allow(unused)]
	pub(crate) fn new(kind: SyntaxErrorKind, sql: String) -> Self {
		Self { kind, sql }
	}
}

impl std::error::Error for SyntaxError {}

impl std::fmt::Display for SyntaxError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let mut msg = format!("SQL syntax error: {}", &self.kind);

		if !self.sql.is_empty() {
			msg.push_str(": Near \"");
			msg.push_str(self.sql.as_str());
			msg.push('"');
		}

		f.write_str(msg.as_str())
	}
}

#[derive(Debug)]
pub enum SyntaxErrorKind {
	MissingSelectPredicates,
	MissingOrderByPredicates,
	MissingGroupByPredicates,
	Other,
}

impl std::fmt::Display for SyntaxErrorKind {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			SyntaxErrorKind::MissingSelectPredicates => {
				write!(f, "SQL SELECT statement must specify at least one column/expression for retrieval")
			}
			SyntaxErrorKind::MissingOrderByPredicates => {
				write!(
					f,
					"SQL ORDER BY statement must specify at least one column/expression"
				)
			}
			SyntaxErrorKind::MissingGroupByPredicates => {
				write!(
					f,
					"SQL GROUP BY statement must specify at least one column/expression"
				)
			}
			SyntaxErrorKind::Other => write!(f, "Other"),
		}
	}
}

impl From<SyntaxError> for ExecuteError {
	fn from(e: SyntaxError) -> Self {
		ExecuteError::new(e)
	}
}
