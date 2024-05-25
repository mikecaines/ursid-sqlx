use std::marker::PhantomData;

use crate::{crud, sql_lang, Database};

/// Provides convenience methods on [Pool](sqlx::Pool) and [Transaction](sqlx::Transaction).
///
/// Using these methods to access the builders can help with type inference issues.
pub trait BuilderHelper<DB: Database> {
	fn build_sql(&self) -> SqlHelper<DB> {
		SqlHelper {
			database: PhantomData,
		}
	}

	fn build_crud(&self) -> CrudHelper<DB> {
		CrudHelper {
			database: PhantomData,
		}
	}
}

impl<DB: Database> BuilderHelper<DB> for sqlx::Pool<DB> {}

impl<'c, DB: Database> BuilderHelper<DB> for sqlx::Transaction<'c, DB> {}

impl<DB: Database> BuilderHelper<DB> for &mut DB::Connection {}

pub struct SqlHelper<DB: Database> {
	database: PhantomData<DB>,
}

impl<DB: Database> SqlHelper<DB> {
	pub fn statement(&self) -> SqlStatementHelper<DB> {
		SqlStatementHelper {
			database: PhantomData,
		}
	}

	pub fn clause(&self) -> SqlClauseHelper<DB> {
		SqlClauseHelper {
			database: PhantomData,
		}
	}
}

pub struct SqlStatementHelper<DB: Database> {
	database: PhantomData<DB>,
}

impl<DB: Database> SqlStatementHelper<DB> {
	pub fn select<N: Into<String>>(
		&self,
		table_name: N,
	) -> sql_lang::statement::select::SelectBuilder<DB, false, false, false, false, false> {
		sql_lang::statement::Select::build(table_name)
	}

	pub fn select_with_join<F: Into<sql_lang::clause::SqlFrom<DB>>>(
		&self,
		from_clause: F,
	) -> sql_lang::statement::select::SelectBuilder<DB, false, false, false, false, true> {
		sql_lang::statement::Select::build_with_join(from_clause)
	}

	pub fn insert<N: Into<String>>(
		&self,
		table_name: N,
	) -> sql_lang::statement::insert::InsertBuilder<DB> {
		sql_lang::statement::Insert::build(table_name)
	}

	pub fn update<N: Into<String>>(
		&self,
		table_name: N,
	) -> sql_lang::statement::update::UpdateBuilder<DB, false> {
		sql_lang::statement::Update::build(table_name)
	}

	pub fn delete<N: Into<String>>(
		&self,
		table_name: N,
	) -> sql_lang::statement::delete::DeleteBuilder<DB> {
		sql_lang::statement::Delete::build(table_name)
	}
}

pub struct SqlClauseHelper<DB: Database> {
	database: PhantomData<DB>,
}

impl<DB: Database> SqlClauseHelper<DB> {
	pub fn sql_where(&self) -> sql_lang::clause::sql_where::WhereBuilder<DB, false, false> {
		sql_lang::clause::sql_where::Where::build()
	}

	pub fn sql_where_with_join(
		&self,
	) -> sql_lang::clause::sql_where::WhereBuilder<DB, false, true> {
		sql_lang::clause::sql_where::Where::build_with_join()
	}
}

pub struct CrudHelper<DB: Database> {
	database: PhantomData<DB>,
}

impl<DB: Database> CrudHelper<DB> {
	pub fn insert_row<N: Into<String>>(&self, table_name: N) -> crud::insert::InsertBuilder<DB> {
		crud::insert_row(table_name)
	}

	pub fn update_rows<N: Into<String>>(
		&self,
		table_name: N,
	) -> crud::update::UpdateBuilder<DB, false> {
		crud::update_rows(table_name)
	}

	pub fn replace_row<N: Into<String>>(
		&self,
		table_name: N,
	) -> crud::replace::ReplaceBuilder<DB, false, false> {
		crud::replace_row(table_name)
	}

	pub fn delete_rows<N: Into<String>>(&self, table_name: N) -> crud::delete::DeleteBuilder<DB> {
		crud::delete_rows(table_name)
	}
}
