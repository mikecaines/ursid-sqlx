use crate::sql_lang::clause::where_like::{render, WhereLike, WhereLikeBuilder};
use crate::{Database, Sql};

pub type Where<DB> = WhereLike<DB, 'w'>;

impl<DB: Database> Where<DB> {
	pub fn build() -> WhereLikeBuilder<DB, 'w', false, false> {
		WhereLikeBuilder { predicates: vec![] }
	}

	pub fn build_with_join() -> WhereLikeBuilder<DB, 'w', false, true> {
		WhereLikeBuilder { predicates: vec![] }
	}
}

impl<DB: Database> From<Where<DB>> for Sql<DB> {
	fn from(where_clause: Where<DB>) -> Self {
		render(where_clause, Some("where "))
	}
}

pub type WhereBuilder<DB, const HAS_PREDICATES: bool, const HAS_JOIN: bool> =
	WhereLikeBuilder<DB, 'w', HAS_PREDICATES, HAS_JOIN>;
