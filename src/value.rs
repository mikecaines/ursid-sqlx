use crate::Database;

pub trait IntoSqlValue<DB: Database> {
	fn into_sql_value(self) -> Option<Value<DB>>;
}

impl<DB: Database> IntoSqlValue<DB> for bool {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_bool(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for u8 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_u8(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for u16 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_u16(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for u32 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_u32(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for i8 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_i8(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for i16 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_i16(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for i32 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_i32(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for i64 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_i64(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for f32 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_f32(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for f64 {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_f64(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for char {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_char(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for &str {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_ref_str(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for String {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_string(self)
	}
}

impl<DB: Database> IntoSqlValue<DB> for Vec<u8> {
	fn into_sql_value(self) -> Option<Value<DB>> {
		DB::value_from_bytes(self)
	}
}

#[cfg(feature = "chrono-datetime")]
mod chrono {
	use crate::value::Value;
	use crate::{Database, IntoSqlValue};

	impl<DB: Database> IntoSqlValue<DB> for chrono::NaiveDateTime {
		fn into_sql_value(self) -> Option<Value<DB>> {
			DB::value_from_chrono_native_datetime(self)
		}
	}

	impl<DB: Database> IntoSqlValue<DB> for chrono::NaiveDate {
		fn into_sql_value(self) -> Option<Value<DB>> {
			DB::value_from_chrono_native_date(self)
		}
	}

	impl<DB: Database> IntoSqlValue<DB> for chrono::NaiveTime {
		fn into_sql_value(self) -> Option<Value<DB>> {
			DB::value_from_chrono_native_time(self)
		}
	}
}

impl<DB: Database> IntoSqlValue<DB> for Value<DB> {
	fn into_sql_value(self) -> Option<Value<DB>> {
		Some(self)
	}
}

/// Blanket conversion from Option<T:IntoSqlValue>
impl<DB: Database, T: IntoSqlValue<DB>> IntoSqlValue<DB> for Option<T> {
	fn into_sql_value(self) -> Option<Value<DB>> {
		self.and_then(|v| v.into_sql_value())
	}
}

/// Blanket conversion from &T:IntoSqlValue
// Using .to_owned() instead of .clone() here would seem more relevant, but it causes
// recursion.
impl<DB: Database, T: IntoSqlValue<DB> + Clone> IntoSqlValue<DB> for &T {
	fn into_sql_value(self) -> Option<Value<DB>> {
		self.clone().into_sql_value()
	}
}

// TODO: can't impl this (or generic version) due to orphan rules
/*impl<DB: Database> From<u32> for Option<Value<DB>> {
	fn from(value: u32) -> Self {
		DB::value_from_u32(value)
	}
}*/

pub(crate) mod requirements {
	use crate::query::requirements::SqlxQuery;

	/// Implemented by [ValueStorage](crate::vendor::requirements::ValueStorage) types.
	///
	/// Used in combination with [SqlxQuery] to bind any `ursid` value storage type,
	/// to any `sqlx` query type.
	pub trait SqlxBindable<DB: sqlx::Database> {
		/// Binds self to the specified `sqlx` query.
		fn bind_to_sqlx<'q, Q: SqlxQuery<'q, DB>>(self, query: Q) -> Q;
	}
}

#[derive(Debug)]
pub struct Value<DB: Database> {
	pub(crate) logical_kind: ValueLogicalKind,
	pub(crate) storage_kind: DB::ValueStorage,
}

impl<DB: Database> Value<DB> {
	pub(crate) fn new(logical_kind: ValueLogicalKind, storage_kind: DB::ValueStorage) -> Self {
		Self {
			logical_kind,
			storage_kind,
		}
	}
}

impl<DB: Database> Clone for Value<DB> {
	fn clone(&self) -> Self {
		Self {
			logical_kind: self.logical_kind,
			storage_kind: self.storage_kind.clone(),
		}
	}
}

impl<DB: Database> PartialEq for Value<DB> {
	fn eq(&self, other: &Self) -> bool {
		self.logical_kind == other.logical_kind && self.storage_kind == other.storage_kind
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ValueLogicalKind {
	Bool,
	U8,
	U16,
	U32,
	I8,
	I16,
	I32,
	I64,
	F32,
	F64,
	Datetime,
	Date,
	Time,
	Text,
	Bytes,
}
