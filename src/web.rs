use std::thread::{spawn, JoinHandle};
use std::sync::{mpsc, Mutex};

use actix_web::{
    rt,
    get,
    post,
    web,
    http::header,
    App,
    HttpResponse,
    HttpServer,
    Responder,
};

use crate::state::InputEvent;
use crate::broadcast::Broadcaster;

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

#[get("/api/events")]
async fn events(broadcaster: web::Data<Broadcaster>) -> impl Responder {
    let client = broadcaster.new_client();

    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/event-stream"))
        .streaming(client)
}

async fn init_server(sender: mpsc::Sender<InputEvent>) -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        sender: Mutex::new(sender),
    });

    let broadcaster = Broadcaster::create();

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(broadcaster.clone())
            .service(start_timer)
            .service(stop_timer)
            .service(reset_timer)
            .service(events)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    Ok(())
}

pub fn spawn_server(sender: mpsc::Sender<InputEvent>) -> JoinHandle<()> {
    spawn(|| {
        rt::System::new().block_on(init_server(sender)).unwrap();
    })
}
