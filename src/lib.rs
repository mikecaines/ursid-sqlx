#![forbid(unsafe_code)]
#![allow(clippy::tabs_in_doc_comments)]

//! `ursid_sqlx`: Utilities for SQL to execute CRUD-like operations, safely build dynamic queries,
//! and more. Based on the [`sqlx`](https://crates.io/crates/sqlx) crate.
//!
//!
//! # About this crate
//! The `ursid_sqlx` crate aims to provide a database-vendor agnostic, safe, and convenient way to
//! build and execute SQL queries. It provides multiple layers of abstraction, so that you can
//! opt in to high-level API's to do simpler things, and lower-level API's when you need to get
//! closer to the SQL. It doesn't get in your way, and serves to compliment direct use of `sqlx`.
//!
//! The name `URSID` comes from: Update, Replace, Select, Insert, Delete.
//!
//!
//! ## Features
//! - Builders to *safely* (via parameterized queries) construct dynamic SQL statements &
//!   clauses such as `SELECT`, `INSERT`, `WHERE`, `JOIN`, `ON`.
//! - Higher level builders to execute CRUD-like operations, including emulation of "replace",
//!   which inserts or updates a row as required.
//! - An expressions module full of `rust` functions & traits that mirror SQL functions, such as
//!   `count()`, `coalesce()`, etc.
//!   These can be used to compose arbitrary SQL expressions & grammar.
//! - An `Sql` type, that all the API's produce, which safely manages the coupling of
//!   the text & values components of a parameterized query.
//! - These `Sql` "fragments" can also be concatenated together similar to Strings.
//! - Functions such as `query()`, `query_as()`, etc., just like in `sqlx`, but work with the
//!   `Sql` type.
//!
//!
//! ## Design principals
//! - All API's must generate SQL *safely* using parameterized queries. Although you can
//!   still create SQL directly from text/string, you are forced to opt-in to this in a clear and
//!   explicit way.
//! - The API surface should be database-vendor agnostic. You can swap out
//!   your app's underlying `sqlx` database vendor/connection (e.g. from `mysql` to `postgres`),
//!   without having to modify any SQL generation code.
//! - Supported database-vendors, serializable types, etc. should generally be on par with `sqlx`
//!   itself. There are however some cases where vendor-specific features are excluded as the focus
//!   of this crate is on cross-vendor compatibility.
//! - Providing a builder interface for absolutely every feature of SQL is a non-goal.
//!   Instead, you can use lower-level API's, and ultimately `sqlx` itself, to achieve any
//!   obscure or database-vendor specific behaviour.
//! - `ursid` is not an ORM, nor is it a database library/driver such as `sqlx` itself. It sits
//!   somewhere in between these layers, providing convenient API's to do *SQL* focused operations.
//!   Not as high-level as an ORM, and not as low-level as text-based SQL queries.
//!
//!
//! # Quickstart
//!
//! ## Using the CRUD features
//!
//! ### Updating one or more rows
//! ```rust
//! # use sqlx::{MySql, Pool};
//! # use sqlx::pool::PoolOptions;
//! use ursid_sqlx::BuilderHelper;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! let db: Pool<MySql> = PoolOptions::new().connect("...").await?;
//!
//! db.build_crud()
//! 	.update_rows("some_table")
//! 	.update_column("some_column", 123)
//! 	.update_column("another_column", "some value")
//! 	.where_column_equal_to("id", 555)
//! 	.execute(&mut *db.acquire().await?)
//! 	.await?;
//! #
//! # Ok(())
//! # }
//! ```
//!
//! - [insert_row()](crate::crud::insert_row), [delete_rows()](crate::crud::delete_rows)
//!   work in a similar manner.
//! - The method
//! 	[with_where_clause()](crate::crud::update::UpdateBuilder::with_where_clause)
//! 	exists to specify more complex WHERE conditions.
//!
//! ### Replacing a single, uniquely identifiable, row (insert or update automatically)
//! ```rust
//! # use sqlx::{MySql, Pool};
//! # use sqlx::pool::PoolOptions;
//! use ursid_sqlx::BuilderHelper;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! let db: Pool<MySql> = PoolOptions::new().connect("...").await?;
//! let mut transaction = db.begin().await?;
//!
//! db.build_crud()
//! 	.replace_row("some_table")
//! 	.key_columns(["id"]) // names of columns used to uniquely identify the row
//! 	.replace_column("another_column", "some value") // will be inserted and/or updated
//! 	.insert_column("id", 123) // will only be inserted, never updated
//! 	.update_column("column3", 93845) // will only be updated, never inserted
//! 	.execute(&mut transaction)
//! 	.await?;
//!
//! transaction.commit().await?;
//! #
//! # Ok(())
//! # }
//! ```
//! - The CRUD builder's `execute()` method's will accept an `sqlx` connection or transaction.
//!
//! ## Using the builders for SQL statements, clauses, etc.
//!
//! ### Building a simple SELECT statement
//! ```rust
//! # use sqlx::{MySql, Pool};
//! # use sqlx::pool::PoolOptions;
//! use ursid_sqlx::{query, BuilderHelper};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! let db: Pool<MySql> = PoolOptions::new().connect("...").await?;
//!
//! // build the statement
//! let mut sql = db
//! 	.build_sql()
//! 	.statement()
//! 	.select("some_table")
//! 	.select_columns(["id", "name"])
//! 	.where_column_equal_to("username", "some_user")
//! 	.finalize_and_freeze()?;
//!
//! assert_eq!(
//! 	sql.query(),
//! 	"select `id`, `name` from `some_table` where `username`=?"
//! );
//!
//! // use the statement to query the database
//! let _rows = query(&mut sql)?.fetch_all(&db).await?;
//! #
//! # Ok(())
//! # }
//! ```
//!
//! ### A more complex SELECT statement
//! - This SELECT includes a join in the FROM clause, and a more complex WHERE clause.
//! - Note that the `select_column()` methods, etc. now require both a table and column reference,
//!   due to the use of a join.
//! ```rust
//! # use sqlx::{MySql, Pool};
//! # use sqlx::pool::PoolOptions;
//! use ursid_sqlx::sql_lang::clause::{SqlFrom, Where};
//! use ursid_sqlx::{query, BuilderHelper, IntoRawSql};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! let db: Pool<MySql> = PoolOptions::new().connect("...").await?;
//!
//! // build the statement
//! let mut sql = db
//! 	.build_sql()
//! 	.statement()
//! 	.select_with_join(
//! 		SqlFrom::build("user", "u")
//! 			.inner_join("employee", "e", ("e", "user_id", "u", "id").try_into()?)
//! 			.finalize()?,
//! 	)
//! 	.select_column("u", "id")
//! 	.select_column("u", "name")
//! 	.select_column_with_alias("e", "start_date", "employee_date")
//! 	.where_clause(|clause| {
//! 		clause
//! 			.column_equal_to("u", "username", "some_user")
//! 			.and_column_equal_to("e", "is_active", 1)
//! 			.and_column_in("e", "foo_id", [1, 2, 3])
//! 	})
//! 	.finalize_and_freeze()?;
//!
//! assert_eq!(
//! 	sql.query(),
//! 	"\
//! 	select `u`.`id`, `u`.`name` \
//! 	from `user` `u` inner join `employee` `e` on `e`.`user_id` = `u`.`id` \
//! 	where \
//! 		`u`.`username`=? \
//! 		and `e`.`is_active`=? \
//! 		and `e`.`foo_id` in (?,?,?)\
//! 	"
//! );
//!
//! // use the statement to query the database
//! let _rows = query(&mut sql)?.fetch_all(&db).await?;
//! #
//! # Ok(())
//! # }
//! ```
//!
//! ## Using the sql expressions module
//! - Sometimes it can be cumbersome to use builders to create complex SQL expressions.
//!   The expressions module contains a series of rust fn's and traits to do this a different way.
//! - The two approaches can be combined as desired. The various builders have methods to integrate
//!   arbitrary expressions, such as `SelectBuilder::select_expression()` in the example below.
//!
//! ### Using an arbitrary expression in a SELECT statement
//! ```rust
//! # use sqlx::{MySql, Pool};
//! # use sqlx::pool::PoolOptions;
//! use ursid_sqlx::{query, BuilderHelper};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! #
//! let db: Pool<MySql> = PoolOptions::new().connect("...").await?;
//!
//! // build the statement
//! let mut sql = db
//! 	.build_sql()
//! 	.statement()
//! 	.select("user")
//! 	.select_expression(
//! 		{
//! 			use ursid_sqlx::sql_lang::expression::prelude::*;
//!
//! 			concat(
//! 				"USER-",
//! 				coalesce(ColumnReference::new("start_date"), "2000-01-01"),
//! 			)
//! 		},
//! 		"badge",
//! 	)
//! 	.where_column_equal_to("username", "some_user")
//! 	.finalize_and_freeze()?;
//!
//! assert_eq!(
//! 	sql.query(),
//! 	"\
//! 	select concat(?, coalesce(`start_date`, ?)) `badge`\
//! 	from `user` \
//! 	where `username`=?\
//! 	"
//! );
//!
//! // use the statement to query the database
//! let _rows = query(&mut sql)?.fetch_all(&db).await?;
//! #
//! # Ok(())
//! # }
//! ```
//!
//! - The `concat()` and `coalesce()` `fn`'s in the example above, are just normal rust functions
//!   that mirror the corresponding SQL function.
//! - They leverage the [IntoSql](crate::sql_lang::IntoSql) trait to safely accept arbitrary values as arguments.
//! - Simple argument values are safely integrated into the generated SQL as parameter bindings.
//! - The fn's return various "AST" types, which can be converted into the
//!   [Sql](crate::sql_lang::Sql)
//!   type, just like the output of the various builders, etc.
//!
//!
//! # Project status
//!
//! Feature completion for this crate has two aspects; general SQL builder/API coverage, and
//! database-vendor coverage for *rendering* those builders to the vendors dialect.
//!
//! The former is easy to implement and is being done as needed. The latter is currently
//! focused on `mysql` & `postgres` coverage. Contributions are welcome on either!
//!
//!
//! # A note on the documentation for this crate
//!
//! The builders throughout this crate use rust const generics a lot. This enables compile-time
//! guarantees that you don't build a `SELECT` statement without any selected columns, or a
//! `WHERE` clause without any predicates, etc. They also provide some method variations, with a
//! goal of enforcing best-practices. For example, `.select_column()` requires *two* arguments
//! when building a `SELECT` that involves a `JOIN`, the column reference *and* the table reference.
//!
//! A side-effect of this however, is that the builder types have many separate `impl`'s, and
//! the generated rust documentation can be hard to navigate.

pub mod crud;
pub mod error;
pub mod helper;
pub mod query;
pub mod sql_lang;
#[cfg(test)]
mod tests;
pub mod value;
mod vendor;

pub use error::{CrudError, ExecuteError, QueryError, SyntaxError};
pub use helper::BuilderHelper;
pub use query::{query, query_as, query_scalar};
pub use sql_lang::{FrozenSql, IntoRawSql, IntoSql, Sql};
pub use ursid_sqlx_macros::IntoSqlValue;
pub use value::IntoSqlValue;

pub trait Database: sqlx::Database + vendor::requirements::DatabaseVendor<Self> {}
