use crate::tests::compare_sql;
use crate::{sql_lang, IntoSql, IntoSqlValue, SyntaxError};

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	compare_sql::<DB>(
		&sql_lang::clause::Where::build()
			.expression({
				use sql_lang::expression::*;

				coalesce(TableAndColumnReference::new("tbl", "col"), 123i32).greater_than(456i32)
			})
			.or_expression({
				use sql_lang::expression::*;

				TableAndColumnReference::new("tbl", "col2").less_than_equal_to(current_datetime())
			})
			.finalize()?
			.into_sql(),
		r#"where coalesce("tbl"."col", $1) > $2 or "tbl"."col2" <= current_timestamp()"#,
		&[123i32.into_sql_value(), 456i32.into_sql_value()],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	compare_sql::<DB>(
		&sql_lang::clause::Where::build()
			.expression({
				use sql_lang::expression::*;

				coalesce(TableAndColumnReference::new("tbl", "col"), 123i32).greater_than(456i32)
			})
			.or_expression({
				use sql_lang::expression::*;

				TableAndColumnReference::new("tbl", "col2").less_than_equal_to(current_datetime())
			})
			.finalize()?
			.into_sql(),
		r#"where coalesce(`tbl`.`col`, ?) > ? or `tbl`.`col2` <= current_timestamp()"#,
		&[123i32.into_sql_value(), 456i32.into_sql_value()],
	)
}
