pub mod db_tables;
pub mod db_types;
pub use platt_macros::*;

#[macro_export]
macro_rules! activate_models {
    ( $($x:ident),+ ) => {
        pub fn get_schema() -> ::platt::db_tables::Schema{
            let mut schema = ::platt::db_tables::Schema::empty();
            $(
                <$x as ::platt::db_tables::DbTable>::activate(&mut schema);
            )+
            schema
        }
    };
}