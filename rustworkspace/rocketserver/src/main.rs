use repository::{Repository, models::ImageSource};
use rocket::{Build, State, futures::lock::Mutex, get, response::Responder, routes, serde::json::Json};

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500)]
    Rocket(String),
    #[response(status = 500)]
    LowLevel(String),
    #[response(status = 404)]
    NotFound(()),
}

type SafeRepo = Mutex<Repository>;

impl From<repository::error::Error> for Error {
    fn from(err: repository::error::Error) -> Self {
        match err {
            repository::error::Error::LowLevel(err) => Error::LowLevel(err),
            repository::error::Error::NotFound => Error::NotFound(())
        }
    }
}

impl From<rocket::Error> for Error {
    fn from(err: rocket::Error) -> Self {
        Error::Rocket(format!("{}", err))
    }
}

#[rocket::main]
async fn main() -> Result<(), crate::Error> {
    Ok(rocket().await?.launch().await?)
}

async fn rocket() -> Result<rocket::Rocket<Build>, crate::Error> {
    let repo = Repository::open(None).await?;
    let state = Mutex::new(repo);

    let routes = routes![
        index,
        get_image_sources, get_image_source
        ];

    Ok(rocket::build()
        .mount("/", routes)
        .manage(state))
}

#[get("/")]
async fn index() -> &'static str {
    "Hello world!"
}

#[get("/api/imagesources")]
async fn get_image_sources(repo: &State<SafeRepo>) -> Result<Json<Vec<ImageSource>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut image_sources = repo.get_all_image_sources().await?;
    image_sources.sort_by(|a,b| a.id.cmp(&b.id));
    Ok(Json(image_sources))
}

#[get("/api/imagesource/<id>")]
async fn get_image_source(repo: &State<SafeRepo>, id: u8) -> Result<Json<ImageSource>, crate::Error> {
    let mut repo = repo.lock().await;
    let image_source = repo.get_image_source(id).await?;
    Ok(Json(image_source))
}
