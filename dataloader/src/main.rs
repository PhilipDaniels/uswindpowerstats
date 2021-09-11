use chrono::{DateTime, Utc};
use env_logger::Builder;
use logging_timer::{finish, stime, stimer};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tiberius::Client;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

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
        load_us_states(file).await?;
    }
    if let Some(file) = opt.turbines_file {
        load_turbines(file);
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

async fn load_us_states(file: PathBuf) -> Result<(), Box<dyn Error>> {
    let tmr = stimer!("LOAD_US_STATES");
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)?;

    let mut states = Vec::new();
    for result in rdr.deserialize() {
        let state: UsState = result?;
        states.push(state);
    }

    finish!(tmr, "Loaded {} US states", states.len());

    let conn = open_ms_sql_connection().await?;
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
    
    // let stream = client.query("SELECT @P1", &[&42_i32]).await?;
    // let row = stream.into_row().await?.unwrap();

    // println!("{:?}", row);
    // assert_eq!(Some(42), row.get(0));

    Ok(client)
}


#[stime]
fn load_turbines(file: PathBuf) {
    
}

