use crate::sql_lang::IntoRawSql;
use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::clause::OrderBy<DB>>,
{
	let sql: Sql<DB> = sql_lang::clause::OrderBy::build()
		.order_by_column_asc("foo")
		.order_by_column_desc("bar")
		.order_by_expression("coalesce(a, b) desc".into_raw_sql())
		.limit(10, 200)
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"order by "foo" asc, "bar" desc, coalesce(a, b) desc limit $1 offset $2"#,
		&[10u32.into_sql_value(), 200u32.into_sql_value()],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"order by `foo` asc, `bar` desc, coalesce(a, b) desc limit ? offset ?",
		&[10u32.into_sql_value(), 200u32.into_sql_value()],
	)
}
