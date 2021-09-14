use chrono::{DateTime, Utc};
use env_logger::Builder;
use itertools::Itertools;
use logging_timer::{executing, finish, stimer};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use tiberius::{Client, ToSql};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tokio::net::TcpStream;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    us_states_file: Option<PathBuf>,
    #[structopt(short, long, parse(from_os_str))]
    turbines_file: Option<PathBuf>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    configure_logging();
    
    let opt = Opt::from_args();
    if let Some(file) = opt.us_states_file {
        let states = load_us_states_from_csv(file)?;
        load_us_states_to_database(&states).await?;
    }
    if let Some(file) = opt.turbines_file {
        let turbines = load_turbines_from_csv(file)?;
        load_all_csv_data_to_database(&turbines).await?;
    }

    Ok(())
}

fn configure_logging() {
    let mut builder = Builder::from_default_env();
    builder.format(|buf, record| {
        let utc: DateTime<Utc> = Utc::now();

        match (record.file(), record.line()) {
        

        (Some(file), Some(line)) => writeln!(
            buf,
            "{:?} {} [{}/{}] {}",
            utc,
            record.level(),
            file,
            line,
            record.args()
        ),
        (Some(file), None) => writeln!(buf, "{:?} {} [{}] {}", utc, record.level(), file, record.args()),
        (None, Some(_line)) => writeln!(buf, "{:?} {} {}", utc, record.level(), record.args()),
        (None, None) => writeln!(buf, "{:?} {} {}", utc, record.level(), record.args()),
    }});

    builder.init();
}

/// Represents a US state as read from the CSV file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UsState {
    state_type: String,
    name: String,
    abbreviation: String,
    capital: Option<String>,
    population: Option<i32>,
    area: Option<i32>,
}

impl UsState {
    /// Return the area in km² (the area in the CSV is in square miles).
    fn area_in_square_km(&self) -> Option<i32> {
        self.area.map(|a| (a as f32 * 2.58999) as i32)
    }
}

fn load_us_states_from_csv(file: PathBuf) -> Result<Vec<UsState>, Box<dyn Error>> {
    let tmr = stimer!("LOAD_US_STATES_FROM_CSV");
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)?;

    let mut states = Vec::new();
    for result in rdr.deserialize() {
        let state: UsState = result?;
        states.push(state);
    }

    finish!(tmr, "Loaded {} US states from CSV", states.len());

    Ok(states)
}

async fn load_us_states_to_database(states: &[UsState]) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_US_STATES_TO_DATABASE");

    let mut client = open_ms_sql_connection().await?;
    for state in states {
        let stmt = "
        BEGIN TRANSACTION;

        UPDATE dbo.State WITH (UPDLOCK, SERIALIZABLE) SET Name = @P1, Capital = @P2, Population = @P3, AreaSquareKm = @P4, StateType = @P5
        WHERE Id = @P6;

        IF @@ROWCOUNT = 0 BEGIN
            INSERT INTO dbo.State (Id, Name, Capital, Population, AreaSquareKm, StateType)
            VALUES (@P6, @P1, @P2, @P3, @P4, @P5);
        END

        COMMIT TRANSACTION;
        ";

        let state_type = state.state_type.chars().nth(0).unwrap().to_ascii_uppercase().to_string();
        let _result = client.execute(stmt, 
            &[&state.name, &state.capital, &state.population, &state.area_in_square_km(), &state_type, &state.abbreviation]).await?;
    }

    executing!(tmr, "Loaded {} US states into database", states.len());

    let ids = states.iter()
        .fold("".to_string(),
        |a,b| if a.len() == 0 { format!("'{}'", b.abbreviation) } else { format!("{}, '{}'", a, b.abbreviation)});
        
    let stmt = format!("DELETE dbo.State WHERE Id NOT IN ({})", ids);
    let result = client.execute(stmt, &[]).await?;
    executing!(tmr, "Deleted extraneous {} US states from the database", result.rows_affected()[0]);

    let row = client.simple_query("SELECT COUNT(*) FROM dbo.State").await?.into_row().await?.unwrap();
    let num_states : i32 = row.get(0).unwrap();
    finish!(tmr, "There are now {} US states in the database", num_states);

    Ok(())
}

static CONN_STR: Lazy<String> = Lazy::new(|| {
    std::env::var("MSSQL_CONNECTION_STRING").unwrap_or_else(|_| {
        "server=tcp:localhost,1433;User Id=SA;Password=EawRsi2PCfurVZi7dym9;Initial Catalog=UsWindPowerStats;TrustServerCertificate=true".to_owned()
    })
});

async fn open_ms_sql_connection() -> Result<Client<Compat<TcpStream>>, Box<dyn std::error::Error>> {
    let config = tiberius::Config::from_ado_string(&CONN_STR)?;
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let client = Client::connect(config, tcp.compat_write()).await?;
    Ok(client)
}

