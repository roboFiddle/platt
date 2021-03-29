use proc_macro::TokenStream;
use darling::FromField;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, Clone, FromField)]
#[darling(attributes(platt), forward_attrs)]
struct PlattField {
    ident: Option<syn::Ident>,
    ty: syn::Type
 }

pub fn inner(model: TokenStream) -> TokenStream {
    let model = parse_macro_input!(model as DeriveInput);
    let mut req_composites = quote! { let mut composites = ::std::collections::HashSet::new();  };
    let mut fields = quote!{ let mut fields = ::std::vec::Vec::new(); };

    if model.generics.lt_token.is_some() {
        panic!("Platt does not support models with generics. Perhaps use an enum instead.")
    }
    let model_name = model.ident.clone();
    let model_name_str = format!("{}", model_name);
    let data = if let syn::Data::Struct(data) = model.data {
        data
    } else {
        panic!("Platt models can only be derived on structs. To store an enum in a database, create a struct with the enum as a field.")
    };
    let fields_named = if let syn::Fields::Named(fields) = data.fields {
        fields 
    } else {
        panic!("Platt models are only currently enabled for structs with named fields.");
    };
    for field in &fields_named.named {
        let p_field = PlattField::from_field(field).unwrap();
        let column_name = format!("{}", p_field.ident.unwrap());
        match p_field.ty {
            // syn::Type::Array(ty_array) =>  { }
            syn::Type::Path(ty_path) =>  { 
                req_composites.extend(quote! { composites.extend(<#ty_path as ::platt::db_types::HasDbType>::composites()); });
                fields.extend(quote! {
                    fields.push(::platt::db_tables::Column::new(#column_name.to_string(), <#ty_path as ::platt::db_types::HasDbType>::db_type()));
                });
            }
            _ => panic!("Platt models can only contain type paths.")
        }
    }

    let result = quote::quote! { 
        impl ::platt::db_tables::DbTable for #model_name {
            fn activate(schema: &mut ::platt::db_tables::Schema) {
                #req_composites
                #fields
                let table = ::platt::db_tables::Table::new(#model_name_str.to_string(), fields);
                schema.add_table(table, composites)
            }
        }
    };
    result.into()
}