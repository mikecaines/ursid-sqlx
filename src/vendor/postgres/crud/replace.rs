use crate::crud::replace::ReplaceBuilder;
use crate::error::{CrudError, CrudErrorKind, ExecuteError};
use crate::sql_lang::clause::sql_where::WhereBuilder;
use crate::sql_lang::statement::Select;
use crate::sql_lang::{IntoRawSql, IntoSql};
use crate::{query, sql_lang};
use sqlx::Postgres;

pub async fn execute(
	builder: ReplaceBuilder<Postgres, true, true>,
	database: &mut <Postgres as sqlx::Database>::Connection,
) -> Result<(), ExecuteError> {
	builder.validate()?;

	let mut where_clause: WhereBuilder<_, true, false> = WhereBuilder {
		predicates: Vec::new(),
	};

	for key_name in &builder.keys {
		let value = builder
			.modifications
			.iter()
			.find_map(|modification| {
				if modification.name == *key_name {
					return Some(modification.value.clone());
				}
				None
			})
			.unwrap_or_default();

		where_clause = where_clause.and_column_equal_to(key_name, value);
	}
	let where_clause = where_clause.finalize()?;

	let mut sql = Select::build(builder.table_name.clone())
		.select_expression(
			{
				use crate::sql_lang::expression::prelude::*;
				count("*".into_raw_sql())
			},
			"cnt",
		)
		.with_where_clause(where_clause.clone())
		.finalize()?
		.into_sql()
		.freeze();

	let existing_row_count: i64 = crate::query_scalar(&mut sql)?
		.fetch_one(&mut *database)
		.await
		.map_err(ExecuteError::new)?;

	if existing_row_count == 0 {
		let mut insert_statement = sql_lang::statement::Insert::build(builder.table_name);

		for modification in builder.modifications {
			if modification.insert {
				insert_statement = insert_statement.column(modification.name, modification.value);
			}
		}

		let insert_statement = insert_statement.finalize()?.into_sql();

		let mut sql = insert_statement.freeze();

		query(&mut sql)?
			.execute(&mut *database)
			.await
			.map_err(ExecuteError::new)?;

		Ok(())
	} else {
		if existing_row_count > 1 {
			return Err(ExecuteError::new(CrudError::new(
				CrudErrorKind::MultipleRowsWouldBeUpdated,
			)));
		}

		let update_builder = {
			// NOTE: we init a builder with the first (nth(0)) value, and then add the rest
			// This is done so that update_builder can be of type UpdateBuilder<_, true> throughout.

			let mut modifications = builder.modifications.into_iter().filter(|m| m.update);

			if let Some(modification) = modifications.nth(0) {
				Some(
					modifications.fold(
						sql_lang::statement::Update::build(builder.table_name)
							.with_where_clause(where_clause)
							.update_column(modification.name, modification.value),
						|update_builder, modification| {
							update_builder.update_column(modification.name, modification.value)
						},
					),
				)
			} else {
				None
			}
		};

		// if there are any columns to update
		if let Some(update_builder) = update_builder {
			let update_statement = update_builder.finalize()?.into_sql();

			let mut sql = update_statement.freeze();

			query(&mut sql)?
				.execute(database)
				.await
				.map_err(ExecuteError::new)?;
		}

		Ok(())
	}
}
