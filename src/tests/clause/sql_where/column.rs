use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::clause::Where<DB>>,
{
	let sql: Sql<DB> = sql_lang::clause::Where::build()
		.column_equal_to("one", 1i32)
		.and_column_equal_to("two", 2i32)
		.and_column_equal_to("three", 3i32)
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"where "one"=$1 and "two"=$2 and "three"=$3"#,
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			3i32.into_sql_value(),
		],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"where `one`=? and `two`=? and `three`=?",
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			3i32.into_sql_value(),
		],
	)
}
