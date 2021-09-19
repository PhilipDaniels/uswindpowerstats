use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageSource {
    pub id: u8,
    pub name: String,
}

impl From<repository::models::ImageSource> for ImageSource {
    fn from(val: repository::models::ImageSource) -> Self {
        Self {
            id: val.id,
            name: val.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateType {
    State,
    Territory,
    FederalCapital,
}

impl From<repository::models::StateType> for StateType {
    fn from(val: repository::models::StateType) -> Self {
        match val {
            repository::models::StateType::State => Self::State,
            repository::models::StateType::Territory => Self::Territory,
            repository::models::StateType::FederalCapital => Self::FederalCapital,
        }
    }
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

impl From<repository::models::State> for State {
    fn from(val: repository::models::State) -> Self {
        Self {
            id: val.id,
            name: val.name,
            capital: val.capital,
            population: val.population,
            area_square_km: val.area_square_km,
            state_type: val.state_type.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct County {
    pub id: i32,
    pub state_id: String,
    pub name: String,
}

impl From<repository::models::County> for County {
    fn from(val: repository::models::County) -> Self {
        Self {
            id: val.id,
            state_id: val.state_id,
            name: val.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl From<repository::models::ConfidenceLevel> for ConfidenceLevel {
    fn from(val: repository::models::ConfidenceLevel) -> Self {
        match val {
            repository::models::ConfidenceLevel::Low => Self::Low,
            repository::models::ConfidenceLevel::Medium => Self::Medium,
            repository::models::ConfidenceLevel::High => Self::High,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Manufacturer {
    pub id: i32,
    pub name: String,
}

impl From<repository::models::Manufacturer> for Manufacturer {
    fn from(val: repository::models::Manufacturer) -> Self {
        Self {
            id: val.id,
            name: val.name
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub num_turbines: Option<i16>,
    pub capacity_mw: Option<Decimal>,
}

impl From<repository::models::Project> for Project {
    fn from(val: repository::models::Project) -> Self {
        Self {
            id: val.id,
            name: val.name,
            num_turbines: val.num_turbines,
            capacity_mw: val.capacity_mw,
        }
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

impl From<repository::models::Model> for Model {
    fn from(val: repository::models::Model) -> Self {
        Self {
            id: val.id,
            manufacturer_id: val.manufacturer_id,
            name: val.name,
            capacity_kw: val.capacity_kw,
            hub_height: val.hub_height,
            rotor_diameter: val.rotor_diameter,
            rotor_swept_area: val.rotor_swept_area,
            total_height_to_tip: val.total_height_to_tip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub image_date: Option<String>,
    pub latitude: Decimal,
    pub longitude: Decimal,
}

impl From<repository::models::Turbine> for Turbine {
    fn from(val: repository::models::Turbine) -> Self {
        Self {
            id: val.id,
            county_id: val.county_id,
            project_id: val.project_id,
            model_id: val.model_id,
            image_source_id: val.image_source_id,
            retrofit: val.retrofit,
            retrofit_year: val.retrofit_year,
            attributes_confidence_level: val.attributes_confidence_level.into(),
            location_confidence_level: val.location_confidence_level.into(),
            image_date: val.image_date.map(|d| d.format("%Y-%m-%d").to_string()),
            latitude: val.latitude,
            longitude: val.longitude,
        }
    }
}
