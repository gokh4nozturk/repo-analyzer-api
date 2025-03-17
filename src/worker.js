// Simple worker implementation without WASM
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  try {
    // Parse URL manually to avoid URL constructor issues
    const url = request.url;
    const urlParts = url.split('?');
    const path = urlParts[0].split('/').slice(3).join('/');
    const fullPath = '/' + path;
    
    console.log(`Request received for path: ${fullPath}`);
    
    // Handle routes
    if (fullPath === '/' || fullPath === '') {
      return jsonResponse(200, {
        message: "Welcome to Repo Analyzer API!",
        version: "0.1.0",
        status: "success"
      });
    }
    
    if (fullPath === '/api/analyze') {
      if (request.method !== 'POST') {
        return jsonResponse(405, {
          status: "error",
          message: "Method not allowed. Use POST."
        });
      }
      
      // Mock response for analyze endpoint
      return jsonResponse(202, {
        status: "queued",
        job_id: "mock-job-id-" + Date.now(),
        message: "Analysis has been queued"
      });
    }
    
    if (fullPath === '/api/status') {
      // Parse query params manually
      let jobId = null;
      if (urlParts.length > 1) {
        const queryParams = urlParts[1].split('&');
        for (const param of queryParams) {
          const [key, value] = param.split('=');
          if (key === 'job_id') {
            jobId = value;
            break;
          }
        }
      }
      
      if (!jobId) {
        return jsonResponse(400, {
          status: "error",
          message: "Missing job_id parameter"
        });
      }
      
      // Mock response for status endpoint
      return jsonResponse(200, {
        status: "in_progress",
        job_id: jobId,
        progress: 50,
        message: "Analysis in progress"
      });
    }
    
    // Not found
    return jsonResponse(404, {
      status: "error",
      message: "Not found"
    });
  } catch (e) {
    console.error('Error in worker:', e);
    return jsonResponse(500, {
      error: 'Internal Server Error',
      message: e.message || 'Unknown error'
    });
  }
}

// Helper function to create JSON responses
function jsonResponse(status, data) {
  return new Response(JSON.stringify(data), {
    status: status,
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type'
    }
  });
}