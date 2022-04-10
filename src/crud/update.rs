use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, ExecuteError};

pub fn update_rows<DB: Database, N: Into<String>>(table_name: N) -> UpdateBuilder<DB, false> {
	UpdateBuilder {
		statement: sql_lang::statement::Update::build(table_name),
	}
}

pub struct UpdateBuilder<DB: Database, const HAS_UPDATES: bool> {
	pub(crate) statement: sql_lang::statement::update::UpdateBuilder<DB, HAS_UPDATES>,
}

impl<DB: Database, const HAS_UPDATES: bool> UpdateBuilder<DB, HAS_UPDATES> {
	pub fn update_column<N: Into<String>, V: IntoSqlValue<DB>>(
		self,
		name: N,
		value: V,
	) -> UpdateBuilder<DB, true> {
		UpdateBuilder {
			statement: self.statement.update_column(name, value),
		}
	}

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
}

impl<DB: Database> UpdateBuilder<DB, true> {
	pub async fn execute(self, database: &mut DB::Connection) -> Result<(), ExecuteError> {
		DB::execute_crud_update(self, database).await
	}
}
