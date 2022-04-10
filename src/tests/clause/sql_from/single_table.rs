use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::clause::SqlFrom<DB>>,
{
	let sql: Sql<DB> = sql_lang::clause::SqlFrom::build("some_table", "st")
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(r#"from "some_table" "st""#, &[])
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>("from `some_table` `st`", &[])
}
