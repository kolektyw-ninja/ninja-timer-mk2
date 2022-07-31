use std::thread::{spawn, JoinHandle};
use std::sync::{mpsc, Mutex};

use actix_web::{rt, get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::state::InputEvent;

struct AppState {
    sender: Mutex<mpsc::Sender<InputEvent>>,
}

impl AppState {
    fn send(&self, event: InputEvent) {
        self.sender.lock().unwrap().send(event).unwrap();
    }
}

#[post("/api/start_timer")]
async fn start_timer(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::StartTimer(0));
    HttpResponse::Ok().body("OK")
}

#[post("/api/stop_timer")]
async fn stop_timer(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::StopTimer(0));
    HttpResponse::Ok().body("OK")
}

#[post("/api/reset_timer")]
async fn reset_timer(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::ResetTimer(0));
    HttpResponse::Ok().body("OK")
}


pub fn run_server(sender: mpsc::Sender<InputEvent>) -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        sender: Mutex::new(sender),
    });

    rt::System::new().block_on(
        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .service(start_timer)
                .service(stop_timer)
                .service(reset_timer)
        })
        .bind(("0.0.0.0", 8080))?
        .run()
    )
}

pub fn spawn_server(sender: mpsc::Sender<InputEvent>) -> JoinHandle<()> {
    spawn(|| {
        run_server(sender).unwrap();
    })
}
