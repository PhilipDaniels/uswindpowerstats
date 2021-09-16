pub mod models;

use once_cell::sync::Lazy;
use std::convert::TryInto;
use tiberius::Client;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tokio::net::TcpStream;

use models::ImageSource;

/// Represents a connection to the MS SQL US Wind Power Stats database.
pub struct Repository {
    client: Client<Compat<TcpStream>>,
}

pub mod error {
    pub enum Error {
        LowLevel(String),
        NotFound
    }

    impl From<tiberius::error::Error> for Error {
        fn from(err: tiberius::error::Error) -> Self {
            Error::LowLevel(format!("{}", err))
        }
    }

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            let err: tiberius::error::Error = err.into();
            err.into()
        }
    }
}

static CONN_STR: Lazy<String> = Lazy::new(|| {
    std::env::var("MSSQL_CONNECTION_STRING").unwrap_or_else(|_| {
        "server=tcp:localhost,1433;User Id=SA;Password=EawRsi2PCfurVZi7dym9;Initial Catalog=UsWindPowerStats;TrustServerCertificate=true".to_owned()
    })
});

impl Repository {
    /// Opens a new connection.
    pub async fn open(connection_string: Option<&str>) -> Result<Self, crate::error::Error> {
        let connection_string = connection_string.unwrap_or_else(|| { CONN_STR.as_ref() });

        let config = tiberius::Config::from_ado_string(&connection_string)?;
        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let client = Client::connect(config, tcp.compat_write()).await?;
        
        Ok(Repository {
            client
        })
    }

    /// Gets all ImageSource rows.
    pub async fn get_all_image_sources(&mut self) -> Result<Vec<ImageSource>, crate::error::Error> {
        let mut image_sources = Vec::new();

        let stream = self.client.simple_query("SELECT Id, Name FROM dbo.ImageSource").await?;
        let rows = stream.into_first_result().await?;

        for row in &rows {
            image_sources.push(row.try_into()?);
        }
        
        Ok(image_sources)
    }

    /// Gets the ImageSource with the specific Id. Returns None if no match found.
    pub async fn get_image_source(&mut self, id: u8) -> Result<ImageSource, crate::error::Error> {
        let stream = self.client.query("SELECT Id, Name FROM dbo.ImageSource WHERE Id = @P1", &[&id]).await?;
        let rows = stream.into_first_result().await?;
        match rows.iter().next() {
            Some(row) => Ok(row.try_into()?),
            None => Err(error::Error::NotFound)
        }
    }

    /// Update a row in the ImageSource table. Returns the number of rows affected (0 or 1).
    pub async fn update_image_source(&mut self, id: u8, name: &str) -> Result<u64, crate::error::Error> {
        let stmt = "UPDATE dbo.ImageSource SET Name = @P1 WHERE Id = @P2;";
        let result = self.client.execute(stmt, &[&name, &id]).await?;
        Ok(result.total())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
