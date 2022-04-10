use crate::error::SyntaxError;
use crate::sql_lang::clause::Where;
use crate::sql_lang::expression::{ColumnReference, TableReference};
use crate::sql_lang::ColRef;
use crate::value::{IntoSqlValue, Value};
use crate::{sql_lang, Database, FrozenSql, IntoRawSql, IntoSql, Sql};

#[derive(Debug)]
pub struct Update<DB: Database> {
	pub(crate) table_name: String,
	pub(crate) set_pairs: Vec<(ColRef, Option<Value<DB>>)>,
	pub(crate) where_clause: Option<sql_lang::clause::sql_where::Where<DB>>,
}

impl<DB: Database> Update<DB> {
	pub fn build<N: Into<String>>(table_name: N) -> UpdateBuilder<DB, false> {
		UpdateBuilder {
			table_name: table_name.into(),
			set_pairs: vec![],
			where_clause_builder: None,
		}
	}
}

impl<DB: Database> Clone for Update<DB> {
	fn clone(&self) -> Self {
		Self {
			table_name: self.table_name.clone(),
			set_pairs: self.set_pairs.clone(),
			where_clause: self.where_clause.clone(),
		}
	}
}

impl<DB: Database> From<Update<DB>> for Sql<DB> {
	fn from(statement: Update<DB>) -> Self {
		let mut sql: Sql<DB> = "update ".into_raw_sql();

		sql = sql.append(TableReference::new(statement.table_name));

		sql = sql.raw_append(" set ");

		for (i, (column, value)) in statement.set_pairs.into_iter().enumerate() {
			let ColRef {
				table_name,
				column_name,
			} = column;

			if i > 0 {
				sql = sql.raw_append(',');
			}

			if let Some(table_name) = table_name {
				sql = sql.append(TableReference::new(table_name));
				sql = sql.raw_append('.');
			}

			sql = sql.append(ColumnReference::new(column_name));
			sql = sql.raw_append('=');
			sql = sql.append(value);
		}

		if let Some(where_clause) = statement.where_clause {
			sql = sql.raw_append(' ').append(where_clause)
		}

		sql
	}
}

pub struct UpdateBuilder<DB: Database, const HAS_UPDATES: bool> {
	table_name: String,
	set_pairs: Vec<(ColRef, Option<Value<DB>>)>,
	where_clause_builder: Option<sql_lang::clause::sql_where::WhereBuilder<DB, true, false>>,
}

impl<DB: Database, const HAS_UPDATES: bool> UpdateBuilder<DB, HAS_UPDATES> {
	pub fn update_column<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> UpdateBuilder<DB, true> {
		self.set_pairs.push((
			ColRef {
				table_name: None,
				column_name: name.into(),
			},
			value.into_sql_value(),
		));

		UpdateBuilder {
			table_name: self.table_name,
			set_pairs: self.set_pairs,
			where_clause_builder: self.where_clause_builder,
		}
	}

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

impl<DB: Database> UpdateBuilder<DB, true> {
	pub fn finalize(self) -> Result<Update<DB>, SyntaxError> {
		Ok(Update {
			table_name: self.table_name,
			set_pairs: self.set_pairs,
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
