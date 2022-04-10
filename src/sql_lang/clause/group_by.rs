use crate::error::SyntaxErrorKind;
use crate::sql_lang::expression::{ColumnReference, TableAndColumnReference};
use crate::sql_lang::Sql;
use crate::{Database, IntoRawSql, IntoSql, SyntaxError};

#[derive(Debug)]
pub struct GroupBy<DB: Database> {
	pub(crate) predicates: Vec<PredicateKind<DB>>,
}

impl<DB: Database> GroupBy<DB> {
	pub fn build() -> GroupByBuilder<DB, false, false> {
		GroupByBuilder { predicates: vec![] }
	}

	pub fn build_with_join() -> GroupByBuilder<DB, false, true> {
		GroupByBuilder { predicates: vec![] }
	}
}

impl<DB: Database> Clone for GroupBy<DB> {
	fn clone(&self) -> Self {
		Self {
			predicates: self.predicates.clone(),
		}
	}
}

impl<DB: Database, Col: Into<String>> TryFrom<Vec<Col>> for GroupBy<DB> {
	type Error = SyntaxError;

	fn try_from(predicates: Vec<Col>) -> Result<Self, Self::Error> {
		predicates
			.into_iter()
			.collect::<GroupByBuilder<DB, true, false>>()
			.finalize()
	}
}

// https://github.com/mikecaines/ursid-sqlx/issues/1
/*
impl<DB: Database, Tab: Into<String>, Col: Into<String>, const LEN: usize>
	TryFrom<[(Tab, Col); LEN]> for GroupBy<DB>
{
	type Error = SyntaxError;

	fn try_from(predicates: [(Tab, Col); LEN]) -> Result<Self, Self::Error> {
		predicates
			.into_iter()
			.collect::<GroupByBuilder<DB, true, true>>()
			.finalize()
	}
}*/
/*impl<DB: Database, Tab: Into<String>, Col: Into<String>> TryFrom<Vec<(Tab, Col)>> for GroupBy<DB> {
	type Error = SyntaxError;

	fn try_from(predicates: Vec<(Tab, Col)>) -> Result<Self, Self::Error> {
		predicates
			.into_iter()
			.collect::<GroupByBuilder<DB, true, true>>()
			.finalize()
	}
}*/

impl<DB: Database> From<GroupBy<DB>> for Sql<DB> {
	fn from(group_by: GroupBy<DB>) -> Self {
		let GroupBy { predicates } = group_by;

		assert!(!predicates.is_empty());

		let mut sql: Sql<DB> = "group by ".into_raw_sql();

		for (i, predicate) in predicates.into_iter().enumerate() {
			if i > 0 {
				sql = sql.raw_append(", ");
			}

			match predicate {
				PredicateKind::TableAndColumn(table_name, column_name) => {
					sql = sql.append(TableAndColumnReference::new(table_name, column_name));
				}

				PredicateKind::Column(column_name) => {
					sql = sql.append(ColumnReference::new(column_name));
				}

				PredicateKind::Expression(expr) => {
					sql = sql.append(expr);
				}
			}
		}

		sql
	}
}

pub struct GroupByBuilder<DB: Database, const HAS_PREDICATES: bool, const HAS_JOIN: bool> {
	predicates: Vec<PredicateKind<DB>>,
}

impl<DB: Database, const HAS_PREDICATES: bool, const HAS_JOIN: bool>
	GroupByBuilder<DB, HAS_PREDICATES, HAS_JOIN>
{
	pub fn group_by_expression<E: Into<Sql<DB>>>(
		mut self,
		expr: E,
	) -> GroupByBuilder<DB, true, HAS_JOIN> {
		self.predicates
			.push(PredicateKind::Expression(expr.into_sql()));

		GroupByBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const HAS_PREDICATES: bool> GroupByBuilder<DB, HAS_PREDICATES, false> {
	pub fn group_by_column<C: Into<String>>(
		mut self,
		column_name: C,
	) -> GroupByBuilder<DB, true, false> {
		self.predicates
			.push(PredicateKind::Column(column_name.into()));

		GroupByBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const HAS_PREDICATES: bool> GroupByBuilder<DB, HAS_PREDICATES, true> {
	pub fn group_by_column<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
	) -> GroupByBuilder<DB, true, false> {
		self.predicates.push(PredicateKind::TableAndColumn(
			table_name.into(),
			column_name.into(),
		));

		GroupByBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const HAS_JOIN: bool> GroupByBuilder<DB, true, HAS_JOIN> {
	pub fn finalize(self) -> Result<GroupBy<DB>, SyntaxError> {
		if self.predicates.is_empty() {
			return Err(SyntaxError::new(
				SyntaxErrorKind::MissingGroupByPredicates,
				"".to_string(),
			));
		}

		Ok(GroupBy {
			predicates: self.predicates,
		})
	}
}

impl<DB: Database, Col: Into<String>> FromIterator<Col> for GroupByBuilder<DB, true, false> {
	fn from_iter<T: IntoIterator<Item = Col>>(iter: T) -> Self {
		Self {
			predicates: iter
				.into_iter()
				.map(|column| PredicateKind::Column(column.into()))
				.collect(),
		}
	}
}

impl<DB: Database, Tab: Into<String>, Col: Into<String>> FromIterator<(Tab, Col)>
	for GroupByBuilder<DB, true, true>
{
	fn from_iter<T: IntoIterator<Item = (Tab, Col)>>(iter: T) -> Self {
		Self {
			predicates: iter
				.into_iter()
				.map(|(table, column)| PredicateKind::TableAndColumn(table.into(), column.into()))
				.collect(),
		}
	}
}

#[derive(Debug)]
pub(crate) enum PredicateKind<DB: Database> {
	TableAndColumn(String, String),
	Column(String),
	Expression(Sql<DB>),
}

impl<DB: Database> Clone for PredicateKind<DB> {
	fn clone(&self) -> Self {
		match self {
			Self::TableAndColumn(t, c) => Self::TableAndColumn(t.clone(), c.clone()),
			Self::Column(c) => Self::Column(c.clone()),
			Self::Expression(e) => Self::Expression(e.clone()),
		}
	}
}
