use std::env;
use axum::{
    routing::{get, post},
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::Router,
    Json,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use redis::{Client, Commands, RedisError, RedisResult};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use axum::response::IntoResponse;
use tokio::sync::Mutex;
use utoipa::{OpenApi, ToSchema};


#[derive(Clone)]
struct AppState {
    redis_client: Arc<Mutex<Client>>,
}

#[derive(Deserialize, ToSchema)]
struct ShortenerCreationRequest {
    destination_url: String,
}



#[derive(Serialize, ToSchema)]
struct ShortenerCreationResponse {
    short_id: String,
    full_url: String,
    destination_url: String,
}
#[utoipa::path(
    get,
    path = "/{shortened_id}",
    responses(
            (status = 307, description = "Redirect to found link"),
            (status = NOT_FOUND, description = "Supplied shortened ID was not found")
    ),
    params(
            ("shortened_id" = String, Path, description = "Shortened ID that corresponds to a link"),
    )
)]
async fn get_link(
    State(state): State<AppState>,
    Path(shortened_id): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    match get_full_url(state.redis_client.clone(), shortened_id).await {
        Ok(url) => Ok(Redirect::temporary(&url)),
        Err(err) => Err((StatusCode::NOT_FOUND, err.to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/",
    request_body = ShortenerCreationRequest,
    responses(
        (status = 200, description = "Shortened URL created successfully", body = ShortenerCreationResponse),
        (status = 500, description = "Failed to create shortened URL")
    )
)]
async fn add_entry(
    State(state): State<AppState>,
    Json(shortener_req): Json<ShortenerCreationRequest>,
) -> Result<Json<ShortenerCreationResponse>, (StatusCode, String)> {
    let host = "http://localhost:8080"; // Replace with actual host if needed

    let res = create_shortened_url(state.redis_client.clone(), shortener_req).await;

    match res {
        Ok((key, value)) => {
            let full_url = format!("{}/{}", host, key);
            Ok(Json(ShortenerCreationResponse {
                short_id: key,
                full_url,
                destination_url: value,
            }))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create shortened URL".into())),
    }
}

async fn create_shortened_url(
    client: Arc<Mutex<Client>>,
    shortener_req: ShortenerCreationRequest,
) -> Result<(String, String), RedisError> {
    let mut con = client.lock().await.get_connection()?;
    let key = generate_random_id();
    let value = shortener_req.destination_url;
    let result: RedisResult<()> = con.set(&key, &value);

    match result {
        Ok(_) => Ok((key, value)),
        Err(err) => Err(err),
    }
}

async fn get_full_url(client: Arc<Mutex<Client>>, shortened_url: String) -> Result<String, RedisError> {
    let mut con = client.lock().await.get_connection()?;
    let result: Option<String> = con.get(&shortened_url)?;
    match result {
        Some(url) => Ok(url),
        None => Err(RedisError::from((redis::ErrorKind::TypeError, "Key not found"))),
    }
}

fn generate_random_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| c.is_ascii_alphanumeric() && c.is_ascii_lowercase())
        .take(10)
        .map(char::from)
        .collect()
}
async fn get_openapi() -> impl IntoResponse {
    let openapi = ApiDoc::openapi();
    let json = (&openapi).to_pretty_json().expect("Failed to serialize OpenAPI schema");
    (StatusCode::OK, [("Content-Type", "application/json")], json)
}
#[derive(OpenApi)]
#[openapi(
    paths(
        add_entry,
        get_link
    ),
    components(
        schemas(
            ShortenerCreationRequest,
            ShortenerCreationResponse
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let redis_conn_string = env::var("REDIS_CONN_STRING").expect("REDIS_CONN_STRING must be set");
    let bind_host = env::var("BIND_HOST").expect("BIND_HOST must be set");
    let bind_port: u16 = env::var("BIND_PORT").expect("BIND_PORT must be set").parse().expect("BIND_PORT must be a valid u16");


    let redis_client = Client::open(redis_conn_string).expect("Invalid Redis URL");
    let app_state = AppState {
        redis_client: Arc::new(Mutex::new(redis_client)),
    };

    let app = Router::new()
        .route("/:shortened", get(get_link))
        .route("/", post(add_entry))
        .route("/openapi.json", get(get_openapi))
        .with_state(app_state);

    let ip_addr: IpAddr = bind_host.parse().expect("Invalid BIND_HOST");
    let addr = SocketAddr::from((ip_addr, bind_port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}