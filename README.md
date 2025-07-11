
# Insecure Temporary Files in Rust with Actix-Web

This project demonstrates the **insecure temporary file vulnerability** and its mitigation in a Rust web application built with the `actix-web` framework.

[cite\_start]An insecure temporary file is a vulnerability that occurs when an application creates temporary files in a way that allows them to be accessed or modified by other users on the system[cite: 6]. [cite\_start]This typically happens when files are created with overly permissive access rights or in a predictable, well-known location[cite: 7].

This lesson will guide you through exploiting this vulnerability in a sample application and then show you how to fix it using secure coding practices in Rust.

## The Demonstration Application 🧑‍💻

The application provides two API services:

  * `/vulnerable_path`: A service that insecurely writes data to a predictable file path (`/tmp/sensitive_data.csv`).
  * `/secure_path`: A service that securely handles temporary data using Rust's `tempfile` crate.

### 1\. Project Setup

First, ensure you have Rust and `cargo` installed on your system.

Create a new Rust project:

```bash
cargo new rust-insecure-temp-file-lesson
cd rust-insecure-temp-file-lesson
```

### 2\. Dependencies

Add the required dependencies to your `Cargo.toml` file.

**`Cargo.toml`**

```toml
[dependencies]
actix-web = "4"
serde = { version = "1.0", features = ["derive"] }
tempfile = "3"
tokio = { version = "1", features = ["full"] }
```

### 3\. Application Code

Replace the content of `src/main.rs` with the following code. Note that we now use `.service()` instead of `.route()` to register our handlers, which is a common and clean way to structure `actix-web` applications.

**`src/main.rs`**

```rust
use actix_web::{web, App, HttpResponse, HttpServer, Responder, post};
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
                return HttpResponse::InternalServerError().body("Failed to write to temporary file");
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
                return HttpResponse::InternalServerError().body("Failed to write to secure temporary file");
            }
            
            // The `tempfile` crate automatically creates the file with restrictive permissions.
            // No need to manually set them.

            // Simulate processing time.
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // The temporary file is automatically deleted when `temp_file` goes out of scope.
            // This is a key security feature of the RAII pattern in Rust.

            HttpResponse::Ok().body("Data processed (secure) 安全")
        }
        Err(_) => HttpResponse::InternalServerError().body("Could not create secure temporary file"),
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
```

-----

## Running and Testing the Application ⚙️

### 1\. Run the Server

Launch the application with `cargo`:

```bash
cargo run
```

### 2\. Exploit the Vulnerable Service 😈

The `vulnerable_service` creates a race condition. An attacker can read the temporary file between its creation and deletion.

**In a new terminal**, start watching the predictable file location:

```bash
# This command will print the file's content the moment it appears
watch -n 0.1 'cat /tmp/sensitive_data.csv 2>/dev/null'
```

**In a third terminal**, send a POST request with mock sensitive data to the vulnerable service:

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{"content":"user,password123,secret_token"}' \
http://127.0.0.1:8080/vulnerable_path
```

You will see the `watch` command's output briefly flash the sensitive data: `user,password123,secret_token`. [cite\_start]This confirms the vulnerability; an attacker on the same machine could steal this data[cite: 9].

### 3\. Verify the Secure Service ✅

The `secure_service` mitigates this risk by using the `tempfile` crate, which creates files with **random names** and **secure permissions**.

Send a request to the secure service:

```bash
curl -X POST -H "Content-Type: application/json" \
-d '{"content":"user,password123,secret_token"}' \
http://127.0.0.1:8080/secure_path
```

The request will succeed, but your `watch` command will show nothing. The temporary file is created securely and cannot be easily found or accessed by other users.

-----

## Key Takeaways and Mitigation

  * **Avoid Predictable File Paths**: Never use hardcoded or easily guessable names for temporary files.
  * **Use Secure Crates**: Leverage well-vetted libraries like **`tempfile`** in Rust. They are designed to handle temporary files securely by default.
  * [cite\_start]**Enforce Restrictive Permissions**: If you must create files manually, always set the most restrictive file permissions possible (e.g., `chmod 600` on Unix), allowing access only to the file owner[cite: 68, 69].
  * **Minimize File Lifetime**: Ensure temporary files are deleted immediately after use. The RAII (Resource Acquisition Is Initialization) pattern in Rust, used by `tempfile`, makes this cleanup automatic and reliable.