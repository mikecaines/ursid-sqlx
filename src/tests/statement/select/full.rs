use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::statement::Select<DB>>,
	Sql<DB>: From<sql_lang::expression::identifier::ColumnReference<DB>>,
	Sql<DB>: From<sql_lang::expression::function::ast::Count<DB>>,
	Sql<DB>: From<i32>,
	Sql<DB>: From<
		sql_lang::expression::grammar::ComparisonCombo<
			DB,
			sql_lang::expression::function::ast::Count<DB>,
			i32,
		>,
	>,
{
	let sql: Sql<DB> = sql_lang::statement::Select::build("some_table")
		.select_column("col1")
		.select_expression(
			{
				use crate::sql_lang::expression::prelude::*;
				count(ColumnReference::new("col2"))
			},
			"some_total",
		)
		.where_column_equal_to("foo_id", 1i32)
		.where_clause(|clause| clause.column_equal_to("foo555", "bar555"))
		.with_group_by_clause(
			sql_lang::clause::GroupBy::build()
				.group_by_column("col1")
				.finalize()?,
		)
		.with_having_clause(
			sql_lang::clause::Having::build()
				.expression({
					use crate::sql_lang::expression::prelude::*;
					count(ColumnReference::new("col2")).greater_than_equal_to(5i32)
				})
				.finalize()?,
		)
		.with_order_by_clause(
			crate::sql_lang::clause::OrderBy::build()
				.order_by_column_desc("some_total")
				.finalize()?,
		)
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"select "col1", count("col2") as "some_total" from "some_table" where "foo_id"=$1 and "foo555"=$2 group by "col1" having count("col2") >= $3 order by "some_total" desc"#,
		&[
			1i32.into_sql_value(),
			"bar555".into_sql_value(),
			5i32.into_sql_value(),
		],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"select `col1`, count(`col2`) as `some_total` \
		from `some_table` \
		where `foo_id`=? \
		and `foo555`=? \
		group by `col1` \
		having count(`col2`) >= ? \
		order by `some_total` desc",
		&[
			1i32.into_sql_value(),
			"bar555".into_sql_value(),
			5i32.into_sql_value(),
		],
	)
}
