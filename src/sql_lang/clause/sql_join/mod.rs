pub(crate) use self::joined_table::JoinedTable;
pub use self::table_reference::TableReferenceKind;
use crate::sql_lang::clause::SqlOn;
use crate::sql_lang::expression::{ColumnReference, TableReference};
use crate::{Database, IntoRawSql, IntoSql, Sql, SyntaxError};

mod joined_table;
mod table_reference;

#[derive(Debug, Clone)]
pub(crate) enum JoinKind {
	Inner,
	Left,
	Right,
}

#[derive(Debug)]
pub struct Join<DB: Database> {
	pub(crate) joined_tables: Vec<JoinedTable<DB>>,
}

impl<DB: Database> Join<DB> {
	pub fn build<T, A>(table_reference_kind: T, alias: A) -> JoinBuilder<DB>
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		JoinBuilder {
			joined_tables: vec![JoinedTable {
				table_reference_kind: table_reference_kind.into(),
				alias: Some(alias.into()),
				join: None,
			}],
		}
	}

	pub fn from_table_name<N: Into<String>>(table_name: N) -> Self {
		Self {
			joined_tables: vec![JoinedTable {
				table_reference_kind: TableReferenceKind::TableName(table_name.into()),
				alias: None,
				join: None,
			}],
		}
	}
}

impl<DB: Database> Clone for Join<DB> {
	fn clone(&self) -> Self {
		Self {
			joined_tables: self.joined_tables.clone(),
		}
	}
}

impl<DB: Database> From<Join<DB>> for Sql<DB> {
	fn from(from_clause: Join<DB>) -> Self {
		let Join { joined_tables } = from_clause;

		let mut sql: Sql<DB> = "".into_raw_sql();

		for joined_table in joined_tables {
			let JoinedTable {
				table_reference_kind,
				alias,
				join,
			} = joined_table;

			let mut join_on = None;

			if let Some((kind, on)) = join {
				match kind {
					JoinKind::Inner => {
						sql = sql.raw_append(" inner join ");
					}
					JoinKind::Left => {
						sql = sql.raw_append(" left join ");
					}
					JoinKind::Right => {
						sql = sql.raw_append(" right join ");
					}
				}

				join_on = on;
			}

			sql = sql.append(match table_reference_kind {
				TableReferenceKind::TableName(table_name) => {
					TableReference::new(table_name).into_sql()
				}
				TableReferenceKind::Subquery(select_statement) => {
					IntoRawSql::<DB>::into_raw_sql("(")
						.append(select_statement)
						.raw_append(")")
				}
			});

			if let Some(alias) = alias {
				sql = sql.raw_append(' ').append(ColumnReference::new(alias));
			}

			if let Some(on) = join_on {
				sql = sql.raw_append(' ').append(on);
			}
		}

		sql
	}
}

pub struct JoinBuilder<DB: Database> {
	joined_tables: Vec<JoinedTable<DB>>,
}

impl<DB: Database> JoinBuilder<DB> {
	pub fn inner_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.joined_tables.push(JoinedTable {
			table_reference_kind: table_reference.into(),
			alias: Some(alias.into()),
			join: Some((JoinKind::Inner, Some(on))),
		});

		self
	}

	pub fn left_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.joined_tables.push(JoinedTable {
			table_reference_kind: table_reference.into(),
			alias: Some(alias.into()),
			join: Some((JoinKind::Left, Some(on))),
		});

		self
	}

	pub fn right_join<T, A>(mut self, table_reference: T, alias: A, on: SqlOn<DB>) -> Self
	where
		T: Into<TableReferenceKind<DB>>,
		A: Into<String>,
	{
		self.joined_tables.push(JoinedTable {
			table_reference_kind: table_reference.into(),
			alias: Some(alias.into()),
			join: Some((JoinKind::Right, Some(on))),
		});

		self
	}

	pub fn finalize(self) -> Result<Join<DB>, SyntaxError> {
		Ok(Join {
			joined_tables: self.joined_tables,
		})
	}
}
