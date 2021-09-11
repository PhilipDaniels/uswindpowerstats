use chrono::{DateTime, Utc};
use env_logger::Builder;
use log::info;
use logging_timer::stime;
use serde::Deserialize;
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

fn main() -> Result<(), Box<dyn Error>> {
    configure_logging();
    
    let opt = Opt::from_args();
    if let Some(file) = opt.us_states_file {
        load_us_states(file)?;
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

#[stime]
fn load_us_states(file: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file)?;

    for record in rdr.deserialize() {
        let record: UsState = record?;
    }

    Ok(())
}

#[stime]
fn load_turbines(file: PathBuf) {
    
}

