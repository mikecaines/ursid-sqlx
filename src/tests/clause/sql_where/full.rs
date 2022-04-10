use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	&'static str: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::clause::Where<DB>>,
{
	let sql: Sql<DB> = sql_lang::clause::Where::build()
		.column_equal_to("one", 1i32)
		.and_column_equal_to("two", 2i32)
		.and_column_equal_to("three", 3i32)
		.and_column_in("foo", [4i32, 5i32, 6i32])
		.and_column_in("bar", [7i32, 8i32, 9i32])
		.and_column_equal_to("four", "10")
		.or_group(
			sql_lang::clause::Where::build()
				.column_equal_to("five", 11i32)
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
		r#"where "one"=$1 and "two"=$2 and "three"=$3 and "foo" in ($4,$5,$6) and "bar" in ($7,$8,$9) and "four"=$10 or ("five"=$11)"#,
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			3i32.into_sql_value(),
			4i32.into_sql_value(),
			5i32.into_sql_value(),
			6i32.into_sql_value(),
			7i32.into_sql_value(),
			8i32.into_sql_value(),
			9i32.into_sql_value(),
			"10".into_sql_value(),
			11i32.into_sql_value(),
		],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"where `one`=? and `two`=? and `three`=? and `foo` in (?,?,?) and `bar` in (?,?,?) and `four`=? or (`five`=?)", 
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			3i32.into_sql_value(),
			4i32.into_sql_value(),
			5i32.into_sql_value(),
			6i32.into_sql_value(),
			7i32.into_sql_value(),
			8i32.into_sql_value(),
			9i32.into_sql_value(),
			"10".into_sql_value(),
			11i32.into_sql_value(),
		],
	)
}
