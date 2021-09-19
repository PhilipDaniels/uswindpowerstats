use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tiberius::{numeric::Decimal, time::chrono::NaiveDate, Row};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateType {
    State,
    Territory,
    FederalCapital,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct State {
    pub id: String,
    pub name: String,
    pub capital: Option<String>,
    pub population: Option<i32>,
    pub area_square_km: Option<i32>,
    pub state_type: StateType,
}

impl TryFrom<&Row> for State {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<&str, _>(0)?.unwrap().to_string();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        let capital = row.try_get::<&str, _>(2)?.map(|s| s.to_string());
        let population = row.try_get::<i32, _>(3)?;
        let area_square_km = row.try_get::<i32, _>(4)?;
        let state_type = match row.try_get::<&str, _>(5)? {
            Some("S") => StateType::State,
            Some("T") => StateType::Territory,
            Some("F") => StateType::FederalCapital,
            x @ _ => return Err(Self::Error::UnknownStateType(format!("{:?}", x))),
        };

        Ok(State {
            id,
            name,
            capital,
            population,
            area_square_km,
            state_type,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl TryFrom<Option<u8>> for ConfidenceLevel {
    type Error = crate::error::Error;

    fn try_from(value: Option<u8>) -> Result<Self, Self::Error> {
        match value {
            Some(1) => Ok(ConfidenceLevel::Low),
            Some(2) => Ok(ConfidenceLevel::Medium),
            Some(3) => Ok(ConfidenceLevel::High),
            x @ _ => Err(Self::Error::UnknownConfidenceLevel(format!("{:?}", x))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Manufacturer {
    pub id: i32,
    pub name: String,
}

impl TryFrom<&Row> for Manufacturer {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<i32, _>(0)?.unwrap();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        Ok(Manufacturer { id, name })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub num_turbines: Option<i16>,
    pub capacity_mw: Option<Decimal>,
}

impl TryFrom<&Row> for Project {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<i32, _>(0)?.unwrap();
        let name = row.try_get::<&str, _>(1)?.unwrap().to_string();
        let num_turbines = row.try_get::<i16, _>(2)?;
        let capacity_mw = row.try_get::<Decimal, _>(3)?;
        Ok(Project {
            id,
            name,
            num_turbines,
            capacity_mw,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub manufacturer_id: i32,
    pub name: String,
    pub capacity_kw: Option<i32>,
    pub hub_height: Option<Decimal>,
    pub rotor_diameter: Option<Decimal>,
    pub rotor_swept_area: Option<Decimal>,
    pub total_height_to_tip: Option<Decimal>,
}

impl TryFrom<&Row> for Model {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<i32, _>(0)?.unwrap();
        let manufacturer_id = row.try_get::<i32, _>(1)?.unwrap();
        let name = row.try_get::<&str, _>(2)?.unwrap().to_string();
        let capacity_kw = row.try_get::<i32, _>(3)?;
        let hub_height = row.try_get::<Decimal, _>(4)?;
        let rotor_diameter = row.try_get::<Decimal, _>(5)?;
        let rotor_swept_area = row.try_get::<Decimal, _>(6)?;
        let total_height_to_tip = row.try_get::<Decimal, _>(7)?;

        Ok(Model {
            id,
            manufacturer_id,
            name,
            capacity_kw,
            hub_height,
            rotor_diameter,
            rotor_swept_area,
            total_height_to_tip,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Turbine {
    pub id: i32,
    pub county_id: i32,
    pub project_id: i32,
    pub model_id: i32,
    pub image_source_id: u8,
    pub retrofit: bool,
    pub retrofit_year: Option<i16>,
    pub attributes_confidence_level: ConfidenceLevel,
    pub location_confidence_level: ConfidenceLevel,
    pub image_date: Option<NaiveDate>,
    pub latitude: Decimal,
    pub longitude: Decimal,
}

impl TryFrom<&Row> for Turbine {
    type Error = crate::error::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let id = row.try_get::<i32, _>(0)?.unwrap();
        let county_id = row.try_get::<i32, _>(1)?.unwrap();
        let project_id = row.try_get::<i32, _>(2)?.unwrap();
        let model_id = row.try_get::<i32, _>(3)?.unwrap();
        let image_source_id = row.try_get::<u8, _>(4)?.unwrap();
        let retrofit = row.try_get::<bool, _>(5)?.unwrap();
        let retrofit_year = row.try_get::<i16, _>(6)?;
        let attributes_confidence_level = ConfidenceLevel::try_from(row.try_get::<u8, _>(7)?)?;
        let location_confidence_level = ConfidenceLevel::try_from(row.try_get::<u8, _>(8)?)?;
        let image_date = row.try_get::<NaiveDate, _>(9)?;
        let latitude = row.try_get::<Decimal, _>(10)?.unwrap();
        let longitude = row.try_get::<Decimal, _>(11)?.unwrap();

        Ok(Turbine {
            id,
            county_id,
            project_id,
            model_id,
            image_source_id,
            retrofit,
            retrofit_year,
            attributes_confidence_level,
            location_confidence_level,
            image_date,
            latitude,
            longitude,
        })
    }
}
