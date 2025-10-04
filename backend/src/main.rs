use actix_files::NamedFile;
use actix_web::http::header::{Charset, ExtendedValue};
use actix_web::middleware::Logger;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    post, web, App, HttpResponse, HttpServer, Responder, Result,
};
use serde::Deserialize;
use std::path::PathBuf;

use file_serve::db::{CreateShareReq, Db, FileEntry, PublicShare, Share};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// ——— User section ———

// Structs
#[derive(Deserialize)]
struct DownloadQuery {
    // Optional: password or not
    password: Option<String>,
}

#[get("/api/download/{slug}")]
async fn download_file(path: web::Path<String>, q: web::Query<DownloadQuery>) -> Result<NamedFile> {
    let slug = path.into_inner();
    let password = q.password.as_deref().unwrap_or("");

    let db = Db::new().map_err(ErrorInternalServerError)?;

    match db.get_download_target(&slug, password) {
        Ok(Some((abs_path, file_name))) => {
            let path: PathBuf = abs_path.into();
            let mut file = NamedFile::open(path).map_err(ErrorInternalServerError)?;

            // Set Content-type
            let ct = mime_guess::from_path(file.path()).first_or_octet_stream();
            file = file.set_content_type(ct);

            // Force download with UTF-8 filename
            file = file.set_content_disposition(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::FilenameExt(ExtendedValue {
                    charset: Charset::Ext("UTF-8".to_owned()),
                    language_tag: None,
                    value: file_name.as_bytes().to_vec(),
                })],
            });

            Ok(file)
        }
        Ok(None) => Err(actix_web::error::ErrorNotFound("share not found")),
        Err(e) => {
            //UnwindingPanic sentinel for incorrect password
            let is_auth_error = matches!(e, rusqlite::Error::UnwindingPanic);
            if is_auth_error {
                Err(ErrorUnauthorized("Invalid password"))
            } else {
                Err(ErrorInternalServerError(e))
            }
        }
    }
}

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

#[delete("/admin/file/{file_id}")]
async fn delete_file(path: web::Path<String>) -> Result<HttpResponse> {
    let file_id = path.into_inner();
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let deleted = db.delete_file(&file_id).map_err(ErrorInternalServerError)?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[post("/admin/share")]
async fn create_share(
    body: web::Json<CreateShareReq>,
) -> Result<web::Json<Share>, actix_web::Error> {
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let share = db.create_share(&body).map_err(ErrorInternalServerError)?;

    Ok(web::Json(share))
}

#[delete("/admin/share/{slug}")]
async fn delete_share(path: web::Path<String>) -> Result<HttpResponse> {
    let slug = path.into_inner();
    let db = Db::new().map_err(ErrorInternalServerError)?;
    let deleted = db.delete_share(&slug).map_err(ErrorInternalServerError)?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

// ——— Bind + Serve ———

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(hello)
            // Customer services
            .service(get_public_share)
            .service(download_file)
            // Admin service
            .service(get_shares)
            .service(create_file)
            .service(delete_file)
            .service(create_share)
            .service(delete_share)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
