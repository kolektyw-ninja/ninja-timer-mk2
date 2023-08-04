use std::thread::{spawn, JoinHandle};
use std::sync::{mpsc, Mutex};
use std::fs::remove_file;
use std::io::Write;

use serde_json::json;
use serde::Deserialize;

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
    Error,
};
use actix_files as fs;
use actix_multipart::Multipart;

use futures_util::TryStreamExt as _;

use chrono::Utc;

use crate::state::{InputEvent, OutputEvent};
use crate::broadcast::Broadcaster;
use crate::assets::get_background_path;

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

#[post("api/enable_debug")]
async fn enable_debug(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::SetDebug(true));
    HttpResponse::Ok().body("OK")
}

#[post("api/disable_debug")]
async fn disable_debug(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::SetDebug(false));
    HttpResponse::Ok().body("OK")
}

#[derive(Deserialize)]
struct Countdown {
    countdown: u64,
}

#[post("api/set_countdown")]
async fn set_countdown(data: web::Data<AppState>, info: web::Json<Countdown>) -> impl Responder {
    data.send(InputEvent::SetCountdown(info.countdown));
    HttpResponse::Ok().body("OK")
}

#[post("api/delete_background")]
async fn delete_background(data: web::Data<AppState>) -> impl Responder {
    remove_file(get_background_path()).unwrap();
    data.send(InputEvent::ReloadBackground);
    HttpResponse::Ok().body("OK")
}

#[post("api/upload_background")]
async fn upload_background(data: web::Data<AppState>, mut payload: Multipart) -> Result<HttpResponse, Error> {
    let filepath = get_background_path();

    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // File::create is blocking operation, use threadpool
        let path = filepath.clone();
        let mut f = web::block(move || std::fs::File::create(path)).await??;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
    }

    println!("file uploaded");
    data.send(InputEvent::ReloadBackground);
    Ok(HttpResponse::Ok().body("OK").into())
}

#[post("api/toggle_display")]
async fn toggle_display(data: web::Data<AppState>) -> impl Responder {
    data.send(InputEvent::ToggleDisplay);
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
            match event {
                OutputEvent::SyncTimers(timers) => {
                    let payload = json!({
                        "timers": [
                            {
                                "id": 0,
                                "startedAt": timers[0].started_at_datetime.map(|x| x.timestamp_millis()),
                                "stoppedAt": timers[0].stopped_at_datetime.map(|x| x.timestamp_millis()),
                                "countdown": timers[0].countdown_duration.as_secs(),
                                "state": format!("{:?}", timers[0].get_state()),
                                "formatted": timers[0].format(),
                            },
                            {
                                "id": 1,
                                "startedAt": timers[0].started_at_datetime.map(|x| x.timestamp_millis()),
                                "stoppedAt": timers[0].stopped_at_datetime.map(|x| x.timestamp_millis()),
                                "countdown": timers[0].countdown_duration.as_secs(),
                                "state": format!("{:?}", timers[0].get_state()),
                                "formatted": timers[0].format(),
                            },
                        ],
                        "now": Utc::now().timestamp_millis(),
                    });

                    clone.send("syncTimers", &payload.to_string());
                },
                OutputEvent::SyncSettings(settings) => {
                    let payload = json!({
                        "settings": settings,
                    });

                    clone.send("syncSettings", &payload.to_string());
                },
                #[allow(unreachable_patterns)]
                _ => clone.send("outputEvent", &format!("{:?}", event))
            };
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
            .service(enable_debug)
            .service(disable_debug)
            .service(set_countdown)
            .service(delete_background)
            .service(upload_background)
            .service(toggle_display)
            .service(fs::Files::new("/", "./client/dist").index_file("index.html"))
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
