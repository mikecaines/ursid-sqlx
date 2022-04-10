use crate::tests::compare_sql;
use crate::{sql_lang, IntoSql, IntoSqlValue, SyntaxError};

#[test]
#[cfg(feature = "postgres")]
fn postgres() -> Result<(), SyntaxError> {
	type DB = sqlx::Postgres;

	compare_sql::<DB>(
		&{
			use sql_lang::expression::*;

			coalesce(
				TableAndColumnReference::new("tbl", "created_datetime"),
				current_datetime(),
			)
			.greater_than("2000-01-01")
			.and(
				sql_lang::statement::Select::build("tbl")
					.select_column("view_count")
					.finalize()?
					.wrap_in_parenthesis()
					.less_than(123i32),
			)
			.and(TableAndColumnReference::new("tbl", "status").not_equal_to(Option::<u32>::None))
			.and(TableAndColumnReference::new("tbl", "status").is_not_null())
			.into_sql()
		},
		r#"coalesce("tbl"."created_datetime", current_timestamp()) > $1 and (select "view_count" from "tbl") < $2 and "tbl"."status" is not null and "tbl"."status" is not null"#,
		&["2000-01-01".into_sql_value(), 123i32.into_sql_value()],
	)
}

#[test]
#[cfg(feature = "mysql")]
fn mysql() -> Result<(), SyntaxError> {
	type DB = sqlx::MySql;

	compare_sql::<DB>(
		&{
			use sql_lang::expression::*;

			coalesce(
				TableAndColumnReference::new("tbl", "created_datetime"),
				current_datetime(),
			)
				.greater_than("2000-01-01")
				.and(
					sql_lang::statement::Select::build("tbl")
						.select_column("view_count")
						.finalize()?
						.wrap_in_parenthesis()
						.less_than(123i32),
				)
				.and(TableAndColumnReference::new("tbl", "status").not_equal_to(Option::<u32>::None))
				.and(TableAndColumnReference::new("tbl", "status").is_not_null())
				.into_sql()
		},
		"coalesce(`tbl`.`created_datetime`, current_timestamp()) > ? and (select `view_count` from `tbl`) < ? and `tbl`.`status` is not null and `tbl`.`status` is not null",
		&[
			"2000-01-01".into_sql_value(),
			123i32.into_sql_value(),
		],
	)
}
