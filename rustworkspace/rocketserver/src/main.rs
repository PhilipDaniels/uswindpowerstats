use repository::{Repository, models::ImageSource};
use rocket::{get, launch, routes, response::Responder, serde::json::Json};
use rocket::tokio::time::{sleep, Duration};

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500)]
    LowLevel(String),
    #[response(status = 404)]
    NotFound(()),           // Get rid of this String?
}

impl From<repository::error::Error> for Error {
    fn from(err: repository::error::Error) -> Self {
        match err {
            repository::error::Error::LowLevel(err) => Error::LowLevel(err),
            repository::error::Error::NotFound => Error::NotFound(())
        }
    }
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[get("/")]
async fn index() -> Result<Json<Vec<ImageSource>>, crate::Error> {
    let mut repo = Repository::open(None).await?;
    let mut image_sources = repo.get_all_image_sources().await?;
    image_sources.sort_by(|a,b| a.id.cmp(&b.id));
    Ok(Json(image_sources))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, delay])
}
