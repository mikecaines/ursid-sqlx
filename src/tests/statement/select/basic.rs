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
{
	let sql: Sql<DB> = sql_lang::statement::Select::build("some_table")
		.select_column("col1")
		.select_column_with_alias("col2", "foo")
		.where_column_equal_to("one", 1i32)
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, &target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"select "col1", "col2" as "foo" from "some_table" where "one"=$1"#,
		&[1i32.into_sql_value()],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"select `col1`, `col2` as `foo` from `some_table` where `one`=?",
		&[1i32.into_sql_value()],
	)
}
