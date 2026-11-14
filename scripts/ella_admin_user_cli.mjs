#!/usr/bin/env node
/**
 * ELLA World — Admin User Provisioning CLI (`scripts/ella_admin_user_cli.mjs`)
 * Creates or updates an in-game player account via `/api/ella-world/admin/users`
 * and verifies login + logout cleanly.
 */

import { execFileSync } from 'node:child_process';

process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';

const PRESET_AVATARS = {
  jack_tech49: {
    avatarMode: 'stylized',
    bodyType: 'male',
    characterPreset: 'jack_tech49',
    skinColor: '#eab692',
    eyeColor: '#38bdf8',
    hairStyle: 'tech_crop',
    hairColor: '#3b2516',
    topStyle: 'tech_jacket',
    topColor: '#e2e8f0',
    torsoColor: '#e2e8f0',
    bottomStyle: 'tech_pants',
    bottomColor: '#64748b',
    legsColor: '#64748b',
    shoeStyle: 'flight_boots',
    shoeColor: '#334155',
    hat: 'none',
    hatColor: '#334155',
    accessory: 'aviators',
    accessoryColor: '#38bdf8',
    fabricFinish: 'matte',
    face: 'hero'
  },
  maverick_cabin: {
    avatarMode: 'stylized',
    bodyType: 'male',
    characterPreset: 'maverick_cabin',
    skinColor: '#eab692',
    eyeColor: '#38bdf8',
    hairStyle: 'maverick_quiff',
    hairColor: '#3b2516',
    topStyle: 'hero_henley',
    topColor: '#cbd5e1',
    torsoColor: '#cbd5e1',
    bottomStyle: 'tech_pants',
    bottomColor: '#2c3e5a',
    legsColor: '#2c3e5a',
    shoeStyle: 'flight_boots',
    shoeColor: '#3f2e24',
    hat: 'recon_cap',
    hatColor: '#334155',
    accessory: 'aviators',
    accessoryColor: '#38bdf8',
    fabricFinish: 'matte',
    face: 'hero'
  }
};

function parseArgs(argv) {
  const out = {
    baseUrl: 'https://192.168.11.24:4444',
    sshHost: 'robert@mail',
    role: 'friend',
    preset: 'jack_tech49'
  };
  for (let i = 2; i < argv.length; i++) {
    const k = argv[i];
    const v = argv[i + 1];
    if (k === '--base-url') { out.baseUrl = v; i++; }
    else if (k === '--ssh-host') { out.sshHost = v; i++; }
    else if (k === '--admin-password') { out.adminPassword = v; i++; }
    else if (k === '--username') { out.username = v; i++; }
    else if (k === '--display-name') { out.displayName = v; i++; }
    else if (k === '--password') { out.password = v; i++; }
    else if (k === '--role') { out.role = v; i++; }
    else if (k === '--preset') { out.preset = v; i++; }
  }
  return out;
}

async function main() {
  const opts = parseArgs(process.argv);
  if (!opts.username || !opts.password) {
    console.error('Usage: node scripts/ella_admin_user_cli.mjs --username <u > --password <p> [--display-name <n>] [--preset jack_tech49]');
    process.exit(1);
  }

  let adminPassword = opts.adminPassword || process.env.ELLA_ADMIN_PASSWORD || '';
  if (!adminPassword) {
    const rawJson = execFileSync(
      'ssh',
      ['-o', 'BatchMode=yes', opts.sshHost, 'cat /var/lib/ella-world/world_users.json'],
      { encoding: 'utf8' }
    );
    adminPassword = JSON.parse(rawJson).adminPassword;
  }

  // 1. Authenticate to ELLA World Admin API
  const adminLoginRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ password: adminPassword })
  });
  if (!adminLoginRes.ok) {
    throw new Error(`Admin login failed: HTTP ${adminLoginRes.status}`);
  }
  const { adminToken } = await adminLoginRes.json();

  // 2. Upsert the requested in-game user + avatar
  const avatar = PRESET_AVATARS[opts.preset] || PRESET_AVATARS.jack_tech49;
  const upsertRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/users`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify({
      username: opts.username,
      displayName: opts.displayName || opts.username,
      password: opts.password,
      role: opts.role,
      avatar
    })
  });
  if (!upsertRes.ok) {
    throw new Error(`User upsert failed: HTTP ${upsertRes.status} ${await upsertRes.text()}`);
  }

  // 3. Verify player login works with exact credentials, then immediately logout so no ghost avatar remains
  const loginRes = await fetch(`${opts.baseUrl}/api/ella-world/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: opts.username, password: opts.password, clientId: 'verify_cli' })
  });
  if (!loginRes.ok) {
    throw new Error(`Player login verification failed: HTTP ${loginRes.status}`);
  }
  const loginData = await loginRes.json();
  await fetch(`${opts.baseUrl}/api/ella-world/logout`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token: loginData.token })
  });

  console.log(JSON.stringify({
    ok: true,
    username: loginData.self?.user?.username,
    displayName: loginData.self?.user?.displayName,
    bodyType: loginData.self?.user?.avatar?.bodyType,
    characterPreset: loginData.self?.user?.avatar?.characterPreset,
    topStyle: loginData.self?.user?.avatar?.topStyle,
    accessory: loginData.self?.user?.avatar?.accessory
  }, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
