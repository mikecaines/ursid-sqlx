use crate::tests::compare_sql;
use crate::value::Value;
use crate::{sql_lang, Database, IntoSql, Sql, SyntaxError};

fn test<DB: Database>(
	target_text: &str,
	target_params: &[Option<Value<DB>>],
) -> Result<(), SyntaxError>
where
	Sql<DB>: From<sql_lang::statement::Select<DB>>,
{
	let sql: Sql<DB> = sql_lang::statement::Select::build("some_table")
		.select_column("col1")
		.with_group_by_clause(
			sql_lang::clause::GroupBy::build()
				.group_by_column("col2")
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

	test::<DB>(r#"select "col1" from "some_table" group by "col2""#, &[])
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>("select `col1` from `some_table` group by `col2`", &[])
}
