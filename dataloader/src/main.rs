use chrono::{DateTime, Utc};
use env_logger::Builder;
use logging_timer::{executing, finish, stimer};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_aux::prelude::deserialize_bool_from_anything;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use tiberius::Client;
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
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    retrofit: bool,
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

async fn load_us_turbines_to_database(states: &[TurbineCsv]) -> Result<(), Box<dyn Error>> {
    // let mut client = open_ms_sql_connection().await?;
    // for state in &states {
    //     let stmt = "
    //     BEGIN TRANSACTION;

    //     UPDATE dbo.State WITH (UPDLOCK, SERIALIZABLE) SET Name = @P1, Capital = @P2, Population = @P3, AreaSquareKm = @P4, StateType = @P5
    //     WHERE Id = @P6;

    //     IF @@ROWCOUNT = 0 BEGIN
    //         INSERT INTO dbo.State (Id, Name, Capital, Population, AreaSquareKm, StateType)
    //         VALUES (@P6, @P1, @P2, @P3, @P4, @P5);
    //     END

    //     COMMIT TRANSACTION;
    //     ";

    //     let state_type = state.state_type.chars().nth(0).unwrap().to_ascii_uppercase().to_string();
    //     let _result = client.execute(stmt, 
    //         &[&state.name, &state.capital, &state.population, &state.area_in_square_km(), &state_type, &state.abbreviation]).await?;
    // }

    // executing!(tmr, "Loaded {} US states into database", states.len());

    // let ids = states.iter()
    //     .fold("".to_string(),
    //     |a,b| if a.len() == 0 { format!("'{}'", b.abbreviation) } else { format!("{}, '{}'", a, b.abbreviation)});
        
    // let stmt = format!("DELETE dbo.State WHERE Id NOT IN ({})", ids);
    // let result = client.execute(stmt, &[]).await?;
    // executing!(tmr, "Deleted extraneous {} US states from the database", result.rows_affected()[0]);

    // let row = client.simple_query("SELECT COUNT(*) FROM dbo.State").await?.into_row().await?.unwrap();
    // let num_states : i32 = row.get(0).unwrap();
    // finish!(tmr, "There are now {} US states in the database", num_states);

    Ok(())
}