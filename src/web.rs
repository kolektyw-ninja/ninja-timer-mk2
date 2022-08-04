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

use actix_files as fs;

use crate::state::{InputEvent, OutputEvent};
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

#[post("/api/request_sync")]
async fn request_sync(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::RequestSync);
    HttpResponse::Ok().body("OK")
}

#[get("/api/events")]
async fn events(broadcaster: web::Data<Broadcaster>) -> impl Responder {
    let client = broadcaster.new_client();

    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/event-stream"))
        .streaming(client)
}

async fn init_server(sender: mpsc::Sender<InputEvent>, receiver: mpsc::Receiver<OutputEvent>) -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        sender: Mutex::new(sender),
    });

    let broadcaster = Broadcaster::create();
    let clone = broadcaster.clone();

    spawn(move || {
        for event in receiver {
            clone.send("stateChanged", &format!("{:?}", event));
        }
    });

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(broadcaster.clone())
            .service(start_timer)
            .service(stop_timer)
            .service(reset_timer)
            .service(request_sync)
            .service(events)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    Ok(())
}

pub fn spawn_server(sender: mpsc::Sender<InputEvent>, receiver: mpsc::Receiver<OutputEvent>) -> JoinHandle<()> {
    spawn(|| {
        rt::System::new().block_on(init_server(sender, receiver)).unwrap();
    })
}
