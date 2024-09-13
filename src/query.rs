use crate::query::requirements::SqlxQuery;
use crate::sql_lang::FrozenSql;
use crate::value::requirements::SqlxBindable;
use crate::{Database, ExecuteError, QueryError};

pub(crate) mod requirements {
	/// Implemented by the various [sqlx] query types.
	/// Such as [Query](sqlx::query::Query), [QueryAs](sqlx::query::QueryAs).
	///
	/// Used in combination with [SqlxBindable] to bind any `ursid` value storage type,
	/// to any `sqlx` query type.
	pub trait SqlxQuery<'q, DB: sqlx::Database> {
		/// Binds the specified value to self.
		fn bind_to_sqlx<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(
			self,
			value: T,
		) -> Self;
	}
}

pub fn query_scalar<'q, DB, O>(
	sql: &'q mut FrozenSql<DB>,
) -> Result<
	sqlx::query::QueryScalar<'q, DB, O, <DB as sqlx::database::Database>::Arguments<'q>>,
	ExecuteError,
>
where
	DB: Database,
	(O,): for<'r> sqlx::FromRow<'r, DB::Row>,
	Option<Vec<u8>>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
	let mut query = sqlx::query_scalar(sql.text.as_str());

	if let Some(values) = sql.values.take() {
		for value in values {
			if let Some(value) = value {
				query = value.storage_kind.bind_to_sqlx(query);
			} else {
				query = query.bind(Option::<Vec<u8>>::None);
			}
		}

		Ok(query)
	} else {
		Err(QueryError::new().into())
	}
}

impl<'q, DB, O> SqlxQuery<'q, DB>
	for sqlx::query::QueryScalar<'q, DB, O, <DB as sqlx::database::Database>::Arguments<'q>>
where
	DB: sqlx::Database,
	(O,): for<'r> sqlx::FromRow<'r, DB::Row>,
{
	fn bind_to_sqlx<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(self, value: T) -> Self {
		self.bind(value)
	}
}

pub fn query_as<'q, DB, O>(
	sql: &'q mut FrozenSql<DB>,
) -> Result<
	sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::Database>::Arguments<'q>>,
	ExecuteError,
>
where
	DB: Database,
	O: for<'r> sqlx::FromRow<'r, DB::Row>,
	Option<Vec<u8>>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
	let mut query = sqlx::query_as(sql.text.as_str());

	if let Some(values) = sql.values.take() {
		for value in values {
			if let Some(value) = value {
				query = value.storage_kind.bind_to_sqlx(query);
			} else {
				query = query.bind(Option::<Vec<u8>>::None);
			}
		}

		Ok(query)
	} else {
		Err(QueryError::new().into())
	}
}

impl<'q, DB, O> SqlxQuery<'q, DB>
	for sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::Database>::Arguments<'q>>
where
	DB: Database,
	O: for<'r> sqlx::FromRow<'r, DB::Row>,
{
	fn bind_to_sqlx<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(self, value: T) -> Self {
		self.bind(value)
	}
}

pub fn query<'q, DB>(
	sql: &'q mut FrozenSql<DB>,
) -> Result<sqlx::query::Query<'q, DB, <DB as sqlx::database::Database>::Arguments<'q>>, ExecuteError>
where
	DB: Database,
	Option<Vec<u8>>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
	let mut query = sqlx::query(sql.text.as_str());

	if let Some(values) = sql.values.take() {
		for value in values {
			if let Some(value) = value {
				query = value.storage_kind.bind_to_sqlx(query);
			} else {
				query = query.bind(Option::<Vec<u8>>::None);
			}
		}

		Ok(query)
	} else {
		Err(QueryError::new().into())
	}
}

impl<'q, DB: sqlx::Database> SqlxQuery<'q, DB>
	for sqlx::query::Query<'q, DB, <DB as sqlx::database::Database>::Arguments<'q>>
{
	fn bind_to_sqlx<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(self, value: T) -> Self {
		self.bind(value)
	}
}
