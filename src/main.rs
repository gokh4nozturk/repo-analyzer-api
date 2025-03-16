use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use dotenv::dotenv;
use futures::{StreamExt, TryStreamExt};
use log::{error, info};
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::str::FromStr;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct UploadResponse {
    url: String,
    bucket: String,
    key: String,
    region: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct UploadParams {
    bucket: Option<String>,
    region: Option<String>,
    key: Option<String>,
}

// Simple API key authentication middleware
async fn authenticate(
    req: HttpRequest,
    api_key: web::Data<Option<String>>,
    node_env: web::Data<String>,
) -> Result<(), HttpResponse> {
    // Skip authentication in development mode
    if node_env.as_str() == "development" {
        return Ok(());
    }

    let header_api_key = req.headers().get("x-api-key");

    match (header_api_key, &*api_key) {
        (Some(header_value), Some(expected_key)) => {
            if let Ok(header_str) = header_value.to_str() {
                if header_str == expected_key {
                    return Ok(());
                }
            }
            Err(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"})))
        }
        _ => Err(HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))),
    }
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

// Upload endpoint
async fn upload_file(
    req: HttpRequest,
    mut payload: Multipart,
    s3_client: web::Data<S3Client>,
    api_key: web::Data<Option<String>>,
    node_env: web::Data<String>,
    default_bucket: web::Data<String>,
    default_region: web::Data<String>,
) -> Result<HttpResponse, Error> {
    // Authenticate request
    if let Err(response) = authenticate(req, api_key, node_env).await {
        return Ok(response);
    }

    // Extract query parameters
    let query = web::Query::<UploadParams>::from_query(req.query_string()).unwrap_or_default();
    let bucket = query
        .bucket
        .clone()
        .unwrap_or_else(|| default_bucket.to_string());
    let region_str = query
        .region
        .clone()
        .unwrap_or_else(|| default_region.to_string());

    let mut file_data: Option<(NamedTempFile, String, String)> = None;

    // Process multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.get_name() {
            if name == "file" {
                let filename = content_disposition
                    .get_filename()
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| {
                        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
                        format!("report-{}", timestamp)
                    });

                let content_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let mut temp_file = NamedTempFile::new()?;

                // Read file data
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    temp_file.write_all(&data)?;
                }

                file_data = Some((temp_file, filename, content_type));
            }
        }
    }

    // Check if file was uploaded
    let (temp_file, filename, content_type) = match file_data {
        Some(data) => data,
        None => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "No file uploaded"
            })));
        }
    };

    // Generate a key if not provided
    let key = match query.key {
        Some(k) => k,
        None => {
            let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
            let uuid_part = Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>();
            format!("reports/{}-{}-{}", timestamp, uuid_part, filename)
        }
    };

    info!("Uploading file to S3: {}/{}", bucket, key);

    // Read the file
    let file_path = temp_file.path();
    let file_content = std::fs::read(file_path)?;

    // Set up S3 upload parameters
    let put_request = PutObjectRequest {
        bucket: bucket.clone(),
        key: key.clone(),
        body: Some(file_content.into()),
        content_type: Some(content_type),
        acl: Some("public-read".to_string()),
        ..Default::default()
    };

    // Upload to S3
    match s3_client.put_object(put_request).await {
        Ok(_) => {
            // Construct the URL
            let url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region_str, key);
            info!("File uploaded successfully: {}", url);

            // Return the URL
            Ok(HttpResponse::Ok().json(UploadResponse {
                url,
                bucket,
                key,
                region: region_str,
            }))
        }
        Err(e) => {
            error!("Error uploading file: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to upload file: {}", e)
            })))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Get configuration from environment
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let aws_access_key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set");
    let aws_secret_key =
        env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set");
    let aws_region = env::var("AWS_REGION").unwrap_or_else(|_| "eu-central-1".to_string());
    let aws_s3_bucket = env::var("AWS_S3_BUCKET").unwrap_or_else(|_| "repo-analyzer".to_string());
    let api_key = env::var("API_KEY").ok();
    let node_env = env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string());

    // Parse region
    let region = Region::from_str(&aws_region).unwrap_or(Region::EuCentral1);

    // Create S3 client
    let credentials_provider = StaticProvider::new_minimal(aws_access_key, aws_secret_key);
    let s3_client = S3Client::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        credentials_provider,
        region,
    );

    info!("Repo Analyzer API running on port {}", port);
    info!("Upload endpoint: http://localhost:{}/upload", port);

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(s3_client.clone()))
            .app_data(web::Data::new(api_key.clone()))
            .app_data(web::Data::new(node_env.clone()))
            .app_data(web::Data::new(aws_s3_bucket.clone()))
            .app_data(web::Data::new(aws_region.clone()))
            .route("/health", web::get().to(health_check))
            .route("/upload", web::post().to(upload_file))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
