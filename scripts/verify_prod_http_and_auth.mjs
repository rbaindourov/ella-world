#!/usr/bin/env node
/**
 * ELLA World — Production HTTP, Static Asset & Token-Revocation Verifier
 * (`scripts/verify_prod_http_and_auth.mjs`)
 *
 * Verifies against `ELLA_VERIFY_ORIGIN` (default `https://robertscreations.com:4444`):
 *   1. Root HTML (`/ella-world`) returns HTTP 200 OK and renders `#admin-revoke-all-btn`.
 *   2. All HTML `<link>`/`<script>` assets plus ES module imports (`ws_spine.js`, `voice.js`,
 *      `device_sig.js`, `world3d.js`) return HTTP 200 OK.
 *   3. Health endpoint (`/api/ella-world/health`) returns HTTP 200 OK.
 *   4. Stale / legacy session tokens presented to `POST /api/ella-world/resume` return
 *      HTTP 401 Unauthorized.
 *   5. Logout endpoint (`POST /api/ella-world/logout`) returns HTTP 200 OK and omits
 *      `Clear-Site-Data` so cryptographic machine seals survive sign-out.
 */

import https from 'node:https';
import { URL } from 'node:url';

const BASE_ORIGIN = process.env.ELLA_VERIFY_ORIGIN || 'https://robertscreations.com:4444';

function requestUrl(targetUrl, { method = 'GET', headers = {}, body = null } = {}) {
  return new Promise((resolve, reject) => {
    const parsed = new URL(targetUrl);
    const req = https.request(
      {
        protocol: parsed.protocol,
        hostname: parsed.hostname,
        port: parsed.port || 443,
        path: parsed.pathname + parsed.search,
        method,
        headers,
        rejectUnauthorized: false,
        timeout: 10000,
      },
      (res) => {
        const chunks = [];
        res.on('data', (c) => chunks.push(c));
        res.on('end', () => {
          const rawBuf = Buffer.concat(chunks);
          resolve({
            status: res.statusCode,
            headers: res.headers,
            rawBytes: rawBuf.length,
            body: rawBuf.toString('utf8'),
          });
        });
      }
    );
    req.on('error', reject);
    req.on('timeout', () => {
      req.destroy(new Error(`Timeout requesting ${targetUrl}`));
    });
    if (body) req.write(body);
    req.end();
  });
}

function extractAssetPaths(html) {
  const paths = new Set([
    'js/ws_spine.js',
    'js/voice.js',
    'js/device_sig.js',
    'js/world3d.js',
    'models/ella_dti_wardrobe.glb',
    'models/ella_world_props_loot.glb',
  ]);
  const linkRe = /<link[^>]+href=["']([^"']+\.css[^"']*)["']/gi;
  const scriptRe = /<script[^>]+src=["']([^"']+\.js[^"']*)["']/gi;
  let m;
  while ((m = linkRe.exec(html)) !== null) {
    if (!m[1].startsWith('http://') && !m[1].startsWith('https://')) {
      paths.add(m[1]);
    }
  }
  while ((m = scriptRe.exec(html)) !== null) {
    if (!m[1].startsWith('http://') && !m[1].startsWith('https://')) {
      paths.add(m[1]);
    }
  }
  return Array.from(paths);
}

async function main() {
  const report = {
    origin: BASE_ORIGIN,
    htmlStatus: 0,
    hasRevokeAllButtonInHtml: false,
    assets: [],
    healthStatus: 0,
    healthBody: null,
    staleResumeStatus: 0,
    logoutStatus: 0,
    logoutClearSiteData: null,
  };

  // 1. Verify root HTML
  const htmlUrl = `${BASE_ORIGIN}/ella-world`;
  const htmlRes = await requestUrl(htmlUrl);
  report.htmlStatus = htmlRes.status;
  report.hasRevokeAllButtonInHtml = htmlRes.body.includes('admin-revoke-all-btn');
  if (htmlRes.status !== 200) {
    throw new Error(`Root HTML ${htmlUrl} returned HTTP ${htmlRes.status}`);
  }
  if (!report.hasRevokeAllButtonInHtml) {
    throw new Error('Root HTML is missing #admin-revoke-all-btn');
  }

  // 2. Verify all local CSS, JS, and ES module assets return 200 OK
  const extracted = extractAssetPaths(htmlRes.body);
  for (const rel of extracted) {
    const resolved = new URL(rel, `${BASE_ORIGIN}/ella-world/`).toString();
    const assetRes = await requestUrl(resolved);
    report.assets.push({
      url: resolved,
      status: assetRes.status,
      bytes: assetRes.rawBytes,
    });
    if (assetRes.status !== 200) {
      throw new Error(`Static asset ${resolved} returned HTTP ${assetRes.status}`);
    }
  }

  // 3. Verify health endpoint
  const healthRes = await requestUrl(`${BASE_ORIGIN}/api/ella-world/health`);
  report.healthStatus = healthRes.status;
  report.healthBody = JSON.parse(healthRes.body);
  if (healthRes.status !== 200) {
    throw new Error(`/api/ella-world/health returned HTTP ${healthRes.status}`);
  }

  // 4. Verify stale/legacy token is rejected with 401 on /api/ella-world/resume
  const legacyPayload = Buffer.from(
    JSON.stringify({ selfId: 'peer_stale01', username: 'ella', role: 'host', iat: Date.now() })
  ).toString('base64url');
  const staleBody = JSON.stringify({ token: `${legacyPayload}.stale_invalid_sig` });
  const resumeRes = await requestUrl(`${BASE_ORIGIN}/api/ella-world/resume`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(staleBody),
    },
    body: staleBody,
  });
  report.staleResumeStatus = resumeRes.status;
  if (resumeRes.status !== 401) {
    throw new Error(`Expected HTTP 401 on stale /resume, got ${resumeRes.status}`);
  }

  // 5. Verify /api/ella-world/logout emits Clear-Site-Data
  const logoutPayload = JSON.stringify({ token: 'stale_or_empty' });
  const logoutRes = await requestUrl(`${BASE_ORIGIN}/api/ella-world/logout`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(logoutPayload),
    },
    body: logoutPayload,
  });
  report.logoutStatus = logoutRes.status;
  report.logoutClearSiteData = logoutRes.headers['clear-site-data'] || null;
  if (logoutRes.status !== 200) {
    throw new Error(`Expected HTTP 200 on /logout, got ${logoutRes.status}`);
  }
  if (report.logoutClearSiteData !== null) {
    throw new Error(
      `Expected /logout to preserve machine seals (no Clear-Site-Data), got ${report.logoutClearSiteData}`
    );
  }

  console.log(JSON.stringify(report, null, 2));
}

main().catch((err) => {
  console.error('[verify_prod_http_and_auth] FAILED:', err.message);
  process.exit(1);
});
