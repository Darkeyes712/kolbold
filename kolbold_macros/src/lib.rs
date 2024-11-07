extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Timer)]
pub fn timer_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = syn::parse(input).expect("Failed to parse input");

    // Build the trait implementation
    impl_timer_macro(&ast)
}

fn impl_timer_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Timer for #name {}
    };
    gen.into()
}
