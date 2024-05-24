use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(IntoSqlValue)]
pub fn into_sql_value_derive(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	impl_macro(&ast)
}

fn impl_macro(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;

	let gen = quote! {
		impl<DB: ursid_sqlx::Database> ursid_sqlx::IntoSqlValue<DB> for #name {
			fn into_sql_value(self) -> Option<ursid_sqlx::value::Value<DB>> {
				self.0.into_sql_value()
			}
		}
	};

	gen.into()
}
