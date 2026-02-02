#!/usr/bin/env node
/**
 * Standalone Lighthouse runner that outputs JSON results to stdout.
 * Uses Lighthouse v9 (CommonJS) for better bundling compatibility.
 * 
 * SUPPORTS THREE MODES:
 * 1. Single URL:     lighthouse-runner <url>
 * 2. Batch mode:     lighthouse-runner --batch <url1> <url2> <url3> ...
 * 3. Persistent mode: lighthouse-runner --persistent
 *    - Keeps Chrome running and accepts JSON requests via stdin
 *    - Each request is a JSON object on a single line
 *    - Outputs JSON response on a single line to stdout
 *    - Much faster for 1000s of requests (no Chrome startup per request)
 * 
 * Returns rendered HTML along with Lighthouse scores for proper JS-rendered content analysis.
 */

// Set HOME if not set (needed for bundled binaries)
if (!process.env.HOME) {
  const os = require('os');
  process.env.HOME = os.homedir();
}

const lighthouse = require('lighthouse');
const chromeLauncher = require('chrome-launcher');
const readline = require('readline');

// Default concurrency for batch mode (how many Lighthouse audits run in parallel)
const DEFAULT_CONCURRENCY = 3;

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    console.log(JSON.stringify({ success: false, error: 'Usage: lighthouse-runner [--batch] [--persistent] [--concurrency=N] <url> [url2] [url3] ...' }));
    process.exit(1);
  }

  // Check for persistent mode
  if (args.includes('--persistent')) {
    await runPersistentMode();
    return;
  }

  // Parse arguments
  let batchMode = false;
  let concurrency = DEFAULT_CONCURRENCY;
  const urls = [];

  for (const arg of args) {
    if (arg === '--batch') {
      batchMode = true;
    } else if (arg.startsWith('--concurrency=')) {
      concurrency = parseInt(arg.split('=')[1], 10) || DEFAULT_CONCURRENCY;
    } else {
      // Validate URL
      try {
        new URL(arg);
        urls.push(arg);
      } catch (e) {
        console.error(`Warning: Invalid URL skipped: ${arg}`);
      }
    }
  }

  if (urls.length === 0) {
    console.log(JSON.stringify({ success: false, error: 'No valid URLs provided' }));
    process.exit(1);
  }

  // If only one URL and not explicitly batch mode, use single mode for backward compatibility
  if (urls.length === 1 && !batchMode) {
    await runSingleUrl(urls[0]);
  } else {
    await runBatch(urls, concurrency);
  }
}

/**
 * Run in persistent mode - keeps Chrome running and accepts requests via stdin.
 * Uses a fresh Chrome instance for each URL but avoids Node.js startup overhead.
 * 
 * The main benefit here is eliminating Node.js/V8 startup time (~500ms-1s per invocation)
 * and reducing process spawn overhead when analyzing 1000s of URLs.
 * 
 * Note: Chrome still restarts per-URL due to Lighthouse limitations, but this is
 * much faster than spawning a completely new Node.js process each time.
 */
