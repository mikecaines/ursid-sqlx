use sqlx::MySql;

use crate::crud::insert::InsertBuilder;
use crate::error::ExecuteError;
use crate::{query, IntoSql};

pub async fn execute(
	builder: InsertBuilder<MySql>,
	database: &mut <MySql as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	let mut sql = builder.statement.finalize()?.into_sql().freeze();

	query(&mut sql)?
		.execute(database)
		.await
		.map_err(ExecuteError::new)?;

	Ok(())
}
