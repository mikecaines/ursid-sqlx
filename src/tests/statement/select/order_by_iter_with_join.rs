use crate::sql_lang::clause::SqlFrom;
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
	let sql: Sql<DB> =
		sql_lang::statement::Select::build_with_join(SqlFrom::build("foo", "f").finalize()?)
			.select_column("f", "col1")
			.order_by([("f", "col1", true)])
			.finalize()?
			.into_sql();

	compare_sql(&sql, target_text, target_params)
}

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	test::<DB>(
		r#"select "f"."col1" from "foo" "f" order by "f"."col1" asc"#,
		&[],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	test::<DB>(
		"select `f`.`col1` from `foo` `f` order by `f`.`col1` asc",
		&[],
	)
}
