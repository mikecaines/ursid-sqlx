use crate::sql_lang::clause::where_like::{render, WhereLike, WhereLikeBuilder};
use crate::{Database, Sql};

pub type Having<DB> = WhereLike<DB, 'h'>;

impl<DB: Database> Having<DB> {
	pub fn build() -> WhereLikeBuilder<DB, 'h', false, false> {
		WhereLikeBuilder { predicates: vec![] }
	}

	pub fn build_with_join() -> WhereLikeBuilder<DB, 'h', false, true> {
		WhereLikeBuilder { predicates: vec![] }
	}
}

impl<DB: Database> From<Having<DB>> for Sql<DB> {
	fn from(having_clause: Having<DB>) -> Self {
		render(having_clause, Some("having "))
	}
}

pub type HavingBuilder<DB, const HAS_PREDICATES: bool, const HAS_JOIN: bool> =
	WhereLikeBuilder<DB, 'h', HAS_PREDICATES, HAS_JOIN>;
