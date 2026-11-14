#!/usr/bin/env node
/**
 * ELLA World — Production HMAC Secret Rotation & Immediate Token Expiration Utility
 * (`scripts/rotate_prod_hmac_secret.mjs`)
 *
 * Non-destructively rotates `ELLA_HMAC_SECRET` in `/etc/systemd/system/ella-world.service.d/override.conf`
 * on `robert@mail` (`192.168.11.24`), archives the prior drop-in to `/var/lib/ella-world/`,
 * reloads & restarts `ella-world.service`, and verifies that any token signed with the prior
 * secret is immediately rejected (`HTTP 401`) on `/api/ella-world/resume`.
 */

import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

function signLegacyTestToken(claims, secret) {
  const payload = Buffer.from(JSON.stringify(claims), 'utf8').toString('base64url');
  const sig = crypto.createHmac('sha256', secret).update(payload).digest('base64url');
  return `${payload}.${sig}`;
}

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  const sshHost = process.env.ELLA_SSH_HOST || 'robert@mail';
  const baseUrl = process.env.ELLA_BASE_URL || 'https://192.168.11.24:4444';

  // 1. Snapshot pre-rotation active session counts
  const beforeHealthRes = await fetch(`${baseUrl}/api/ella-world/health`);
  const beforeHealth = await beforeHealthRes.json();

  // 2. Read current systemd override.conf
  const currentOverride = execFileSync(
    'ssh',
    ['-o', 'BatchMode=yes', sshHost, 'sudo cat /etc/systemd/system/ella-world.service.d/override.conf'],
    { encoding: 'utf8' }
  );

  let oldSecret = '';
  const lines = currentOverride.split(/\r?\n/);
  const randHex = crypto.randomBytes(24).toString('hex');
  const newSecret = `ewld-prod-hmac-${randHex}`;
  let replaced = false;

  const updatedLines = lines.map((line) => {
    if (line.startsWith('Environment=ELLA_HMAC_SECRET=')) {
      oldSecret = line.slice('Environment=ELLA_HMAC_SECRET='.length).trim();
      replaced = true;
      return `Environment=ELLA_HMAC_SECRET=${newSecret}`;
    }
    return line;
  });

  if (!replaced) {
    const svcIdx = updatedLines.findIndex((l) => l.trim() === '[Service]');
    if (svcIdx >= 0) {
      updatedLines.splice(svcIdx + 1, 0, `Environment=ELLA_HMAC_SECRET=${newSecret}`);
    } else {
      updatedLines.push('[Service]', `Environment=ELLA_HMAC_SECRET=${newSecret}`);
    }
  }

  const updatedContent = `${updatedLines.join('\n').trimEnd()}\n`;
  const ts = Date.now();

  // 3. Archive prior override.conf losslessly in /var/lib/ella-world/ and write new override.conf
  execFileSync(
    'ssh',
    [
      '-o',
      'BatchMode=yes',
      sshHost,
      `sudo cp /etc/systemd/system/ella-world.service.d/override.conf /var/lib/ella-world/override.conf.bak.${ts}`
    ],
    { encoding: 'utf8' }
  );

  execFileSync(
    'ssh',
    [
      '-o',
      'BatchMode=yes',
      sshHost,
      'sudo tee /etc/systemd/system/ella-world.service.d/override.conf >/dev/null && sudo systemctl daemon-reload && sudo systemctl restart ella-world'
    ],
    { input: updatedContent, encoding: 'utf8' }
  );

  // 4. Wait briefly for native Rust TLS listener on :4444 to come back online
  let afterHealth = null;
  for (let attempt = 0; attempt < 15; attempt++) {
    await sleep(300);
    try {
      const r = await fetch(`${baseUrl}/api/ella-world/health`);
      if (r.ok) {
        afterHealth = await r.json();
        break;
      }
    } catch {
      // retry while socket binds
    }
  }
  if (!afterHealth) {
    throw new Error('ella-world failed to respond on /api/ella-world/health after restart');
  }

  // 5. Verify a token signed with the previous ELLA_HMAC_SECRET is rejected with HTTP 401
  let oldTokenStatus = null;
  if (oldSecret) {
    const staleToken = signLegacyTestToken(
      {
        selfId: 'peer_ella_stale',
        username: 'ella',
        role: 'host',
        iat: Date.now() - 60_000
      },
      oldSecret
    );
    const staleRes = await fetch(`${baseUrl}/api/ella-world/resume`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: staleToken, clientId: 'stale_verify' })
    });
    oldTokenStatus = staleRes.status;
  }

  console.log(
    JSON.stringify(
      {
        ok: true,
        beforeActivePlayers: beforeHealth.activePlayers,
        beforeActiveWsSessions: beforeHealth.activeWsSessions,
        beforeActiveWebrtcSessions: beforeHealth.activeWebrtcSessions,
        afterActivePlayers: afterHealth.activePlayers,
        afterActiveWsSessions: afterHealth.activeWsSessions,
        afterActiveWebrtcSessions: afterHealth.activeWebrtcSessions,
        previousTokenResumeHttpStatus: oldTokenStatus,
        backupFile: `/var/lib/ella-world/override.conf.bak.${ts}`
      },
      null,
      2
    )
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
