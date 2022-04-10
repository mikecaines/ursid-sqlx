use crate::error::SyntaxErrorKind;
use crate::sql_lang::expression::{ColumnReference, TableAndColumnReference};
use crate::sql_lang::Sql;
use crate::{Database, IntoRawSql, IntoSql, SyntaxError};

#[derive(Debug)]
pub struct OrderBy<DB: Database> {
	pub(crate) predicates: Vec<PredicateKind<DB>>,
	pub(crate) limit_and_offset: Option<(u32, u32)>,
}

impl<DB: Database> OrderBy<DB> {
	pub fn build() -> OrderByBuilder<DB, false, false, false> {
		OrderByBuilder {
			predicates: vec![],
			limit_and_offset: None,
		}
	}

	pub fn build_with_join() -> OrderByBuilder<DB, false, false, true> {
		OrderByBuilder {
			predicates: vec![],
			limit_and_offset: None,
		}
	}
}

impl<DB: Database> Clone for OrderBy<DB> {
	fn clone(&self) -> Self {
		Self {
			predicates: self.predicates.clone(),
			limit_and_offset: self.limit_and_offset,
		}
	}
}

impl<DB: Database, Col: Into<String>> TryFrom<Vec<(Col, bool)>> for OrderBy<DB> {
	type Error = SyntaxError;

	fn try_from(predicates: Vec<(Col, bool)>) -> Result<Self, Self::Error> {
		predicates
			.into_iter()
			.collect::<OrderByBuilder<DB, true, false, false>>()
			.finalize()
	}
}

impl<DB: Database, Tab: Into<String>, Col: Into<String>> TryFrom<Vec<(Tab, Col, bool)>>
	for OrderBy<DB>
{
	type Error = SyntaxError;

	fn try_from(predicates: Vec<(Tab, Col, bool)>) -> Result<Self, Self::Error> {
		predicates
			.into_iter()
			.collect::<OrderByBuilder<DB, true, false, true>>()
			.finalize()
	}
}

impl<DB: Database> From<OrderBy<DB>> for Sql<DB> {
	fn from(order_by: OrderBy<DB>) -> Self {
		let OrderBy {
			predicates,
			limit_and_offset,
		} = order_by;

		assert!(!predicates.is_empty());

		let mut sql: Sql<DB> = "order by ".into_raw_sql();

		for (i, predicate) in predicates.into_iter().enumerate() {
			if i > 0 {
				sql = sql.raw_append(", ");
			}

			match predicate {
				PredicateKind::TableAndColumn(table_name, column_name, ascending) => {
					sql = sql.append(TableAndColumnReference::new(table_name, column_name));

					sql = sql.raw_append(if ascending { " asc" } else { " desc" });
				}

				PredicateKind::Column(column_name, ascending) => {
					sql = sql.append(ColumnReference::new(column_name));
					sql = sql.raw_append(if ascending { " asc" } else { " desc" });
				}

				PredicateKind::Expression(expr) => {
					sql = sql.append(expr);
				}
			}
		}

		if let Some((limit, offset)) = limit_and_offset {
			sql = sql.raw_append(" limit ").append(limit);

			if offset > 0 {
				sql = sql.raw_append(" offset ").append(offset);
			}
		}

		sql
	}
}

pub struct OrderByBuilder<
	DB: Database,
	const HAS_PREDICATES: bool,
	const HAS_LIMIT: bool,
	const HAS_JOIN: bool,
> {
	predicates: Vec<PredicateKind<DB>>,
	limit_and_offset: Option<(u32, u32)>,
}

impl<DB: Database, const HAS_PREDICATES: bool, const HAS_LIMIT: bool, const HAS_JOIN: bool>
	OrderByBuilder<DB, HAS_PREDICATES, HAS_LIMIT, HAS_JOIN>
{
	pub fn order_by_expression<E: Into<Sql<DB>>>(
		mut self,
		expr: E,
	) -> OrderByBuilder<DB, true, HAS_LIMIT, HAS_JOIN> {
		self.predicates
			.push(PredicateKind::Expression(expr.into_sql()));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}
}

