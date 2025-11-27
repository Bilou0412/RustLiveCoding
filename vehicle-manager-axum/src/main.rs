use axum::{
    Json, Router, routing::{get, post}
};
use serde::{Deserialize,Serialize};

#[derive(Deserialize)]
struct VehicleData {
    brand: String,
    years: u32,
    model: String,
}

#[derive(Serialize)]
struct ResponseMessage {
    status: String,
    message: String,
}
#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
    .route("/ok", get(hello_world))
    .route("/vehicle_data", post(vehicle_data));
  
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> Json<ResponseMessage> {
    Json(ResponseMessage { status: "success".to_string(), message: "hello world".to_string()})
}

async fn vehicle_data(Json(payload): Json<VehicleData>) -> Json<ResponseMessage> 
{
    Json(ResponseMessage { status: "success".to_string(), message: format!("On a bien save le vehicule: {},{},{}",payload.brand,payload.model,payload.years)  })
}