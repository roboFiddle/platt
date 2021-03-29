use platt::{PlattModel, PlattEnum};

#[derive(PlattEnum)]
pub enum BlogPostStatus {
    Draft,
    Published { at: u16 },
    Removed { at: u16, reason: String }
}

#[derive(PlattModel)]
pub struct BlogPost {
    name: String,
    content: platt::db_types::Varchar<255>,
    status: BlogPostStatus
}

platt::activate_models!(BlogPost);