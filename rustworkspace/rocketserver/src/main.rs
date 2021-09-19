use repository::Repository;
use rocket::{
    futures::lock::Mutex, get, put, response::Responder, routes, serde::json::Json, Build, State,
};

mod results;
use results::*;

#[derive(Debug, Responder)]
pub enum Error {
    #[response(status = 500)]
    Rocket(String),
    #[response(status = 500)]
    LowLevel(String),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    ServerError(String),
}

impl From<repository::error::Error> for Error {
    fn from(err: repository::error::Error) -> Self {
        match err {
            repository::error::Error::LowLevel(msg) => Error::LowLevel(msg),
            repository::error::Error::NotFound => Error::NotFound(()),
            repository::error::Error::UnknownStateType(msg) => Error::ServerError(msg),
            repository::error::Error::UnknownConfidenceLevel(msg) => Error::ServerError(msg),
        }
    }
}

impl From<rocket::Error> for Error {
    fn from(err: rocket::Error) -> Self {
        Error::Rocket(format!("{}", err))
    }
}

type SafeRepo = Mutex<Repository>;

#[rocket::main]
async fn main() -> Result<(), crate::Error> {
    Ok(rocket().await?.launch().await?)
}

async fn rocket() -> Result<rocket::Rocket<Build>, crate::Error> {
    let repo = Repository::open(None).await?;
    let state = Mutex::new(repo);

    let routes = routes![
        index,
        get_image_sources,
        get_image_source,
        update_image_source,
        get_states,
        get_counties,
        get_projects,
        get_manufacturers,
        get_models,
        get_turbines,
    ];

    Ok(rocket::build().mount("/", routes).manage(state))
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
    image_sources.sort_by(|a, b| a.id.cmp(&b.id));
    let image_sources = image_sources.into_iter().map(|i| i.into()).collect();
    Ok(Json(image_sources))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/imagesources/1
#[get("/api/imagesources/<id>")]
async fn get_image_source(
    repo: &State<SafeRepo>,
    id: u8,
) -> Result<Json<ImageSource>, crate::Error> {
    let mut repo = repo.lock().await;
    let image_source = repo.get_image_source(id).await?;
    Ok(Json(image_source.into()))
}

/// curl -X PUT http://localhost:8000/api/imagesources/1 -d '"Digital Globe XXX"' -H "Content-Type: application/json"
#[put("/api/imagesources/<id>", format = "json", data = "<name>")]
async fn update_image_source(
    repo: &State<SafeRepo>,
    id: u8,
    name: Json<String>,
) -> Result<(), crate::Error> {
    let mut repo = repo.lock().await;
    match repo.update_image_source(id, &name).await? {
        0 => Err(Error::NotFound(())),
        1 => Ok(()),
        n @ _ => Err(Error::ServerError(format!("Unexpected row count {}", n))),
    }
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/states
#[get("/api/states")]
async fn get_states(repo: &State<SafeRepo>) -> Result<Json<Vec<results::State>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut states = repo.get_all_states().await?;
    states.sort_by(|a, b| a.id.cmp(&b.id));
    let states = states.into_iter().map(|i| i.into()).collect();
    Ok(Json(states))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/counties
#[get("/api/counties")]
async fn get_counties(repo: &State<SafeRepo>) -> Result<Json<Vec<County>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut counties = repo.get_all_counties().await?;
    counties.sort_by(|a, b| a.id.cmp(&b.id));
    let counties = counties.into_iter().map(|i| i.into()).collect();
    Ok(Json(counties))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/projects
#[get("/api/projects")]
async fn get_projects(repo: &State<SafeRepo>) -> Result<Json<Vec<Project>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut projects = repo.get_all_projects().await?;
    projects.sort_by(|a, b| a.id.cmp(&b.id));
    let projects = projects.into_iter().map(|i| i.into()).collect();
    Ok(Json(projects))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/manufacturers
#[get("/api/manufacturers")]
async fn get_manufacturers(
    repo: &State<SafeRepo>,
) -> Result<Json<Vec<Manufacturer>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut manufacturers = repo.get_all_manufacturers().await?;
    manufacturers.sort_by(|a, b| a.id.cmp(&b.id));
    let manufacturers = manufacturers.into_iter().map(|i| i.into()).collect();
    Ok(Json(manufacturers))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/models
#[get("/api/models")]
async fn get_models(repo: &State<SafeRepo>) -> Result<Json<Vec<Model>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut models = repo.get_all_models().await?;
    models.sort_by(|a, b| a.id.cmp(&b.id));
    let models = models.into_iter().map(|i| i.into()).collect();
    Ok(Json(models))
}

/// curl -w "\n" -i -X GET http://localhost:8000/api/turbines
#[get("/api/turbines")]
async fn get_turbines(repo: &State<SafeRepo>) -> Result<Json<Vec<Turbine>>, crate::Error> {
    let mut repo = repo.lock().await;
    let mut turbines = repo.get_all_turbines().await?;
    turbines.sort_by(|a, b| a.id.cmp(&b.id));
    let turbines = turbines.into_iter().map(|i| i.into()).collect();
    Ok(Json(turbines))
}
