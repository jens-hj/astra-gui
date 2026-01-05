//! Proc-macros for astra-gui.
//!
//! Currently provided:
//! - `#[derive(WithBuilders)]`: generates `with_<field>(...)` builder-style methods
//!   for each named field in a struct.
//!
//! ## Example
//! ```ignore
//! use astra_gui_macros::WithBuilders;
//!
//! #[derive(Clone, Debug, WithBuilders)]
//! pub struct Style {
//!     pub padding: f32,
//!     pub color: Color,
//! }
//!
//! let s = Style { padding: 1.0, color: Color::WHITE }
//!     .with_padding(2.0)
//!     .with_color(Color::BLACK);
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

/// Derive that generates `with_<field>` builder methods for structs with named fields.
///
/// Generated methods take `self` by value (builder style) and return `Self`.
#[proc_macro_derive(WithBuilders, attributes(with_builders))]
pub fn derive_with_builders(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let generics = &input.generics;

    let Data::Struct(data_struct) = &input.data else {
        return syn::Error::new(
            input.span(),
            "#[derive(WithBuilders)] only supports structs",
        )
        .to_compile_error()
        .into();
    };

    let Fields::Named(fields_named) = &data_struct.fields else {
        return syn::Error::new(
            data_struct.fields.span(),
            "#[derive(WithBuilders)] only supports structs with named fields",
        )
        .to_compile_error()
        .into();
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut methods = Vec::with_capacity(fields_named.named.len());
    for field in fields_named.named.iter() {
        let Some(field_ident) = field.ident.as_ref() else {
            // Named fields always have idents, but keep this defensive.
            continue;
        };
        let field_ty = &field.ty;

        // Method name: with_<field_name>
        let method_ident = format_ident!("with_{}", field_ident);

        methods.push(quote! {
            #[inline]
            pub fn #method_ident(mut self, value: #field_ty) -> Self {
                self.#field_ident = value;
                self
            }
        });
    }

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    }
    .into()
}
