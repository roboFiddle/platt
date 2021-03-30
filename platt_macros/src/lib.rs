use proc_macro::TokenStream;
mod derive_enum;
mod derive_model;

#[proc_macro_derive(PlattEnum, attributes(platt))]
pub fn derive_enum(model: TokenStream) -> TokenStream {
    derive_enum::inner(model)   
}

#[proc_macro_derive(PlattModel, attributes(platt))]
pub fn derive_model(model: TokenStream) -> TokenStream {
    derive_model::inner(model)
}