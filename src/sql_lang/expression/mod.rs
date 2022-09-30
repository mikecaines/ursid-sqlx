use crate::sql_lang::expression::grammar::{IsNull, LogicalCombo};
use crate::sql_lang::Sql;
use crate::Database;
pub use function::{
	abs, coalesce, coalesce3, concat, concat3, count, current_datetime, day_diff, lower, max, min,
	minute_diff,
};
pub use grammar::{ComparisonOp, LogicalNot, LogicalOp, Parenthesis};
pub use identifier::*;
use std::marker::PhantomData;

pub mod function;
pub mod grammar;
pub mod identifier;

pub mod prelude {
	pub use super::*;
}

pub trait SqlExpression<DB: Database>: Into<Sql<DB>> {
	fn greater_than<T: Into<Sql<DB>>>(self, rhs: T) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::GreaterThan,
			rhs,
		}
	}

	fn greater_than_equal_to<T: Into<Sql<DB>>>(
		self,
		rhs: T,
	) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::GreaterThanEqualTo,
			rhs,
		}
	}

	fn less_than<T: Into<Sql<DB>>>(self, rhs: T) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::LessThan,
			rhs,
		}
	}

	fn less_than_equal_to<T: Into<Sql<DB>>>(self, rhs: T) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::LessThanEqualTo,
			rhs,
		}
	}

	fn equal_to<T: Into<Sql<DB>>>(self, rhs: T) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::EqualTo,
			rhs,
		}
	}

	fn not_equal_to<T: Into<Sql<DB>>>(self, rhs: T) -> grammar::ComparisonCombo<DB, Self, T> {
		grammar::ComparisonCombo {
			db: PhantomData,
			lhs: self,
			op: ComparisonOp::NotEqualTo,
			rhs,
		}
	}

	fn and<T: Into<Sql<DB>>>(self, rhs: T) -> LogicalCombo<DB, Self, T> {
		LogicalCombo {
			db: PhantomData,
			lhs: self,
			op: LogicalOp::And,
			rhs,
		}
	}

	fn or<T: Into<Sql<DB>>>(self, rhs: T) -> LogicalCombo<DB, Self, T> {
		LogicalCombo {
			db: PhantomData,
			lhs: self,
			op: LogicalOp::Or,
			rhs,
		}
	}

	#[allow(clippy::wrong_self_convention)]
	fn is_null(self) -> IsNull<DB, Self> {
		IsNull {
			db: PhantomData,
			lhs: self,
			not: false,
		}
	}

	#[allow(clippy::wrong_self_convention)]
	fn is_not_null(self) -> IsNull<DB, Self> {
		IsNull {
			db: PhantomData,
			lhs: self,
			not: true,
		}
	}

	fn wrap_in_parenthesis(self) -> grammar::Parenthesis<DB, Self> {
		grammar::Parenthesis::new(self)
	}

	fn wrap_in_not(self) -> grammar::LogicalNot<DB, Self> {
		grammar::LogicalNot::new(self)
	}
}

impl<DB: Database, T> SqlExpression<DB> for T where T: Into<Sql<DB>> {}
