pub(crate) use self::predicate::PredicateKind;
use crate::error::SyntaxError;
pub(crate) use crate::sql_lang::expression::grammar::LogicalOp;
use crate::sql_lang::expression::{
	ColumnReference, LogicalNot, SqlExpression, TableAndColumnReference,
};
use crate::sql_lang::{ColRef, Sql};
use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, IntoRawSql, IntoSql};

mod predicate;

#[derive(Debug)]
pub struct WhereLike<DB: Database, const MODE: char> {
	pub(crate) predicates: Vec<(LogicalOp, PredicateKind<DB, MODE>)>,
}

impl<DB: Database, const MODE: char> WhereLike<DB, MODE> {
	pub(crate) fn into_builder<const HAS_JOIN: bool>(
		self,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char> Clone for WhereLike<DB, MODE> {
	fn clone(&self) -> Self {
		Self {
			predicates: self.predicates.clone(),
		}
	}
}

pub fn render<DB: Database, const MODE: char>(
	where_clause: WhereLike<DB, MODE>,
	prefix: Option<&str>,
) -> Sql<DB> {
	let WhereLike { predicates } = where_clause;

	assert!(!predicates.is_empty());

	let mut sql: Sql<DB> = prefix.unwrap_or("").into_raw_sql();

	for (i, (combinator, predicate)) in predicates.into_iter().enumerate() {
		if i > 0 {
			sql = sql.raw_append(' ').append(combinator).raw_append(' ');
		}

		match predicate {
			PredicateKind::Pair((column, value)) => {
				let ColRef {
					table_name,
					column_name,
				} = column;

				if let Some(table_name) = table_name {
					sql = sql.append(TableAndColumnReference::new(table_name, column_name));
				} else {
					sql = sql.append(ColumnReference::new(column_name));
				}

				if value.is_some() {
					sql = sql.raw_append('=').append(value);
				} else {
					sql = sql.raw_append(" is null");
				}
			}

			PredicateKind::In((column, in_clause)) => {
				let ColRef {
					table_name,
					column_name,
				} = column;

				if let Some(table_name) = table_name {
					sql = sql.append(TableAndColumnReference::new(table_name, column_name));
				} else {
					sql = sql.append(ColumnReference::new(column_name));
				}

				sql = sql.raw_append(' ').append(in_clause)
			}

			PredicateKind::Expression(expr) => {
				sql = sql.append(expr);
			}

			PredicateKind::Group(where_clause) => {
				sql = sql.raw_append('(');
				sql = sql.append(render(where_clause, None));
				sql = sql.raw_append(')');
			}
		}
	}

	sql
}

#[derive(Debug)]
pub struct WhereLikeBuilder<
	DB: Database,
	const MODE: char,
	const HAS_PREDICATES: bool,
	const HAS_JOIN: bool,
> {
	pub(crate) predicates: Vec<(LogicalOp, PredicateKind<DB, MODE>)>,
}

impl<DB: Database, const MODE: char, const HAS_PREDICATES: bool, const HAS_JOIN: bool>
	WhereLikeBuilder<DB, MODE, HAS_PREDICATES, HAS_JOIN>
{
	pub(crate) fn merge_with_builder(
		mut self,
		other: WhereLikeBuilder<DB, MODE, HAS_PREDICATES, HAS_JOIN>,
	) -> Self {
		let WhereLikeBuilder { predicates } = other;

		self.predicates.extend(predicates);

		self
	}

	pub(crate) fn merge_with_clause(
		mut self,
		other: WhereLike<DB, MODE>,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		let WhereLike { predicates } = other;

		self.predicates.extend(predicates);

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char, const HAS_JOIN: bool>
	WhereLikeBuilder<DB, MODE, false, HAS_JOIN>
{
	pub fn group(
		mut self,
		other: WhereLike<DB, MODE>,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::And, PredicateKind::Group(other)));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn expression<E: Into<Sql<DB>>>(
		mut self,
		expr: E,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::And, PredicateKind::Expression(expr.into())));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char, const HAS_JOIN: bool>
	WhereLikeBuilder<DB, MODE, true, HAS_JOIN>
{
	pub fn and_group(
		mut self,
		other: WhereLike<DB, MODE>,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::And, PredicateKind::Group(other)));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_group(
		mut self,
		other: WhereLike<DB, MODE>,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::Or, PredicateKind::Group(other)));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_expression<E: Into<Sql<DB>>>(
		mut self,
		expr: E,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::And, PredicateKind::Expression(expr.into())));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_expression<E: Into<Sql<DB>>>(
		mut self,
		expr: E,
	) -> WhereLikeBuilder<DB, MODE, true, HAS_JOIN> {
		self.predicates
			.push((LogicalOp::Or, PredicateKind::Expression(expr.into())));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn finalize(self) -> Result<WhereLike<DB, MODE>, SyntaxError> {
		Ok(WhereLike {
			predicates: self.predicates,
		})
	}
}

impl<DB: Database, const MODE: char> WhereLikeBuilder<DB, MODE, false, false> {
	pub fn column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Pair((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_not_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				ColumnReference::new(name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_is_null<N: Into<String>>(
		mut self,
		name: N,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::new(name.into()).is_null()
					} else {
						ColumnReference::new(name.into()).is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::In((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_not_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::new(name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char> WhereLikeBuilder<DB, MODE, false, true> {
	pub fn column_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Pair((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_not_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				TableAndColumnReference::new(table_name.into(), column_name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_is_null<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::with_table(table_name.into(), column_name.into()).is_null()
					} else {
						ColumnReference::with_table(table_name.into(), column_name.into())
							.is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_in<T: Into<String>, C: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::In((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn column_not_in<T: Into<String>, C: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::with_table(table_name.into(), column_name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char> WhereLikeBuilder<DB, MODE, true, false> {
	pub fn and_column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Pair((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_not_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				ColumnReference::new(name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Pair((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_not_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				ColumnReference::new(name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_is_null<N: Into<String>>(
		mut self,
		name: N,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::new(name.into()).is_null()
					} else {
						ColumnReference::new(name.into()).is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_is_null<N: Into<String>>(
		mut self,
		name: N,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::new(name.into()).is_null()
					} else {
						ColumnReference::new(name.into()).is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::In((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_not_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::new(name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::In((
				ColRef {
					table_name: None,
					column_name: name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_not_in<N: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		name: N,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, false> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::new(name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char> WhereLikeBuilder<DB, MODE, true, true> {
	pub fn and_column_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Pair((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_not_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				TableAndColumnReference::new(table_name.into(), column_name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Pair((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				value.into_sql_value(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_not_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				TableAndColumnReference::new(table_name.into(), column_name.into())
					.not_equal_to(value.into_sql_value())
					.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_is_null<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::with_table(table_name.into(), column_name.into()).is_null()
					} else {
						ColumnReference::with_table(table_name.into(), column_name.into())
							.is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_is_null<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
		is_null: bool,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				{
					use sql_lang::expression::*;

					if is_null {
						ColumnReference::with_table(table_name.into(), column_name.into()).is_null()
					} else {
						ColumnReference::with_table(table_name.into(), column_name.into())
							.is_not_null()
					}
				}
				.into(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_in<T: Into<String>, C: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::In((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn and_column_not_in<
		T: Into<String>,
		C: Into<String>,
		I: Into<sql_lang::clause::In<DB>>,
	>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::And,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::with_table(table_name.into(), column_name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_in<T: Into<String>, C: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::In((
				ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				sql_in_clause.into(),
			)),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}

	pub fn or_column_not_in<T: Into<String>, C: Into<String>, I: Into<sql_lang::clause::In<DB>>>(
		mut self,
		table_name: T,
		column_name: C,
		sql_in_clause: I,
	) -> WhereLikeBuilder<DB, MODE, true, true> {
		self.predicates.push((
			LogicalOp::Or,
			PredicateKind::Expression(
				LogicalNot::new(
					ColumnReference::with_table(table_name.into(), column_name.into())
						.into_sql()
						.append(sql_in_clause.into()),
				)
				.into_sql(),
			),
		));

		WhereLikeBuilder {
			predicates: self.predicates,
		}
	}
}

impl<DB: Database, const MODE: char, const HAS_PREDICATES: bool, const HAS_JOIN: bool> Clone
	for WhereLikeBuilder<DB, MODE, HAS_PREDICATES, HAS_JOIN>
{
	fn clone(&self) -> Self {
		Self {
			predicates: self.predicates.clone(),
		}
	}
}
