use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, IntoSqlValue, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	i32: IntoSqlValue<DB>,
	Sql<DB>: From<sql_lang::statement::Delete<DB>>,
{
	let sql: Sql<DB> = sql_lang::statement::Delete::build("some_table")
		.with_where_clause(
			sql_lang::clause::Where::build()
				.column_equal_to("col1", 1i32)
				.or_column_equal_to("col2", 2i32)
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
		r#"delete from "some_table" where "col1"=$1 or "col2"=$2"#,
		&[1i32.into_sql_value(), 2i32.into_sql_value()],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"delete from `some_table` where `col1`=? or `col2`=?",
		&[1i32.into_sql_value(), 2i32.into_sql_value()],
	)
}
