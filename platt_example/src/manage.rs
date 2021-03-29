use std::fs::File;
use std::io::Read;
mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let new_schema = models::get_schema();
    println!("{}", new_schema.to_sql());
    Ok(())
}