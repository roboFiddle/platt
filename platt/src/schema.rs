#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DbType {
    pub base: String,
    pub nullable: bool,
    pub indexed: bool,
    pub primary_key: bool,
    pub unique: bool
}

impl DbType {
    pub(crate) fn db_type_string(&self) -> String {
        let mut base = self.base.clone();
        if !self.nullable {
            base += " NOT NULL";
        }
        if self.indexed {
            base += " INDEXED";
        }
        if self.primary_key {
            base += " PRIMARY KEY";
        }
        if self.unique {
            base += " UNIQUE";
        }
        base
    }

    pub(crate) fn db_type_string_simple(&self) -> String {
        self.base.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Composite {
    pub name: String,
    pub fields: Vec<(String, DbType)>
}

pub trait HasDbType {
    fn composites() -> Vec<Composite> {
        Vec::default()
    }
    fn tables() -> Vec<Table> { 
        Vec::default()
    }
    fn db_type() -> DbType;
}

macro_rules! basic_db_type {
    ($rust:ty, $db:expr) => {
        basic_db_type!($rust, $db, false);
    };
    ($rust:ty, $db:expr, $null:expr) => {
        impl HasDbType for $rust {
            fn db_type() -> DbType {
                DbType {
                    base: $db.to_string(),
                    nullable: $null,
                    indexed: false,
                    primary_key: false,
                    unique: false
                }
            }
        }
    };
}

basic_db_type!(bool, "BOOLEAN");
basic_db_type!(u8, "SMALLINT");
basic_db_type!(u16, "SMALLINT");
basic_db_type!(u32, "INT");
basic_db_type!(u64, "BIGINT");
basic_db_type!(i8, "SMALLINT");
basic_db_type!(i16, "SMALLINT");
basic_db_type!(i32, "INT");
basic_db_type!(i64, "BIGINT");
basic_db_type!(String, "TEXT");
basic_db_type!(f32, "REAL");
basic_db_type!(f64, "DOUBLE PRECISION");

pub mod non_zero {
    use std::num::*;
    use super::*;
    basic_db_type!(NonZeroI8, "SMALLINT");
    basic_db_type!(NonZeroI16, "SMALLINT");
    basic_db_type!(NonZeroI32, "INT");
    basic_db_type!(NonZeroI64, "BIGINT");
    basic_db_type!(NonZeroU8, "SMALLINT");
    basic_db_type!(NonZeroU16, "SMALLINT");
    basic_db_type!(NonZeroU32, "INT");
    basic_db_type!(NonZeroU64, "BIGINT");
}

pub struct Varchar<const SIZE: usize>(String);
impl<const SIZE: usize> HasDbType for Varchar<SIZE> {
    fn db_type() -> DbType {
        DbType {
            base: format!("VARCHAR({})", SIZE),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

pub struct ExactString<const SIZE: usize>(String);
impl<const SIZE: usize> HasDbType for ExactString<SIZE> {
    fn db_type() -> DbType {
        DbType {
            base: format!("CHAR({})", SIZE),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

pub struct Decimal<const BEFORE: u16, const AFTER: u16>();
impl<const BEFORE: u16, const AFTER: u16> HasDbType for Decimal<BEFORE, AFTER> {
    fn db_type() -> DbType {
        DbType {
            base: format!("DECIMAL({}, {})", BEFORE, AFTER),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

pub struct BitStringFixed<const SIZE: usize>([bool; SIZE]);
impl<const SIZE: usize> HasDbType for BitStringFixed<SIZE> {
    fn db_type() -> DbType {
        DbType {
            base: format!("BIT({})", SIZE),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

pub struct BitString(Vec<bool>);
impl HasDbType for BitString {
    fn db_type() -> DbType {
        DbType {
            base: format!("BIT VARYING"),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

#[cfg(feature = "chrono_type")]
pub mod chrono_type {
    use super::*;
    basic_db_type!(chrono::NaiveDate, "DATE");
    basic_db_type!(chrono::NaiveTime, "TIME");
    basic_db_type!(chrono::NaiveDateTime, "TIMESTAMP");
    impl<Tz: chrono::TimeZone> HasDbType for chrono::DateTime<Tz> {
        fn db_type() -> DbType {
            DbType {
                base: "TIMESTAMPTZ".to_string(),
                nullable: false,
                indexed: false,
                primary_key: false,
                unique: false
            }
        }
    }
}
#[cfg(feature = "uuid_type")]
pub mod uuid_type {
    use super::*;
    basic_db_type!(uuid::Uuid, "Uuid");
}

#[cfg(feature = "json")]
pub mod json {
    use super::*;
    pub struct LenientJson(serde_json::Value);
    basic_db_type!(LenientJson, "JSON");
    basic_db_type!(serde_json::Value, "JSONB");
}

impl<T: HasDbType> HasDbType for Option<T> {
    fn composites() -> Vec<Composite> {
        T::composites()
    }
    fn db_type() -> DbType {
        let mut raw = T::db_type();
        raw.nullable = true;
        raw
    }
}

impl<T: HasDbType> HasDbType for Vec<T> {
    fn composites() -> Vec<Composite> {
        T::composites()
    }
    fn db_type() -> DbType {
        let mut raw = T::db_type();
        raw.base = format!("{}[]", raw.base);
        raw
    }
}

impl<T: HasDbType, const N: usize> HasDbType for [T; N] {
    fn composites() -> Vec<Composite> {
        T::composites()
    }
    fn db_type() -> DbType {
        let mut raw = T::db_type();
        raw.base = format!("{}[{}]", raw.base, N);
        raw
    }
}

pub struct ForeignKey<Other: DbModel>(std::marker::PhantomData<Other>);
impl<Other: DbModel> HasDbType for ForeignKey<Other> {
    fn db_type() -> DbType {
        DbType {
            base: format!("UUID REFERENCES {}(\"id\")", Other::table_name()),
            nullable: false,
            indexed: false,
            primary_key: false,
            unique: false
        }
    }
}

pub struct ManyToManyRelationship<Other: DbModel>(std::marker::PhantomData<Other>);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Column {
    name: String,
    db_type: DbType
}

impl Column {
    pub fn new(name: String, db_type: DbType) -> Self {
        Self { 
            name, 
            db_type
        }
    }
}

pub trait DbModel {
    fn table_name() -> String;
    fn activate(schema: &mut Schema);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Table {
    name: String,
    columns: Vec<Column>
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        Self { 
            name, 
            columns
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    composites: Vec<Composite>,
    tables: Vec<Table>
}

impl Schema {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn add_tables(&mut self, table: Vec<Table>) {
        self.tables.extend(table);
    }

    pub fn add_composites(&mut self, composites: Vec<Composite>) {
        self.composites.extend(composites);
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        for composite in &self.composites {
            let mut composite_sql = format!("CREATE TYPE \"{}\" AS (", composite.name);
            for (name, db_type) in &composite.fields {
                composite_sql += &format!("\"{}\" {}, ", name, db_type.db_type_string_simple());
            }
            composite_sql.pop();
            composite_sql.pop();
            composite_sql += ");\n";
            sql += &composite_sql;
        }

        for table in &self.tables {
            let mut table_sql = format!("CREATE TABLE \"{}\" (", table.name);
            for column in &table.columns {
                table_sql += &format!("\"{}\" {}, ", column.name, column.db_type.db_type_string());
            }
            table_sql.pop();
            table_sql.pop();
            table_sql += ");\n";
            sql += &table_sql;
        }
        sql
    }

    pub fn diff(&self) -> Result<(), Box<dyn std::error::Error>>
    {
        use std::fs::File;
        use std::io::Read;
        let current_schema: Self = {
            if let Ok(mut file) = File::open("current_schema.json") {
                let mut current_schema = String::new();
                file.read_to_string(&mut current_schema)?;
                serde_json::from_str(&current_schema)?
            } else {
                Self::empty()
            }
        };

        dbg!(current_schema);
        dbg!(self);
        Ok(())
    }
}

pub fn primary_key() -> DbType {
    DbType {
        base: "UUID".to_string(),
        nullable: false,
        indexed: false,
        primary_key: true,
        unique: true
    }
}