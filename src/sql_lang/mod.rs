use crate::value::Value;
use crate::{Database, IntoSqlValue};

pub mod clause;
pub mod expression;
pub mod statement;

/// Represents a fragment of SQL.
///
/// Internally composed of a String/parameterized-query part, and a Vec/bound-values part.
///
/// This type is not normally created directly, and is instead produced by the various builder
/// types for statements, clauses, etc.
#[derive(Debug)]
pub struct Sql<DB: Database> {
	pub(crate) text: String,
	pub(crate) values: Vec<Option<Value<DB>>>,
	pub(crate) placeholder_counter: u16,
}

impl<DB: Database> Sql<DB> {
	/// Determines whether the specified SQL fragment represents SQL NULL.
	pub fn is_null(expr: &Sql<DB>) -> bool {
		expr.text.trim().eq_ignore_ascii_case("null") && expr.values.is_empty()
	}

	pub(crate) fn new<T: Into<String>>(text: T, values: Vec<Option<Value<DB>>>) -> Self {
		Self {
			text: text.into(),
			values,
			placeholder_counter: 0,
		}
	}

	pub fn query(&self) -> &str {
		self.text.as_str()
	}

	pub fn params(&self) -> &[Option<Value<DB>>] {
		self.values.as_slice()
	}

	/// Appends the specified value to the fragment, creating a new fragment.
	///
	/// The value must implement [IntoSql](crate::IntoSql).
	pub fn append<S: IntoSql<DB>>(self, sql: S) -> Self {
		DB::sql_append(self, sql.into_sql())
	}

	/// Appends the specified "raw" value to the fragment, creating a new fragment.
	///
	/// The value must implement [IntoRawSql].
	pub fn raw_append<S: IntoRawSql<DB>>(self, raw_sql: S) -> Self {
		DB::sql_append(self, raw_sql.into_raw_sql())
	}

	pub fn freeze(self) -> FrozenSql<DB> {
		FrozenSql {
			text: self.text,
			values: Some(self.values),
		}
	}
}

impl<DB: Database> Clone for Sql<DB> {
	fn clone(&self) -> Self {
		Self {
			text: self.text.clone(),
			values: self.values.clone(),
			placeholder_counter: self.placeholder_counter,
		}
	}
}

/// Performs conversion of a value into [Sql], in an (SQL) safe manner.
///
/// The implementer of this trait guarantees that the conversion never introduces an
/// SQL-injection vulnerability.
///
/// Example use is to convert a SELECT statement builder into an SQL parameterized query.
///
/// This trait is automatically implemented for any type that implements `Into<Sql>`.
/// See also the IntoRawSql trait, for a more direct, less safe conversion process.
pub trait IntoSql<DB: Database>: Sized {
	fn into_sql(self) -> Sql<DB>;
}

impl<DB: Database> From<bool> for Sql<DB> {
	fn from(value: bool) -> Self {
		Sql::new(
			DB::sql_value_placeholder(),
			vec![DB::value_from_bool(value)],
		)
	}
}

impl<DB: Database> From<u8> for Sql<DB> {
	fn from(value: u8) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_u8(value)])
	}
}

impl<DB: Database> From<u16> for Sql<DB> {
	fn from(value: u16) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_u16(value)])
	}
}

impl<DB: Database> From<u32> for Sql<DB> {
	fn from(value: u32) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_u32(value)])
	}
}

impl<DB: Database> From<i8> for Sql<DB> {
	fn from(value: i8) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_i8(value)])
	}
}

impl<DB: Database> From<i16> for Sql<DB> {
	fn from(value: i16) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_i16(value)])
	}
}

impl<DB: Database> From<i32> for Sql<DB> {
	fn from(value: i32) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_i32(value)])
	}
}

impl<DB: Database> From<i64> for Sql<DB> {
	fn from(value: i64) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![DB::value_from_i64(value)])
	}
}

impl<DB: Database> From<char> for Sql<DB> {
	fn from(value: char) -> Self {
		Sql::new(
			DB::sql_value_placeholder(),
			vec![DB::value_from_char(value)],
		)
	}
}

impl<DB: Database> From<&str> for Sql<DB> {
	fn from(value: &str) -> Self {
		Sql::new(
			DB::sql_value_placeholder(),
			vec![DB::value_from_ref_str(value)],
		)
	}
}

impl<DB: Database> From<String> for Sql<DB> {
	fn from(value: String) -> Self {
		Sql::new(
			DB::sql_value_placeholder(),
			vec![DB::value_from_string(value)],
		)
	}
}

impl<DB: Database> From<Vec<u8>> for Sql<DB> {
	fn from(value: Vec<u8>) -> Self {
		Sql::new(
			DB::sql_value_placeholder(),
			vec![DB::value_from_bytes(value)],
		)
	}
}

/// Blanket conversion from Option<T:IntoSqlValue>
impl<DB: Database, T: IntoSqlValue<DB>> From<Option<T>> for Sql<DB> {
	fn from(value: Option<T>) -> Self {
		match value {
			None => Self::new("null", vec![]),
			Some(v) => Self::new(DB::sql_value_placeholder(), vec![v.into_sql_value()]),
		}
	}
}

/// Blanket conversion from &T:IntoSqlValue
impl<DB: Database, T: IntoSqlValue<DB> + Clone> From<&T> for Sql<DB> {
	fn from(value: &T) -> Self {
		Sql::new(DB::sql_value_placeholder(), vec![value.into_sql_value()])
	}
}

/// Blanket implementation of IntoSql for any type that implements `Into<Sql>`.
impl<DB: Database, T> IntoSql<DB> for T
where
	T: Into<Sql<DB>>,
{
	fn into_sql(self) -> Sql<DB> {
		self.into()
	}
}

/// Performs conversion of a value directly into [Sql], in an (SQL) unsafe manner.
///
/// The user of this trait must ensure that its use does not introduce an SQL-injection
/// vulnerability.
///
/// Example use is to generate an SQL query directly from an &str.
pub trait IntoRawSql<DB: Database>: Sized {
	fn into_raw_sql(self) -> Sql<DB>;
}

impl<DB: Database> IntoRawSql<DB> for String {
	fn into_raw_sql(self) -> Sql<DB> {
		Sql::new(self, vec![])
	}
}

impl<DB: Database> IntoRawSql<DB> for &str {
	fn into_raw_sql(self) -> Sql<DB> {
		Sql::new(self, vec![])
	}
}

impl<DB: Database> IntoRawSql<DB> for char {
	fn into_raw_sql(self) -> Sql<DB> {
		Sql::new(self, vec![])
	}
}

#[derive(Debug, Clone)]
pub(crate) struct ColRef {
	pub(crate) table_name: Option<String>,
	pub(crate) column_name: String,
}

#[derive(Debug)]
pub struct FrozenSql<DB: Database> {
	pub(crate) text: String,
	pub(crate) values: Option<Vec<Option<Value<DB>>>>,
}

impl<DB: Database> FrozenSql<DB> {
	pub fn query(&self) -> &str {
		self.text.as_str()
	}

	pub fn params(&self) -> Option<&[Option<Value<DB>>]> {
		self.values.as_deref()
	}
}

impl<DB: Database> Clone for FrozenSql<DB> {
	fn clone(&self) -> Self {
		Self {
			text: self.text.clone(),
			values: self.values.clone(),
		}
	}
}
