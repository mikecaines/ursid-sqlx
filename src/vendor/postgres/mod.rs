use std::future::Future;
use std::pin::Pin;

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use sqlx::Postgres;

use self::value::PostgresValueStorage;
use crate::sql_lang::expression::function;
use crate::value::{Value, ValueLogicalKind};
use crate::{ExecuteError, Sql};

mod crud;
mod sql_lang;
mod value;

fn quote_identifier<I: Into<String>>(identifier: I) -> String {
	let mut identifier: String = identifier.into().replace("\"", "\"\"");
	identifier.insert(0, '"');
	identifier.push('"');
	identifier
}

impl crate::Database for Postgres {}

lazy_static! {
	static ref PLACEHOLDER_REGEX: Regex = Regex::new(r"\$\d+").expect("Placeholder regex failed");
}

impl crate::vendor::requirements::DatabaseVendor<Postgres> for Postgres {
	type ValueStorage = PostgresValueStorage;

	fn value_from_bool(value: bool) -> Option<Value<Postgres>> {
		// Postgres boolean type is selected as 't'/'f'.
		// This has no transparent equivalent in other vendors.
		// Store as smallest int type instead.
		Some(Value::new(
			ValueLogicalKind::Bool,
			PostgresValueStorage::I16(value as i16),
		))
	}

	fn value_from_u8(value: u8) -> Option<Value<Postgres>> {
		// Postgres has no unsigned int types.
		// Use smallest signed type instead.
		Some(Value::new(
			ValueLogicalKind::U8,
			PostgresValueStorage::I16(value as i16),
		))
	}

	fn value_from_u16(value: u16) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::U16,
			PostgresValueStorage::I32(value as i32),
		))
	}

	fn value_from_u32(value: u32) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::U32,
			PostgresValueStorage::I64(value as i64),
		))
	}

	fn value_from_i8(value: i8) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::I8,
			PostgresValueStorage::I16(value as i16),
		))
	}

	fn value_from_i16(value: i16) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::I16,
			PostgresValueStorage::I16(value),
		))
	}

	fn value_from_i32(value: i32) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::I32,
			PostgresValueStorage::I32(value),
		))
	}

	fn value_from_i64(value: i64) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::I64,
			PostgresValueStorage::I64(value),
		))
	}

	fn value_from_f32(value: f32) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::F32,
			PostgresValueStorage::F32(value),
		))
	}

	fn value_from_f64(value: f64) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::F64,
			PostgresValueStorage::F64(value),
		))
	}

	fn value_from_char(value: char) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			PostgresValueStorage::Text(value.to_string()),
		))
	}

	fn value_from_ref_str(value: &str) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			PostgresValueStorage::Text(value.to_string()),
		))
	}

	fn value_from_string(value: String) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			PostgresValueStorage::Text(value),
		))
	}

	fn value_from_bytes(value: Vec<u8>) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Bytes,
			PostgresValueStorage::Bytes(value),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_datetime(value: chrono::NaiveDateTime) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Datetime,
			PostgresValueStorage::Text(value.format("%Y-%m-%d %H:%M:%S.%f").to_string()),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_date(value: chrono::NaiveDate) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Date,
			PostgresValueStorage::Text(value.format("%Y-%m-%d").to_string()),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_time(value: chrono::NaiveTime) -> Option<Value<Postgres>> {
		Some(Value::new(
			ValueLogicalKind::Time,
			PostgresValueStorage::Text(value.format("%H:%M:%S.%f").to_string()),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_datetime<T>(value: chrono::DateTime<T>) -> Option<Value<Postgres>>
	where
		T: chrono::TimeZone,
		T::Offset: std::fmt::Display,
	{
		Some(Value::new(
			ValueLogicalKind::Datetime,
			PostgresValueStorage::Text(value.format("%Y-%m-%d %H:%M:%S.%f %:z").to_string()),
		))
	}

	fn sql_value_placeholder() -> &'static str {
		"$0"
	}

	fn sql_quote_identifier<I: Into<String>>(id: I) -> String {
		quote_identifier(id)
	}

	fn sql_append(mut lhs: Sql<Postgres>, rhs: Sql<Postgres>) -> Sql<Postgres> {
		let Sql {
			text,
			values,
			placeholder_counter: _,
		} = rhs;

		let text = PLACEHOLDER_REGEX.replace_all(text.as_str(), |_captures: &Captures| {
			lhs.placeholder_counter += 1;
			format!("${}", lhs.placeholder_counter)
		});

		lhs.text.push_str(text.as_ref());
		lhs.values.extend(values);
		lhs
	}

	fn sql_from_expr_date_diff(ast: function::ast::DateDiff<Postgres>) -> Sql<Postgres> {
		sql_lang::expression::function::render_date_diff(ast)
	}

	fn execute_crud_insert<'a>(
		builder: crate::crud::insert::InsertBuilder<Postgres>,
		connection: &'a mut <Postgres as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::insert::execute(builder, connection))
	}

	fn execute_crud_update<'a>(
		builder: crate::crud::update::UpdateBuilder<Postgres, true>,
		connection: &'a mut <Postgres as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::update::execute(builder, connection))
	}

	fn execute_crud_replace<'a>(
		builder: crate::crud::replace::ReplaceBuilder<Postgres, true, true>,
		connection: &'a mut <Postgres as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::replace::execute(builder, connection))
	}

	fn execute_crud_delete<'a>(
		builder: crate::crud::delete::DeleteBuilder<Postgres>,
		connection: &'a mut <Postgres as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::delete::execute(builder, connection))
	}
}
