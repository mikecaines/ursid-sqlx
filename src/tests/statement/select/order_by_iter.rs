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
	let sql: Sql<DB> = sql_lang::statement::Select::build("foo")
		.select_column("col1")
		.order_by([("col1", false)])
		.finalize()?
		.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(r#"select "col1" from "foo" order by "col1" desc"#, &[])
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>("select `col1` from `foo` order by `col1` desc", &[])
}
