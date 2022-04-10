use crate::{Database, IntoRawSql, Sql};
use std::marker::PhantomData;

pub struct TableAndColumnReference<DB: Database> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) table_name: String,
	pub(crate) column_name: String,
}

impl<DB: Database> TableAndColumnReference<DB> {
	pub fn new<T: Into<String>, C: Into<String>>(table_name: T, column_name: C) -> Self {
		Self {
			db: Default::default(),
			table_name: table_name.into(),
			column_name: column_name.into(),
		}
	}
}

impl<DB: Database> From<TableAndColumnReference<DB>> for Sql<DB> {
	fn from(expr: TableAndColumnReference<DB>) -> Self {
		let TableAndColumnReference {
			db: _,
			table_name,
			column_name,
		} = expr;

		(DB::sql_quote_identifier(table_name) + "." + &DB::sql_quote_identifier(column_name))
			.into_raw_sql()
	}
}

pub struct TableReference<DB: Database> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) table_name: String,
}

impl<DB: Database> TableReference<DB> {
	pub fn new<C: Into<String>>(table_name: C) -> Self {
		Self {
			db: Default::default(),
			table_name: table_name.into(),
		}
	}
}

impl<DB: Database> From<TableReference<DB>> for Sql<DB> {
	fn from(expr: TableReference<DB>) -> Self {
		let TableReference { db: _, table_name } = expr;

		DB::sql_quote_identifier(table_name).into_raw_sql()
	}
}

pub struct ColumnReference<DB: Database> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) column_name: String,
}

impl<DB: Database> ColumnReference<DB> {
	pub fn new<C: Into<String>>(column_name: C) -> Self {
		Self {
			db: Default::default(),
			column_name: column_name.into(),
		}
	}

	pub fn with_table<T: Into<String>, C: Into<String>>(
		table_name: T,
		column_name: C,
	) -> TableAndColumnReference<DB> {
		TableAndColumnReference {
			db: Default::default(),
			table_name: table_name.into(),
			column_name: column_name.into(),
		}
	}
}

impl<DB: Database> From<ColumnReference<DB>> for Sql<DB> {
	fn from(expr: ColumnReference<DB>) -> Self {
		let ColumnReference { db: _, column_name } = expr;

		DB::sql_quote_identifier(column_name).into_raw_sql()
	}
}
