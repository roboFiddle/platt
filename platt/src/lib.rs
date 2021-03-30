pub mod query;
pub mod schema;
pub use platt_macros::*;
pub use derive_builder::Builder;

pub trait HasBuilder {
    type Builder;
}

#[macro_export]
macro_rules! activate_models {
    ( $($x:ident),+ ) => {
        pub fn get_schema() -> ::platt::schema::Schema{
            let mut schema = ::platt::schema::Schema::empty();
            $(
                <$x as ::platt::schema::DbModel>::activate(&mut schema);
            )+
            schema
        }
    };
}