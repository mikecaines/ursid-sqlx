use sqlx::Postgres;

use crate::crud::delete::DeleteBuilder;
use crate::error::ExecuteError;
use crate::query;
use crate::sql_lang::IntoSql;

pub async fn execute(
	builder: DeleteBuilder<Postgres>,
	database: &mut <Postgres as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	let mut sql = builder.statement.finalize()?.into_sql().freeze();

	query(&mut sql)?
		.execute(database)
		.await
		.map_err(ExecuteError::new)?;

	Ok(())
}
