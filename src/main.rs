use std::io;
use std::sync::{Mutex, Arc};

use actix_web::{web, middleware, App, HttpServer, HttpResponse};

mod lib;

use lib::*;

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Todo>,
}

/// Gets all active todo items.
async fn get(data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    HttpResponse::Ok().json(&data.lock().unwrap().items) 
}

/// Deletes all active todo items.
async fn delete(data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    data.lock().unwrap().items = Vec::default();
    HttpResponse::Ok().finish()
}

/// Gets active todo by its id.
async fn get_todo(path: web::Path<Id>, data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    let todos = &data.lock().unwrap().items;
    match todos.iter().find(|x| x.id == path.id) {
        Some(todo) => HttpResponse::Ok().json(&todo),
        None       => HttpResponse::NotFound().finish(),
    }
}

/// Adds new todo item.
async fn add(todo: web::Json<NewTodo>, data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    let todo    = Todo::from_new(&todo.0);
    let todo_id = todo.id;
    data.lock().unwrap().items.push(todo);
    HttpResponse::Created().json(Id { id: todo_id })
}

/// Deletes active todo item by its id.
async fn delete_todo(path: web::Path<Id>, data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    let index = data.lock().unwrap().items.iter().position(|x| x.id == path.id);
    match index {
        Some(index) => {
            data.lock().unwrap().items.remove(index);
            HttpResponse::Ok().finish()
        },
        None => HttpResponse::NotFound().finish(),
    }
}

/// Updates todo item identified by provided id.
async fn update_todo(path: web::Path<Id>, update: web::Json<UpdateTodo>, data: web::Data<Arc<Mutex<AppState>>>) -> HttpResponse {
    for item in &mut data.lock().unwrap().items {
        if item.id == path.id {
            if let Some(t) = &update.title { item.title = t.clone(); }
            if let Some(c) = update.completed { item.completed = c; }
            return HttpResponse::Ok().finish();
        }
    }
    HttpResponse::NotFound().finish()
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
                    .route(web::delete().to(delete))
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_todo))
                    .route(web::patch().to(update_todo))
                    .route(web::delete().to(delete_todo))
            )
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {

    use super::*;
    use actix_web::http;

    #[actix_rt::test]
    async fn get_status_code() {
        let state    = AppState::default();
        let data     = web::Data::new(Arc::new(Mutex::new(state)));
        let response = get(data).await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_empty_todos_json() {
        let state    = AppState::default();
        let data     = web::Data::new(Arc::new(Mutex::new(state)));
        let response = get(data).await;
        let body: Vec<Todo> = match response.body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => serde_json::from_slice(bytes).unwrap(),
            _ => panic!("Response body error!")
        };
        assert_eq!(body, Vec::default());
    }

    #[actix_rt::test]
    async fn get_todos_json() { 
        let state = AppState { 
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
                Todo::from_new(&NewTodo { title: String::from("test 3") }),
            ] 
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let response = get(data.clone()).await;
        let body: Vec<Todo> = match response.body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => serde_json::from_slice(bytes).unwrap(),
            _ => panic!("Response body error!")
        };
        assert_eq!(body, data.lock().unwrap().items);
    }

    #[actix_rt::test]
    async fn delete_active_todo_items() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
                Todo::from_new(&NewTodo { title: String::from("test 3") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let response = delete(data.clone()).await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(data.lock().unwrap().items, Vec::default());
    }

    #[actix_rt::test]
    async fn get_todo_not_found_status() {
        let state = AppState::default();
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let path = web::Path::from(Id { id: uuid::Uuid::new_v4() });
        let response = get_todo(path, data).await;
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_todo_status() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let path = web::Path::from(Id { id: data.lock().unwrap().items[0].id });
        let response = get_todo(path, data.clone()).await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_todo_json() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let path = web::Path::from(Id { id: data.lock().unwrap().items[0].id });
        let response = get_todo(path, data.clone()).await;
        let body: Todo = match response.body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => serde_json::from_slice(bytes).unwrap(),
            _ => panic!("Response body error!")
        };
        assert_eq!(body, data.lock().unwrap().items[0]);
    }

    #[actix_rt::test]
    async fn add_todo_status() {
        let state = AppState::default();
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let new = web::Json(NewTodo { title: String::from("Learn Rust") });
        let response = add(new, data).await;
        assert_eq!(response.status(), http::StatusCode::CREATED);
    } 

    #[actix_rt::test]
    async fn add_todo_json() {
        let state = AppState::default();
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let new = web::Json(NewTodo { title: String::from("Learn Rust") });
        let response = add(new, data.clone()).await;
        let body: Id = match response.body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => serde_json::from_slice(bytes).unwrap(),
            _ => panic!("Response body error!")
        };
        let expected = Todo {
            id: body.id,
            title: String::from("Learn Rust"),
            completed: false,
        };
        assert_eq!(body, Id { id: expected.id });
        assert_eq!(expected, data.lock().unwrap().items[0]);
    }

    #[actix_rt::test]
    async fn delete_todo_not_found_status() {
        let state = AppState::default();
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let path = web::Path::from(Id { id: uuid::Uuid::new_v4() });
        let response = delete_todo(path, data).await;
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn delete_todo_status() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let todo_id = data.lock().unwrap().items[0].id;
        let path = web::Path::from(Id { id: todo_id });
        let response = delete_todo(path, data.clone()).await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn delete_todo_updated_state() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let todo_id = data.lock().unwrap().items[0].id;
        let path = web::Path::from(Id { id: todo_id });
        delete_todo(path, data.clone()).await;
        assert_eq!(data.lock().unwrap().items.len(), 1);
        assert_ne!(data.lock().unwrap().items[0].id, todo_id);
    }

    #[actix_rt::test]
    async fn update_todo_not_found_status() {
        let state = AppState::default();
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let path = web::Path::from(Id { id: uuid::Uuid::new_v4() });
        let json = web::Json(UpdateTodo { title: None, completed: Some(true) });
        let response = update_todo(path, json, data).await;
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn update_todo_updated_state() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let todo_id = data.lock().unwrap().items[0].id;
        let path = web::Path::from(Id { id: todo_id });
        let json = web::Json(UpdateTodo { title: Some(String::from("test 1 updated")), completed: Some(true) });
        update_todo(path, json, data.clone()).await;
        assert_eq!(data.lock().unwrap().items[0], Todo { id: todo_id, title: String::from("test 1 updated"), completed: true });
    }

    #[actix_rt::test]
    async fn update_todo_status() {
        let state = AppState {
            items: vec![
                Todo::from_new(&NewTodo { title: String::from("test 1") }),
                Todo::from_new(&NewTodo { title: String::from("test 2") }),
            ]
        };
        let data = web::Data::new(Arc::new(Mutex::new(state)));
        let todo_id = data.lock().unwrap().items[0].id;
        let path = web::Path::from(Id { id: todo_id });
        let json = web::Json(UpdateTodo { title: Some(String::from("test 1 updated")), completed: Some(true) });
        let response = update_todo(path, json, data.clone()).await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }
}