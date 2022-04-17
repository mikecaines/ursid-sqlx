use crate::error::{SyntaxError, SyntaxErrorKind};
use crate::sql_lang::clause::having::HavingBuilder;
use crate::sql_lang::clause::sql_where::WhereBuilder;
use crate::sql_lang::clause::{GroupBy, Having, OrderBy, SqlFrom, Where};
use crate::sql_lang::expression::{ColumnReference, TableAndColumnReference};
use crate::sql_lang::{ColRef, Sql};
use crate::value::IntoSqlValue;
use crate::{sql_lang, Database, FrozenSql, IntoRawSql, IntoSql};

#[derive(Debug)]
pub struct Select<DB: Database> {
	pub(crate) from_clause: SqlFrom<DB>,
	pub(crate) select_columns: Vec<SelectPredicateKind<DB>>,
	pub(crate) where_clause: Option<sql_lang::clause::Where<DB>>,
	pub(crate) group_by_clause: Option<sql_lang::clause::GroupBy<DB>>,
	pub(crate) having_clause: Option<sql_lang::clause::Having<DB>>,
	pub(crate) order_by_clause: Option<sql_lang::clause::OrderBy<DB>>,
}

impl<DB: Database> Select<DB> {
	pub fn build<N: Into<String>>(
		table_name: N,
	) -> SelectBuilder<DB, false, false, false, false, false> {
		SelectBuilder {
			from_clause: SqlFrom::from_table_name(table_name),
			select_columns: vec![],
			where_clause_builder: None,
			group_by_clause: None,
			having_clause_builder: None,
			order_by_clause: None,
		}
	}

	pub fn build_with_join<F: Into<SqlFrom<DB>>>(
		from_clause: F,
	) -> SelectBuilder<DB, false, false, false, false, true> {
		SelectBuilder {
			from_clause: from_clause.into(),
			select_columns: vec![],
			where_clause_builder: None,
			group_by_clause: None,
			having_clause_builder: None,
			order_by_clause: None,
		}
	}
}

impl<DB: Database> Clone for Select<DB> {
	fn clone(&self) -> Self {
		Self {
			from_clause: self.from_clause.clone(),
			select_columns: self.select_columns.clone(),
			where_clause: self.where_clause.clone(),
			group_by_clause: self.group_by_clause.clone(),
			having_clause: self.having_clause.clone(),
			order_by_clause: self.order_by_clause.clone(),
		}
	}
}

impl<DB: Database> From<Select<DB>> for Sql<DB> {
	fn from(statement: Select<DB>) -> Self {
		let Select {
			from_clause,
			select_columns,
			where_clause,
			group_by_clause,
			having_clause,
			order_by_clause,
		} = statement;

		let mut sql: Sql<DB> = "select ".into_raw_sql();

		for (i, predicate_kind) in select_columns.into_iter().enumerate() {
			match predicate_kind {
				SelectPredicateKind::Column(selected_column) => {
					let SelectedColumn {
						column: ColRef {
							table_name,
							column_name,
						},
						alias,
					} = selected_column;

					if i > 0 {
						sql = sql.raw_append(", ");
					}

					if let Some(table_name) = table_name {
						sql = sql.append(TableAndColumnReference::new(table_name, column_name));
					} else {
						sql = sql.append(ColumnReference::new(column_name));
					}

					if let Some(alias) = alias {
						sql = sql.raw_append(" as ");
						sql = sql.append(ColumnReference::new(alias));
					}
				}

				SelectPredicateKind::Expression(selected_expr) => {
					let SelectedExpression { expr, alias } = selected_expr;

					if i > 0 {
						sql = sql.raw_append(", ");
					}

					sql = sql.append(expr);

					sql = sql.raw_append(" as ");
					sql = sql.append(ColumnReference::new(alias));
				}
			}
		}

		sql = sql.raw_append(' ').append(from_clause);

		if let Some(where_clause) = where_clause {
			sql = sql.raw_append(' ').append(where_clause)
		}

		if let Some(group_by_clause) = group_by_clause {
			sql = sql.raw_append(' ').append(group_by_clause);
		}

		if let Some(having_clause) = having_clause {
			sql = sql.raw_append(' ').append(having_clause);
		}

		if let Some(order_by_clause) = order_by_clause {
			sql = sql.raw_append(' ').append(order_by_clause);
		}

		sql
	}
}

pub struct SelectBuilder<
	DB: Database,
	const HAS_COLUMNS: bool,
	const HAS_GROUP_BY: bool,
	const HAS_HAVING: bool,
	const HAS_ORDER_BY: bool,
	const HAS_JOIN: bool,
