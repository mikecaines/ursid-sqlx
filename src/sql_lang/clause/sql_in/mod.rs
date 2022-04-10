pub(crate) use crate::sql_lang::clause::sql_in::in_kind::InKind;
use crate::sql_lang::statement::Select;
use crate::value::IntoSqlValue;
use crate::{Database, IntoRawSql, Sql};

mod in_kind;

#[derive(Debug)]
pub struct In<DB: Database> {
	pub(crate) kind: InKind<DB>,
}

impl<DB: Database> In<DB> {
	pub fn build() -> InBuilder<DB> {
		InBuilder {
			kind: InKind::Values(vec![]),
		}
	}

	pub fn from_select_statement(select_statement: Select<DB>) -> Self {
		Self {
			kind: InKind::Subquery(select_statement),
		}
	}
}

impl<DB: Database> Clone for In<DB> {
	fn clone(&self) -> Self {
		Self {
			kind: self.kind.clone(),
		}
	}
}

impl<DB: Database, I: IntoIterator<Item = T>, T: IntoSqlValue<DB>> From<I> for In<DB> {
	fn from(values: I) -> Self {
		In::build().values(values.into_iter()).finalize()
	}
}

impl<DB: Database> From<Select<DB>> for In<DB> {
	fn from(select_statement: Select<DB>) -> Self {
		In::from_select_statement(select_statement)
	}
}

impl<DB: Database> From<In<DB>> for Sql<DB> {
	fn from(in_clause: In<DB>) -> Self {
		match in_clause.kind {
			InKind::Values(values) => {
				let mut sql: Sql<DB>;

				if values.is_empty() {
					sql = "in (select null where 1 = 0)".into_raw_sql();
				} else {
					sql = "in (".into_raw_sql();

					for (i, value) in values.into_iter().enumerate() {
						if i > 0 {
							sql = sql.raw_append(',');
						}

						sql = sql.append(value);
					}

					sql = sql.raw_append(')');
				}

				sql
			}

			InKind::Subquery(select_clause) => IntoRawSql::<DB>::into_raw_sql("in (")
				.append(select_clause)
				.raw_append(')'),
		}
	}
}

#[derive(Debug)]
pub struct InBuilder<DB: Database> {
	kind: InKind<DB>,
}

impl<DB: Database> InBuilder<DB> {
	pub fn value<V: IntoSqlValue<DB>>(mut self, value: V) -> Self {
		if let InKind::Values(ref mut v) = self.kind {
			v.push(value.into_sql_value());
		} else {
			panic!("impossible: InBuilder::value() encountered !InKind::Values");
		}
		self
	}

	pub fn values<I: IntoIterator<Item = V>, V: IntoSqlValue<DB>>(mut self, values: I) -> Self {
		if let InKind::Values(ref mut v) = self.kind {
			for value in values.into_iter() {
				v.push(value.into_sql_value());
			}
		} else {
			panic!("impossible: InBuilder::value() encountered !InKind::Values");
		}

		self
	}

	pub fn finalize(self) -> In<DB> {
		In { kind: self.kind }
	}
}

impl<DB: Database> Clone for InBuilder<DB> {
	fn clone(&self) -> Self {
		Self {
			kind: self.kind.clone(),
		}
	}
}
