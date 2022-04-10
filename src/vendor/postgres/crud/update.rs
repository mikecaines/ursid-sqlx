use crate::crud::update::UpdateBuilder;
use crate::{query, ExecuteError, IntoSql};
use sqlx::Postgres;

pub async fn execute(
	builder: UpdateBuilder<Postgres, true>,
	connection: &mut <Postgres as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	let mut sql = builder.statement.finalize()?.into_sql().freeze();

	query(&mut sql)?
		.execute(connection)
		.await
		.map_err(ExecuteError::new)?;

	Ok(())
}
