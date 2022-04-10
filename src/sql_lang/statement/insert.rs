use crate::error::SyntaxError;
use crate::sql_lang::expression::{ColumnReference, TableReference};
use crate::value::{IntoSqlValue, Value};
use crate::{Database, FrozenSql, IntoRawSql, IntoSql, Sql};

#[derive(Debug)]
pub struct Insert<DB: Database> {
	pub(crate) table_name: String,
	pub(crate) pairs: Vec<(String, Option<Value<DB>>)>,
}

impl<DB: Database> Insert<DB> {
	pub fn build<N: Into<String>>(table_name: N) -> InsertBuilder<DB> {
		InsertBuilder {
			table_name: table_name.into(),
			pairs: vec![],
		}
	}
}

impl<DB: Database> Clone for Insert<DB> {
	fn clone(&self) -> Self {
		Self {
			table_name: self.table_name.clone(),
			pairs: self.pairs.clone(),
		}
	}
}

impl<DB: Database> From<Insert<DB>> for Sql<DB> {
	fn from(insert_statement: Insert<DB>) -> Self {
		let mut sql: Sql<DB> = "insert into ".into_raw_sql();
		sql = sql.append(TableReference::new(insert_statement.table_name));
		sql = sql.raw_append(" (");

		for (i, (name, _value)) in insert_statement.pairs.iter().enumerate() {
			if i > 0 {
				sql = sql.raw_append(',');
			}

			sql = sql.append(ColumnReference::new(name));
		}

		sql = sql.raw_append(") values (");

		for (i, (_name, value)) in insert_statement.pairs.into_iter().enumerate() {
			if i > 0 {
				sql = sql.raw_append(',');
			}

			sql = sql.append(value);
		}

		sql = sql.raw_append(')');

		sql
	}
}

pub struct InsertBuilder<DB: Database> {
	table_name: String,
	pairs: Vec<(String, Option<Value<DB>>)>,
}

impl<DB: Database> InsertBuilder<DB> {
	pub fn column<N: Into<String>, V: IntoSqlValue<DB>>(mut self, name: N, value: V) -> Self {
		self.pairs.push((name.into(), value.into_sql_value()));
		self
	}

	pub fn finalize(self) -> Result<Insert<DB>, SyntaxError> {
		Ok(Insert {
			table_name: self.table_name,
			pairs: self.pairs,
		})
	}

	pub fn finalize_and_freeze(self) -> Result<FrozenSql<DB>, SyntaxError> {
		Ok(self.finalize()?.into_sql().freeze())
	}
}
