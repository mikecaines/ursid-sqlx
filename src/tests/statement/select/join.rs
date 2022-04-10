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
	Sql<DB>: From<
		crate::sql_lang::expression::grammar::ComparisonCombo<
			DB,
			sql_lang::expression::identifier::TableAndColumnReference<DB>,
			sql_lang::expression::identifier::TableAndColumnReference<DB>,
		>,
	>,
	Sql<DB>: From<sql_lang::expression::identifier::TableAndColumnReference<DB>>,
{
	let sql: Sql<DB> = sql_lang::statement::Select::build_with_join(
		sql_lang::clause::SqlFrom::build("table1", "t1")
			.inner_join("table2", "t2", ("t2", "fk_t1_id", "t1", "id").try_into()?)
			.finalize()?,
	)
	.select_column("t1", "col1")
	.select_columns([("t2", "col2")])
	.where_column_equal_to("t1", "col1", 1i32)
	.with_where_clause(
		sql_lang::clause::Where::build_with_join()
			.column_equal_to("t1", "col2", 2i32)
			.and_column_equal_to("t2", "col1", 3i32)
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
		r#"select "t1"."col1", "t2"."col2" from "table1" "t1" inner join "table2" "t2" on "t2"."fk_t1_id" = "t1"."id" where "t1"."col1"=$1 and "t1"."col2"=$2 and "t2"."col1"=$3"#,
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
		"select `t1`.`col1`, `t2`.`col2` from `table1` `t1` inner join `table2` `t2` on `t2`.`fk_t1_id` = `t1`.`id` where `t1`.`col1`=? and `t1`.`col2`=? and `t2`.`col1`=?",
		&[
			1i32.into_sql_value(),
			2i32.into_sql_value(),
			3i32.into_sql_value(),
		],
	)
}
