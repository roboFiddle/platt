use platt::{PlattModel, PlattEnum};

#[derive(PlattEnum)]
pub enum BlogPostStatus {
    Draft,
    Published { at: u16 },
    Removed { at: u16, reason: String }
}

#[derive(PlattModel)]
#[platt(not_clonable)]
pub struct User {
    email: String,
    password: String
}

#[derive(PlattModel)]
#[platt(not_clonable)]
pub struct BlogPost {
    name: String,
    content: platt::schema::Varchar<255>,
    status: BlogPostStatus,
    #[platt(reverse = "User")]
    posted_by: platt::schema::ForeignKey<User>
}

platt::activate_models!(User, BlogPost);