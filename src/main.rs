use actix_web::{
    get,
    web::{Data, Json, Path},
    App, HttpResponse, HttpServer, Responder,
};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, StatementCache};
use serde::{Deserialize, Serialize};
use tokio_postgres::{self, NoTls};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    id: i32,
    name: String,
    email: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertUser {
    name: String,
    email: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    email: String,
    address: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut con = Config::new();
    con.dbname = Some("Demo".to_string());
    con.password = Some("sanjay".to_string());
    con.user = Some("postgres".to_string());
    con.host = Some("localhost".to_string());
    con.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool_manager = con.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool_manager.clone()))
            .service(home)
            .service(add_data)
            .service(select_data)
            .service(all_data)
            .service(delete_data)
            .service(update_data)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/insert")]
async fn add_data(state: Data<Pool>, val: Json<InsertUser>) -> impl Responder {
    let client = state.get().await.unwrap();
    match client
        .execute(
            "INSERT INTO user_info(name, email, address) VALUES ($1, $2, $3)",
            &[&val.name, &val.email, &val.address],
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Data Inserted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/user/{id}")]
async fn select_data(state: Data<Pool>, id: Path<i32>) -> impl Responder {
    let client = state.get().await.unwrap();
    let mut show = vec![];
    match client
        .query("SELECT * FROM user_info WHERE id = $1", &[&id.to_owned()])
        .await
    {
        Ok(val) => {
            for i in val {
                let users = UserData {
                    id: i.get(0),
                    name: i.get(1),
                    email: i.get(2),
                    address: i.get(3),
                };
                show.push(users);
            }
            HttpResponse::Ok().json(show)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/alluser")]
async fn all_data(state: Data<Pool>) -> impl Responder {
    let client = state.get().await.unwrap();
    let mut show = vec![];
    match client.query("SELECT * FROM user_info", &[]).await {
        Ok(val) => {
            for i in val {
                let users = UserData {
                    id: i.get(0),
                    name: i.get(1),
                    email: i.get(2),
                    address: i.get(3),
                };
                show.push(users);
            }
            HttpResponse::Ok().json(show)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/delete/{id}")]
async fn delete_data(state: Data<Pool>, id: Path<i32>) -> impl Responder {
    let client = state.get().await.unwrap();
    match client
        .execute("DELETE FROM user_info WHERE id = $1", &[&id.to_owned()])
        .await
    {
        Ok(_) => HttpResponse::Ok().body("User deleted."),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/update/{name}")]
async fn update_data(
    state: Data<Pool>,
    val: Json<UpdateUser>,
    name: Path<String>,
) -> impl Responder {
    let client = state.get().await.unwrap();
    match client
        .execute(
            "UPDATE user_info SET email = $1, address = $2 WHERE name = $3",
            &[&val.email, &val.address, &name.to_string()],
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Data Updated."),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
