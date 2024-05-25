use sqlx::MySql;

use crate::query::requirements::SqlxQuery;
use crate::value::requirements::SqlxBindable;

#[derive(Clone, Debug, PartialEq)]
pub enum MySqlValueStorage {
	U8(u8),
	U16(u16),
	U32(u32),
	I8(i8),
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

impl SqlxBindable<MySql> for MySqlValueStorage {
	fn bind_to_sqlx<'q, Q: SqlxQuery<'q, MySql>>(self, query: Q) -> Q {
		match self {
			Self::U8(v) => query.bind_to_sqlx(v),
			Self::U16(v) => query.bind_to_sqlx(v),
			Self::U32(v) => query.bind_to_sqlx(v),
			Self::I8(v) => query.bind_to_sqlx(v),
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
