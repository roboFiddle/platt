use std::ops::{Deref, DerefMut};

pub trait Queryable {
    type Data;
    type Insertable;
    type Filters;
    type Update;
}

pub trait GetFilterState<FieldType> {
    fn get(&mut self) -> &mut FilterState<FieldType>;
}
pub trait Filters<FieldType>: GetFilterState<FieldType> { }

#[derive(Debug, Default, Clone)]
pub struct FilterState<FieldType>(Vec<FieldType>);

pub struct DatabaseResult<Model: Queryable>(Model::Data);

#[derive(Clone, Debug, Default)]
pub struct TrackingMut<FieldType> {
    data: FieldType,
    edited: bool
}

impl<FieldType> Deref for TrackingMut<FieldType> {
    type Target = FieldType;
    fn deref(&self) -> &FieldType {
        &self.data
    }
}
impl<FieldType> DerefMut for TrackingMut<FieldType> {
    fn deref_mut(&mut self) -> &mut FieldType {
        self.edited = true;
        &mut self.data
    }
}

pub struct QuerySet<DB, Model> {
    db: DB,
    _model: std::marker::PhantomData<Model>
}

impl<DB, Model: Queryable> QuerySet<DB, Model> {
    pub fn all() -> Self { todo!() }
    pub fn none() -> Self { todo!() }
    pub fn get_one() -> Model::Data { todo!()  } 
    pub fn create() -> Model::Data { todo!()  }
    pub fn get_or_create() -> Model::Data { todo!() }
    pub fn update_or_create() -> Model::Data { todo!() }
    pub fn bulk_create() -> Model::Data { todo!() }

    pub fn filter(self, filters: Model::Filters) -> Self {
        todo!()
    }

    pub fn exclude(self, filters: Model::Filters) -> Self {
        todo!()
    }

    pub fn order_by(self) -> Self {
        todo!()
    }

    pub fn reverse(self) -> Self {
        todo!()
    }

    pub fn sum(self, other: Self) -> Self { 
        todo!()
    }

    pub fn intersection(self, other: Self) -> Self { 
        todo!()
    }

    pub fn difference(self, other: Self) -> Self { 
        todo!()
    }

    pub fn count(self) -> Self {
        todo!()
    }

    pub fn first(self) -> Self {
        todo!()
    }

    pub fn last(self) -> Self {
        todo!()
    }

    pub fn exists(self) -> Self {
        todo!()
    }
}