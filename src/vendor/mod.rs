pub(crate) mod requirements {
	use crate::sql_lang::expression::function;
	use crate::value::requirements::SqlxBindable;
	use crate::value::Value;
	use crate::{crud, Database, ExecuteError, Sql};
	use std::fmt::Debug;
	use std::future::Future;
	use std::pin::Pin;

	pub trait DatabaseVendor<DB: Database> {
		type ValueStorage: SqlxBindable<DB> + Send + Clone + Debug + PartialEq;

		fn value_from_bool(value: bool) -> Option<Value<DB>>;
		fn value_from_u8(value: u8) -> Option<Value<DB>>;
		fn value_from_u16(value: u16) -> Option<Value<DB>>;
		fn value_from_u32(value: u32) -> Option<Value<DB>>;
		fn value_from_i8(value: i8) -> Option<Value<DB>>;
		fn value_from_i16(value: i16) -> Option<Value<DB>>;
		fn value_from_i32(value: i32) -> Option<Value<DB>>;
		fn value_from_i64(value: i64) -> Option<Value<DB>>;
		fn value_from_f32(value: f32) -> Option<Value<DB>>;
		fn value_from_f64(value: f64) -> Option<Value<DB>>;
		fn value_from_char(value: char) -> Option<Value<DB>>;
		fn value_from_ref_str(value: &str) -> Option<Value<DB>>;
		fn value_from_string(value: String) -> Option<Value<DB>>;
		fn value_from_bytes(value: Vec<u8>) -> Option<Value<DB>>;

		#[cfg(feature = "chrono-datetime")]
		fn value_from_chrono_native_datetime(value: chrono::NaiveDateTime) -> Option<Value<DB>>;

		#[cfg(feature = "chrono-datetime")]
		fn value_from_chrono_native_date(value: chrono::NaiveDate) -> Option<Value<DB>>;

		#[cfg(feature = "chrono-datetime")]
		fn value_from_chrono_native_time(value: chrono::NaiveTime) -> Option<Value<DB>>;

		#[cfg(feature = "chrono-datetime")]
		fn value_from_chrono_datetime<T>(value: chrono::DateTime<T>) -> Option<Value<DB>>
			where T: chrono::TimeZone,
						T::Offset: std::fmt::Display;

		fn sql_value_placeholder() -> &'static str;

		fn sql_quote_identifier<I: Into<String>>(id: I) -> String;

		fn sql_append(lhs: Sql<DB>, rhs: Sql<DB>) -> Sql<DB>;

		fn sql_from_expr_date_diff(ast: function::ast::DateDiff<DB>) -> Sql<DB>;

		fn execute_crud_insert<'a>(
			builder: crud::insert::InsertBuilder<DB>,
			connection: &'a mut DB::Connection,
		) -> Pin<Box<dyn Future<Output=Result<(), ExecuteError>> + Send + 'a>>;

		fn execute_crud_update<'a>(
			builder: crud::update::UpdateBuilder<DB, true>,
			connection: &'a mut DB::Connection,
		) -> Pin<Box<dyn Future<Output=Result<(), ExecuteError>> + Send + 'a>>;

		fn execute_crud_replace<'a>(
			builder: crud::replace::ReplaceBuilder<DB, true, true>,
			connection: &'a mut DB::Connection,
		) -> Pin<Box<dyn Future<Output=Result<(), ExecuteError>> + Send + 'a>>;

		fn execute_crud_delete<'a>(
			builder: crud::delete::DeleteBuilder<DB>,
			connection: &'a mut DB::Connection,
		) -> Pin<Box<dyn Future<Output=Result<(), ExecuteError>> + Send + 'a>>;
	}
}

#[cfg(any(feature = "mysql", feature = "doc"))]
pub mod mysql;

#[cfg(any(feature = "postgres", feature = "doc"))]
pub mod postgres;
