use std::fmt;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, ResponseError};
use tokio_postgres::{Client, NoTls, Error};
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for MyError {}

async fn connect() -> Result<Client, Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=password dbname=prueba1",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

// Obtener todas las categorías
async fn get_categories(db: web::Data<Arc<Mutex<Client>>>) -> Result<impl Responder, MyError> {
    let rows = db.lock().unwrap().query("SELECT id, name FROM categories", &[]).await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;
    let mut result = String::new();
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        result.push_str(&format!("id: {}, name: {}\n", id, name));
    }
    Ok(result)
}

// Crear una nueva categoría
#[derive(Debug, Deserialize)]
struct CategoryData {
    name: String,
}

async fn create_category(
    db: web::Data<Arc<Mutex<Client>>>,
    category_data: web::Json<CategoryData>,
) -> Result<impl Responder, MyError> {
    let name = &category_data.name;

    let client = db.lock().unwrap();

    client
        .execute(
            "INSERT INTO categories (name) VALUES ($1)",
            &[name],
        )
        .await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().body("Category created successfully"))
}

// Obtener todos los ingredientes
async fn get_ingredients(db: web::Data<Arc<Mutex<Client>>>) -> Result<impl Responder, MyError> {
    let rows = db.lock().unwrap().query("SELECT id, name FROM ingredients", &[]).await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;
    let mut result = String::new();
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        result.push_str(&format!("id: {}, name: {}\n", id, name));
    }
    Ok(result)
}

// Crear un nuevo ingrediente
#[derive(Debug, Deserialize)]
struct IngredientData {
    name: String,
}

async fn create_ingredient(
    db: web::Data<Arc<Mutex<Client>>>,
    ingredient_data: web::Json<IngredientData>,
) -> Result<impl Responder, MyError> {
    let name = &ingredient_data.name;

    let client = db.lock().unwrap();

    client
        .execute(
            "INSERT INTO ingredients (name) VALUES ($1)",
            &[name],
        )
        .await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().body("Ingredient created successfully"))
}

// Obtener todas las recetas
async fn get_recipes(db: web::Data<Arc<Mutex<Client>>>) -> Result<impl Responder, MyError> {
    let rows = db.lock().unwrap().query("SELECT id, name, description, category_id FROM recipes", &[]).await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;
    let mut result = String::new();
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let description: &str = row.get(2);
        let category_id: i32 = row.get(3);
        result.push_str(&format!("id: {}, name: {}, description: {}, category_id: {}\n", id, name, description, category_id));
    }
    Ok(result)
}

// Crear una nueva receta
#[derive(Debug, Deserialize)]
struct RecipeData {
    name: String,
    description: String,
    category_id: i32,
}

async fn create_recipe(
    db: web::Data<Arc<Mutex<Client>>>,
    recipe_data: web::Json<RecipeData>,
) -> Result<impl Responder, MyError> {
    let name = &recipe_data.name;
    let description = &recipe_data.description;
    let category_id = recipe_data.category_id;

    let client = db.lock().unwrap();

    client
        .execute(
            "INSERT INTO recipes (name, description, category_id) VALUES ($1, $2, $3)",
            &[name, description, &category_id],
        )
        .await
        .map_err(|e| MyError(format!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().body("Recipe created successfully"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Arc::new(Mutex::new(connect().await.unwrap_err()));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(
                web::resource("/categories")
                    .route(web::get().to(get_categories))
                    .route(web::post().to(create_category))
            )
            .service(
                web::resource("/ingredients")
                    .route(web::get().to(get_ingredients))
                    .route(web::post().to(create_ingredient))
            )
            .service(
                web::resource("/recipes")
                    .route(web::get().to(get_recipes))
                    .route(web::post().to(create_recipe))
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
