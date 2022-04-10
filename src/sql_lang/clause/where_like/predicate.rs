use crate::sql_lang::{ColRef, Sql};
use crate::value::Value;
use crate::{sql_lang, Database};

#[derive(Debug)]
pub(crate) enum PredicateKind<DB: Database, const MODE: char> {
	Pair((ColRef, Option<Value<DB>>)),
	In((ColRef, sql_lang::clause::In<DB>)),
	Expression(Sql<DB>),
	Group(super::WhereLike<DB, MODE>),
}

impl<DB: Database, const MODE: char> Clone for PredicateKind<DB, MODE> {
	fn clone(&self) -> Self {
		match self {
			PredicateKind::Pair(pair) => PredicateKind::Pair(pair.clone()),
			PredicateKind::In(in_clause) => PredicateKind::In(in_clause.clone()),
			PredicateKind::Expression(expr) => PredicateKind::Expression(expr.clone()),
			PredicateKind::Group(where_clause) => PredicateKind::Group(where_clause.clone()),
		}
	}
}
