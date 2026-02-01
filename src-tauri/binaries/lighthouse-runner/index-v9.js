#!/usr/bin/env node
/**
 * Standalone Lighthouse runner that outputs JSON results to stdout.
 * Uses Lighthouse v9 (CommonJS) for better bundling compatibility.
 * Directly spawns Chrome and connects Lighthouse to it.
 */

// Set HOME if not set (needed for bundled binaries)
if (!process.env.HOME) {
  const os = require('os');
  process.env.HOME = os.homedir();
}

const lighthouse = require('lighthouse');
const chromeLauncher = require('chrome-launcher');

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    console.log(JSON.stringify({ success: false, error: 'Usage: lighthouse-runner <url>' }));
    process.exit(1);
  }

  const url = args[0];

  // Validate URL
  try {
    new URL(url);
  } catch (e) {
    console.log(JSON.stringify({ success: false, error: `Invalid URL: ${url}` }));
    process.exit(1);
  }

  await runLighthouse(url);
}

async function runLighthouse(url) {
  let chrome = null;
  
  try {
    // Launch Chrome using chrome-launcher (handles finding Chrome automatically)
    chrome = await chromeLauncher.launch({
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

    // Run Lighthouse
    const result = await lighthouse(url, {
      port: chrome.port,
      output: 'json',
      logLevel: 'error',
      onlyCategories: ['performance', 'accessibility', 'best-practices', 'seo'],
    });

    if (!result || !result.lhr) {
      throw new Error('Lighthouse did not return valid results');
    }

    const { lhr } = result;

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

    // Get the final URL
    const finalUrl = lhr.finalDisplayedUrl || lhr.finalUrl || url;

    // Output result
    const output = {
      success: true,
      url: finalUrl,
      requested_url: url,
      fetch_time: lhr.fetchTime,
      scores,
      seo_audits: seoAudits,
      performance_metrics: performanceMetrics,
    };

    console.log(JSON.stringify(output));
    
  } catch (error) {
    const output = {
      success: false,
      url,
      error: error.message,
    };
    console.log(JSON.stringify(output));
    process.exit(1);
  } finally {
    if (chrome) {
      await chrome.kill();
    }
  }
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