impl<DB: Database, const HAS_PREDICATES: bool, const HAS_JOIN: bool>
	OrderByBuilder<DB, HAS_PREDICATES, false, HAS_JOIN>
{
	pub fn limit(mut self, limit: u32, offset: u32) -> OrderByBuilder<DB, true, true, HAS_JOIN> {
		self.limit_and_offset = Some((limit, offset));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}
}

impl<DB: Database, const HAS_PREDICATES: bool, const HAS_LIMIT: bool>
	OrderByBuilder<DB, HAS_PREDICATES, HAS_LIMIT, false>
{
	pub fn order_by_column_asc<C: Into<String>>(
		mut self,
		column_name: C,
	) -> OrderByBuilder<DB, true, HAS_LIMIT, false> {
		self.predicates
			.push(PredicateKind::Column(column_name.into(), true));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}

	pub fn order_by_column_desc<C: Into<String>>(
		mut self,
		column_name: C,
	) -> OrderByBuilder<DB, true, HAS_LIMIT, false> {
		self.predicates
			.push(PredicateKind::Column(column_name.into(), false));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}
}

impl<DB: Database, const HAS_PREDICATES: bool, const HAS_LIMIT: bool>
	OrderByBuilder<DB, HAS_PREDICATES, HAS_LIMIT, true>
{
	pub fn order_by_column_asc<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
	) -> OrderByBuilder<DB, true, HAS_LIMIT, false> {
		self.predicates.push(PredicateKind::TableAndColumn(
			table_name.into(),
			column_name.into(),
			true,
		));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}

	pub fn order_by_column_desc<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
	) -> OrderByBuilder<DB, true, HAS_LIMIT, false> {
		self.predicates.push(PredicateKind::TableAndColumn(
			table_name.into(),
			column_name.into(),
			false,
		));

		OrderByBuilder {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		}
	}
}

impl<DB: Database, const HAS_LIMIT: bool, const HAS_JOIN: bool>
	OrderByBuilder<DB, true, HAS_LIMIT, HAS_JOIN>
{
	pub fn finalize(self) -> Result<OrderBy<DB>, SyntaxError> {
		if self.predicates.is_empty() {
			return Err(SyntaxError::new(
				SyntaxErrorKind::MissingOrderByPredicates,
				"".to_string(),
			));
		}

		Ok(OrderBy {
			predicates: self.predicates,
			limit_and_offset: self.limit_and_offset,
		})
	}
}

impl<DB: Database, Col: Into<String>> FromIterator<(Col, bool)>
	for OrderByBuilder<DB, true, false, false>
{
	fn from_iter<T: IntoIterator<Item = (Col, bool)>>(iter: T) -> Self {
		Self {
			predicates: iter
				.into_iter()
				.map(|(column, ascending)| PredicateKind::Column(column.into(), ascending))
				.collect(),
			limit_and_offset: None,
		}
	}
}

impl<DB: Database, Tab: Into<String>, Col: Into<String>> FromIterator<(Tab, Col, bool)>
	for OrderByBuilder<DB, true, false, true>
{
	fn from_iter<T: IntoIterator<Item = (Tab, Col, bool)>>(iter: T) -> Self {
		Self {
			predicates: iter
				.into_iter()
				.map(|(table, column, ascending)| {
					PredicateKind::TableAndColumn(table.into(), column.into(), ascending)
				})
				.collect(),
			limit_and_offset: None,
		}
	}
}

#[derive(Debug)]
pub(crate) enum PredicateKind<DB: Database> {
	TableAndColumn(String, String, bool),
	Column(String, bool),
	Expression(Sql<DB>),
}

impl<DB: Database> Clone for PredicateKind<DB> {
	fn clone(&self) -> Self {
		match self {
			Self::TableAndColumn(t, c, o) => Self::TableAndColumn(t.clone(), c.clone(), *o),
			Self::Column(c, o) => Self::Column(c.clone(), *o),
			Self::Expression(e) => Self::Expression(e.clone()),
		}
	}
}
