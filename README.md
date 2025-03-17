# Repo Analyzer API (Cloudflare Workers with R2 Storage)

This is the API service for Repo Analyzer that handles file uploads to Cloudflare R2 storage, deployed as a Cloudflare Worker.

## Features

- Health check endpoint
- File upload to Cloudflare R2
- Serving uploaded files
- API key authentication
- CORS support
- Environment variable configuration

## Requirements

- Cloudflare account with Workers and R2 enabled
- Wrangler CLI

## Development

### Setup

1. Clone the repository
2. Install Wrangler CLI: `npm install -g wrangler`
3. Login to Cloudflare: `wrangler login`
4. Create a local .env file based on .env.example
5. Run the development server: `wrangler dev`

### Environment Variables

- `AWS_REGION`: Region identifier (default: eu-central-1)
- `AWS_S3_BUCKET`: R2 bucket name (default: repo-analyzer)
- `API_KEY`: API key for authentication
- `NODE_ENV`: Environment (development/production)

## API Endpoints

### Root Endpoint

```
GET /
```

Returns a welcome message with API version information.

### Upload Report

```
POST /api/upload
```

Headers:
- `x-api-key`: API key for authentication (not required in development mode)

Form data:
- `file`: The file to upload
- `key`: (Optional) Custom key for the file, defaults to a timestamped filename in the reports directory

Response:
```json
{
  "status": "success",
  "url": "https://api.analyzer.gokhanozturk.io/reports/filename.json",
  "key": "reports/filename.json",
  "message": "File uploaded successfully"
}
```

### Serve File

```
GET /reports/{filename}
```

Serves the file directly from R2 storage with appropriate content type headers.

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