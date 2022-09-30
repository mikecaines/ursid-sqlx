use crate::sql_lang::Sql;
use crate::Database;
use std::marker::PhantomData;

pub fn current_datetime<DB: Database>() -> ast::CurrentDatetime<DB> {
	ast::CurrentDatetime { db: PhantomData }
}

pub fn day_diff<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>>(
	datetime1: T1,
	datetime2: T2,
) -> ast::DateDiff<DB> {
	ast::DateDiff {
		interval: ast::DateDiffInterval::Day,
		datetime1: datetime1.into(),
		datetime2: datetime2.into(),
	}
}

pub fn minute_diff<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>>(
	datetime1: T1,
	datetime2: T2,
) -> ast::DateDiff<DB> {
	ast::DateDiff {
		interval: ast::DateDiffInterval::Minute,
		datetime1: datetime1.into(),
		datetime2: datetime2.into(),
	}
}

pub fn lower<DB: Database, T: Into<Sql<DB>>>(value: T) -> ast::Lower<DB> {
	ast::Lower {
		value: value.into(),
	}
}

pub fn coalesce<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>>(
	value1: T1,
	value2: T2,
) -> ast::Coalesce<DB> {
	ast::Coalesce {
		args: vec![value1.into(), value2.into()],
	}
}

pub fn coalesce3<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>, T3: Into<Sql<DB>>>(
	value1: T1,
	value2: T2,
	value3: T3,
) -> ast::Coalesce<DB> {
	ast::Coalesce {
		args: vec![value1.into(), value2.into(), value3.into()],
	}
}

pub fn concat<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>>(
	value1: T1,
	value2: T2,
) -> ast::Concat<DB> {
	ast::Concat {
		args: vec![value1.into(), value2.into()],
	}
}

pub fn concat3<DB: Database, T1: Into<Sql<DB>>, T2: Into<Sql<DB>>, T3: Into<Sql<DB>>>(
	value1: T1,
	value2: T2,
	value3: T3,
) -> ast::Concat<DB> {
	ast::Concat {
		args: vec![value1.into(), value2.into(), value3.into()],
	}
}

pub fn count<DB: Database, E: Into<Sql<DB>>>(expr: E) -> ast::Count<DB> {
	ast::Count { expr: expr.into() }
}

pub fn min<DB: Database, E: Into<Sql<DB>>>(expr: E) -> ast::Min<DB> {
	ast::Min { expr: expr.into() }
}

pub fn max<DB: Database, E: Into<Sql<DB>>>(expr: E) -> ast::Max<DB> {
	ast::Max { expr: expr.into() }
}

pub fn abs<DB: Database, E: Into<Sql<DB>>>(expr: E) -> ast::Abs<DB> {
	ast::Abs { expr: expr.into() }
}

pub mod ast {
	use crate::sql_lang::Sql;
	use crate::{Database, IntoRawSql};
	use std::marker::PhantomData;

	pub struct CurrentDatetime<DB: Database> {
		pub(crate) db: PhantomData<DB>,
	}

	impl<DB: Database> From<CurrentDatetime<DB>> for Sql<DB> {
		fn from(_ast: CurrentDatetime<DB>) -> Self {
			"current_timestamp()".into_raw_sql()
		}
	}

	pub struct DateDiff<DB: Database> {
		pub(crate) interval: DateDiffInterval,
		pub(crate) datetime1: Sql<DB>,
		pub(crate) datetime2: Sql<DB>,
	}

	impl<DB: Database> From<DateDiff<DB>> for Sql<DB> {
		fn from(ast: DateDiff<DB>) -> Self {
			DB::sql_from_expr_date_diff(ast)
		}
	}

	pub enum DateDiffInterval {
		Day,
		Minute,
	}

	pub struct Lower<DB: Database> {
		pub(crate) value: Sql<DB>,
	}

	impl<DB: Database> From<Lower<DB>> for Sql<DB> {
		fn from(ast: Lower<DB>) -> Self {
			let Lower { value } = ast;

			let sql: Sql<DB> = "lower(".into_raw_sql();
			sql.append(value).raw_append(')')
		}
	}

	pub struct Coalesce<DB: Database> {
		pub(crate) args: Vec<Sql<DB>>,
	}

	impl<DB: Database> From<Coalesce<DB>> for Sql<DB> {
		fn from(expr: Coalesce<DB>) -> Self {
			let mut sql: Sql<DB> = "coalesce(".into_raw_sql();

			for (i, arg) in expr.args.into_iter().enumerate() {
				if i > 0 {
					sql = sql.raw_append(", ");
				}

				sql = sql.append(arg);
			}

			sql = sql.raw_append(')');
			sql
		}
	}

	pub struct Concat<DB: Database> {
		pub(crate) args: Vec<Sql<DB>>,
	}

	impl<DB: Database> From<Concat<DB>> for Sql<DB> {
		fn from(expr: Concat<DB>) -> Self {
			let mut sql: Sql<DB> = "concat(".into_raw_sql();

			for (i, arg) in expr.args.into_iter().enumerate() {
				if i > 0 {
					sql = sql.raw_append(", ");
				}

				sql = sql.append(arg);
			}

			sql = sql.raw_append(')');
			sql
		}
	}

	pub struct Count<DB: Database> {
		pub(crate) expr: Sql<DB>,
	}

	impl<DB: Database> From<Count<DB>> for Sql<DB> {
		fn from(ast: Count<DB>) -> Self {
			let Count { expr } = ast;

			IntoRawSql::<DB>::into_raw_sql("count(")
				.append(expr)
				.raw_append(")")
		}
	}

	pub struct Min<DB: Database> {
		pub(crate) expr: Sql<DB>,
	}

	impl<DB: Database> From<Min<DB>> for Sql<DB> {
		fn from(ast: Min<DB>) -> Self {
			let Min { expr } = ast;

			IntoRawSql::<DB>::into_raw_sql("min(")
				.append(expr)
				.raw_append(")")
		}
	}

	pub struct Max<DB: Database> {
		pub(crate) expr: Sql<DB>,
	}

	impl<DB: Database> From<Max<DB>> for Sql<DB> {
		fn from(ast: Max<DB>) -> Self {
			let Max { expr } = ast;

			IntoRawSql::<DB>::into_raw_sql("max(")
				.append(expr)
				.raw_append(")")
		}
	}

	pub struct Abs<DB: Database> {
		pub(crate) expr: Sql<DB>,
	}

	impl<DB: Database> From<Abs<DB>> for Sql<DB> {
		fn from(ast: Abs<DB>) -> Self {
			let Abs { expr } = ast;

			IntoRawSql::<DB>::into_raw_sql("abs(")
				.append(expr)
				.raw_append(")")
		}
	}
}
