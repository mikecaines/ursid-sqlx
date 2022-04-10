use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, ExecuteError};

pub fn delete_rows<DB: Database, N: Into<String>>(table_name: N) -> DeleteBuilder<DB> {
	DeleteBuilder {
		statement: sql_lang::statement::Delete::build(table_name),
	}
}

pub struct DeleteBuilder<DB: Database> {
	pub(crate) statement: sql_lang::statement::delete::DeleteBuilder<DB>,
}

impl<DB: Database> DeleteBuilder<DB> {
	pub fn where_column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> Self {
		self.statement = self.statement.where_column_equal_to(name, value);
		self
	}

	pub fn with_where_clause(mut self, where_clause: sql_lang::clause::Where<DB>) -> Self {
		self.statement = self.statement.with_where_clause(where_clause);
		self
	}

	pub async fn execute(self, database: &mut DB::Connection) -> Result<(), ExecuteError> {
		DB::execute_crud_delete(self, database).await
	}
}