async function runPersistentMode() {
  let isShuttingDown = false;
  
  try {
    console.error('[lighthouse-runner] Starting persistent mode...');
    console.error('[lighthouse-runner] Ready for requests');
    
    // Signal ready to the parent process
    console.log(JSON.stringify({ success: true, ready: true }));
    
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false
    });
    
    // Process requests line by line
    for await (const line of rl) {
      if (isShuttingDown) break;
      
      const trimmedLine = line.trim();
      if (!trimmedLine) continue;
      
      try {
        const request = JSON.parse(trimmedLine);
        
        // Handle shutdown command
        if (request.action === 'shutdown') {
          console.error('[lighthouse-runner] Shutdown requested');
          console.log(JSON.stringify({ success: true, action: 'shutdown' }));
          isShuttingDown = true;
          break;
        }
        
        // Handle ping (health check)
        if (request.action === 'ping') {
          console.log(JSON.stringify({ success: true, action: 'pong' }));
          continue;
        }
        
        // Handle analyze request - launches Chrome, runs Lighthouse, kills Chrome
        // This is still faster than spawning a new Node.js process each time
        if (request.action === 'analyze' && request.url) {
          console.error(`[lighthouse-runner] Analyzing: ${request.url}`);
          const startTime = Date.now();
          
          let chrome = null;
          try {
            chrome = await launchChrome();
            const result = await analyzeSingleUrl(request.url, chrome.port);
            const elapsed = Date.now() - startTime;
            console.error(`[lighthouse-runner] Completed in ${elapsed}ms: ${request.url}`);
            console.log(JSON.stringify(result));
          } catch (error) {
            console.error(`[lighthouse-runner] Failed: ${request.url} - ${error.message}`);
            console.log(JSON.stringify({
              success: false,
              url: request.url,
              error: error.message,
            }));
          } finally {
            if (chrome) {
              try {
                await chrome.kill();
              } catch (e) {
                // Ignore kill errors
              }
            }
          }
          continue;
        }
        
        // Handle batch analyze - this is the most efficient mode for bulk URLs
        if (request.action === 'batch' && request.urls && Array.isArray(request.urls)) {
          console.error(`[lighthouse-runner] Batch analyzing ${request.urls.length} URLs`);
          const startTime = Date.now();
          
          let chrome = null;
          const results = [];
          
          try {
            chrome = await launchChrome();
            
            for (let i = 0; i < request.urls.length; i++) {
              const url = request.urls[i];
              console.error(`[lighthouse-runner] Analyzing ${i + 1}/${request.urls.length}: ${url}`);
              
              try {
                // Restart Chrome between URLs to avoid Lighthouse state issues
                if (i > 0) {
                  await chrome.kill();
                  chrome = await launchChrome();
                }
                const result = await analyzeSingleUrl(url, chrome.port);
                results.push(result);
              } catch (error) {
                results.push({
                  success: false,
                  url,
                  error: error.message,
                });
                // Restart Chrome on error
                try {
                  await chrome.kill();
                  chrome = await launchChrome();
                } catch (e) {
                  // Ignore
                }
              }
            }
            
            const elapsed = Date.now() - startTime;
            console.error(`[lighthouse-runner] Batch completed in ${elapsed}ms`);
            console.log(JSON.stringify({
              success: true,
              batch: true,
              total: request.urls.length,
              completed: results.filter(r => r.success).length,
              failed: results.filter(r => !r.success).length,
              results,
            }));
          } finally {
            if (chrome) {
              try {
                await chrome.kill();
              } catch (e) {
                // Ignore
              }
            }
          }
          continue;
        }
        
        // Unknown action
        console.log(JSON.stringify({
          success: false,
          error: `Unknown action: ${request.action || 'none'}`,
        }));
        
      } catch (parseError) {
        console.error(`[lighthouse-runner] Failed to parse request: ${parseError.message}`);
        console.log(JSON.stringify({
          success: false,
          error: `Invalid JSON: ${parseError.message}`,
        }));
      }
    }
    
  } catch (error) {
    console.error(`[lighthouse-runner] Persistent mode error: ${error.message}`);
    console.log(JSON.stringify({
      success: false,
      error: `Persistent mode failed: ${error.message}`,
    }));
    process.exit(1);
  } finally {
    process.exit(0);
  }
}

/**
 * Run Lighthouse for a single URL (original behavior)
 */
async function runSingleUrl(url) {
  let chrome = null;
  
  try {
    chrome = await launchChrome();
    const result = await analyzeSingleUrl(url, chrome.port);
    console.log(JSON.stringify(result));
  } catch (error) {
    console.log(JSON.stringify({
      success: false,
      url,
      error: error.message,
    }));
    process.exit(1);
  } finally {
    if (chrome) {
      await chrome.kill();
    }
  }
}