#[derive(Debug, Deserialize)]
struct TurbineCsv {
    case_id: i32,
    faa_ors: String,
    faa_asn: String,
    usgs_pr_id: Option<i32>,
    eia_id: Option<i32>,
    t_state: String,
    t_county: String,
    t_fips: i32,
    p_name: String,
    p_year: Option<i32>,
    p_tnum: i32,
    p_cap: Option<f32>,
    t_manu: String,
    t_model: String,
    t_cap: Option<i32>,
    t_hh: Option<f32>,
    t_rd: Option<f32>,
    t_rsa: Option<f32>,
    t_ttlh: Option<f32>,
    retrofit: u8,
    retrofit_year: Option<i32>,
    t_conf_atr: u8,
    t_conf_loc: u8,
    t_img_date: String,
    t_img_srce: String,
    xlong: f32,
    ylat: f32,
}

fn load_turbines_from_csv(file: PathBuf) -> Result<Vec<TurbineCsv>, Box<dyn Error>> {
    let tmr = stimer!("LOAD_US_TURBINES_FROM_CSV");
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)?;

    let mut turbines = Vec::new();
    for result in rdr.deserialize() {
        let turbine: TurbineCsv = result?;
        turbines.push(turbine);
    }

    finish!(tmr, "Loaded {} US turbines from CSV", turbines.len());
    Ok(turbines)

}

/// An auxiliary type so we don't have to pass a huge tuple to the database load function.
#[derive(Debug, Copy, Clone)]
struct Model<'a> {
    t_manu: &'a String,
    t_model: &'a String,
    t_cap: Option<i32>,
    t_hh: Option<f32>,
    t_rd: Option<f32>,
    t_rsa: Option<f32>,
    t_ttlh: Option<f32>,
}

/// We consider models equivalent based on manufacturer and name only.
/// This means we don't have to worry about the floats being only PartialEq.
/// We assume things are named uniquely in the spreadsheet.
impl<'a> PartialEq for Model<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.t_manu == other.t_manu && self.t_model == other.t_model
    }
}

impl<'a> Eq for Model<'a> {}

impl TurbineCsv {
    fn to_model(&self) -> Model {
        Model {
            t_manu: &self.t_manu,
            t_model: &self.t_model,
            t_cap: self.t_cap,
            t_hh: self.t_hh,
            t_rd: self.t_rd,
            t_rsa: self.t_rsa,
            t_ttlh: self.t_ttlh,
        }
    }
}

impl<'a> std::hash::Hash for Model<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.t_manu.hash(state);
        self.t_model.hash(state);
    }
}

async fn load_all_csv_data_to_database(turbines: &[TurbineCsv]) -> Result<(), Box<dyn Error>> {
    let _tmr = stimer!("LOAD_ALL_CSV_DATA_TO_DATABASE");

    let mut client = open_ms_sql_connection().await?;

    let counties = turbines.iter()
        .map(|t| (&t.t_state, &t.t_county))
        .unique()
        .collect::<Vec<_>>();

    load_counties_to_database(&mut client, counties).await?;

    let manufacturers = turbines.iter()
        .map(|t| &t.t_manu)
        .unique()
        .collect::<Vec<_>>();

    load_manufacturers_to_database(&mut client, &manufacturers).await?;

    let models = turbines.iter()
        .map(|t| t.to_model())
        .unique()
        .collect::<Vec<_>>();

    load_turbine_models_to_database(&mut client, &models).await?;

    let image_sources = turbines.iter()
        .map(|t| &t.t_img_srce)
        .unique()
        .collect::<Vec<_>>();

    load_image_sources_to_database(&mut client, &image_sources).await?;

    // Temporarily multiply all capacities by 1000 so that we can convert them to ints
    // and hence use unique().
    let projects = turbines.iter()
        .map(|t| (&t.p_name, t.p_tnum, t.p_cap.map(|c| (c * 1000.0) as i32)))
        .unique()
        .map(|(nm, tn, cap)| (nm, tn, cap.map(|c| (c as f32) / 1000.0)))
        .collect::<Vec<_>>();

    load_projects_to_database(&mut client, &projects).await?;
    load_turbines_to_database(&mut client, turbines).await?;

    Ok(())
}

async fn load_counties_to_database(client: &mut Client<Compat<TcpStream>>, counties: Vec<(&String, &String)>) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_US_COUNTIES_TO_DATABASE");

    for county in &counties {
        let stmt = "
        IF NOT EXISTS (SELECT 1 FROM dbo.County C2 WHERE C2.StateId = @P1 and C2.Name = @P2)
        INSERT INTO dbo.County(StateId, Name)
        VALUES (@P1, @P2)
        ";

        let _result = client.execute(stmt, 
             &[county.0, county.1]).await?;
    }

    finish!(tmr, "Loaded {} US counties into the database", counties.len());
    Ok(())
}

