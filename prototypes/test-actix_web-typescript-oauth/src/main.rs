use actix_files::Files;
use actix_web::{App, HttpServer, Responder, web, HttpResponse};
use std::fs;

async fn index() -> impl Responder {
    let content = fs::read_to_string("index.html").expect("Unable to read file");
    HttpResponse::Ok().content_type("text/html").body(content)
}

async fn script() -> impl Responder {
    let content = fs::read_to_string("script.js").expect("Unable to read file");
    HttpResponse::Ok().content_type("application/javascript").body(content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/script.js", web::get().to(script))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