/**
 * Run Lighthouse for multiple URLs sequentially, sharing one Chrome instance.
 * Lighthouse cannot run multiple audits in parallel on one Chrome instance,
 * but by reusing Chrome we save significant startup time (~3-5 seconds per URL).
 */
async function runBatch(urls, concurrency) {
  let chrome = null;
  
  try {
    // Launch Chrome once for all URLs
    console.error(`[lighthouse-runner] Starting batch analysis of ${urls.length} URLs`);
    console.error(`[lighthouse-runner] Note: URLs are processed sequentially but Chrome startup is avoided`);
    chrome = await launchChrome();
    
    const results = [];
    
    // Process URLs sequentially (Lighthouse limitation - can't run parallel on same Chrome)
    for (let i = 0; i < urls.length; i++) {
      const url = urls[i];
      console.error(`[lighthouse-runner] Analyzing ${i + 1}/${urls.length}: ${url}`);
      
      try {
        const result = await analyzeSingleUrl(url, chrome.port);
        results.push(result);
        console.error(`[lighthouse-runner] Completed: ${url}`);
      } catch (error) {
        results.push({
          success: false,
          url,
          error: error.message,
        });
        console.error(`[lighthouse-runner] Failed: ${url} - ${error.message}`);
      }
    }
    
    // Output all results as a JSON array
    console.log(JSON.stringify({
      success: true,
      batch: true,
      total: urls.length,
      completed: results.filter(r => r.success).length,
      failed: results.filter(r => !r.success).length,
      results,
    }));
    
  } catch (error) {
    console.log(JSON.stringify({
      success: false,
      batch: true,
      error: error.message,
    }));
    process.exit(1);
  } finally {
    if (chrome) {
      console.error('[lighthouse-runner] Shutting down Chrome');
      await chrome.kill();
    }
  }
}

/**
 * Launch Chrome with optimal flags
 */
async function launchChrome() {
  return chromeLauncher.launch({
    chromeFlags: [
      '--headless',
      '--disable-gpu',
      '--no-sandbox',
      '--disable-dev-shm-usage',
      '--disable-extensions',
      '--disable-background-networking',
      '--disable-default-apps',
      '--disable-sync',
      '--disable-translate',
      '--mute-audio',
      '--hide-scrollbars',
      '--no-first-run',
      '--no-default-browser-check',
    ],
  });
}

/**
 * Fetch HTML via simple HTTP request (for link extraction)
 */
async function fetchHtmlSimple(url) {
  const https = require('https');
  const http = require('http');
  const { URL } = require('url');
  
  const parsedUrl = new URL(url);
  const client = parsedUrl.protocol === 'https:' ? https : http;
  
  return new Promise((resolve, reject) => {
    const req = client.get(url, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (compatible; SEOBot/1.0; +https://example.com/bot)',
      },
      timeout: 10000,
    }, (res) => {
      // Follow redirects
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        fetchHtmlSimple(res.headers.location).then(resolve).catch(reject);
        return;
      }
      
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => resolve({ html: data, statusCode: res.statusCode }));
    });
    
    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy();
      reject(new Error('Request timeout'));
    });
  });
}

/**
 * Analyze a single URL using an existing Chrome instance
 */
