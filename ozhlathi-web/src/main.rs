mod config;
use config::{Config, Credentials};
use lazy_static::lazy_static;
use ozhlathi_base::{MachineStatus, Notification};
use rocket::request::Outcome;
use rocket::serde::json::{serde_json, Json};
use rocket_cors::AllowedOrigins;
use std::collections::HashMap;
use std::{
    net::IpAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};
#[macro_use]
extern crate rocket;

lazy_static! {
    static ref CONFIG: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config {
        ..Default::default()
    }));
    static ref CREDENTIALS: Mutex<config::Credentials> = Mutex::new(config::Credentials {
        ..Default::default()
    });
    static ref MACHINE_STATUSES: Mutex<HashMap<String, MachineStatus>> = Mutex::new(HashMap::new());
}

#[launch]
fn rocket() -> _ {
    *CONFIG.lock().unwrap() = {
        let config_file = std::fs::File::open("config.json").expect("Failed to open config file");
        serde_json::from_reader::<std::fs::File, Config>(config_file)
            .expect("Failed to parse config file")
    };
    *CREDENTIALS.lock().unwrap() = {
        let credentials_file =
            std::fs::File::open("credentials.json").expect("Failed to open config file");
        serde_json::from_reader::<std::fs::File, Credentials>(credentials_file)
            .expect("Failed to parse credentials file")
    };

    let config = CONFIG.lock().unwrap();

    println!("url: {}", config.url);

    let figment = rocket::Config::figment()
        .merge((
            "address",
            IpAddr::from_str(config.url.split(':').nth(0).unwrap()).expect("Failed to parse IP"),
        ))
        .merge((
            "port",
            u16::from_str(config.url.split(':').nth(1).unwrap()).unwrap(),
        ));

    drop(config);

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_headers: rocket_cors::AllOrSome::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::custom(figment).attach(cors).mount(
        "/",
        routes![notifications, machine_report, get_config, set_config],
    )
}

#[get("/notifications")]
fn notifications(_authed: Authenticated) -> Json<Vec<Notification>> {
    let mut notifications = Vec::new();
    let machine_statuses = MACHINE_STATUSES.lock().unwrap();
    for status in machine_statuses.iter() {
        let status = status.1;
        notifications.push(Notification {
            color: {
                if (status.timestamp.unwrap() - chrono::Utc::now().timestamp()).abs() > 60 * 10 {
                    "red".to_string()
                } else {
                    "green".to_string()
                }
            },
            title: format!("{} status", status.name),
            content: {
                let mut str = String::new();
                str.push_str("mem: ");
                str.push_str(&format_usage(
                    status.memory.available_memory,
                    status.memory.total_memory,
                ));
                str.push('\n');
                str.push_str("swap: ");
                str.push_str(&format_usage(
                    status.memory.free_swap,
                    status.memory.total_swap,
                ));
                str
            },
        })
    }
    Json(notifications)
}

fn format_usage(available: u64, total: u64) -> String {
    format!(
        "{avail}MB / {total}MB ({used_percentage}%)\n",
        avail = available / 1000 / 1000,
        total = total / 1000 / 1000,
        used_percentage = (available as f64 / total as f64 * 100.0).floor() as u8
    )
}

#[post("/machine/report", data = "<data>")]
fn machine_report(data: Json<MachineStatus>, _authed: Authenticated) -> Result<(), &'static str> {
    let mut data = data.into_inner();
    let mut machine_statuses = MACHINE_STATUSES.lock().unwrap();
    data.timestamp = Some(chrono::Utc::now().timestamp());
    machine_statuses.insert(data.name.clone(), data);
    Ok(())
}

#[get("/config")]
fn get_config(_authed: Authenticated) -> Json<Config> {
    Json(CONFIG.lock().unwrap().clone())
}

#[post("/config", data = "<data>")]
fn set_config(data: Json<Config>, _authed: Authenticated) -> Result<(), &'static str> {
    let data = data.into_inner();
    *CONFIG.lock().unwrap() = data.clone();
    // save to file
    let config_file = std::fs::File::create("config.json").expect("Failed to open config file");
    serde_json::to_writer(&config_file, &data).expect("Failed to write config file");
    config_file.sync_all().expect("Failed to sync config file");
    Ok(())
}

struct Authenticated;

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for Authenticated {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let credentials = CONFIG.lock().unwrap();
        if credentials.password == "" {
            return Outcome::Success(Authenticated);
        }
        let auth_header = request.headers().get_one("Authorization");
        if auth_header.is_none() {
            return Outcome::Failure((rocket::http::Status::Unauthorized, ()));
        }
        let auth_header = auth_header.unwrap();
        if auth_header != credentials.password {
            return Outcome::Failure((rocket::http::Status::Unauthorized, ()));
        }
        Outcome::Success(Authenticated)
    }
}
