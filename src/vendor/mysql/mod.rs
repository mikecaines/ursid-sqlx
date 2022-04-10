use self::value::MySqlValueStorage;
use crate::sql_lang::expression::function;
use crate::value::{Value, ValueLogicalKind};
use crate::{ExecuteError, Sql};
use sqlx::MySql;
use std::future::Future;
use std::pin::Pin;

mod crud;
mod sql_lang;
mod value;

fn quote_identifier<I: Into<String>>(identifier: I) -> String {
	let mut identifier: String = identifier.into().replace("`", "``");
	identifier.insert(0, '`');
	identifier.push('`');
	identifier
}

impl crate::Database for MySql {}

impl crate::vendor::requirements::DatabaseVendor<MySql> for MySql {
	type ValueStorage = MySqlValueStorage;

	fn value_from_bool(value: bool) -> Option<Value<MySql>> {
		// Mysql boolean/bit type is has special literal syntax.
		// This has no transparent equivalent in other vendors.
		// Store as smallest int type instead.
		Some(Value::new(
			ValueLogicalKind::Bool,
			MySqlValueStorage::U8(value as u8),
		))
	}

	fn value_from_u8(value: u8) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::U8,
			MySqlValueStorage::U8(value),
		))
	}

	fn value_from_u16(value: u16) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::U16,
			MySqlValueStorage::U16(value),
		))
	}

	fn value_from_u32(value: u32) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::U32,
			MySqlValueStorage::U32(value),
		))
	}

	fn value_from_i8(value: i8) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::I8,
			MySqlValueStorage::I8(value),
		))
	}

	fn value_from_i16(value: i16) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::I16,
			MySqlValueStorage::I16(value),
		))
	}

	fn value_from_i32(value: i32) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::I32,
			MySqlValueStorage::I32(value),
		))
	}

	fn value_from_i64(value: i64) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::I64,
			MySqlValueStorage::I64(value),
		))
	}

	fn value_from_f32(value: f32) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::F32,
			MySqlValueStorage::F32(value),
		))
	}

	fn value_from_f64(value: f64) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::F64,
			MySqlValueStorage::F64(value),
		))
	}

	fn value_from_char(value: char) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			MySqlValueStorage::Text(value.to_string()),
		))
	}

	fn value_from_ref_str(value: &str) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			MySqlValueStorage::Text(value.to_string()),
		))
	}

	fn value_from_string(value: String) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Text,
			MySqlValueStorage::Text(value),
		))
	}

	fn value_from_bytes(value: Vec<u8>) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Bytes,
			MySqlValueStorage::Bytes(value),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_datetime(value: chrono::NaiveDateTime) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Datetime,
			MySqlValueStorage::Text(value.format("%Y-%m-%d %H:%M:%S.%f").to_string()),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_date(value: chrono::NaiveDate) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Date,
			MySqlValueStorage::Text(value.format("%Y-%m-%d").to_string()),
		))
	}

	#[cfg(feature = "chrono-datetime")]
	fn value_from_chrono_native_time(value: chrono::NaiveTime) -> Option<Value<MySql>> {
		Some(Value::new(
			ValueLogicalKind::Time,
			MySqlValueStorage::Text(value.format("%H:%M:%S.%f").to_string()),
		))
	}

	fn sql_value_placeholder() -> &'static str {
		"?"
	}

	fn sql_quote_identifier<I: Into<String>>(id: I) -> String {
		quote_identifier(id)
	}

	fn sql_append(mut lhs: Sql<MySql>, rhs: Sql<MySql>) -> Sql<MySql> {
		let Sql {
			text,
			values,
			placeholder_counter: _,
		} = rhs;

		lhs.text.push_str(text.as_str());
		lhs.values.extend(values);
		lhs
	}

	fn sql_from_expr_date_diff(ast: function::ast::DateDiff<MySql>) -> Sql<MySql> {
		sql_lang::expression::function::render_date_diff(ast)
	}

	fn execute_crud_insert<'a>(
		builder: crate::crud::insert::InsertBuilder<MySql>,
		connection: &'a mut <MySql as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::insert::execute(builder, connection))
	}

	fn execute_crud_update<'a>(
		builder: crate::crud::update::UpdateBuilder<MySql, true>,
		connection: &'a mut <MySql as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::update::execute(builder, connection))
	}

	fn execute_crud_replace<'a>(
		builder: crate::crud::replace::ReplaceBuilder<MySql, true, true>,
		connection: &'a mut <MySql as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::replace::execute(builder, connection))
	}

	fn execute_crud_delete<'a>(
		builder: crate::crud::delete::DeleteBuilder<MySql>,
		connection: &'a mut <MySql as sqlx::Database>::Connection,
	) -> Pin<Box<dyn Future<Output = Result<(), ExecuteError>> + Send + 'a>> {
		Box::pin(crud::delete::execute(builder, connection))
	}
}
