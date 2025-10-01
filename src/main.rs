use actix_files::NamedFile;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};

use file_serve::db::{Db, Share};
use serde::{Deserialize, Serialize};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// Admin related endpoints

#[get("/admin")]
async fn admin_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/admin.html")?)
}

#[get("/admin/api/shares")]
async fn list_shares() -> actix_web::Result<impl Responder> {
    let db = Db::new().map_err(to_ise)?;
    let shares = db.list_shares().map_err(to_ise)?;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(admin_page)
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
