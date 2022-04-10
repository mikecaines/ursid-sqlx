use crate::sql_lang::clause::sql_join::{JoinBuilder, TableReferenceKind};
use crate::sql_lang::clause::{Join, SqlOn};
use crate::{Database, IntoRawSql, Sql, SyntaxError};

#[derive(Debug)]
pub struct SqlFrom<DB: Database> {
	pub(crate) join: Join<DB>,
}

impl<DB: Database> SqlFrom<DB> {
	pub fn build<T, A>(table_reference_kind: T, alias: A) -> SqlFromBuilder<DB>
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		SqlFromBuilder {
			join: Join::build(table_reference_kind, alias),
		}
	}

	pub fn from_table_name<N: Into<String>>(table_name: N) -> Self {
		Self {
			join: Join::from_table_name(table_name),
		}
	}
}

impl<DB: Database> Clone for SqlFrom<DB> {
	fn clone(&self) -> Self {
		Self {
			join: self.join.clone(),
		}
	}
}

impl<DB: Database> From<SqlFrom<DB>> for Sql<DB> {
	fn from(from_clause: SqlFrom<DB>) -> Self {
		let SqlFrom { join } = from_clause;
		IntoRawSql::<DB>::into_raw_sql("from ").append(join)
	}
}

pub struct SqlFromBuilder<DB: Database> {
	join: JoinBuilder<DB>,
}

impl<DB: Database> SqlFromBuilder<DB> {
	pub fn inner_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.join = self.join.inner_join(table_reference, alias, on);
		self
	}

	pub fn left_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.join = self.join.left_join(table_reference, alias, on);
		self
	}

	pub fn right_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.join = self.join.right_join(table_reference, alias, on);
		self
	}

	pub fn finalize(self) -> Result<SqlFrom<DB>, SyntaxError> {
		Ok(SqlFrom {
			join: self.join.finalize()?,
		})
	}
}
