# Repo Analyzer API (Rust + Cloudflare Workers)

This is a Rust implementation of the Repo Analyzer API service that handles file uploads to Cloudflare R2 storage, deployed as a Cloudflare Worker.

## Features

- Health check endpoint
- File upload to Cloudflare R2
- API key authentication
- CORS support
- Environment variable configuration
- WebAssembly (WASM) compilation for Cloudflare Workers

## Requirements

- Rust 1.70 or higher
- wasm-pack
- Cloudflare account with Workers and R2 enabled
- Wrangler CLI

## Development

### Setup

1. Clone the repository
2. Install wasm-pack: `cargo install wasm-pack`
3. Install Wrangler CLI: `npm install -g wrangler`
4. Login to Cloudflare: `wrangler login`
5. Build the project: `wasm-pack build --target web`
6. Run the development server: `wrangler dev`

### Environment Variables

- `PORT`: The port to run the server on (default: 3000)
- `AWS_REGION`: AWS region (default: eu-central-1)
- `AWS_S3_BUCKET`: S3 bucket name (default: repo-analyzer)
- `API_KEY`: API key for authentication
- `NODE_ENV`: Environment (development/production)
- `R2_DOMAIN`: Custom domain for R2 bucket (optional)

## API Endpoints

### Health Check

```
GET /health
```

Returns a 200 OK response with a JSON body: `{"status": "ok"}`.

### Upload File

```
POST /upload
```

Headers:
- `x-api-key`: API key for authentication (not required in development mode)

Form data:
- `file`: The file to upload

Query parameters (optional):
- `bucket`: S3 bucket name (overrides environment variable)
- `region`: AWS region (overrides environment variable)
- `key`: Custom S3 key for the file

Response:
```json
{
  "url": "https://bucket-name.r2.cloudflarestorage.com/key",
  "bucket": "bucket-name",
  "key": "key",
  "region": "region"
}
```

## Deployment

Deploy to Cloudflare Workers:

```bash
wrangler publish
```

For production deployment:

```bash
wrangler publish --env production
```

## License

MIT