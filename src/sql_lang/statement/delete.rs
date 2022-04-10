use crate::error::SyntaxError;
use crate::sql_lang::clause::Where;
use crate::sql_lang::expression::TableReference;
use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, FrozenSql, IntoRawSql, IntoSql, Sql};

#[derive(Debug)]
pub struct Delete<DB: Database> {
	pub(crate) table_name: String,
	pub(crate) where_clause: Option<sql_lang::clause::sql_where::Where<DB>>,
}

impl<DB: Database> Delete<DB> {
	pub fn build<N: Into<String>>(table_name: N) -> DeleteBuilder<DB> {
		DeleteBuilder {
			table_name: table_name.into(),
			where_clause_builder: None,
		}
	}
}

impl<DB: Database> Clone for Delete<DB> {
	fn clone(&self) -> Self {
		Self {
			table_name: self.table_name.clone(),
			where_clause: self.where_clause.clone(),
		}
	}
}

impl<DB: Database> From<Delete<DB>> for Sql<DB> {
	fn from(delete_statement: Delete<DB>) -> Self {
		let mut sql = IntoRawSql::<DB>::into_raw_sql("delete from ")
			.append(TableReference::new(delete_statement.table_name));

		if let Some(where_clause) = delete_statement.where_clause {
			sql = sql.raw_append(' ').append(where_clause)
		}

		sql
	}
}

pub struct DeleteBuilder<DB: Database> {
	table_name: String,
	where_clause_builder: Option<sql_lang::clause::sql_where::WhereBuilder<DB, true, false>>,
}

impl<DB: Database> DeleteBuilder<DB> {
	pub fn where_column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> Self {
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.and_column_equal_to(name, value)
		} else {
			Where::build().column_equal_to(name, value)
		});

		self
	}

	pub fn with_where_clause(mut self, clause: sql_lang::clause::Where<DB>) -> Self {
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.merge_with_clause(clause)
		} else {
			clause.into_builder()
		});

		self
	}
}

impl<DB: Database> DeleteBuilder<DB> {
	pub fn finalize(self) -> Result<Delete<DB>, SyntaxError> {
		Ok(Delete {
			table_name: self.table_name,
			where_clause: self
				.where_clause_builder
				.map(|w| w.finalize())
				.transpose()?,
		})
	}

	pub fn finalize_and_freeze(self) -> Result<FrozenSql<DB>, SyntaxError> {
		Ok(self.finalize()?.into_sql().freeze())
	}
}
