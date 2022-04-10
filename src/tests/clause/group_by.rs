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
	Sql<DB>: From<sql_lang::clause::GroupBy<DB>>,
{
	let sql: Sql<DB> = sql_lang::clause::GroupBy::build()
		.group_by_column("foo")
		.group_by_column("bar")
		.group_by_expression("coalesce(a, b)".into_raw_sql())
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(r#"group by "foo", "bar", coalesce(a, b)"#, &[])
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>("group by `foo`, `bar`, coalesce(a, b)", &[])
}
