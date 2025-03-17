import { fetch } from '../build/repo_analyzer_api.js';

addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  try {
    return await fetch(request);
  } catch (e) {
    console.error('Error in worker:', e);
    return new Response(JSON.stringify({
      error: 'Internal Server Error',
      message: e.message
    }), {
      status: 500,
      headers: {
        'Content-Type': 'application/json'
      }
    });
  }
}