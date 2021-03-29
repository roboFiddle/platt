use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


pub fn inner(model: TokenStream) -> TokenStream {
    let model = parse_macro_input!(model as DeriveInput);
    let mut req_composites = quote! { let mut composites = ::std::collections::HashSet::new();  };

    if model.generics.lt_token.is_some() {
        panic!("Platt does not support models with generics. Perhaps use an enum instead.")
    }
    let enum_name = model.ident.clone();
    let data = if let syn::Data::Enum(data) = model.data {
        data
    } else {
        panic!("Platt enums can only be derived on enums.")
    };

    let mut composites = Vec::new();
    for variant in &data.variants {
        match variant.fields {
            syn::Fields::Named(ref fields) => {
                if fields.named.len() > 0 {
                    let composite_name = format!("{}_{}", enum_name, variant.ident);
                    let mut composite = quote! { let mut composite = ::platt::db_types::Composite { name: #composite_name.to_string(), fields: Vec::new() }; };
                    for p_field in variant.fields.iter() {
                        let column_name = format!("{}", p_field.ident.as_ref().unwrap());
                        match &p_field.ty {
                            // syn::Type::Array(ty_array) =>  { }
                            syn::Type::Path(ty_path) =>  { 
                                req_composites.extend(quote! { composites.extend(<#ty_path as ::platt::db_types::HasDbType>::composites()); });
                                composite.extend(quote!{
                                    composite.fields.push( (#column_name.to_string(), <#ty_path as ::platt::db_types::HasDbType>::db_type()) );
                                })
                            }
                            _ => panic!("Platt models can only contain type paths.")
                        }
                    }
                    req_composites.extend(quote! {
                        {
                            #composite
                            composites.insert(composite);
                        }
                    });
                    composites.push((variant.ident.to_string(), composite_name));
                }
            }
            syn::Fields::Unnamed(ref _fields) => { panic!(); }
            syn::Fields::Unit => ()
        }
    }

    let enum_name_str = enum_name.to_string();
    let enum_composite_name = format!("{}__Composite", enum_name);
    let mut enum_composite = quote! { 
        let mut composite = ::platt::db_types::Composite { 
            name: #enum_composite_name.to_string(), 
            fields: vec![(#enum_name_str.to_string(), <u32 as ::platt::db_types::HasDbType>::db_type())]
        }; 
    };
    for (variant_name, composite_name) in composites {
        enum_composite.extend(quote![
            composite.fields.push( (#variant_name.to_string(), ::platt::db_types::DbType {
                base: #composite_name.to_string(),
                nullable: false,
                indexed: false,
                primary_key: false,
                unique: false
            })
         );
        ]);
    }

    let result = quote::quote! {
        impl ::platt::db_types::HasDbType for #enum_name {
            fn composites() -> ::std::collections::HashSet<::platt::db_types::Composite> {
                #req_composites
                {
                    #enum_composite
                    composites.insert(composite);
                }
                composites
            }
            fn db_type() -> ::platt::db_types::DbType {
                ::platt::db_types::DbType {
                    base: #enum_composite_name.to_string(),
                    nullable: false,
                    indexed: false,
                    primary_key: false,
                    unique: false
                }
            }
        }
    };
    result.into()
}