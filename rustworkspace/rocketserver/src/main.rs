use repository::{Repository, models::ImageSource};
use rocket::{get, launch, routes, response::Responder, serde::json::Json};

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500)]
    LowLevel(String),
    #[response(status = 404)]
    NotFound(()),
}

impl From<repository::error::Error> for Error {
    fn from(err: repository::error::Error) -> Self {
        match err {
            repository::error::Error::LowLevel(err) => Error::LowLevel(err),
            repository::error::Error::NotFound => Error::NotFound(())
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![
        index,
        get_image_sources, get_image_source
    ])
}

#[get("/")]
async fn index() -> &'static str {
    "Hello world!"
}

#[get("/api/imagesources")]
async fn get_image_sources() -> Result<Json<Vec<ImageSource>>, crate::Error> {
    let mut repo = Repository::open(None).await?;
    let mut image_sources = repo.get_all_image_sources().await?;
    image_sources.sort_by(|a,b| a.id.cmp(&b.id));
    Ok(Json(image_sources))
}

#[get("/api/imagesource/<id>")]
async fn get_image_source(id: u8) -> Result<Json<ImageSource>, crate::Error> {
    let mut repo = Repository::open(None).await?;
    let image_source = repo.get_image_source(id).await?;
    Ok(Json(image_source))
}
