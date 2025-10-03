use actix_web::middleware::Logger;
use actix_web::{
    error::ErrorInternalServerError, get, post, web, App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use file_serve::db::{CreateShareReq, Db, FileEntry, PublicShare, Share};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// ——— User section ———

#[get("/api/share/{slug}")]
async fn get_public_share(
    path: web::Path<String>,
) -> Result<web::Json<PublicShare>, actix_web::Error> {
    let slug = path.into_inner();
    let db = Db::new().map_err(ErrorInternalServerError)?;

    match db
        .get_public_share(&slug)
        .map_err(ErrorInternalServerError)?
    {
        Some(ps) => Ok(web::Json(ps)),
        None => Err(actix_web::error::ErrorNotFound("share not found")),
    }
}

// ——— Admin section ———

// Structs

#[derive(Deserialize)]
struct CreateFileReq {
    abs_path: String,
}

// Endpoints

#[get("/admin/shares")]
async fn get_shares() -> Result<web::Json<Vec<Share>>, actix_web::Error> {
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let shares = db.list_shares().map_err(ErrorInternalServerError)?;
    Ok(web::Json(shares))
}

#[post("/admin/file")]
async fn create_file(
    body: web::Json<CreateFileReq>,
) -> Result<web::Json<FileEntry>, actix_web::Error> {
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let file = db
        .create_or_get_file(&body.abs_path)
        .map_err(ErrorInternalServerError)?;
    Ok(web::Json(file))
}

#[post("/admin/share")]
async fn create_share(
    body: web::Json<CreateShareReq>,
) -> Result<web::Json<Share>, actix_web::Error> {
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let share = db.create_share(&body).map_err(ErrorInternalServerError)?;

    Ok(web::Json(share))
}

// ——— Bind + Serve ———

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            // Customer services
            .service(get_public_share)
            .service(hello)
            // Admin service
            .service(get_shares)
            .service(create_file)
            .service(create_share)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
