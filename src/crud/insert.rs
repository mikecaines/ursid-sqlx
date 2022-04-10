use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, ExecuteError};

pub fn insert_row<DB: Database, N: Into<String>>(table_name: N) -> InsertBuilder<DB> {
	InsertBuilder {
		statement: sql_lang::statement::Insert::build(table_name),
	}
}

pub struct InsertBuilder<DB: Database> {
	pub(crate) statement: sql_lang::statement::insert::InsertBuilder<DB>,
}

impl<DB: Database> InsertBuilder<DB> {
	pub fn column<N: Into<String>, V: IntoSqlValue<DB>>(mut self, name: N, value: V) -> Self {
		self.statement = self.statement.column(name, value);
		self
	}

	pub async fn execute(self, database: &mut DB::Connection) -> Result<(), ExecuteError> {
		DB::execute_crud_insert(self, database).await
	}
}
