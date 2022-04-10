use crate::error::SyntaxErrorKind;
use crate::value::Value;
use crate::{Database, Sql, SyntaxError};

mod clause;
mod expression;
mod statement;

pub fn compare_sql<DB: Database>(
	sql: &Sql<DB>,
	target_text: &str,
	target_values: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	Value<DB>: PartialEq,
{
	if sql.query() != target_text {
		return Err(SyntaxError::new(
			SyntaxErrorKind::Other,
			format!("SQL text differs: {}\nvs\n{}", sql.query(), target_text),
		));
	};

	if sql.params() != target_values {
		return Err(SyntaxError::new(
			SyntaxErrorKind::Other,
			format!(
				"SQL values differ: {:?}\nvs\n{:?}",
				sql.params(),
				target_values
			),
		));
	}

	return Ok(());
}
