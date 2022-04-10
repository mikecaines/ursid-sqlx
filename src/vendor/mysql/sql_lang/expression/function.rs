use crate::sql_lang::expression::function::ast::{DateDiff, DateDiffInterval};
use crate::sql_lang::{IntoRawSql, Sql};
use sqlx::MySql;

pub fn render_date_diff(ast: DateDiff<MySql>) -> Sql<MySql> {
	"timestampdiff("
		.into_raw_sql()
		.raw_append(match ast.interval {
			DateDiffInterval::Day => "DAY",
			DateDiffInterval::Minute => "MINUTE",
		})
		.raw_append(',')
		.append(ast.datetime1)
		.raw_append(',')
		.append(ast.datetime2)
		.raw_append(')')
}
