use crate::crud::insert::InsertBuilder;
use crate::error::ExecuteError;
use crate::{query, IntoSql};
use sqlx::Postgres;

pub async fn execute(
	builder: InsertBuilder<Postgres>,
	database: &mut <Postgres as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	let mut sql = builder.statement.finalize()?.into_sql().freeze();

	query(&mut sql)?
		.execute(database)
		.await
		.map_err(ExecuteError::new)?;

	Ok(())
}
