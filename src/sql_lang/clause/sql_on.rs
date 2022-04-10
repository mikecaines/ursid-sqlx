use crate::sql_lang::clause::where_like::{render, WhereLike, WhereLikeBuilder};
use crate::sql_lang::expression::grammar::ComparisonCombo;
use crate::sql_lang::expression::TableAndColumnReference;
use crate::{Database, IntoSql, Sql, SyntaxError};

pub type SqlOn<DB> = WhereLike<DB, 'o'>;

impl<DB: Database> SqlOn<DB> {
	pub fn build() -> WhereLikeBuilder<DB, 'o', false, true> {
		WhereLikeBuilder { predicates: vec![] }
	}
}
impl<DB: Database> From<SqlOn<DB>> for Sql<DB> {
	fn from(where_clause: SqlOn<DB>) -> Self {
		render(where_clause, Some("on "))
	}
}

pub type SqlOnBuilder<DB, const HAS_PREDICATES: bool> =
	WhereLikeBuilder<DB, 'o', HAS_PREDICATES, true>;

impl<DB: Database> SqlOnBuilder<DB, false> {
	pub fn fk<FkTab: Into<String>, FkCol: Into<String>, PkTab: Into<String>, PkCol: Into<String>>(
		self,
		fk_table_name: FkTab,
		fk_column_name: FkCol,
		pk_table_name: PkTab,
		pk_column_name: PkCol,
	) -> SqlOnBuilder<DB, true>
	where
		Sql<DB>:
			From<ComparisonCombo<DB, TableAndColumnReference<DB>, TableAndColumnReference<DB>>>,
		Sql<DB>: From<TableAndColumnReference<DB>>,
	{
		self.expression({
			use crate::sql_lang::expression::prelude::*;

			ColumnReference::with_table(fk_table_name, fk_column_name)
				.equal_to(ColumnReference::with_table(pk_table_name, pk_column_name))
				.into_sql()
		})
	}
}

impl<
		DB: Database,
		FkTab: Into<String>,
		FkCol: Into<String>,
		PkTab: Into<String>,
		PkCol: Into<String>,
	> TryFrom<(FkTab, FkCol, PkTab, PkCol)> for SqlOn<DB>
where
	Sql<DB>: From<ComparisonCombo<DB, TableAndColumnReference<DB>, TableAndColumnReference<DB>>>,
	Sql<DB>: From<TableAndColumnReference<DB>>,
{
	type Error = SyntaxError;

	fn try_from(parts: (FkTab, FkCol, PkTab, PkCol)) -> Result<Self, Self::Error> {
		crate::sql_lang::clause::SqlOn::build()
			.fk(parts.0, parts.1, parts.2, parts.3)
			.finalize()
	}
}
