use crate::sql_lang::expression::function::ast::{DateDiff, DateDiffInterval};
use crate::{IntoRawSql, Sql};
use sqlx::Postgres;

pub fn render_date_diff(ast: DateDiff<Postgres>) -> Sql<Postgres> {
	match ast.interval {
		DateDiffInterval::Day => "date_part('day',"
			.into_raw_sql()
			.append(ast.datetime2)
			.raw_append("::timestamp - ")
			.append(ast.datetime1)
			.raw_append("::timestamp)"),
		DateDiffInterval::Minute => "(date_part('day',"
			.into_raw_sql()
			.append(ast.datetime2.clone())
			.raw_append("::timestamp - ")
			.append(ast.datetime1.clone())
			.raw_append("::timestamp) * 24 * 60")
			.raw_append(" + date_part('hour',")
			.append(ast.datetime2.clone())
			.raw_append("::timestamp - ")
			.append(ast.datetime1.clone())
			.raw_append("::timestamp) * 60")
			.raw_append(" + date_part('minute',")
			.append(ast.datetime2)
			.raw_append("::timestamp - ")
			.append(ast.datetime1)
			.raw_append("::timestamp))"),
	}
}
