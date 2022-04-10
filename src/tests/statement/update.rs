use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::statement::Update<DB>>,
{
	let sql: Sql<DB> = sql_lang::statement::Update::build("some_table")
		.update_column("one", 1i32)
		.update_column("two", 2i32)
		.where_column_equal_to("10", 10i32)
		.where_column_equal_to("11", 11i32)
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"update "some_table" set "one"=$1,"two"=$2 where "10"=$3 and "11"=$4"#,
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			10i32.into_sql_value(),
			11i32.into_sql_value(),
		],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"update `some_table` set `one`=?,`two`=? where `10`=? and `11`=?",
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			10i32.into_sql_value(),
			11i32.into_sql_value(),
		],
	)
}
