use crate::sql_lang::clause::sql_join::{JoinKind, TableReferenceKind};
use crate::sql_lang::clause::SqlOn;
use crate::Database;

#[derive(Debug)]
pub struct JoinedTable<DB: Database> {
	pub(crate) table_reference_kind: TableReferenceKind<DB>,
	pub(crate) alias: Option<String>,
	pub(crate) join: Option<(JoinKind, Option<SqlOn<DB>>)>,
}

impl<DB: Database> Clone for JoinedTable<DB> {
	fn clone(&self) -> Self {
		Self {
			table_reference_kind: self.table_reference_kind.clone(),
			alias: self.alias.clone(),
			join: self.join.clone(),
		}
	}
}
