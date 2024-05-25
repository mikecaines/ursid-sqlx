use sqlx::MySql;

use crate::crud::update::UpdateBuilder;
use crate::{query, ExecuteError, IntoSql};

pub async fn execute(
	builder: UpdateBuilder<MySql, true>,
	connection: &mut <MySql as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	let mut sql = builder.statement.finalize()?.into_sql().freeze();

	query(&mut sql)?
		.execute(connection)
		.await
		.map_err(ExecuteError::new)?;

	Ok(())
}
