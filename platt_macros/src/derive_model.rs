use proc_macro::TokenStream;
use darling::{FromDeriveInput, FromField};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, punctuated::Punctuated};
use heck::{CamelCase, SnakeCase};

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(
    attributes(platt),
    forward_attrs(doc, cfg, allow),
    supports(struct_named)
)]
pub struct PlattModel {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: darling::ast::Data<darling::util::Ignored, PlattField>,
    #[darling(default)]
    not_clonable: Option<()>
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(platt), forward_attrs(doc, cfg, allow))]
struct PlattField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    reverse: Option<String>
 }

pub fn inner(model: TokenStream) -> TokenStream {
    let model = parse_macro_input!(model as DeriveInput);
    let platt_model = PlattModel::from_derive_input(&model).unwrap();
    let model_name = platt_model.ident;
    let model_name_str = model_name.to_string();
    let model_name_snake = model_name_str.to_snake_case();
    let filters_mod = format_ident!("{}_filters_mod", model_name_snake);
    let updates_mod = format_ident!("{}_updates_mod", model_name_snake);
    let data_mod = format_ident!("{}_data_mod", model_name_snake);
    let filter_name_struct = format_ident!("{}Filters", model_name);
    let update_name_struct = format_ident!("{}Updates", model_name);
    let data_name_struct = format_ident!("{}Data", model_name);

    let mut req_composites = quote! { let mut composites = ::std::vec::Vec::new();  };
    let mut tables = quote!{ let mut tables = ::std::vec::Vec::new(); };
    let mut fields = quote!{ 
        let mut fields = ::std::vec::Vec::new(); 
        fields.push(::platt::schema::Column::new("id".to_string(), ::platt::schema::primary_key()));
    };
    let mut filter_structs = quote! { };
    let mut filter_struct_getters = quote! { };
    let mut filter_struct_fields = syn::FieldsNamed {
        brace_token: syn::token::Brace::default(),
        named: Punctuated::default()
    };
    let mut update_struct_fields = quote! { };
    let mut data_struct_fields = quote! { };

    if model.generics.lt_token.is_some() {
        panic!("Platt does not support models with generics. Perhaps use an enum instead.")
    }

    if platt_model.data.is_enum() {
        panic!("");
    }

    let struct_data = platt_model.data.take_struct().expect("Platt models are only currently enabled for structs with named fields.");
    if !struct_data.style.is_struct() {
        panic!("Platt models are only currently enabled for structs with named fields.");
    }
    
    for p_field in &struct_data.fields {
        let column_name = format!("{}", p_field.ident.as_ref().unwrap());
        let column_name_ident = format_ident!("{}", p_field.ident.as_ref().unwrap());
        let column_name_type_ident = format_ident!("{}", column_name_ident.to_string().to_camel_case());
        if column_name == "id" {
            panic!("Platt uses a column name of 'id' internally. Please use a different name.")
        }
        match &p_field.ty {
            // syn::Type::Array(ty_array) =>  { }
            syn::Type::Path(ref ty_path) =>  { 
                req_composites.extend(quote! { composites.extend(<#ty_path as ::platt::schema::HasDbType>::composites()); });
                tables.extend(quote! {
                    tables.extend(<#ty_path as ::platt::schema::HasDbType>::tables());
                });
                fields.extend(quote! {
                    fields.push(::platt::schema::Column::new(#column_name.to_string(), <#ty_path as ::platt::schema::HasDbType>::db_type()));
                });
                filter_structs.extend(quote! {
                    pub struct #column_name_type_ident(::platt::query::FilterState<#ty_path>);
                    impl ::platt::query::GetFilterState<#ty_path> for #column_name_type_ident {
                        fn get(&mut self) -> &mut ::platt::query::FilterState<#ty_path> { &mut self.0 }
                    }
                    impl ::platt::query::Filters<#ty_path> for #column_name_type_ident { }
                });
                let mut filter_field_path = Punctuated::new();
                filter_field_path.push(syn::PathSegment {
                    ident: column_name_type_ident.clone(),
                    arguments: syn::PathArguments::None
                });
                filter_struct_fields.named.push(syn::Field {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Inherited,
                    ident: Some(column_name_ident.clone()),
                    colon_token: Some(<syn::Token![:]>::default()),
                    ty: syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: filter_field_path
                        }
                    }.into()
                });
                filter_struct_getters.extend(quote!{
                    pub fn #column_name_ident(&mut self) -> &mut #column_name_type_ident {
                        &mut self.#column_name_ident
                    }
                });
                update_struct_fields.extend(quote!{
                    #column_name_ident: ::std::option::Option<#ty_path>,
                });
                data_struct_fields.extend(quote!{
                    pub #column_name_ident: ::platt::query::TrackingMut<#ty_path>,
                })
            }
            _ => panic!("Platt models can only contain type paths.")
        }
    }

    let mut filters_struct_path_types = Punctuated::new();
    filters_struct_path_types.push(syn::PathSegment {
        ident: filters_mod.clone(),
        arguments: syn::PathArguments::None
    });
    filters_struct_path_types.push(syn::PathSegment {
        ident: filter_name_struct.clone(),
        arguments: syn::PathArguments::None
    });

    let filters_struct_path = syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: filters_struct_path_types
        }
    };

    let mut update_struct_path_path = Punctuated::new();
    update_struct_path_path.push(syn::PathSegment {
        ident: updates_mod.clone(),
        arguments: syn::PathArguments::None
    });
    update_struct_path_path.push(syn::PathSegment {
        ident: update_name_struct.clone(),
        arguments: syn::PathArguments::None
    });

    let update_struct_path = syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: update_struct_path_path
        }
    };
    let update_builder_name = format_ident!("{}Builder", update_name_struct);
    let update_builder_attr = if platt_model.not_clonable.is_some() { quote!{ #[builder(pattern = "owned")] } } else { quote! { } };

    let mut data_struct_path_path = Punctuated::new();
    data_struct_path_path.push(syn::PathSegment {
        ident: data_mod.clone(),
        arguments: syn::PathArguments::None
    });
    data_struct_path_path.push(syn::PathSegment {
        ident: data_name_struct.clone(),
        arguments: syn::PathArguments::None
    });

    let data_struct_path = syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: data_struct_path_path
        }
    };

    let result = quote::quote! { 
        pub mod #data_mod {
            use super::*;

            pub struct #data_name_struct {
                #data_struct_fields
            }
        }

        pub mod #filters_mod {
            use super::*;

            #filter_structs
            pub struct #filter_name_struct #filter_struct_fields
            impl #filter_name_struct { #filter_struct_getters }
        }

        pub mod #updates_mod {
            use super::*;
            #[derive(::platt::Builder)]
            #update_builder_attr
            pub struct #update_name_struct {
                #update_struct_fields
            }
            impl ::platt::HasBuilder for #update_name_struct {
                type Builder = #update_builder_name;
            }
        }

        impl ::platt::schema::DbModel for #model_name {
            fn table_name() -> String {
                #model_name_str.to_string()
            }

            fn activate(schema: &mut ::platt::schema::Schema) {
                #req_composites
                #fields
                #tables
                tables.push(::platt::schema::Table::new(#model_name_str.to_string(), fields));
                schema.add_tables(tables);
                schema.add_composites(composites);
            }
        }

        impl ::platt::query::Queryable for #model_name {
            type Data = #data_struct_path;
            type Insertable = #model_name;
            type Filters = #filters_struct_path;
            type Update = #update_struct_path;
        }
    };
    result.into()
}