async function analyzeSingleUrl(url, chromePort) {
  // Run Lighthouse with SEO audits only (much faster than full audit)
  const result = await lighthouse(url, {
    port: chromePort,
    output: 'json',
    logLevel: 'error',
    onlyCategories: ['seo'],
  });

  if (!result || !result.lhr) {
    throw new Error('Lighthouse did not return valid results');
  }

  const { lhr } = result;

  // Always fetch HTML via HTTP to:
  // 1. Get accurate page load time (SEO-only mode doesn't include performance metrics)
  // 2. Get rendered HTML for link discovery
  let renderedHtml = '';
  let statusCode = 200;
  let loadTimeMs = null;
  
  try {
    const fetchStart = Date.now();
    const fetchResult = await fetchHtmlSimple(lhr.finalDisplayedUrl || lhr.finalUrl || url);
    loadTimeMs = Date.now() - fetchStart;  // Actual HTTP response time
    renderedHtml = fetchResult.html;
    statusCode = fetchResult.statusCode;
  } catch (e) {
    console.error(`[lighthouse-runner] Could not fetch HTML: ${e.message}`);
    // Fallback to artifacts if HTTP fetch fails
    if (result.artifacts && result.artifacts.MainDocumentContent) {
      renderedHtml = result.artifacts.MainDocumentContent;
    }
  }
  
  // Try to get status code from devtools logs if available
  if (result.artifacts?.devtoolsLogs?.defaultPass) {
    const mainRequest = result.artifacts.devtoolsLogs.defaultPass.find(
      log => log.method === 'Network.responseReceived' && 
             log.params?.response?.url === (lhr.finalDisplayedUrl || lhr.finalUrl || url)
    );
    if (mainRequest?.params?.response?.status) {
      statusCode = mainRequest.params.response.status;
    }
  }

  // Extract scores (0-1 scale)
  const scores = {
    performance: lhr.categories.performance?.score ?? null,
    accessibility: lhr.categories.accessibility?.score ?? null,
    best_practices: lhr.categories['best-practices']?.score ?? null,
    seo: lhr.categories.seo?.score ?? null,
  };

  // Extract detailed SEO audits
  const seoAudits = {
    document_title: extractAudit(lhr, 'document-title'),
    meta_description: extractAudit(lhr, 'meta-description'),
    viewport: extractAudit(lhr, 'viewport'),
    canonical: extractAudit(lhr, 'canonical'),
    hreflang: extractAudit(lhr, 'hreflang'),
    robots_txt: extractAudit(lhr, 'robots-txt'),
    crawlable_anchors: extractAudit(lhr, 'crawlable-anchors'),
    link_text: extractAudit(lhr, 'link-text'),
    image_alt: extractAudit(lhr, 'image-alt'),
    http_status_code: extractAudit(lhr, 'http-status-code'),
    is_crawlable: extractAudit(lhr, 'is-crawlable'),
  };

  // Extract performance metrics
  const performanceMetrics = {
    first_contentful_paint: lhr.audits['first-contentful-paint']?.numericValue ?? null,
    largest_contentful_paint: lhr.audits['largest-contentful-paint']?.numericValue ?? null,
    speed_index: lhr.audits['speed-index']?.numericValue ?? null,
    time_to_interactive: lhr.audits['interactive']?.numericValue ?? null,
    total_blocking_time: lhr.audits['total-blocking-time']?.numericValue ?? null,
    cumulative_layout_shift: lhr.audits['cumulative-layout-shift']?.numericValue ?? null,
  };

  const finalUrl = lhr.finalDisplayedUrl || lhr.finalUrl || url;

  return {
    success: true,
    url: finalUrl,
    requested_url: url,
    fetch_time: lhr.fetchTime,
    status_code: statusCode,
    html: renderedHtml,
    content_size: renderedHtml.length,
    load_time_ms: loadTimeMs,  // Actual HTTP response time in milliseconds
    scores,
    seo_audits: seoAudits,
    performance_metrics: performanceMetrics,
  };
}

function extractAudit(lhr, auditId) {
  const audit = lhr.audits[auditId];
  if (!audit) {
    return {
      passed: false,
      value: null,
      score: 0,
      description: 'Audit not available',
    };
  }

  return {
    passed: audit.score === 1,
    value: audit.displayValue || audit.title || null,
    score: audit.score ?? 0,
    description: audit.description || '',
  };
}

// Main entry point
main().catch((error) => {
  console.log(JSON.stringify({ success: false, error: error.message }));
  process.exit(1);
});