> {
	from_clause: SqlFrom<DB>,
	select_columns: Vec<SelectPredicateKind<DB>>,
	where_clause_builder: Option<sql_lang::clause::sql_where::WhereBuilder<DB, true, HAS_JOIN>>,
	group_by_clause: Option<Result<crate::sql_lang::clause::GroupBy<DB>, SyntaxError>>,
	having_clause_builder: Option<sql_lang::clause::having::HavingBuilder<DB, true, HAS_JOIN>>,
	order_by_clause: Option<Result<crate::sql_lang::clause::OrderBy<DB>, SyntaxError>>,
}

impl<
		DB: Database,
		const HAS_COLUMNS: bool,
		const HAS_GROUP_BY: bool,
		const HAS_HAVING: bool,
		const HAS_ORDER_BY: bool,
		const HAS_JOIN: bool,
	> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, HAS_JOIN>
{
	/// Adds an arbitrary [Sql] expression to the list of SELECT predicates.
	pub fn select_expression<E: Into<Sql<DB>>, A: Into<String>>(
		mut self,
		expr: E,
		alias: A,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, HAS_JOIN> {
		self.select_columns
			.push(SelectPredicateKind::Expression(SelectedExpression {
				expr: expr.into(),
				alias: alias.into(),
			}));

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn with_where_clause(mut self, clause: sql_lang::clause::Where<DB>) -> Self {
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.merge_with_clause(clause)
		} else {
			clause.into_builder()
		});

		self
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_ORDER_BY: bool, const HAS_JOIN: bool>
	SelectBuilder<DB, HAS_COLUMNS, true, false, HAS_ORDER_BY, HAS_JOIN>
{
	pub fn with_having_clause(
		mut self,
		clause: sql_lang::clause::Having<DB>,
	) -> SelectBuilder<DB, HAS_COLUMNS, true, true, HAS_ORDER_BY, HAS_JOIN> {
		self.having_clause_builder = Some(if let Some(builder) = self.having_clause_builder {
			builder.merge_with_clause(clause)
		} else {
			clause.into_builder()
		});

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_ORDER_BY: bool>
	SelectBuilder<DB, HAS_COLUMNS, true, false, HAS_ORDER_BY, false>
{
	pub fn having_clause<
		F: FnOnce(HavingBuilder<DB, false, false>) -> HavingBuilder<DB, true, false>,
	>(
		mut self,
		build: F,
	) -> Self {
		let clause = build(Having::build());
		self.having_clause_builder = Some(if let Some(builder) = self.having_clause_builder {
			builder.merge_with_builder(clause)
		} else {
			clause
		});

		self
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_ORDER_BY: bool>
	SelectBuilder<DB, HAS_COLUMNS, true, false, HAS_ORDER_BY, true>
{
	pub fn having_clause<
		F: FnOnce(HavingBuilder<DB, false, true>) -> HavingBuilder<DB, true, true>,
	>(
		mut self,
		build: F,
	) -> Self {
		let clause = build(Having::build_with_join());
		self.having_clause_builder = Some(if let Some(builder) = self.having_clause_builder {
			builder.merge_with_builder(clause)
		} else {
			clause
		});

		self
	}
}

impl<
		DB: Database,
		const HAS_COLUMNS: bool,
		const HAS_GROUP_BY: bool,
		const HAS_HAVING: bool,
		const HAS_ORDER_BY: bool,
	> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, false>
{
	pub fn select_column<N: Into<String>>(
		mut self,
		name: N,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, false> {
		self.select_columns
			.push(SelectPredicateKind::Column(SelectedColumn {
				column: ColRef {
					table_name: None,
					column_name: name.into(),
				},
				alias: None,
			}));

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_column_with_alias<N: Into<String>, A: Into<String>>(
		mut self,
		name: N,
		alias: A,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, false> {
		self.select_columns
			.push(SelectPredicateKind::Column(SelectedColumn {
				column: ColRef {
					table_name: None,
					column_name: name.into(),
				},
				alias: Some(alias.into()),
			}));

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_columns<N: Into<String>, S: IntoIterator<Item = N>>(
		mut self,
		names: S,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, false> {
		for name in names.into_iter() {
			self.select_columns
				.push(SelectPredicateKind::Column(SelectedColumn {
					column: ColRef {
						table_name: None,
						column_name: name.into(),
					},
					alias: None,
				}));
		}

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_columns_with_alias<
		N: Into<String>,
		A: Into<String>,
		S: IntoIterator<Item = (N, A)>,
	>(
		mut self,
		names: S,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, false> {
		for (name, alias) in names.into_iter() {
			self.select_columns
				.push(SelectPredicateKind::Column(SelectedColumn {
					column: ColRef {
						table_name: None,
						column_name: name.into(),
					},
					alias: Some(alias.into()),
				}));
		}

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn where_column_equal_to<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> Self {
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.and_column_equal_to(name, value)
		} else {
			Where::build().column_equal_to(name, value)
		});

		self
	}

	pub fn where_clause<
		F: FnOnce(WhereBuilder<DB, false, false>) -> WhereBuilder<DB, true, false>,
	>(
		mut self,
		build: F,
	) -> Self {
		let clause = build(Where::build());
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.merge_with_builder(clause)
		} else {
			clause
		});

		self
	}
}

impl<
		DB: Database,
		const HAS_COLUMNS: bool,
		const HAS_GROUP_BY: bool,
		const HAS_HAVING: bool,
		const HAS_ORDER_BY: bool,
	> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, true>
{
	pub fn select_column<T: Into<String>, C: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, true> {
		self.select_columns
			.push(SelectPredicateKind::Column(SelectedColumn {
				column: ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				alias: None,
			}));

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_column_with_alias<T: Into<String>, C: Into<String>, A: Into<String>>(
		mut self,
		table_name: T,
		column_name: C,
		alias: A,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, true> {
		self.select_columns
			.push(SelectPredicateKind::Column(SelectedColumn {
				column: ColRef {
					table_name: Some(table_name.into()),
					column_name: column_name.into(),
				},
				alias: Some(alias.into()),
			}));

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_columns<T: Into<String>, C: Into<String>, S: IntoIterator<Item = (T, C)>>(
		mut self,
		pairs: S,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, true> {
		for (table_name, column_name) in pairs.into_iter() {
			self.select_columns
				.push(SelectPredicateKind::Column(SelectedColumn {
					column: ColRef {
						table_name: Some(table_name.into()),
						column_name: column_name.into(),
					},
					alias: None,
				}));
		}

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn select_columns_with_alias<
		T: Into<String>,
		C: Into<String>,
		A: Into<String>,
		S: IntoIterator<Item = (T, C, A)>,
	>(
		mut self,
		pairs: S,
	) -> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, true> {
		for (table_name, column_name, alias) in pairs.into_iter() {
			self.select_columns
				.push(SelectPredicateKind::Column(SelectedColumn {
					column: ColRef {
						table_name: Some(table_name.into()),
						column_name: column_name.into(),
					},
					alias: Some(alias.into()),
				}));
		}

		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}

	pub fn where_column_equal_to<T: Into<String>, C: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		table_name: T,
		column_name: C,
		value: V,
	) -> Self {
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.and_column_equal_to(table_name, column_name, value)
		} else {
			Where::build_with_join().column_equal_to(table_name, column_name, value)
		});

		self
	}

	pub fn where_clause<
		F: FnOnce(WhereBuilder<DB, false, true>) -> WhereBuilder<DB, true, true>,
	>(
		mut self,
		build: F,
	) -> Self {
		let clause = build(Where::build_with_join());
		self.where_clause_builder = Some(if let Some(builder) = self.where_clause_builder {
			builder.merge_with_builder(clause)
		} else {
			clause
		});

		self
	}
}

impl<
		DB: Database,
		const HAS_COLUMNS: bool,
		const HAS_GROUP_BY: bool,
		const HAS_HAVING: bool,
		const HAS_JOIN: bool,
	> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, false, HAS_JOIN>
{
	pub fn with_order_by_clause(
		self,
		order_by_clause: OrderBy<DB>,
	) -> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, true, HAS_JOIN> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: Some(Ok(order_by_clause)),
		}
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_GROUP_BY: bool, const HAS_HAVING: bool>
	SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, false, false>
{
	pub fn order_by<S: IntoIterator<Item = (Col, bool)>, Col: Into<String>>(
		self,
		predicates: S,
	) -> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, true, false> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: Some(predicates.into_iter().collect::<Vec<_>>().try_into()),
		}
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_GROUP_BY: bool, const HAS_HAVING: bool>
	SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, false, true>
{
	pub fn order_by<
		S: IntoIterator<Item = (Tab, Col, bool)>,
		Tab: Into<String>,
		Col: Into<String>,
	>(
		self,
		predicates: S,
	) -> SelectBuilder<DB, HAS_COLUMNS, HAS_GROUP_BY, HAS_HAVING, true, true> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: self.group_by_clause,
			having_clause_builder: self.having_clause_builder,
			order_by_clause: Some(predicates.into_iter().collect::<Vec<_>>().try_into()),
		}
	}
}

impl<
		DB: Database,
		const HAS_COLUMNS: bool,
		const HAS_HAVING: bool,
		const HAS_ORDER_BY: bool,
		const HAS_JOIN: bool,
	> SelectBuilder<DB, HAS_COLUMNS, false, HAS_HAVING, HAS_ORDER_BY, HAS_JOIN>
{
	pub fn with_group_by_clause(
		self,
		group_by_clause: GroupBy<DB>,
	) -> SelectBuilder<DB, HAS_COLUMNS, true, HAS_HAVING, HAS_ORDER_BY, HAS_JOIN> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: Some(Ok(group_by_clause)),
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}
}

impl<DB: Database, const HAS_COLUMNS: bool, const HAS_HAVING: bool, const HAS_ORDER_BY: bool>
	SelectBuilder<DB, HAS_COLUMNS, false, HAS_HAVING, HAS_ORDER_BY, false>
{
	pub fn group_by<S: IntoIterator<Item = Col>, Col: Into<String>>(
		self,
		predicates: S,
	) -> SelectBuilder<DB, HAS_COLUMNS, true, HAS_HAVING, HAS_ORDER_BY, false> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: Some(predicates.into_iter().collect::<Vec<_>>().try_into()),
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}
}

// see: https://github.com/mikecaines/ursid-sqlx/issues/12
/*impl<DB: Database, const HAS_COLUMNS: bool, const HAS_HAVING: bool, const HAS_ORDER_BY: bool>
	SelectBuilder<DB, HAS_COLUMNS, false, HAS_HAVING, HAS_ORDER_BY, true>
{
	pub fn group_by<S: IntoIterator<Item = (Tab, Col)>, Tab: Into<String>, Col: Into<String>>(
		self,
		predicates: S,
	) -> SelectBuilder<DB, HAS_COLUMNS, true, HAS_HAVING, HAS_ORDER_BY, true> {
		SelectBuilder {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause_builder: self.where_clause_builder,
			group_by_clause: Some(predicates.into_iter().collect::<Vec<_>>().try_into()),
			having_clause_builder: self.having_clause_builder,
			order_by_clause: self.order_by_clause,
		}
	}
}*/

impl<
		DB: Database,
		const HAS_ORDER_BY: bool,
		const HAS_GROUP_BY: bool,
		const HAS_HAVING: bool,
		const HAS_JOIN: bool,
	> SelectBuilder<DB, true, HAS_GROUP_BY, HAS_HAVING, HAS_ORDER_BY, HAS_JOIN>
{
	pub fn finalize(self) -> Result<Select<DB>, SyntaxError> {
		if self.select_columns.is_empty() {
			return Err(SyntaxError::new(
				SyntaxErrorKind::MissingSelectPredicates,
				"".to_string(),
			));
		}

		Ok(Select {
			from_clause: self.from_clause,
			select_columns: self.select_columns,
			where_clause: self
				.where_clause_builder
				.map(|w| w.finalize())
				.transpose()?,
			group_by_clause: self.group_by_clause.transpose()?,
			having_clause: self
				.having_clause_builder
				.map(|w| w.finalize())
				.transpose()?,
			order_by_clause: self.order_by_clause.transpose()?,
		})
	}

	pub fn finalize_and_freeze(self) -> Result<FrozenSql<DB>, SyntaxError> {
		Ok(self.finalize()?.into_sql().freeze())
	}
}

#[derive(Debug)]
pub(crate) enum SelectPredicateKind<DB: Database> {
	Column(SelectedColumn),
	Expression(SelectedExpression<DB>),
}

impl<DB: Database> Clone for SelectPredicateKind<DB> {
	fn clone(&self) -> Self {
		match self {
			Self::Column(column) => Self::Column(column.clone()),
			Self::Expression(expr) => Self::Expression(expr.clone()),
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct SelectedColumn {
	pub(crate) column: ColRef,
	pub(crate) alias: Option<String>,
}

#[derive(Debug)]
pub(crate) struct SelectedExpression<DB: Database> {
	pub(crate) expr: Sql<DB>,
	pub(crate) alias: String,
}

impl<DB: Database> Clone for SelectedExpression<DB> {
	fn clone(&self) -> Self {
		Self {
			expr: self.expr.clone(),
			alias: self.alias.clone(),
		}
	}
}
