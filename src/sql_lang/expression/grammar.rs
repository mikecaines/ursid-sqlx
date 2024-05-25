use std::marker::PhantomData;

use crate::sql_lang::Sql;
use crate::{Database, IntoRawSql, IntoSql};

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
	GreaterThan,
	GreaterThanEqualTo,
	LessThan,
	LessThanEqualTo,
	EqualTo,
	NotEqualTo,
}

impl<DB: Database> From<ComparisonOp> for Sql<DB> {
	fn from(op: ComparisonOp) -> Self {
		match op {
			ComparisonOp::GreaterThan => '>'.into_raw_sql(),
			ComparisonOp::LessThan => '<'.into_raw_sql(),
			ComparisonOp::GreaterThanEqualTo => ">=".into_raw_sql(),
			ComparisonOp::LessThanEqualTo => "<=".into_raw_sql(),
			ComparisonOp::EqualTo => "=".into_raw_sql(),
			ComparisonOp::NotEqualTo => "<>".into_raw_sql(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct ComparisonCombo<DB: Database, L: Into<Sql<DB>>, R: Into<Sql<DB>>> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) lhs: L,
	pub(crate) op: ComparisonOp,
	pub(crate) rhs: R,
}

impl<DB: Database, L, R> From<ComparisonCombo<DB, L, R>> for Sql<DB>
where
	L: Into<Sql<DB>>,
	R: Into<Sql<DB>>,
{
	fn from(expr: ComparisonCombo<DB, L, R>) -> Self {
		let ComparisonCombo {
			db: _,
			lhs,
			op,
			rhs,
		} = expr;

		let rhs = rhs.into_sql();

		if Sql::is_null(&rhs) {
			match op {
				ComparisonOp::EqualTo => lhs.into_sql().raw_append(" is null"),
				ComparisonOp::NotEqualTo => lhs.into_sql().raw_append(" is not null"),
				_ => lhs
					.into_sql()
					.raw_append(' ')
					.append(op)
					.raw_append(' ')
					.append(rhs),
			}
		} else {
			lhs.into_sql()
				.raw_append(' ')
				.append(op)
				.raw_append(' ')
				.append(rhs)
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum LogicalOp {
	And,
	Or,
}

impl<DB: Database> From<LogicalOp> for Sql<DB> {
	fn from(combinator: LogicalOp) -> Self {
		match combinator {
			LogicalOp::And => "and".into_raw_sql(),
			LogicalOp::Or => "or".into_raw_sql(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct LogicalCombo<DB: Database, L: Into<Sql<DB>>, R: Into<Sql<DB>>> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) lhs: L,
	pub(crate) op: LogicalOp,
	pub(crate) rhs: R,
}

impl<DB: Database, L, R> From<LogicalCombo<DB, L, R>> for Sql<DB>
where
	L: Into<Sql<DB>>,
	R: Into<Sql<DB>>,
{
	fn from(expr: LogicalCombo<DB, L, R>) -> Self {
		let LogicalCombo {
			db: _,
			lhs,
			op,
			rhs,
		} = expr;

		lhs.into_sql()
			.raw_append(' ')
			.append(op)
			.raw_append(' ')
			.append(rhs)
	}
}

#[derive(Debug, Clone)]
pub struct LogicalNot<DB: Database, T: Into<Sql<DB>>> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) content: T,
}

impl<DB: Database, T> From<LogicalNot<DB, T>> for Sql<DB>
where
	T: Into<Sql<DB>>,
{
	fn from(expr: LogicalNot<DB, T>) -> Self {
		let LogicalNot { db: _, content } = expr;

		IntoRawSql::<DB>::into_raw_sql("not ").append(content)
	}
}

impl<DB: Database, T: Into<Sql<DB>>> LogicalNot<DB, T> {
	pub fn new(expr: T) -> Self {
		Self {
			db: PhantomData,
			content: expr,
		}
	}
}

#[derive(Debug, Clone)]
pub struct IsNull<DB: Database, T: Into<Sql<DB>>> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) lhs: T,
	pub(crate) not: bool,
}

impl<DB: Database, T> From<IsNull<DB, T>> for Sql<DB>
where
	T: Into<Sql<DB>>,
{
	fn from(expr: IsNull<DB, T>) -> Self {
		let IsNull { db: _, lhs, not } = expr;

		if not {
			lhs.into_sql().raw_append(" is not null")
		} else {
			lhs.into_sql().raw_append(" is null")
		}
	}
}

#[derive(Debug, Clone)]
pub struct Parenthesis<DB: Database, T: Into<Sql<DB>>> {
	pub(crate) db: PhantomData<DB>,
	pub(crate) content: T,
}

impl<DB: Database, T: Into<Sql<DB>>> Parenthesis<DB, T> {
	pub fn new(expr: T) -> Self {
		Self {
			db: PhantomData,
			content: expr,
		}
	}
}

impl<DB: Database, T> From<Parenthesis<DB, T>> for Sql<DB>
where
	T: Into<Sql<DB>>,
{
	fn from(expr: Parenthesis<DB, T>) -> Self {
		let Parenthesis { db: _, content } = expr;

		IntoRawSql::<DB>::into_raw_sql('(')
			.append(content)
			.raw_append(')')
	}
}
