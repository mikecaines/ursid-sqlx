use crate::sql_lang::statement::Select;
use crate::value::Value;
use crate::Database;

#[derive(Debug)]
pub(crate) enum InKind<DB: Database> {
	Values(Vec<Option<Value<DB>>>),
	Subquery(Select<DB>),
}

impl<DB: Database> Clone for InKind<DB> {
	fn clone(&self) -> Self {
		match self {
			InKind::Values(values) => InKind::Values(values.clone()),
			InKind::Subquery(select_statement) => InKind::Subquery(select_statement.clone()),
		}
	}
}
