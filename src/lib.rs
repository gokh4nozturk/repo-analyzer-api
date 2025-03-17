use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, Response, ResponseInit};

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    version: String,
    status: String,
}

#[derive(Serialize)]
struct AnalysisResponse {
    status: String,
    job_id: String,
    message: String,
}

#[derive(Deserialize)]
struct AnalysisRequest {
    repo_url: String,
    branch: Option<String>,
}

#[wasm_bindgen]
pub struct RepoAnalyzerApi {}

#[wasm_bindgen]
impl RepoAnalyzerApi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

#[wasm_bindgen]
pub async fn fetch(request: Request) -> Result<Response, JsValue> {
    // Log request
    web_sys::console::log_1(&"Request received".into());

    // Parse URL
    let url = request.url();
    web_sys::console::log_1(&format!("URL: {}", url).into());

    // Extract path from URL using string operations
    let path = url.split('?').next().unwrap_or("/");
    let path = path.split('#').next().unwrap_or("/");

    // Remove domain part if present
    let path = if path.contains("://") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 3 {
            format!("/{}", parts[3..].join("/"))
        } else {
            "/".to_string()
        }
    } else {
        // For relative paths like "/api/analyze"
        if !path.starts_with('/') {
            format!("/{}", path)
        } else {
            path.to_string()
        }
    };

    web_sys::console::log_1(&format!("Path: {}", path).into());

    // Route request
    match path.as_str() {
        "/api/analyze" => handle_analyze_request(request).await,
        "/api/status" => handle_status_request(request).await,
        "/" | "" => handle_default_request().await,
        _ => handle_default_request().await,
    }
}

async fn handle_analyze_request(request: Request) -> Result<Response, JsValue> {
    // Check if it's a POST request
    if request.method() != "POST" {
        return create_error_response(405, "Method not allowed. Use POST.").await;
    }

    // Parse request body
    let promise = request.text()?;
    let body_jsvalue = JsFuture::from(promise).await?;
    let body = body_jsvalue
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to convert request body to string"))?;

    // Parse JSON
    let analysis_request: AnalysisRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(_) => return create_error_response(400, "Invalid JSON format").await,
    };

    // Validate repo URL
    if analysis_request.repo_url.is_empty() {
        return create_error_response(400, "Repository URL is required").await;
    }

    // Generate job ID
    let job_id = Uuid::new_v4().to_string();

    // In a real implementation, you would:
    // 1. Queue the analysis job
    // 2. Store the job status
    // 3. Return the job ID to the client

    web_sys::console::log_1(
        &format!("Starting analysis for repo: {}", analysis_request.repo_url).into(),
    );

    // Create response
    let response = AnalysisResponse {
        status: "queued".to_string(),
        job_id,
        message: format!("Analysis for {} has been queued", analysis_request.repo_url),
    };

    create_json_response(202, &response).await
}

async fn handle_status_request(request: Request) -> Result<Response, JsValue> {
    // Extract job_id from URL query parameters using string operations
    let url = request.url();
    let job_id = if let Some(query) = url.split('?').nth(1) {
        if let Some(job_id_param) = query.split('&').find(|param| param.starts_with("job_id=")) {
            job_id_param.split('=').nth(1).unwrap_or("").to_string()
        } else {
            return create_error_response(400, "Missing job_id parameter").await;
        }
    } else {
        return create_error_response(400, "Missing job_id parameter").await;
    };

    // In a real implementation, you would:
    // 1. Look up the job status
    // 2. Return the status to the client

    web_sys::console::log_1(&format!("Checking status for job: {}", job_id).into());

    // Mock response for now
    let response = serde_json::json!({
        "status": "in_progress",
        "job_id": job_id,
        "progress": 50,
        "message": "Analysis in progress"
    });

    create_json_response(200, &response).await
}

async fn handle_default_request() -> Result<Response, JsValue> {
    // Create response data
    let data = ApiResponse {
        message: "Welcome to Repo Analyzer API!".to_string(),
        version: "0.1.0".to_string(),
        status: "success".to_string(),
    };

    create_json_response(200, &data).await
}

async fn create_json_response<T: Serialize>(status: u16, data: &T) -> Result<Response, JsValue> {
    // Create response headers
    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;

    let json = serde_json::to_string(data).unwrap();

    // Return response
    let init = ResponseInit::new();
    init.set_status(status);
    init.set_headers(&headers);

    Response::new_with_opt_str_and_init(Some(&json), &init)
}

async fn create_error_response(status: u16, message: &str) -> Result<Response, JsValue> {
    let error_response = serde_json::json!({
        "status": "error",
        "message": message
    });

    create_json_response(status, &error_response).await
}
