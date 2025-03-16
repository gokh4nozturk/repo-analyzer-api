use chrono::Utc;
use js_sys::{Array, Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{console, Request, Response, ResponseInit};

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

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[wasm_bindgen]
pub struct Repo_Analyzer_Api {}

#[wasm_bindgen]
impl Repo_Analyzer_Api {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"Repo Analyzer API initialized".into());
        Self {}
    }

    #[wasm_bindgen]
    pub fn handle_request(&self, request: Request, env: JsValue) -> Promise {
        let request_clone = request.clone();
        let url = web_sys::Url::new(&request.url()).unwrap();
        let path = url.pathname();
        let method = request.method();

        // Health check endpoint
        if path == "/health" && method == "GET" {
            let mut init = ResponseInit::new();
            init.status(200);
            let headers = Object::new();
            Reflect::set(&headers, &"Content-Type".into(), &"application/json".into()).unwrap();
            init.headers(&headers);

            let response = HealthResponse {
                status: "ok".to_string(),
            };
            let json = serde_json::to_string(&response).unwrap();

            return future_to_promise(async move {
                let response = Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
                Ok(response.into())
            });
        }

        // Upload endpoint
        if path == "/upload" && method == "POST" {
            return future_to_promise(async move {
                // Check authentication
                let node_env =
                    js_sys::Reflect::get(&env, &"NODE_ENV".into()).unwrap_or("production".into());
                let node_env_str = node_env
                    .as_string()
                    .unwrap_or_else(|| "production".to_string());

                if node_env_str != "development" {
                    let headers = request_clone.headers();
                    let api_key_js = js_sys::Reflect::get(&headers, &"x-api-key".into())
                        .unwrap_or(JsValue::NULL);
                    let env_api_key =
                        js_sys::Reflect::get(&env, &"API_KEY".into()).unwrap_or(JsValue::NULL);

                    if api_key_js == JsValue::NULL || api_key_js != env_api_key {
                        let mut init = ResponseInit::new();
                        init.status(401);
                        let headers = Object::new();
                        Reflect::set(&headers, &"Content-Type".into(), &"application/json".into())
                            .unwrap();
                        init.headers(&headers);

                        let error = ErrorResponse {
                            error: "Unauthorized".to_string(),
                        };
                        let json = serde_json::to_string(&error).unwrap();
                        let response =
                            Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
                        return Ok(response.into());
                    }
                }

                // Process form data
                let form_data_promise = request_clone.form_data();
                let form_data = wasm_bindgen_futures::JsFuture::from(form_data_promise)
                    .await
                    .unwrap();
                let form_data: web_sys::FormData = form_data.into();

                let file = form_data.get("file");
                if file == JsValue::NULL {
                    let mut init = ResponseInit::new();
                    init.status(400);
                    let headers = Object::new();
                    Reflect::set(&headers, &"Content-Type".into(), &"application/json".into())
                        .unwrap();
                    init.headers(&headers);

                    let error = ErrorResponse {
                        error: "No file uploaded".to_string(),
                    };
                    let json = serde_json::to_string(&error).unwrap();
                    let response = Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
                    return Ok(response.into());
                }

                // Get parameters
                let search_params = url.search_params();
                let bucket_param = search_params.get("bucket");
                let region_param = search_params.get("region");
                let key_param = search_params.get("key");

                let env_bucket = js_sys::Reflect::get(&env, &"AWS_S3_BUCKET".into())
                    .unwrap_or("repo-analyzer".into());
                let env_region = js_sys::Reflect::get(&env, &"AWS_REGION".into())
                    .unwrap_or("eu-central-1".into());

                let bucket = bucket_param.unwrap_or_else(|| {
                    env_bucket
                        .as_string()
                        .unwrap_or_else(|| "repo-analyzer".to_string())
                });
                let region = region_param.unwrap_or_else(|| {
                    env_region
                        .as_string()
                        .unwrap_or_else(|| "eu-central-1".to_string())
                });

                // Generate a key if not provided
                let key = if let Some(k) = key_param {
                    k
                } else {
                    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
                    let uuid_part = Uuid::new_v4()
                        .to_string()
                        .chars()
                        .take(8)
                        .collect::<String>();
                    let file_obj: web_sys::File = file.dyn_into().unwrap();
                    let filename = file_obj.name();
                    format!("reports/{}-{}-{}", timestamp, uuid_part, filename)
                };

                console::log_1(&format!("Uploading file to R2: {}/{}", bucket, key).into());

                // Upload to R2
                let storage = js_sys::Reflect::get(&env, &"STORAGE".into()).unwrap();
                let put_method = js_sys::Reflect::get(&storage, &"put".into()).unwrap();
                let put_function = js_sys::Function::from(put_method);

                let metadata = Object::new();
                let http_metadata = Object::new();

                let file_obj: web_sys::File = file.dyn_into().unwrap();
                let content_type = file_obj.type_();
                let content_type = if content_type.is_empty() {
                    "application/octet-stream".to_string()
                } else {
                    content_type
                };

                Reflect::set(&http_metadata, &"contentType".into(), &content_type.into()).unwrap();
                Reflect::set(&metadata, &"httpMetadata".into(), &http_metadata).unwrap();

                let args = Array::new();
                args.push(&key.into());
                args.push(&file);
                args.push(&metadata);

                let put_promise = put_function.apply(&storage, &args).unwrap();
                wasm_bindgen_futures::JsFuture::from(put_promise)
                    .await
                    .unwrap();

                // Generate public URL
                let r2_domain_js =
                    js_sys::Reflect::get(&env, &"R2_DOMAIN".into()).unwrap_or(JsValue::NULL);
                let r2_domain = if r2_domain_js == JsValue::NULL {
                    format!("{}.{}.r2.cloudflarestorage.com", bucket, region)
                } else {
                    r2_domain_js.as_string().unwrap()
                };

                let url = format!("https://{}/{}", r2_domain, key);
                console::log_1(&format!("File uploaded successfully: {}", url).into());

                // Return the URL
                let mut init = ResponseInit::new();
                init.status(200);
                let headers = Object::new();
                Reflect::set(&headers, &"Content-Type".into(), &"application/json".into()).unwrap();
                init.headers(&headers);

                let response = UploadResponse {
                    url,
                    bucket,
                    key,
                    region,
                };
                let json = serde_json::to_string(&response).unwrap();
                let response = Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
                Ok(response.into())
            });
        }

        // Not found
        future_to_promise(async move {
            let mut init = ResponseInit::new();
            init.status(404);
            let headers = Object::new();
            Reflect::set(&headers, &"Content-Type".into(), &"application/json".into()).unwrap();
            init.headers(&headers);

            let error = ErrorResponse {
                error: "Not found".to_string(),
            };
            let json = serde_json::to_string(&error).unwrap();
            let response = Response::new_with_opt_str_and_init(Some(&json), &init).unwrap();
            Ok(response.into())
        })
    }
}
