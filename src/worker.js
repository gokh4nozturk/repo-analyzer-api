import { Repo_Analyzer_Api } from '../build/repo-analyzer-api.js';

export default {
  async fetch(request, env, ctx) {
    // Initialize the Rust WASM module
    const instance = new Repo_Analyzer_Api();
    
    // Handle the request using the Rust module
    return instance.handle_request(request, env);
  }
};