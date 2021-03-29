use crate::db_types::*;

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

pub trait DbTable {
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

    pub fn add_table(&mut self, table: Table, composites: Vec<Composite>) {
        self.composites.extend(composites);
        self.tables.push(table);
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