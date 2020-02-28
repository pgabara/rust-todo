use std::io;
use std::sync::{Mutex, Arc};

use serde::{Serialize, Deserialize};
use actix_web::{web, middleware, App, HttpServer, HttpResponse};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Todo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    title: String,
    done: bool,
}

async fn get(data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    HttpResponse::Ok().json(&data.lock().unwrap().items) 
}

async fn add(todo: web::Json<Todo>, data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    data.lock().unwrap().items.push(todo.0);
    HttpResponse::Created().finish()
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let data = web::Data::new(Arc::new(Mutex::new(AppState::default())));
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/")
                    .route(web::get().to(get))
                    .route(web::post().to(add))
            )
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}
