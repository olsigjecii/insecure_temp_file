use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Deserialize)]
struct UserData {
    content: String,
}

// VULNERABLE SERVICE
// Creates a temporary file in a predictable location (/tmp/sensitive_data.csv)
// with default (often insecure) permissions.
#[post("/vulnerable_path")]
async fn vulnerable_service(user_data: web::Json<UserData>) -> impl Responder {
    let file_path = "/tmp/sensitive_data.csv";
    match File::create(file_path) {
        Ok(mut file) => {
            if file.write_all(user_data.content.as_bytes()).is_err() {
                return HttpResponse::InternalServerError()
                    .body("Failed to write to temporary file");
            }

            // In a real-world scenario, some processing would happen here.
            // We simulate this with a short delay to create a window for an attack.
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Clean up the file
            let _ = fs::remove_file(file_path);

            HttpResponse::Ok().body("Data processed (vulnerable) 脆弱")
        }
        Err(_) => HttpResponse::InternalServerError().body("Could not create temporary file"),
    }
}

// SECURE SERVICE
// Uses the `tempfile` crate to create a temporary file with a random name and
// secure permissions (0o600 on Unix) by default.
#[post("/secure_path")]
async fn secure_service(user_data: web::Json<UserData>) -> impl Responder {
    match NamedTempFile::new() {
        Ok(mut temp_file) => {
            if temp_file.write_all(user_data.content.as_bytes()).is_err() {
                return HttpResponse::InternalServerError()
                    .body("Failed to write to secure temporary file");
            }

            // The `tempfile` crate automatically creates the file with restrictive permissions.
            // No need to manually set them.

            // Simulate processing time.
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // The temporary file is automatically deleted when `temp_file` goes out of scope.
            // This is a key security feature of the RAII pattern in Rust.

            HttpResponse::Ok().body("Data processed (secure) 安全")
        }
        Err(_) => {
            HttpResponse::InternalServerError().body("Could not create secure temporary file")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Server starting at http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .service(vulnerable_service)
            .service(secure_service)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
