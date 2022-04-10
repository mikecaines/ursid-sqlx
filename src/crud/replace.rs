use crate::error::CrudErrorKind;
use crate::value::{IntoSqlValue, Value};
use crate::{CrudError, Database, ExecuteError};

pub fn replace_row<DB: Database, N: Into<String>>(
	table_name: N,
) -> ReplaceBuilder<DB, false, false> {
	ReplaceBuilder {
		table_name: table_name.into(),
		modifications: Vec::new(),
		keys: Vec::new(),
	}
}

pub struct ReplaceBuilder<DB: Database, const HAS_KEYS: bool, const HAS_UPDATES: bool> {
	pub(crate) table_name: String,
	pub(crate) modifications: Vec<Modification<DB>>,
	pub(crate) keys: Vec<String>,
}

impl<DB: Database, const HAS_KEYS: bool, const HAS_UPDATES: bool>
	ReplaceBuilder<DB, HAS_KEYS, HAS_UPDATES>
{
	pub fn replace_column<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> ReplaceBuilder<DB, HAS_KEYS, true> {
		self.modifications.push(Modification {
			name: name.into(),
			value: value.into_sql_value(),
			update: true,
			insert: true,
		});

		ReplaceBuilder {
			table_name: self.table_name,
			modifications: self.modifications,
			keys: self.keys,
		}
	}

	pub fn update_column<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> ReplaceBuilder<DB, HAS_KEYS, true> {
		self.modifications.push(Modification {
			name: name.into(),
			value: value.into_sql_value(),
			update: true,
			insert: false,
		});

		ReplaceBuilder {
			table_name: self.table_name,
			modifications: self.modifications,
			keys: self.keys,
		}
	}

	pub fn insert_column<N: Into<String>, V: IntoSqlValue<DB>>(
		mut self,
		name: N,
		value: V,
	) -> ReplaceBuilder<DB, HAS_KEYS, true> {
		self.modifications.push(Modification {
			name: name.into(),
			value: value.into_sql_value(),
			update: false,
			insert: true,
		});

		ReplaceBuilder {
			table_name: self.table_name,
			modifications: self.modifications,
			keys: self.keys,
		}
	}
}

impl<DB: Database, const HAS_UPDATES: bool> ReplaceBuilder<DB, false, HAS_UPDATES> {
	pub fn key_columns<V: IntoIterator<Item = K>, K: Into<String>>(
		mut self,
		column_names: V,
	) -> ReplaceBuilder<DB, true, HAS_UPDATES> {
		for name in column_names.into_iter() {
			self.keys.push(name.into());
		}

		ReplaceBuilder {
			table_name: self.table_name,
			modifications: self.modifications,
			keys: self.keys,
		}
	}
}

impl<DB: Database> ReplaceBuilder<DB, true, true> {
	pub(crate) fn validate(&self) -> Result<(), ExecuteError> {
		if self.keys.is_empty() {
			return Err(CrudError::new(CrudErrorKind::MissingKeyColumns).into());
		}

		Ok(())
	}

	pub async fn execute(self, database: &mut DB::Connection) -> Result<(), ExecuteError> {
		DB::execute_crud_replace(self, database).await
	}
}

pub(crate) struct Modification<DB: Database> {
	pub name: String,
	pub value: Option<Value<DB>>,
	pub update: bool,
	pub insert: bool,
}
