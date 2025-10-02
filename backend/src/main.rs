use actix_web::{
    error::ErrorInternalServerError, get, post, web, App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

use file_serve::db::{Db, FileEntry, Share};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// ——— Admin section ———

/// Structs

#[derive(Deserialize)]
struct CreateFileReq {
    abs_path: String,
}

/// Endpoints

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

async fn create_share(
    body: web::Json<CreateShareReq>,
) -> Result<web::Json<Vec<Share>>, actix_web::Error> {
    let db = Db::new().map_err(ErrorInternalServerError)?;
}

/// ——— Bind + Serve ———

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(get_shares))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
