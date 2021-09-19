use std::convert::TryFrom;
use tiberius::Row;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct ImageSource {
    pub id: u8,
    pub name: String,
}

impl TryFrom<&Row> for ImageSource {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<u8, _>(0)?.unwrap();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        Ok(ImageSource { id, name })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct County {
    pub id: i32,
    pub state_id: String,
    pub name: String,
}

impl TryFrom<&Row> for County {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<i32, _>(0)?.unwrap();
        let state_id = row.try_get::<&str, _>(1)?.unwrap().to_string();
        let name = row.try_get::<&str, _>(2)?.unwrap().to_string();
        Ok(County { id, state_id, name })
    }
}
