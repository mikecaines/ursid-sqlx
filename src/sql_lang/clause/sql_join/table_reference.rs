use crate::sql_lang::statement::Select;
use crate::Database;

#[derive(Debug)]
pub enum TableReferenceKind<DB: Database> {
	TableName(String),
	Subquery(Select<DB>),
}

impl<DB: Database> Clone for TableReferenceKind<DB> {
	fn clone(&self) -> Self {
		match self {
			TableReferenceKind::TableName(name) => Self::TableName(name.clone()),
			TableReferenceKind::Subquery(select) => Self::Subquery(select.clone()),
		}
	}
}

impl<DB: Database> From<Select<DB>> for TableReferenceKind<DB> {
	fn from(select_statement: Select<DB>) -> Self {
		Self::Subquery(select_statement)
	}
}

impl<DB: Database> From<&str> for TableReferenceKind<DB> {
	fn from(table_name: &str) -> Self {
		Self::TableName(table_name.to_string())
	}
}

impl<DB: Database> From<String> for TableReferenceKind<DB> {
	fn from(table_name: String) -> Self {
		Self::TableName(table_name)
	}
}
