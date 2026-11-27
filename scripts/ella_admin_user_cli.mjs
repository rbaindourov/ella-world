#!/usr/bin/env node
/**
 * ELLA World — Admin User, Parental Controls, Debtor's Prison & Session Revocation CLI (`scripts/ella_admin_user_cli.mjs`)
 * Supports:
 *   1. Global token expiration:   `node scripts/ella_admin_user_cli.mjs --revoke-all`
 *   2. Per-user token expiration: `node scripts/ella_admin_user_cli.mjs --revoke-user <username>`
 *   3. Roster & judicial status:  `node scripts/ella_admin_user_cli.mjs --list`
 *   4. Provision / rotate user:   `node scripts/ella_admin_user_cli.mjs --username <u> --password <p> [--display-name <n>] [--preset jack_tech49]`
 *   5. Parental & prison actions:
 *      - `node scripts/ella_admin_user_cli.mjs --username <u> --coins -50`
 *      - `node scripts/ella_admin_user_cli.mjs --username <u> --sentence 4 [--reason "..."]`
 *      - `node scripts/ella_admin_user_cli.mjs --username <u> --pardon`
 *      - `node scripts/ella_admin_user_cli.mjs --username <u> --reset-strikes`
 *      - `node scripts/ella_admin_user_cli.mjs --username <u> [--mute-voice true|false] [--chat-mode normal|quick_only|muted] [--can-build true|false] [--can-trade true|false] [--suspended true|false]`
 *   6. Machine digital signatures & anti-cheat evasion:
 *      - `node scripts/ella_admin_user_cli.mjs --devices`
 *      - `node scripts/ella_admin_user_cli.mjs --trust-device <deviceId|shortSig> <true|false>`
 *      - `node scripts/ella_admin_user_cli.mjs --ban-device <deviceId|shortSig> <true|false>`
 *      - `node scripts/ella_admin_user_cli.mjs --unlink-device <deviceId|shortSig>`
 *   7. Multi-Playground Friend Invites (`STAR-XXXX-XXXX`, scoped via `--world <slug>`):
 *      - `node scripts/ella_admin_user_cli.mjs [--world ella] --invites`
 *      - `node scripts/ella_admin_user_cli.mjs [--world ella] --create-friend-invite [--invite-label "For Leo"] [--max-uses 1] [--expires-hours 168] [--starter-coins 120]`
 *      - `node scripts/ella_admin_user_cli.mjs [--world ella] --approve-invite <STAR-XXXX-XXXX>`
 *      - `node scripts/ella_admin_user_cli.mjs [--world ella] --reject-invite <STAR-XXXX-XXXX>`
 *      - `node scripts/ella_admin_user_cli.mjs [--world ella] --revoke-invite <STAR-XXXX-XXXX>`
 *   8. Platform Super-Admin & Parent Founder Invites (`FAMILY-XXXX-XXXX`):
 *      - `node scripts/ella_admin_user_cli.mjs --playgrounds [--superadmin-password <pw>]`
 *      - `node scripts/ella_admin_user_cli.mjs --create-family-invite [--invite-label "Maya Family"] [--max-uses 1] [--expires-hours 336] [--superadmin-password <pw>]`
 *      - `node scripts/ella_admin_user_cli.mjs --revoke-family-invite <FAMILY-XXXX-XXXX> [--superadmin-password <pw>]`
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

function parseBoolFlag(raw) {
  if (raw === undefined || raw === null) return undefined;
  const norm = String(raw).trim().toLowerCase();
  if (norm === 'true' || norm === '1' || norm === 'yes' || norm === 'on') return true;
  if (norm === 'false' || norm === '0' || norm === 'no' || norm === 'off') return false;
  throw new Error(`Expected boolean true/false value, received "${raw}"`);
}

function parseArgs(argv) {
  const out = {
    baseUrl: 'https://192.168.11.24:4444',
    sshHost: 'robert@mail',
    worldSlug: '',
    superadminPassword: '',
    playgroundsOnly: false,
    createFamilyInvite: false,
    revokeFamilyInvite: '',
    invitesOnly: false,
    createFriendInvite: false,
    approveInvite: '',
    rejectInvite: '',
    revokeInvite: '',
    inviteLabel: '',
    maxUses: null,
    expiresHours: null,
    starterCoins: null,
    role: 'friend',
    preset: null,
    revokeAll: false,
    revokeUser: '',
    listOnly: false,
    devicesOnly: false,
    trustDeviceId: '',
    trustDeviceVal: undefined,
    banDeviceId: '',
    banDeviceVal: undefined,
    unlinkDeviceId: '',
    coinDelta: null,
    pardon: false,
    sentenceTasks: null,
    reason: '',
    resetStrikes: false,
    chatMode: null,
    muteVoice: undefined,
    canBuild: undefined,
    canTrade: undefined,
    suspended: undefined,
    setRole: null,
    giftItem: '',
    giftQty: 1,
    allowClone: undefined,
    skipLoginVerify: false
  };
  for (let i = 2; i < argv.length; i++) {
    const k = argv[i];
    const v = argv[i + 1];
    if (k === '--base-url') { out.baseUrl = v; i++; }
    else if (k === '--ssh-host') { out.sshHost = v; i++; }
    else if (k === '--world' || k === '--world-slug') { out.worldSlug = (v || '').trim().toLowerCase(); i++; }
    else if (k === '--admin-password') { out.adminPassword = v; i++; }
    else if (k === '--superadmin-password') { out.superadminPassword = v || ''; i++; }
    else if (k === '--playgrounds') { out.playgroundsOnly = true; }
    else if (k === '--create-family-invite') { out.createFamilyInvite = true; }
    else if (k === '--revoke-family-invite') { out.revokeFamilyInvite = (v || '').trim(); i++; }
    else if (k === '--invites') { out.invitesOnly = true; }
    else if (k === '--create-friend-invite') { out.createFriendInvite = true; }
    else if (k === '--approve-invite') { out.approveInvite = (v || '').trim(); i++; }
    else if (k === '--reject-invite') { out.rejectInvite = (v || '').trim(); i++; }
    else if (k === '--revoke-invite') { out.revokeInvite = (v || '').trim(); i++; }
    else if (k === '--invite-label') { out.inviteLabel = v || ''; i++; }
    else if (k === '--max-uses') { out.maxUses = Math.max(1, Math.floor(Number(v) || 1)); i++; }
    else if (k === '--expires-hours') { out.expiresHours = Math.max(0, Math.floor(Number(v) || 0)); i++; }
    else if (k === '--starter-coins') { out.starterCoins = Math.max(0, Math.floor(Number(v) || 0)); i++; }
    else if (k === '--username') { out.username = v; i++; }
    else if (k === '--display-name') { out.displayName = v; i++; }
    else if (k === '--password') { out.password = v; i++; }
    else if (k === '--role') { out.role = v; i++; }
    else if (k === '--preset') { out.preset = v; i++; }
    else if (k === '--revoke-all') { out.revokeAll = true; }
    else if (k === '--revoke-user') { out.revokeUser = v; i++; }
    else if (k === '--list') { out.listOnly = true; }
    else if (k === '--devices') { out.devicesOnly = true; }
    else if (k === '--trust-device') {
      out.trustDeviceId = v || '';
      out.trustDeviceVal = parseBoolFlag(argv[i + 2] ?? 'true');
      i += argv[i + 2] && !argv[i + 2].startsWith('--') ? 2 : 1;
    }
    else if (k === '--ban-device') {
      out.banDeviceId = v || '';
      out.banDeviceVal = parseBoolFlag(argv[i + 2] ?? 'true');
      i += argv[i + 2] && !argv[i + 2].startsWith('--') ? 2 : 1;
    }
    else if (k === '--unlink-device') { out.unlinkDeviceId = v || ''; i++; }
    else if (k === '--coins') { out.coinDelta = Number(v); i++; }
    else if (k === '--pardon') { out.pardon = true; }
    else if (k === '--sentence') {
      if (v && !v.startsWith('--') && Number.isFinite(Number(v))) {
        out.sentenceTasks = Math.max(1, Math.floor(Number(v)));
        i++;
      } else {
        out.sentenceTasks = 4;
      }
    }
    else if (k === '--reason') { out.reason = v || ''; i++; }
    else if (k === '--reset-strikes') { out.resetStrikes = true; }
    else if (k === '--chat-mode') { out.chatMode = v; i++; }
    else if (k === '--mute-voice') { out.muteVoice = parseBoolFlag(v); i++; }
    else if (k === '--can-build') { out.canBuild = parseBoolFlag(v); i++; }
    else if (k === '--can-trade') { out.canTrade = parseBoolFlag(v); i++; }
    else if (k === '--suspended') { out.suspended = parseBoolFlag(v); i++; }
    else if (k === '--set-role') { out.setRole = v; i++; }
    else if (k === '--gift-item') { out.giftItem = v || ''; i++; }
    else if (k === '--qty') { out.giftQty = Math.max(0, Math.floor(Number(v) || 1)); i++; }
    else if (k === '--allow-clone') { out.allowClone = parseBoolFlag(v); i++; }
    else if (k === '--skip-login-verify') { out.skipLoginVerify = true; }
  }
  return out;
}

async function postAdminModerate(baseUrl, adminToken, payload) {
  const res = await fetch(`${baseUrl}/api/ella-world/admin/moderate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  if (!res.ok) {
    throw new Error(`Admin moderation failed: HTTP ${res.status} ${await res.text()}`);
  }
  return res.json();
}

async function postAdminDevice(baseUrl, adminToken, payload) {
  const res = await fetch(`${baseUrl}/api/ella-world/admin/devices`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  if (!res.ok) {
    throw new Error(`Admin device management failed: HTTP ${res.status} ${await res.text()}`);
  }
  return res.json();
}

async function postAdminInvites(baseUrl, adminToken, payload) {
  const res = await fetch(`${baseUrl}/api/ella-world/admin/invites`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  if (!res.ok) {
    throw new Error(`Admin invites action failed: HTTP ${res.status} ${await res.text()}`);
  }
  return res.json();
}

function resolveDeviceIdFromDashboard(dashboard, queryId) {
  const q = String(queryId || '').trim();
  if (!q) return '';
  const list = Array.isArray(dashboard?.devices) ? dashboard.devices : [];
  const exact = list.find(
    (d) =>
      String(d.deviceId || '').toLowerCase() === q.toLowerCase() ||
      String(d.shortSig || '').toLowerCase() === q.toLowerCase()
  );
  return exact ? exact.deviceId : q;
}

function resolveSuperadminPasswordFromRemote(sshHost) {
  const rawOverride = execFileSync(
    'ssh',
    [
      '-o',
      'BatchMode=yes',
      sshHost,
      'sudo -n grep -h "^Environment=ELLA_SUPERADMIN_PASSWORD=" /etc/systemd/system/ella-world.service.d/*.conf /etc/systemd/system/ella-world.service 2>/dev/null | tail -n 1 || true'
    ],
    { encoding: 'utf8' }
  ).trim();
  if (rawOverride) {
    return rawOverride.replace(/^Environment=ELLA_SUPERADMIN_PASSWORD=/, '').replace(/^"|"$/g, '');
  }
  const rawJson = execFileSync(
    'ssh',
    ['-o', 'BatchMode=yes', sshHost, 'cat /var/lib/ella-world/world_users.json'],
    { encoding: 'utf8' }
  );
  return JSON.parse(rawJson).adminPassword;
}

async function main() {
  const opts = parseArgs(process.argv);
  const hasSuperadminAction =
    opts.playgroundsOnly || opts.createFamilyInvite || Boolean(opts.revokeFamilyInvite);
  const hasInviteAction =
    opts.invitesOnly ||
    opts.createFriendInvite ||
    Boolean(opts.approveInvite) ||
    Boolean(opts.rejectInvite) ||
    Boolean(opts.revokeInvite);
  const hasDeviceAction =
    Boolean(opts.trustDeviceId) || Boolean(opts.banDeviceId) || Boolean(opts.unlinkDeviceId);
  const hasControlUpdates =
    opts.chatMode !== null ||
    opts.muteVoice !== undefined ||
    opts.canBuild !== undefined ||
    opts.canTrade !== undefined ||
    opts.suspended !== undefined ||
    opts.setRole !== null;
  const hasModerationAction =
    Boolean(opts.username) &&
    (opts.pardon ||
      opts.sentenceTasks !== null ||
      opts.resetStrikes ||
      (opts.coinDelta !== null && Number.isFinite(opts.coinDelta) && opts.coinDelta !== 0) ||
      hasControlUpdates);

  const hasAction =
    hasSuperadminAction ||
    hasInviteAction ||
    opts.revokeAll ||
    Boolean(opts.revokeUser) ||
    opts.listOnly ||
    opts.devicesOnly ||
    hasDeviceAction ||
    hasModerationAction ||
    Boolean(opts.username && opts.giftItem) ||
    Boolean(opts.username && opts.password);

  if (!hasAction) {
    console.error(
      [
        'Usage:',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --revoke-all',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --revoke-user <username>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --list',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --invites',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --create-friend-invite [--invite-label "..."] [--max-uses 1] [--expires-hours 168] [--starter-coins 120]',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --approve-invite <STAR-XXXX-XXXX>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --reject-invite <STAR-XXXX-XXXX>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --revoke-invite <STAR-XXXX-XXXX>',
        '  node scripts/ella_admin_user_cli.mjs --playgrounds [--superadmin-password <pw>]',
        '  node scripts/ella_admin_user_cli.mjs --create-family-invite [--invite-label "..."] [--max-uses 1] [--expires-hours 336]',
        '  node scripts/ella_admin_user_cli.mjs --revoke-family-invite <FAMILY-XXXX-XXXX>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --devices',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --trust-device <deviceId|shortSig> <true|false>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --ban-device <deviceId|shortSig> <true|false>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --unlink-device <deviceId|shortSig>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> --password <p> [--display-name <n>] [--preset jack_tech49]',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> --coins <-50|+50>',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> --sentence [tasks] [--reason "..."]',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> --pardon',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> --reset-strikes',
        '  node scripts/ella_admin_user_cli.mjs [--world <slug>] --username <u> [--mute-voice true|false] [--chat-mode normal|quick_only|muted] [--can-build true|false] [--can-trade true|false] [--suspended true|false]'
      ].join('\n')
    );
    process.exit(1);
  }

  if (hasSuperadminAction) {
    let superPw = opts.superadminPassword || process.env.ELLA_SUPERADMIN_PASSWORD || '';
    if (!superPw) {
      superPw = resolveSuperadminPasswordFromRemote(opts.sshHost);
    }
    const superLoginRes = await fetch(`${opts.baseUrl}/api/ella-world/superadmin/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ password: superPw })
    });
    if (!superLoginRes.ok) {
      throw new Error(`Super-Admin login failed: HTTP ${superLoginRes.status} ${await superLoginRes.text()}`);
    }
    const { superadminToken, platform } = await superLoginRes.json();
    if (opts.playgroundsOnly) {
      console.log(JSON.stringify({ ok: true, platform }, null, 2));
      return;
    }
    const payload = opts.createFamilyInvite
      ? {
          action: 'create',
          labelNote: opts.inviteLabel || 'Family Playground Founder Invite',
          maxUses: opts.maxUses ?? 1,
          expiresInHours: opts.expiresHours ?? 336
        }
      : {
          action: 'revoke',
          code: opts.revokeFamilyInvite
        };
    const actRes = await fetch(`${opts.baseUrl}/api/ella-world/superadmin/invites`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${superadminToken}`
      },
      body: JSON.stringify(payload)
    });
    if (!actRes.ok) {
      throw new Error(`Super-Admin invite action failed: HTTP ${actRes.status} ${await actRes.text()}`);
    }
    const actOut = await actRes.json();
    const shareUrl = actOut.invite?.code
      ? `${opts.baseUrl.replace(/\/$/, '')}/?invite=${encodeURIComponent(actOut.invite.code)}`
      : null;
    console.log(
      JSON.stringify(
        {
          ok: true,
          invite: actOut.invite,
          shareUrl,
          playgroundCount: (actOut.platform?.playgrounds || []).length
        },
        null,
        2
      )
    );
    return;
  }

  const targetWorldSlug = opts.worldSlug || '';
  let adminPassword = opts.adminPassword || process.env.ELLA_ADMIN_PASSWORD || '';
  if (!adminPassword) {
    const remoteUsersPath =
      targetWorldSlug && targetWorldSlug !== 'ella'
        ? `/var/lib/ella-world/playgrounds/${targetWorldSlug}/world_users.json`
        : '/var/lib/ella-world/world_users.json';
    const rawJson = execFileSync(
      'ssh',
      ['-o', 'BatchMode=yes', opts.sshHost, `cat ${remoteUsersPath}`],
      { encoding: 'utf8' }
    );
    adminPassword = JSON.parse(rawJson).adminPassword;
  }

  // 1. Authenticate to ELLA World Admin API (scoped to worldSlug when provided)
  const adminLoginPayload = { password: adminPassword };
  if (targetWorldSlug) adminLoginPayload.worldSlug = targetWorldSlug;
  const adminLoginRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(adminLoginPayload)
  });
  if (!adminLoginRes.ok) {
    throw new Error(`Admin login failed: HTTP ${adminLoginRes.status}`);
  }
  const { adminToken, worldSlug: resolvedWorldSlug, dashboard: initialDashboard } = await adminLoginRes.json();

  if (hasInviteAction) {
    if (opts.invitesOnly) {
      const invRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/invites`, {
        headers: { Authorization: `Bearer ${adminToken}` }
      });
      if (!invRes.ok) {
        throw new Error(`Failed to fetch invites: HTTP ${invRes.status} ${await invRes.text()}`);
      }
      const invOut = await invRes.json();
      console.log(
        JSON.stringify(
          {
            ok: true,
            worldSlug: invOut.worldSlug || resolvedWorldSlug || 'ella',
            worldName: initialDashboard?.worldName,
            invites: invOut.invites || []
          },
          null,
          2
        )
      );
      return;
    }

    let invPayload;
    if (opts.createFriendInvite) {
      invPayload = {
        action: 'create',
        labelNote: opts.inviteLabel || 'Friend Invite',
        maxUses: opts.maxUses ?? 1,
        expiresInHours: opts.expiresHours ?? 168,
        presetChatMode: opts.chatMode || 'normal',
        presetVoiceMuted: opts.muteVoice ?? false,
        presetCanBuild: opts.canBuild ?? true,
        presetCanTrade: opts.canTrade ?? true,
        starterCoins: opts.starterCoins ?? 120
      };
    } else if (opts.approveInvite) {
      invPayload = { action: 'approve', code: opts.approveInvite };
    } else if (opts.rejectInvite) {
      invPayload = { action: 'reject', code: opts.rejectInvite };
    } else {
      invPayload = { action: 'revoke', code: opts.revokeInvite };
    }
    const invActOut = await postAdminInvites(opts.baseUrl, adminToken, invPayload);
    const shareUrl = invActOut.invite?.code
      ? `${opts.baseUrl.replace(/\/$/, '')}/?invite=${encodeURIComponent(invActOut.invite.code)}`
      : null;
    console.log(
      JSON.stringify(
        {
          ok: true,
          worldSlug: invActOut.worldSlug || resolvedWorldSlug || 'ella',
          invite: invActOut.invite,
          shareUrl
        },
        null,
        2
      )
    );
    return;
  }

  if (opts.devicesOnly) {
    console.log(
      JSON.stringify(
        {
          ok: true,
          worldSlug: resolvedWorldSlug || 'ella',
          worldName: initialDashboard?.worldName,
          enforceMachineSignatures: initialDashboard?.enforceMachineSignatures,
          deviceCount: (initialDashboard?.devices || []).length,
          deviceEvasionCount: initialDashboard?.deviceEvasionCount ?? 0,
          devices: initialDashboard?.devices || [],
          recentDeviceEvasions: initialDashboard?.recentDeviceEvasions || []
        },
        null,
        2
      )
    );
    return;
  }

  if (hasDeviceAction) {
    const receipts = [];
    if (opts.trustDeviceId) {
      const deviceId = resolveDeviceIdFromDashboard(initialDashboard, opts.trustDeviceId);
      receipts.push(
        await postAdminDevice(opts.baseUrl, adminToken, {
          deviceId,
          action: 'update',
          trustedShared: opts.trustDeviceVal
        })
      );
    }
    if (opts.banDeviceId) {
      const deviceId = resolveDeviceIdFromDashboard(initialDashboard, opts.banDeviceId);
      receipts.push(
        await postAdminDevice(opts.baseUrl, adminToken, {
          deviceId,
          action: 'update',
          banned: opts.banDeviceVal
        })
      );
    }
    if (opts.unlinkDeviceId) {
      const deviceId = resolveDeviceIdFromDashboard(initialDashboard, opts.unlinkDeviceId);
      receipts.push(
        await postAdminDevice(opts.baseUrl, adminToken, {
          deviceId,
          action: 'unlink_users'
        })
      );
    }
    const latest = receipts[receipts.length - 1] || {};
    console.log(
      JSON.stringify(
        {
          ok: true,
          device: latest.device,
          kickedCount: latest.kickedCount ?? 0
        },
        null,
        2
      )
    );
    return;
  }

  if (opts.listOnly) {
    console.log(
      JSON.stringify(
        {
          ok: true,
          worldSlug: resolvedWorldSlug || 'ella',
          worldName: initialDashboard?.worldName,
          sessionEpoch: initialDashboard?.sessionEpoch,
          worldLocked: initialDashboard?.worldLocked,
          globalVoiceEnabled: initialDashboard?.globalVoiceEnabled,
          globalChatMode: initialDashboard?.globalChatMode,
          curfewEnabled: initialDashboard?.curfewEnabled,
          curfewWindow: `${initialDashboard?.curfewStart || '21:00'}-${initialDashboard?.curfewEnd || '07:00'}`,
          profanityFineBase: initialDashboard?.profanityFineBase,
          debtPrisonThreshold: initialDashboard?.debtPrisonThreshold,
          prisonStrikeThreshold: initialDashboard?.prisonStrikeThreshold,
          users: (initialDashboard?.users || []).map((u) => ({
            username: u.username,
            displayName: u.displayName,
            role: u.role,
            online: u.online,
            coins: u.coins,
            violationStrikes: u.violationStrikes,
            inDebtorsPrison: u.inDebtorsPrison,
            prisonTasksRemaining: u.prisonTasksRemaining,
            voiceMutedByParent: u.voiceMutedByParent,
            chatMode: u.chatMode,
            canBuild: u.canBuild,
            canTrade: u.canTrade,
            suspended: u.suspended,
            tokenEpoch: u.tokenEpoch
          }))
        },
        null,
        2
      )
    );
    return;
  }

  if (opts.username && opts.giftItem) {
    const giftRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/gift-item`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${adminToken}`
      },
      body: JSON.stringify({
        username: opts.username,
        itemId: opts.giftItem,
        qty: opts.giftQty,
        allowClone: opts.allowClone
      })
    });
    if (!giftRes.ok) {
      throw new Error(`Gift item failed: HTTP ${giftRes.status} ${await giftRes.text()}`);
    }
    const giftOut = await giftRes.json();
    console.log(
      JSON.stringify(
        {
          ok: true,
          username: giftOut.username,
          itemId: giftOut.itemId,
          itemName: giftOut.itemName,
          qtyGranted: giftOut.qtyGranted,
          allowClone: giftOut.allowClone,
          economy: giftOut.economy
        },
        null,
        2
      )
    );
    return;
  }

  if (hasModerationAction && !opts.password) {
    const receipts = [];
    if (hasControlUpdates) {
      const payload = { username: opts.username, action: 'update_controls' };
      if (opts.chatMode !== null) payload.chatMode = opts.chatMode;
      if (opts.muteVoice !== undefined) payload.voiceMutedByParent = opts.muteVoice;
      if (opts.canBuild !== undefined) payload.canBuild = opts.canBuild;
      if (opts.canTrade !== undefined) payload.canTrade = opts.canTrade;
      if (opts.suspended !== undefined) payload.suspended = opts.suspended;
      if (opts.setRole !== null) payload.role = opts.setRole;
      receipts.push(await postAdminModerate(opts.baseUrl, adminToken, payload));
    }
    if (opts.resetStrikes) {
      receipts.push(
        await postAdminModerate(opts.baseUrl, adminToken, {
          username: opts.username,
          action: 'reset_strikes'
        })
      );
    }
    if (opts.coinDelta !== null && Number.isFinite(opts.coinDelta) && opts.coinDelta !== 0 && opts.sentenceTasks === null) {
      receipts.push(
        await postAdminModerate(opts.baseUrl, adminToken, {
          username: opts.username,
          action: 'adjust_coins',
          coinDelta: Math.trunc(opts.coinDelta)
        })
      );
    }
    if (opts.sentenceTasks !== null) {
      const sentencePayload = {
        username: opts.username,
        action: 'sentence_prison',
        prisonTasks: opts.sentenceTasks
      };
      if (opts.coinDelta !== null && Number.isFinite(opts.coinDelta) && opts.coinDelta !== 0) {
        sentencePayload.coinDelta = Math.trunc(opts.coinDelta);
      }
      if (opts.reason) {
        sentencePayload.reason = opts.reason;
      }
      receipts.push(await postAdminModerate(opts.baseUrl, adminToken, sentencePayload));
    }
    if (opts.pardon) {
      receipts.push(
        await postAdminModerate(opts.baseUrl, adminToken, {
          username: opts.username,
          action: 'pardon_prison'
        })
      );
    }

    const latest = receipts[receipts.length - 1] || {};
    console.log(
      JSON.stringify(
        {
          ok: true,
          username: latest.username || opts.username,
          actionsApplied: receipts.map((r) => r.action),
          economy: latest.economy,
          moderation: latest.moderation,
          kickedCount: latest.kickedCount ?? 0
        },
        null,
        2
      )
    );
    return;
  }

  if (opts.revokeAll || opts.revokeUser) {
    const revokePayload = opts.revokeUser ? { username: opts.revokeUser } : {};
    const revokeRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/sessions/revoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${adminToken}`
      },
      body: JSON.stringify(revokePayload)
    });
    if (!revokeRes.ok) {
      throw new Error(
        `Session revoke failed: HTTP ${revokeRes.status} ${await revokeRes.text()}`
      );
    }
    const revokeOut = await revokeRes.json();
    console.log(
      JSON.stringify(
        {
          ok: true,
          scope: revokeOut.scope,
          username: revokeOut.username || null,
          sessionEpoch: revokeOut.sessionEpoch ?? revokeOut.dashboard?.sessionEpoch,
          tokenEpoch: revokeOut.tokenEpoch ?? null,
          kickedCount: revokeOut.kickedCount,
          onlineUsersAfterRevoke: (revokeOut.dashboard?.users || [])
            .filter((u) => u.online)
            .map((u) => u.username)
        },
        null,
        2
      )
    );
    return;
  }

  // 2. Upsert the requested in-game user + optional avatar preset
  const upsertPayload = {
    username: opts.username,
    displayName: opts.displayName || opts.username,
    password: opts.password,
    role: opts.role
  };
  if (opts.preset && PRESET_AVATARS[opts.preset]) {
    upsertPayload.avatar = PRESET_AVATARS[opts.preset];
  }
  const upsertRes = await fetch(`${opts.baseUrl}/api/ella-world/admin/users`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(upsertPayload)
  });
  if (!upsertRes.ok) {
    throw new Error(`User upsert failed: HTTP ${upsertRes.status} ${await upsertRes.text()}`);
  }
  const upsertOut = await upsertRes.json();

  if (opts.skipLoginVerify) {
    const updatedUser = (upsertOut.dashboard?.users || []).find(
      (u) => u.username?.toLowerCase() === opts.username.toLowerCase()
    );
    console.log(
      JSON.stringify(
        {
          ok: true,
          worldSlug: resolvedWorldSlug || 'ella',
          username: updatedUser?.username || opts.username,
          displayName: updatedUser?.displayName || opts.displayName || opts.username,
          passwordRotated: upsertOut.passwordRotated,
          kickedSessions: upsertOut.kickedSessions,
          tokenEpoch: updatedUser?.tokenEpoch,
          skippedLoginVerify: true
        },
        null,
        2
      )
    );
    return;
  }

  // 3. Verify player login works with exact credentials, then immediately logout so no ghost avatar remains
  const verifyLoginPayload = {
    username: opts.username,
    password: opts.password,
    clientId: 'verify_cli'
  };
  if (resolvedWorldSlug) verifyLoginPayload.worldSlug = resolvedWorldSlug;
  const loginRes = await fetch(`${opts.baseUrl}/api/ella-world/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(verifyLoginPayload)
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
    worldSlug: loginData.worldSlug || resolvedWorldSlug || 'ella',
    username: loginData.self?.user?.username,
    displayName: loginData.self?.user?.displayName,
    passwordRotated: upsertOut.passwordRotated,
    kickedSessions: upsertOut.kickedSessions,
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
