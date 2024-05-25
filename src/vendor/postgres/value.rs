use sqlx::Postgres;

use crate::query::requirements::SqlxQuery;
use crate::value::requirements::SqlxBindable;

#[derive(Clone, Debug, PartialEq)]
pub enum PostgresValueStorage {
	I16(i16),
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
	Text(String),
	Bytes(Vec<u8>),
	/* TODO: Date, Time, Datetime (as chrono types, instead of string?)
	 * TODO: Decimal type (behind feature, use external crate)
	 */
}

impl SqlxBindable<Postgres> for PostgresValueStorage {
	fn bind_to_sqlx<'q, Q: SqlxQuery<'q, Postgres>>(self, query: Q) -> Q {
		match self {
			Self::I16(v) => query.bind_to_sqlx(v),
			Self::I32(v) => query.bind_to_sqlx(v),
			Self::I64(v) => query.bind_to_sqlx(v),
			Self::F32(v) => query.bind_to_sqlx(v),
			Self::F64(v) => query.bind_to_sqlx(v),
			Self::Text(v) => query.bind_to_sqlx(v),
			Self::Bytes(v) => query.bind_to_sqlx(v),
		}
	}
}
