use repository::{Repository, models::ImageSource};
use rocket::{Build, State, futures::lock::Mutex, get, put, response::Responder, routes, serde::json::Json};

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500)]
    Rocket(String),
    #[response(status = 500)]
    LowLevel(String),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    ServerError(String)
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
        get_image_sources, get_image_source, update_image_source
        ];

    Ok(rocket::build()
        .mount("/", routes)
        .manage(state))
        
}

#[get("/")]
async fn index() -> &'static str {
    "Hello world!"
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/imagesources
#[get("/api/imagesources")]
async fn get_image_sources(repo: &State<SafeRepo>) -> Result<Json<Vec<ImageSource>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut image_sources = repo.get_all_image_sources().await?;
    image_sources.sort_by(|a,b| a.id.cmp(&b.id));
    Ok(Json(image_sources))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/imagesources/1
#[get("/api/imagesources/<id>")]
async fn get_image_source(repo: &State<SafeRepo>, id: u8) -> Result<Json<ImageSource>, crate::Error> {
    let mut repo = repo.lock().await;
    let image_source = repo.get_image_source(id).await?;
    Ok(Json(image_source))
}

/// curl -X PUT http://localhost:8000/api/imagesources/1 -d '"Digital Globe XXX"' -H "Content-Type: application/json" 
#[put("/api/imagesources/<id>", format = "json", data = "<name>")]
async fn update_image_source(repo: &State<SafeRepo>, id: u8, name: Json<String>) -> Result<(), crate::Error> {
    let mut repo = repo.lock().await;
    match repo.update_image_source(id, &name).await? {
        0 => Err(Error::NotFound(())),
        1 => Ok(()),
        n @ _ => Err(Error::ServerError(format!("Unexpected row count {}", n))),
    }
}
