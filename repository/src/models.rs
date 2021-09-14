use std::{convert::TryFrom, error::Error};
use tiberius::Row;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSource {
    pub id: u8,
    pub name: String,
}

impl TryFrom<Row> for ImageSource {
    type Error = Box<dyn Error>;
    
    fn try_from(row: Row) -> Result<Self, Box<dyn Error>> {
        let id = row.try_get::<u8, _>(0)?.unwrap();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        Ok(ImageSource { id, name })
    }
}

impl TryFrom<&Row> for ImageSource {
    type Error = Box<dyn Error>;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<u8, _>(0)?.unwrap();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        Ok(ImageSource { id, name })
    }
}