async fn load_manufacturers_to_database(client: &mut Client<Compat<TcpStream>>, manufacturers: &Vec<&String>) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_MANUFACTURERS_TO_DATABASE");

    // We allow blank names. Easier than dealing with NULL.

    for m in manufacturers {
        let stmt = "
        IF NOT EXISTS (SELECT 1 FROM dbo.Manufacturer M2 WHERE M2.Name = @P1)
        INSERT INTO dbo.Manufacturer(Name)
        VALUES (@P1)
        ";
        
        let _result = client.execute(stmt, &[*m]).await?;
    }

    finish!(tmr, "Loaded {} manufacturers into the database", manufacturers.len());
    Ok(())
}

async fn load_turbine_models_to_database<'a>(client: &mut Client<Compat<TcpStream>>, models: &Vec<Model<'a>>) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_TURBINE_MODELS_TO_DATABASE");

    for model in models {
        let stmt = "
        EXEC dbo.model_upsert @P1, @P2, @P3, @P4, @P5, @P6, @P7
        ";

        let params: &[&dyn ToSql] = &[
            model.t_manu,
            model.t_model,
            &model.t_cap,
            &model.t_hh,
            &model.t_rd,
            &model.t_rsa,
            &model.t_ttlh,
        ];

        let _result = client.execute(stmt, params).await?;
    }

    finish!(tmr, "Loaded {} turbine models into the database", models.len());
    Ok(())
}

async fn load_image_sources_to_database(client: &mut Client<Compat<TcpStream>>, image_sources: &Vec<&String>) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_IMAGE_SOURCES_TO_DATABASE");

    // We allow blank names. Easier than dealing with NULL.

    for src in image_sources {
        let stmt = "
        IF NOT EXISTS (SELECT 1 FROM dbo.ImageSource S2 WHERE S2.Name = @P1)
        INSERT INTO dbo.ImageSource(Name)
        VALUES (@P1)
        ";
        
        let _result = client.execute(stmt, &[*src]).await?;
    }

    finish!(tmr, "Loaded {} image sources into the database", image_sources.len());
    Ok(())
}

async fn load_projects_to_database(client: &mut Client<Compat<TcpStream>>, projects: &[(&String, i32, Option<f32>)]) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_PROJECTS_TO_DATABASE");

    for p in projects {
        let stmt = "
        BEGIN TRANSACTION;

        UPDATE dbo.Project WITH (UPDLOCK, SERIALIZABLE) SET NumTurbines = @P1, CapacityMW = @P2
        WHERE Name = @P3;

        IF @@ROWCOUNT = 0 BEGIN
            INSERT INTO dbo.Project(Name, NumTurbines, CapacityMW)
            VALUES (@P3, @P1, @P2);
        END

        COMMIT TRANSACTION;
        ";

        let params: &[&dyn ToSql] = &[
            &p.1,
            &p.2,
            p.0,
        ];
        
        let _result = client.execute(stmt, params).await?;
    }

    finish!(tmr, "Loaded {} projects into the database", projects.len());
    Ok(())
}

async fn load_turbines_to_database(client: &mut Client<Compat<TcpStream>>, turbines: &[TurbineCsv]) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_TURBINES_TO_DATABASE");

    let stmt = "DELETE dbo.Turbine;";
    let _result = client.execute(stmt, &[]).await?;

    for (idx, t) in turbines.iter().enumerate() {
        let stmt = "EXEC dbo.turbine_upsert @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12, @P13";
        let image_date = parse_date(&t.t_img_date);

        let params: &[&dyn ToSql] = &[
            &t.t_state,
            &t.t_county,
            &t.p_name,
            &t.t_manu,
            &t.t_model,
            &t.t_img_srce,
            &t.retrofit,
            &t.retrofit_year,
            &t.t_conf_atr,
            &t.t_conf_loc,
            &image_date,
            &t.ylat,
            &t.xlong,
        ];
        
        let _result = client.execute(stmt, params).await?;

        if idx % 1000 == 0 {
            executing!(tmr, "Loaded {} turbines into the database", idx);        
        }
    }

    finish!(tmr, "Loaded {} turbines into the database", turbines.len());
    Ok(())
}

fn parse_date(d: &str) -> Option<String> {
    let mut parts = d.split('/');
    
    if let Some(m) = parts.next() {
        if let Some(d) = parts.next() {
            if let Some(y) = parts.next() {
                return Some(format!("{}-{:0>2}-{:0>2}", y, m, d));
            }
        }
    }

    None
}
