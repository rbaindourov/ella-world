import { VoiceChatManager } from '/ella-world/js/voice.js';
import { WorldSpineClient } from '/ella-world/js/ws_spine.js';
import { BlockyWorld3D, BLOCK_COLORS } from '/ella-world/js/world3d.js';

// Signature Ella Forensics Cast & Dress-to-Impress Runway Presets
const SIGNATURE_GLAM_PRESETS = {
  ella_idol: {
    avatarMode: 'stylized',
    characterPreset: 'ella_idol',
    skinColor: '#ffdfc4',
    eyeColor: '#60a5fa',
    hairStyle: 'twin_pigtails',
    hairColor: '#fde047',
    topStyle: 'glam_gown',
    topColor: '#f472b6',
    bottomStyle: 'pleated_skirt',
    bottomColor: '#f9a8d4',
    shoeStyle: 'runway_heels',
    shoeColor: '#fbbf24',
    hat: 'crown',
    hatColor: '#fbbf24',
    accessory: 'wings',
    accessoryColor: '#a5f3fc',
    fabricFinish: 'satin',
    face: 'star'
  },
  katey: {
    avatarMode: 'stylized',
    characterPreset: 'katey',
    skinColor: '#ffe4d0',
    eyeColor: '#34d399',
    hairStyle: 'katey_long',
    hairColor: '#78350f',
    topStyle: 'sweater',
    topColor: '#c084fc',
    bottomStyle: 'jeans',
    bottomColor: '#3b82f6',
    shoeStyle: 'sneakers',
    shoeColor: '#f472b6',
    hat: 'beret',
    hatColor: '#a855f7',
    accessory: 'choker',
    accessoryColor: '#312e81',
    fabricFinish: 'matte',
    face: 'happy'
  },
  lizzy: {
    avatarMode: 'stylized',
    characterPreset: 'lizzy',
    skinColor: '#e5b887',
    eyeColor: '#8b5cf6',
    hairStyle: 'lizzy_bob',
    hairColor: '#1e1b4b',
    topStyle: 'puff_blouse',
    topColor: '#f472b6',
    bottomStyle: 'cargo_pants',
    bottomColor: '#475569',
    shoeStyle: 'sneakers',
    shoeColor: '#1e293b',
    hat: 'catears',
    hatColor: '#f472b6',
    accessory: 'star_purse',
    accessoryColor: '#f472b6',
    fabricFinish: 'satin',
    face: 'cat'
  },
  amy: {
    avatarMode: 'stylized',
    characterPreset: 'amy',
    skinColor: '#f3c6a5',
    eyeColor: '#60a5fa',
    hairStyle: 'amy_waves',
    hairColor: '#f472b6',
    topStyle: 'star_tee',
    topColor: '#38bdf8',
    bottomStyle: 'pleated_skirt',
    bottomColor: '#f8fafc',
    shoeStyle: 'platform_boots',
    shoeColor: '#f472b6',
    hat: 'hair_bow',
    hatColor: '#f472b6',
    accessory: 'sunglasses',
    accessoryColor: '#fbbf24',
    fabricFinish: 'satin',
    face: 'cool'
  },
  mom: {
    avatarMode: 'stylized',
    characterPreset: 'mom',
    skinColor: '#f5cba7',
    eyeColor: '#92400e',
    hairStyle: 'mom_bun',
    hairColor: '#92400e',
    topStyle: 'crew_tee',
    topColor: '#34d399',
    bottomStyle: 'overalls',
    bottomColor: '#3b82f6',
    shoeStyle: 'sneakers',
    shoeColor: '#34d399',
    hat: 'none',
    hatColor: '#fbbf24',
    accessory: 'pearls',
    accessoryColor: '#f8fafc',
    fabricFinish: 'matte',
    face: 'happy'
  },
  fairy_queen: {
    avatarMode: 'stylized',
    characterPreset: 'fairy_queen',
    skinColor: '#ffe4d0',
    eyeColor: '#8b5cf6',
    hairStyle: 'glam_ponytail',
    hairColor: '#c084fc',
    topStyle: 'glam_gown',
    topColor: '#38bdf8',
    bottomStyle: 'ruffle_mini',
    bottomColor: '#c084fc',
    shoeStyle: 'runway_heels',
    shoeColor: '#38bdf8',
    hat: 'crown',
    hatColor: '#fbbf24',
    accessory: 'wings',
    accessoryColor: '#67e8f9',
    fabricFinish: 'glitter',
    face: 'star'
  },
  y2k_skater: {
    avatarMode: 'stylized',
    characterPreset: 'y2k_skater',
    skinColor: '#ffdfc4',
    eyeColor: '#ec4899',
    hairStyle: 'twin_pigtails',
    hairColor: '#fb7185',
    topStyle: 'crop_top',
    topColor: '#f472b6',
    bottomStyle: 'ruffle_mini',
    bottomColor: '#f8fafc',
    shoeStyle: 'platform_boots',
    shoeColor: '#f472b6',
    hat: 'hair_bow',
    hatColor: '#ec4899',
    accessory: 'star_purse',
    accessoryColor: '#fbbf24',
    fabricFinish: 'satin',
    face: 'wink'
  },
  runway_gala: {
    avatarMode: 'stylized',
    characterPreset: 'runway_gala',
    skinColor: '#c68652',
    eyeColor: '#34d399',
    hairStyle: 'amy_waves',
    hairColor: '#1e1b4b',
    topStyle: 'glam_gown',
    topColor: '#fbbf24',
    bottomStyle: 'pleated_skirt',
    bottomColor: '#fbbf24',
    shoeStyle: 'runway_heels',
    shoeColor: '#fbbf24',
    hat: 'crown',
    hatColor: '#fbbf24',
    accessory: 'star_purse',
    accessoryColor: '#f59e0b',
    fabricFinish: 'glitter',
    face: 'star'
  }
};

// State
let sessionToken = localStorage.getItem('ella_world_token') || '';
let clientId = sessionStorage.getItem('ella_world_client_id') || '';
if (!clientId) {
  clientId = Math.random().toString(36).slice(2, 8);
  try {
    sessionStorage.setItem('ella_world_client_id', clientId);
  } catch {
    // ignore storage quota
  }
}
let adminToken = '';
let selfId = null;
let selfUser = null;
let draftAvatar = null;
let salonOriginalAvatar = null;
let avatarAutoSaveTimer = null;
let isSalonOpen = false;
let udpTimer = null;
let sharedOutboundSeq = 0;
let sessionEnteredWallMs = 0;
let lastAnnouncedTransportState = '';
let lastRunwayToastKey = '';
let lastRunwayToastAt = 0;
let wsTransportState = { state: 'connecting', rttMs: null };
let udpTransportState = { state: 'connecting', rttMs: null, port: 44444 };
let lastChatTs = 0;
const seenChatIds = new Set();

function nextOutboundSeq() {
  sharedOutboundSeq = (sharedOutboundSeq + 1) >>> 0;
  return sharedOutboundSeq;
}
let currentBlocksVersion = 0;
let latestMoveState = { x: 0, y: 0, z: 0, ry: 0, anim: 'idle', emote: null };
let localVoiceMuted = false;
let localVoiceSpeaking = false;
let toastTimer = null;

// Map of peerId -> player state object
const playersMap = new Map();

const JSON_HEADERS = {
  'Content-Type': 'application/json',
  'X-Cache-Warmup': 'true'
};

// DOM Elements
const loginScreen = document.getElementById('login-screen');
const gameHud = document.getElementById('game-hud');
const worldWelcomeTitle = document.getElementById('world-welcome-title');
const loginOnlineCount = document.getElementById('login-online-count');
const friendLoginForm = document.getElementById('friend-login-form');
const loginUsernameInput = document.getElementById('login-username');
const loginPasswordInput = document.getElementById('login-password');
const toggleLoginPasswordBtn = document.getElementById('toggle-login-password');
const loginErrorEl = document.getElementById('login-error');

const hudWorldTitle = document.getElementById('hud-world-title');
const hudPlayerBadge = document.getElementById('hud-player-badge');
const udpStatusPill = document.getElementById('udp-status-pill');
const udpStatusSr = document.getElementById('udp-status-sr');
const hudToast = document.getElementById('hud-toast');
const hudChatPanel = document.querySelector('.hud-chat-panel');

const runwayHudBar = document.getElementById('runway-hud-bar');
const runwayThemeLabel = document.getElementById('runway-theme-label');
const runwayModelSelect = document.getElementById('runway-model-select');
const runwayLeaderPill = document.getElementById('runway-leader-pill');

const micToggleBtn = document.getElementById('mic-toggle-btn');
const micIcon = document.getElementById('mic-icon');
const micLabel = document.getElementById('mic-label');
const localSpeakingMeter = document.getElementById('local-speaking-meter');
const proximityVoiceSwitch = document.getElementById('proximity-voice-switch');
const pttVoiceSwitch = document.getElementById('ptt-voice-switch');
const voiceModeBadge = document.getElementById('voice-mode-badge');
const voiceConnectionHint = document.getElementById('voice-connection-hint');
const friendsOnlineCount = document.getElementById('friends-online-count');
const friendsListEl = document.getElementById('friends-list');

const buildToolbar = document.getElementById('build-toolbar');
const toggleBuildBtn = document.getElementById('toggle-build-btn');
const buildActionPlaceBtn = document.getElementById('build-action-place');
const buildActionRemoveBtn = document.getElementById('build-action-remove');
const buildSaveStatusEl = document.getElementById('build-save-status');
const colorSwatchBar = document.getElementById('color-swatch-bar');

function updateBuildSaveStatus(blockCount, saving = false) {
  if (!buildSaveStatusEl) return;
  buildSaveStatusEl.classList.toggle('is-saving', Boolean(saving));
  if (saving) {
    buildSaveStatusEl.textContent = '💾 Saving build...';
  } else if (typeof blockCount === 'number') {
    buildSaveStatusEl.textContent = `✅ Saved World (${blockCount} blocks)`;
  } else {
    buildSaveStatusEl.textContent = '✅ Saved World';
  }
}

const chatMessagesEl = document.getElementById('chat-messages');
const chatForm = document.getElementById('chat-form');
const chatInput = document.getElementById('chat-input');

// Dialogs & Popover
const avatarDialog = document.getElementById('avatar-dialog');
const audioDialog = document.getElementById('audio-dialog');
const helpDialog = document.getElementById('help-dialog');
const adminDialog = document.getElementById('admin-dialog');

// 1. Initialize 3D World
const world3d = new BlockyWorld3D({
  container: document.getElementById('canvas-container'),
  onMove: (movePayload) => {
    latestMoveState = { ...latestMoveState, ...movePayload };
    if (selfId && playersMap.has(selfId)) {
      const me = playersMap.get(selfId);
      me.x = movePayload.x;
      me.y = movePayload.y;
      me.z = movePayload.z;
      me.ry = movePayload.ry;
      me.anim = movePayload.anim;
      me.emote = movePayload.emote;
    }
  },
  onPlaceBlock: async (block) => {
    world3d.addBlockMesh(block);
    updateBuildSaveStatus(world3d.voxelMap.size, true);
    const res = await fetch('/api/ella-world/block', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'place', ...block })
    }).catch(() => null);
    if (res && res.ok) {
      const data = await res.json();
      currentBlocksVersion = data.blocksVersion;
      updateBuildSaveStatus(Array.isArray(data.blocks) ? data.blocks.length : world3d.voxelMap.size, false);
    }
  },
  onRemoveBlock: async (coords) => {
    world3d.removeBlockMesh(coords);
    updateBuildSaveStatus(world3d.voxelMap.size, true);
    const res = await fetch('/api/ella-world/block', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'remove', ...coords })
    }).catch(() => null);
    if (res && res.ok) {
      const data = await res.json();
      currentBlocksVersion = data.blocksVersion;
      updateBuildSaveStatus(Array.isArray(data.blocks) ? data.blocks.length : world3d.voxelMap.size, false);
    }
  },
  onObbyCheckpoint: (cpIndex) => {
    showToast(`⭐ Checkpoint ${cpIndex} Reached! Press R anytime to warp here.`);
  },
  onTrophyWin: async () => {
    showToast('🏆 YOU CONQUERED THE SKY OBBY! Celebration sent to all friends! 🎉', 6000);
    await fetch('/api/ella-world/chat', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, isTrophyWin: true })
    }).catch(() => {});
  },
  onRunwayProximity: (isNear) => {
    if (runwayHudBar) {
      runwayHudBar.classList.toggle('near-runway', Boolean(isNear));
    }
    if (isNear) {
      showToast('💃 Welcome to the Fashion Runway! Strike a pose (1–8) & rate outfits 1–5 ⭐!');
    }
  }
});

function refreshConnectionBadge() {
  if (!udpStatusPill) return;
  const udpHealthy = voiceManager.isUdpStateHealthy();
  const wsHealthy = spineClient.isConnected();

  let badgeMode = 'connecting';
  if (udpHealthy) {
    badgeMode = 'udp';
  } else if (wsHealthy) {
    badgeMode = 'ws';
  } else if (
    wsTransportState.state === 'reconnecting' ||
    udpTransportState.state === 'reconnecting'
  ) {
    badgeMode = 'reconnecting';
  }

  udpStatusPill.classList.toggle('is-connecting', badgeMode === 'connecting');
  udpStatusPill.classList.toggle('is-reconnecting', badgeMode === 'reconnecting');

  if (badgeMode === 'udp') {
    const rtt =
      typeof udpTransportState.rttMs === 'number'
        ? udpTransportState.rttMs
        : wsTransportState.rttMs;
    const rttSuffix = typeof rtt === 'number' ? ` • ${rtt}ms` : '';
    const port = udpTransportState.port || voiceManager.udpPort || 44444;
    udpStatusPill.textContent = `🟢 UDP :${port}${rttSuffix}`;
  } else if (badgeMode === 'ws') {
    const rttSuffix =
      typeof wsTransportState.rttMs === 'number' ? ` • ${wsTransportState.rttMs}ms` : '';
    udpStatusPill.textContent = `🟢 WSS Sync${rttSuffix}`;
  } else if (badgeMode === 'reconnecting') {
    udpStatusPill.textContent = `🟡 Reconnecting...`;
  } else {
    udpStatusPill.textContent = `⚡ Connecting...`;
  }

  // Only update screen-reader polite live region on discrete connection state transitions
  // (never spam assistive tech on 20Hz RTT ticks — modern-web-guidance: accessibility Live Regions)
  if (udpStatusSr && badgeMode !== lastAnnouncedTransportState) {
    lastAnnouncedTransportState = badgeMode;
    if (badgeMode === 'udp') {
      udpStatusSr.textContent = `Connected to world fast-lane over UDP port ${udpTransportState.port || 44444}.`;
    } else if (badgeMode === 'ws') {
      udpStatusSr.textContent = 'Connected to world sync over secure WebSocket.';
    } else if (badgeMode === 'reconnecting') {
      udpStatusSr.textContent = 'Reconnecting world sync link.';
    }
  }
}

function handleBinaryWorldStateSnapshot({ rttMs, transport, players }) {
  if (transport === 'udp' && typeof rttMs === 'number') {
    udpTransportState.state = 'connected';
    udpTransportState.rttMs = rttMs;
  } else if (transport === 'ws' && typeof rttMs === 'number') {
    wsTransportState.state = 'connected';
    wsTransportState.rttMs = rttMs;
  }
  refreshConnectionBadge();

  for (const frame of players || []) {
    if (!frame.id || frame.id === selfId) continue;
    const existing = playersMap.get(frame.id);
    if (!existing) {
      // Unknown peer seen in 20 Hz binary tick — trigger throttled authoritative snapshot resync
      spineClient.requestResync();
      continue;
    }
    existing.x = frame.x;
    existing.y = frame.y;
    existing.z = frame.z;
    existing.ry = frame.ry;
    existing.anim = frame.anim;
    existing.emote = frame.emote;
    world3d.upsertRemotePlayer(existing);
    if (existing.muted !== frame.muted || existing.speaking !== frame.speaking) {
      existing.muted = frame.muted;
      existing.speaking = frame.speaking;
      updateFriendCardVoiceIndicator(frame.id, frame.muted, frame.speaking);
      const av = world3d.remoteAvatars.get(frame.id);
      if (av) av.setVoiceState({ muted: frame.muted, speaking: frame.speaking });
    }
  }
}

// 2A. Initialize Authoritative Persistent WebSocket Spine (`wss://<host>/api/ella-world/ws`)
const spineClient = new WorldSpineClient({
  nextSeq: nextOutboundSeq,
  onWorldStateSnapshot: (snap) => {
    handleBinaryWorldStateSnapshot(snap);
  },
  onWorldEvent: (ev) => {
    handleIncomingUdpWorldEvent(ev);
  },
  onTransportStatus: (st) => {
    wsTransportState = { ...wsTransportState, ...st };
    refreshConnectionBadge();
  },
  onClockOffset: (offsetMs) => {
    world3d.setServerClockOffset(offsetMs);
  }
});

// 2B. Initialize Zero-Renegotiation Single-Port UDP Fast-Lane + 8-Slot Opus SFU Manager
const voiceManager = new VoiceChatManager({
  nextSeq: nextOutboundSeq,
  onLocalVoiceState: ({ muted, speaking, level }) => {
    localVoiceMuted = muted;
    localVoiceSpeaking = speaking;
    updateLocalMicUI(muted, speaking, level);
    if (world3d.selfAvatar) {
      world3d.selfAvatar.setVoiceState({ muted, speaking });
    }
    if (selfId && playersMap.has(selfId)) {
      const me = playersMap.get(selfId);
      me.muted = muted;
      me.speaking = speaking;
      updateFriendCardVoiceIndicator(selfId, muted, speaking);
    }
  },
  onPeerVoiceState: (peerId, { speaking }) => {
    const p = playersMap.get(peerId);
    if (p) {
      p.speaking = speaking;
      updateFriendCardVoiceIndicator(peerId, p.muted, speaking);
    }
    const av = world3d.remoteAvatars.get(peerId);
    if (av) {
      av.setVoiceState({ muted: p ? p.muted : false, speaking });
    }
  },
  onWorldStateSnapshot: (snap) => {
    handleBinaryWorldStateSnapshot(snap);
  },
  onWorldEvent: (ev) => {
    handleIncomingUdpWorldEvent(ev);
  },
  onUdpTransportStatus: (status) => {
    udpTransportState = { ...udpTransportState, ...status };
    refreshConnectionBadge();
  },
  onStatusChange: ({ ok, message }) => {
    voiceConnectionHint.textContent = message;
    const statusEl = document.getElementById('mic-permission-status');
    if (statusEl) {
      statusEl.textContent = ok ? 'Mic Ready (UDP Opus)' : 'Check Permission';
    }
  }
});

// Update 3D Spatial Proximity Audio at 20Hz
setInterval(() => {
  if (world3d.selfAvatar) {
    voiceManager.updateSpatialPositions(
      world3d.selfAvatar.group.position,
      world3d.cameraYaw,
      playersMap
    );
  }
}, 50);

// 3. Modern Web Guidance: Dialog Light-Dismiss Fallback for modal dialogs without native `closedby`
if (!('closedBy' in HTMLDialogElement.prototype)) {
  for (const dialog of document.querySelectorAll('dialog[closedby="any"]')) {
    dialog.addEventListener('click', (event) => {
      if (event.target !== dialog) return;
      const rect = dialog.getBoundingClientRect();
      const isDialogContent =
        rect.top <= event.clientY &&
        event.clientY <= rect.top + rect.height &&
        rect.left <= event.clientX &&
        event.clientX <= rect.left + rect.width;
      if (isDialogContent) return;
      dialog.close();
    });
  }
}

for (const btn of document.querySelectorAll('[data-close-dialog]')) {
  btn.addEventListener('click', () => {
    const target = document.getElementById(btn.getAttribute('data-close-dialog'));
    if (target) target.close();
  });
}

// 4. Show/Hide Password Toggles
function bindPasswordToggle(btn, inputEl) {
  btn.addEventListener('click', () => {
    const isPassword = inputEl.type === 'password';
    inputEl.type = isPassword ? 'text' : 'password';
    btn.textContent = isPassword ? 'Hide password' : 'Show password';
  });
}
bindPasswordToggle(toggleLoginPasswordBtn, loginPasswordInput);
bindPasswordToggle(
  document.getElementById('toggle-admin-password'),
  document.getElementById('admin-master-password')
);

// 5. Fetch Initial World Metadata
async function loadPublicMeta() {
  try {
    const res = await fetch('/api/ella-world/meta', {
      headers: { 'X-Cache-Warmup': 'true' }
    });
    const data = await res.json();
    applyWorldMeta(data);
    if (data.runway) applyRunwaySummary(data.runway);
    loginOnlineCount.textContent =
      data.onlineCount === 1
        ? '1 friend in world right now'
        : `${data.onlineCount || 0} friends in world right now`;
  } catch {
    // ignore offline error
  }
}
loadPublicMeta();

function applyWorldMeta(meta) {
  if (!meta) return;
  if (meta.worldName) {
    worldWelcomeTitle.textContent = meta.worldName;
    hudWorldTitle.textContent = meta.worldName;
    document.title = `${meta.worldName} — 3D Friends Hangout & Dress to Impress World`;
  }
}

function applyRunwaySummary(runway) {
  if (!runway) return;
  if (runway.theme && runwayThemeLabel) {
    runwayThemeLabel.textContent = runway.theme;
  }
  if (runwayLeaderPill) {
    const top = Array.isArray(runway.leaderboard) && runway.leaderboard[0];
    if (top && top.votes > 0) {
      const matchedPlayer = Array.from(playersMap.values()).find(
        (p) => String(p.user?.username || '').toLowerCase() === top.username
      );
      const disp = matchedPlayer ? matchedPlayer.user.displayName : top.username;
      runwayLeaderPill.textContent = `🏆 Top Look: ${disp} (${top.avgStars}★)`;
    } else {
      runwayLeaderPill.textContent = '🏆 Rate outfits 1–5★!';
    }
  }
}

// 6. Friend Sign-In & Automatic Persistent Session Resume
async function tryAutoResumeSession() {
  if (!sessionToken) return;
  try {
    const res = await fetch('/api/ella-world/resume', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, clientId })
    });
    if (!res.ok) {
      localStorage.removeItem('ella_world_token');
      sessionToken = '';
      return;
    }
    const data = await res.json();
    sessionToken = data.token;
    selfId = data.selfId;
    selfUser = data.user;
    localStorage.setItem('ella_world_token', sessionToken);
    await enterGameWorld(data);
  } catch {
    // stay on login screen if offline
  }
}
tryAutoResumeSession();

friendLoginForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  loginErrorEl.hidden = true;

  const username = loginUsernameInput.value.trim();
  const password = loginPasswordInput.value;

  try {
    const res = await fetch('/api/ella-world/login', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ username, password, clientId })
    });
    const data = await res.json();
    if (!res.ok) {
      loginErrorEl.textContent = data.error || 'Could not sign in.';
      loginErrorEl.hidden = false;
      return;
    }

    sessionToken = data.token;
    selfId = data.selfId;
    selfUser = data.user;
    localStorage.setItem('ella_world_token', sessionToken);
    await enterGameWorld(data);
  } catch {
    loginErrorEl.textContent = 'Could not reach server. Please try again.';
    loginErrorEl.hidden = false;
  }
});

async function enterGameWorld(loginData) {
  loginScreen.hidden = true;
  gameHud.hidden = false;
  sessionEnteredWallMs = Date.now();

  if (typeof loginData.serverTimeMs === 'number') {
    world3d.setServerClockOffset(loginData.serverTimeMs - Date.now());
  }

  applyWorldMeta(loginData.worldMeta);
  if (loginData.runway) applyRunwaySummary(loginData.runway);
  hudPlayerBadge.textContent = `@${selfUser.username}`;

  const defaultProximity =
    loginData.worldMeta && loginData.worldMeta.voiceDefaultMode === 'proximity';
  proximityVoiceSwitch.checked = defaultProximity;
  voiceManager.setProximityMode(defaultProximity);
  updateVoiceModeBadge();

  playersMap.clear();
  playersMap.set(selfId, loginData.self);
  latestMoveState = {
    x: loginData.self.x || 0,
    y: loginData.self.y || 0,
    z: loginData.self.z || 0,
    ry: loginData.self.ry || 0,
    anim: 'idle',
    emote: null
  };

  world3d.initSelfPlayer(loginData.self);
  world3d.loadBlocks(loginData.blocks || []);
  currentBlocksVersion = loginData.blocksVersion || 1;
  updateBuildSaveStatus((loginData.blocks || []).length, false);

  spineClient.configure({
    sessionToken,
    wsUrl: loginData.worldMeta?.wsUrl || '/api/ella-world/ws'
  });

  voiceManager.configure({
    selfId,
    sessionToken,
    rtcOfferUrl: loginData.worldMeta?.rtcOfferUrl || '/api/ella-world/rtc/offer',
    iceServers: loginData.worldMeta?.iceServers,
    webrtcUdpPort: loginData.worldMeta?.webrtcUdpPort || 44444,
    voiceSlotCount: loginData.worldMeta?.voiceSlotCount || 8,
    defaultMode: defaultProximity ? 'proximity' : 'party'
  });

  for (const p of loginData.players || []) {
    playersMap.set(p.id, p);
    world3d.upsertRemotePlayer(p);
    voiceManager.ensurePeer(p.id);
  }

  renderFriendsList();

  chatMessagesEl.innerHTML = '';
  lastChatTs = Date.now() - 30000;
  seenChatIds.clear();
  for (const c of loginData.chatHistory || []) {
    if (c.id) seenChatIds.add(c.id);
    appendChatMessage(c);
    if (c.ts && c.ts > lastChatTs) lastChatTs = c.ts;
  }
  appendChatMessage({
    isSystem: true,
    text: `👑 Welcome to ${loginData.worldMeta?.worldName || "Ella's Starlight Glam Plaza"}, ${selfUser.displayName}! Open Glam Closet (C) or warp to the Runway (F)!`
  });

  // 1. Immediately connect persistent `:443` WebSocket spine (zero wait on microphone dialog!)
  spineClient.connect();

  // 2. Start 20 Hz hybrid state loop (uses UDP fast-lane when healthy, instant WSS fallback otherwise)
  if (udpTimer) clearInterval(udpTimer);
  udpTimer = setInterval(() => {
    if (!selfId || !selfUser) return;
    const udpHealthy = voiceManager.isUdpStateHealthy();
    spineClient.notifyUdpFastLane(udpHealthy);
    const stateFrame = {
      ...latestMoveState,
      muted: localVoiceMuted,
      speaking: localVoiceSpeaking
    };
    if (udpHealthy) {
      voiceManager.sendBinaryState(stateFrame);
    } else {
      spineClient.sendBinaryState(stateFrame);
      if (voiceManager.isUdpConnected()) {
        // Probe newly opened UDP DataChannel so inbound echo promotes UDP fast-lane
        voiceManager.sendBinaryState(stateFrame);
      }
    }
  }, 50);

  // 3. Connect Zero-Renegotiation UDP SFU + initialize microphone in parallel without blocking world sync
  (async () => {
    await voiceManager.initLocalMicrophone();
    await populateMicDeviceSelect();
  })().catch(() => {});
  voiceManager.connectServerUdp().catch(() => {});
}

function upsertIncomingRemotePlayer(p, silentJoin = false) {
  if (!p || !p.id || p.id === selfId) return false;
  const existing = playersMap.get(p.id);
  if (!existing) {
    playersMap.set(p.id, p);
    world3d.upsertRemotePlayer(p);
    voiceManager.ensurePeer(p.id);
    renderFriendsList();
    if (!silentJoin) {
      showToast(`👋 ${p.user?.displayName || 'A friend'} joined the world!`);
      appendChatMessage({
        isSystem: true,
        text: `🎉 ${p.user?.displayName || 'A friend'} joined the party!`
      });
    }
    return true;
  }

  const prevHash = existing.avatarHash || '';
  const nextHash = p.avatarHash || '';
  const prevAvatarSig = prevHash || JSON.stringify(existing.user?.avatar || {});
  const nextAvatarSig = nextHash || JSON.stringify(p.user?.avatar || {});
  existing.x = p.x;
  existing.y = p.y;
  existing.z = p.z;
  existing.ry = p.ry;
  existing.anim = p.anim;
  existing.emote = p.emote;
  if (p.avatarHash) existing.avatarHash = p.avatarHash;
  if (p.user) existing.user = p.user;
  world3d.upsertRemotePlayer(existing);

  if (p.user?.avatar && prevAvatarSig !== nextAvatarSig) {
    const av = world3d.remoteAvatars.get(p.id);
    if (av) av.updateAvatarAppearance(p.user.avatar);
    renderFriendsList();
  }
  if (existing.muted !== p.muted || existing.speaking !== p.speaking) {
    existing.muted = p.muted;
    existing.speaking = p.speaking;
    updateFriendCardVoiceIndicator(p.id, p.muted, p.speaking);
    const av = world3d.remoteAvatars.get(p.id);
    if (av) av.setVoiceState({ muted: p.muted, speaking: p.speaking });
  }
  return false;
}

function reconcileAuthoritativeRoster(remoteList, silentNewJoins = false) {
  const liveIds = new Set([selfId]);
  let rosterChanged = false;

  for (const p of remoteList || []) {
    if (!p || !p.id) continue;
    liveIds.add(p.id);
    if (p.id === selfId) continue;
    if (upsertIncomingRemotePlayer(p, silentNewJoins)) {
      rosterChanged = true;
    }
  }

  for (const [pid] of Array.from(playersMap.entries())) {
    if (pid !== selfId && !liveIds.has(pid)) {
      playersMap.delete(pid);
      world3d.removeRemotePlayer(pid);
      voiceManager.removePeer(pid);
      rosterChanged = true;
    }
  }

  if (rosterChanged) {
    renderFriendsList();
  }
}

function handleIncomingChatEntry(c, isHistorical = false) {
  if (!c) return;
  if (c.ts && c.ts > lastChatTs) {
    lastChatTs = c.ts;
  }
  if (c.id && seenChatIds.has(c.id)) {
    return;
  }
  if (c.id) seenChatIds.add(c.id);
  appendChatMessage(c);
  if (isHistorical || (c.ts && c.ts < sessionEnteredWallMs - 2500)) {
    return;
  }
  if (c.isTrophyWin || c.isRunwayEvent) {
    showToast(c.text, 5200);
    if (c.isTrophyWin) {
      const winnerAv =
        c.senderId === selfId ? world3d.selfAvatar : world3d.remoteAvatars.get(c.senderId);
      if (winnerAv) {
        world3d.spawnCelebrationConfetti(winnerAv.group.position);
      }
    }
  } else {
    const senderAv =
      c.senderId === selfId ? world3d.selfAvatar : world3d.remoteAvatars.get(c.senderId);
    if (senderAv) {
      senderAv.showSpeechBubble(c.text);
    }
  }
}

function handleIncomingUdpWorldEvent(ev) {
  if (!ev || typeof ev !== 'object') return;
  switch (ev.type) {
    case 'welcome_snapshot': {
      reconcileAuthoritativeRoster(ev.players || [], true);
      if (Array.isArray(ev.blocks) && ev.blocksVersion !== currentBlocksVersion) {
        currentBlocksVersion = ev.blocksVersion;
        world3d.loadBlocks(ev.blocks);
        updateBuildSaveStatus(ev.blocks.length, false);
      }
      for (const c of ev.chatHistory || []) {
        handleIncomingChatEntry(c, true);
      }
      if (ev.runway) {
        applyRunwaySummary(ev.runway);
      }
      if (ev.voiceSlots) {
        voiceManager.applyVoiceSlotMap(ev.voiceSlots);
      }
      break;
    }
    case 'world_sync_digest': {
      reconcileAuthoritativeRoster(ev.players || [], false);
      if (typeof ev.blocksVersion === 'number' && ev.blocksVersion !== currentBlocksVersion) {
        spineClient.requestResync();
      }
      if (ev.runway) {
        applyRunwaySummary(ev.runway);
      }
      break;
    }
    case 'voice_slot_map': {
      if (ev.slots) {
        voiceManager.applyVoiceSlotMap(ev.slots);
      }
      break;
    }
    case 'roster': {
      for (const p of ev.players || []) {
        upsertIncomingRemotePlayer(p, true);
      }
      break;
    }
    case 'player_join': {
      if (ev.player) {
        upsertIncomingRemotePlayer(ev.player, false);
      }
      break;
    }
    case 'player_leave': {
      if (ev.id && ev.id !== selfId && playersMap.has(ev.id)) {
        const departed = playersMap.get(ev.id);
        appendChatMessage({
          isSystem: true,
          text: `👋 ${departed.user?.displayName || 'A friend'} left the world.`
        });
        playersMap.delete(ev.id);
        world3d.removeRemotePlayer(ev.id);
        voiceManager.removePeer(ev.id);
        renderFriendsList();
      }
      break;
    }
    case 'avatar_updated': {
      if (ev.selfId && ev.selfId !== selfId && ev.user) {
        const existing = playersMap.get(ev.selfId);
        if (existing && ev.avatarHash && existing.avatarHash === ev.avatarHash) {
          break;
        }
        if (existing) {
          existing.user = ev.user;
          if (ev.avatarHash) existing.avatarHash = ev.avatarHash;
        }
        const av = world3d.remoteAvatars.get(ev.selfId);
        if (av) av.updateAvatarAppearance(ev.user.avatar);
        renderFriendsList();
      }
      break;
    }
    case 'chat': {
      if (ev.chat) {
        handleIncomingChatEntry(ev.chat, false);
      }
      break;
    }
    case 'block': {
      if (Array.isArray(ev.blocks) && ev.blocksVersion !== currentBlocksVersion) {
        currentBlocksVersion = ev.blocksVersion;
        world3d.loadBlocks(ev.blocks);
        updateBuildSaveStatus(ev.blocks.length, false);
      }
      break;
    }
    case 'runway': {
      if (ev.runway) {
        applyRunwaySummary(ev.runway);
      }
      const toastKey = `${ev.event || ''}:${ev.runway?.theme || ''}:${ev.voterId || ''}:${ev.targetUsername || ''}:${ev.stars || ''}`;
      const nowMs = performance.now();
      if (toastKey === lastRunwayToastKey && nowMs - lastRunwayToastAt < 1800) {
        break;
      }
      lastRunwayToastKey = toastKey;
      lastRunwayToastAt = nowMs;
      if (ev.event === 'theme' && ev.runway?.theme) {
        showToast(`💃 New Runway Theme: "${ev.runway.theme}"!`);
      } else if (ev.event === 'vote' && ev.voterId !== selfId && ev.stars) {
        showToast(
          `⭐ ${ev.voterName || 'A friend'} rated ${ev.targetUsername || 'an outfit'} ${ev.stars}★!`
        );
      }
      break;
    }
    default:
      break;
  }
}

// 7. Friends Online, Runway Model Dropdown & Per-Friend Volume Mixer UI
function syncSliderCssProps(slider) {
  slider.style.setProperty('--attr-min', slider.min || 0);
  slider.style.setProperty('--attr-max', slider.max || 100);
  slider.style.setProperty('--control-value', slider.value);
}

function renderFriendsList() {
  friendsOnlineCount.textContent = String(playersMap.size);
  friendsListEl.innerHTML = '';

  const prevSelectedModel = runwayModelSelect ? runwayModelSelect.value : '';
  if (runwayModelSelect) {
    runwayModelSelect.innerHTML = '';
  }

  for (const [pid, p] of playersMap.entries()) {
    const isMe = pid === selfId;
    const li = document.createElement('li');
    li.className = `friend-card${p.speaking ? ' speaking' : ''}`;
    li.id = `friend-card-${pid}`;

    const topRow = document.createElement('div');
    topRow.className = 'friend-card-top';

    const nameSpan = document.createElement('span');
    nameSpan.className = 'friend-name';
    nameSpan.textContent = `${p.user.role === 'host' ? '👑 ' : '🧒 '}${p.user.displayName}${isMe ? ' (You)' : ''}`;

    const statusTag = document.createElement('span');
    statusTag.className = 'friend-voice-tag';
    statusTag.id = `friend-voice-tag-${pid}`;
    statusTag.textContent = p.muted ? '🔇 Muted' : p.speaking ? '🔊 Talking' : '🎤 Ready';

    topRow.append(nameSpan, statusTag);
    li.appendChild(topRow);

    if (!isMe) {
      const volRow = document.createElement('div');
      volRow.className = 'friend-volume-row';
      const sliderId = `vol-slider-${pid}`;
      const label = document.createElement('label');
      label.setAttribute('for', sliderId);
      label.textContent = 'Vol 100%';

      const slider = document.createElement('input');
      slider.type = 'range';
      slider.id = sliderId;
      slider.min = '0';
      slider.max = '100';
      slider.step = '1';
      slider.value = '100';
      slider.setAttribute('aria-valuetext', '100% volume');
      syncSliderCssProps(slider);

      slider.addEventListener('input', () => {
        syncSliderCssProps(slider);
        label.textContent = `Vol ${slider.value}%`;
        slider.setAttribute('aria-valuetext', `${slider.value}% volume`);
        voiceManager.setPeerUserVolume(pid, Number(slider.value));
      });

      volRow.append(label, slider);
      li.appendChild(volRow);
    }

    friendsListEl.appendChild(li);

    // Populate Runway Star Voting selector (friends listed first, then self)
    if (runwayModelSelect && p.user?.username) {
      const opt = document.createElement('option');
      opt.value = p.user.username;
      opt.textContent = `${p.user.displayName}${isMe ? ' (Me)' : ''}`;
      if (!isMe && runwayModelSelect.firstChild) {
        runwayModelSelect.insertBefore(opt, runwayModelSelect.firstChild);
      } else {
        runwayModelSelect.appendChild(opt);
      }
    }
  }

  if (runwayModelSelect && prevSelectedModel) {
    const exists = Array.from(runwayModelSelect.options).some((o) => o.value === prevSelectedModel);
    if (exists) runwayModelSelect.value = prevSelectedModel;
  }
}

function updateFriendCardVoiceIndicator(pid, muted, speaking) {
  const card = document.getElementById(`friend-card-${pid}`);
  const tag = document.getElementById(`friend-voice-tag-${pid}`);
  if (card) {
    card.classList.toggle('speaking', Boolean(speaking && !muted));
  }
  if (tag) {
    tag.textContent = muted ? '🔇 Muted' : speaking ? '🔊 Talking' : '🎤 Ready';
  }
}

function updateLocalMicUI(muted, speaking, level = 0) {
  micToggleBtn.classList.toggle('mic-muted', muted);
  micToggleBtn.classList.toggle('mic-unmuted', !muted);
  micToggleBtn.classList.toggle('is-speaking', Boolean(speaking && !muted));
  micIcon.textContent = muted ? '🔇' : '🎤';
  if (voiceManager.pushToTalk) {
    micLabel.textContent = muted ? 'Hold V to Talk' : 'Talking (PTT)';
  } else {
    micLabel.textContent = muted ? 'Mic Muted' : 'Mic On';
  }
  const scale = 1 + Math.min(1.6, level * 5);
  localSpeakingMeter.style.transform = `scale(${muted ? 0.6 : scale})`;

  const dialogFill = document.getElementById('mic-dialog-meter-fill');
  if (dialogFill) {
    dialogFill.style.width = `${muted ? 4 : Math.min(100, Math.round(level * 260) + 5)}%`;
  }
}

function updateVoiceModeBadge() {
  const isProx = proximityVoiceSwitch.checked;
  voiceModeBadge.textContent = isProx ? '3D Proximity' : 'Party Voice';
}

// 8. Voice Controls Events
micToggleBtn.addEventListener('click', () => {
  pttVoiceSwitch.checked = false;
  voiceManager.toggleMute();
});

proximityVoiceSwitch.addEventListener('change', () => {
  voiceManager.setProximityMode(proximityVoiceSwitch.checked);
  updateVoiceModeBadge();
  showToast(
    proximityVoiceSwitch.checked
      ? '🔊 Switched to 3D Proximity Voice (Walk closer to friends to hear them louder!)'
      : '🎉 Switched to Party Voice (Everyone hears everyone anywhere in the world!)'
  );
});

pttVoiceSwitch.addEventListener('change', () => {
  voiceManager.setPushToTalk(pttVoiceSwitch.checked);
  showToast(
    pttVoiceSwitch.checked
      ? '🎙️ Push-to-Talk enabled: Hold the V key whenever you want to speak!'
      : '🎤 Open Mic enabled!'
  );
});

async function populateMicDeviceSelect() {
  const select = document.getElementById('mic-device-select');
  if (!select) return;
  const devices = await voiceManager.getAudioInputDevices();
  select.innerHTML = '';
  if (devices.length === 0) {
    const opt = document.createElement('option');
    opt.value = '';
    opt.textContent = 'Default System Microphone';
    select.appendChild(opt);
    return;
  }
  devices.forEach((d, idx) => {
    const opt = document.createElement('option');
    opt.value = d.deviceId;
    opt.textContent = d.label || `Microphone ${idx + 1}`;
    if (d.deviceId === voiceManager.selectedDeviceId) opt.selected = true;
    select.appendChild(opt);
  });
}

document.getElementById('audio-settings-btn').addEventListener('click', async () => {
  await populateMicDeviceSelect();
  audioDialog.showModal();
});

document.getElementById('mic-device-select').addEventListener('change', async (e) => {
  await voiceManager.initLocalMicrophone(e.target.value);
});

document.getElementById('retry-mic-btn').addEventListener('click', async () => {
  const select = document.getElementById('mic-device-select');
  await voiceManager.initLocalMicrophone(select ? select.value : '');
});

// 9. Chat & IME-Safe Enter-to-Submit
chatInput.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    if (event.isComposing || event.keyCode === 229) {
      return;
    }
    chatForm.requestSubmit();
  }
});

async function submitChatText(text) {
  if (!text) return;
  try {
    const res = await fetch('/api/ella-world/chat', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, text })
    });
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      const c = data.message;
      if (c) {
        handleIncomingChatEntry(c);
      }
    } else if (data.error) {
      showToast(`🛡️ ${data.error}`, 3600);
    }
  } catch {
    // ignore transient chat error
  }
}

chatForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const text = chatInput.value.trim();
  if (!text) return;
  chatInput.value = '';
  await submitChatText(text);
});

for (const btn of document.querySelectorAll('.reaction-btn')) {
  btn.addEventListener('click', async () => {
    const text = btn.getAttribute('data-react');
    if (text) {
      await submitChatText(text);
    }
  });
}

function appendChatMessage(msg) {
  const div = document.createElement('div');
  div.className = `chat-line${msg.isSystem ? ' system-msg' : ''}`;
  if (msg.isSystem) {
    div.textContent = msg.text;
  } else {
    const authorSpan = document.createElement('span');
    authorSpan.className = 'chat-author';
    authorSpan.textContent = `${msg.displayName || msg.username}:`;
    const bodySpan = document.createElement('span');
    bodySpan.textContent = msg.text;
    div.append(authorSpan, bodySpan);
  }
  chatMessagesEl.appendChild(div);
  chatMessagesEl.scrollTop = chatMessagesEl.scrollHeight;
}

// 10. Emotes, Runway Voting, Teleports, and Build Mode
function activateEmote(emoteName) {
  const active = world3d.triggerEmote(emoteName);
  latestMoveState.emote = active;
  for (const btn of document.querySelectorAll('.emote-btn')) {
    btn.classList.toggle('active', btn.getAttribute('data-emote') === active);
  }
}

for (const btn of document.querySelectorAll('.emote-btn')) {
  btn.addEventListener('click', () => {
    activateEmote(btn.getAttribute('data-emote'));
  });
}

document.getElementById('warp-plaza-btn').addEventListener('click', () => {
  world3d.warpToPlaza();
  showToast('⛲ Teleported to Central Hangout Plaza!');
});

document.getElementById('warp-runway-btn').addEventListener('click', () => {
  world3d.warpToRunway();
  showToast('💃 Teleported Backstage at the Dress to Impress Fashion Runway!');
});

document.getElementById('warp-obby-btn').addEventListener('click', () => {
  world3d.warpToObbyCheckpoint();
  showToast('🏃‍♀️ Teleported to Sky Obby!');
});

// Dress to Impress Runway Theme Spin & 1-5 Star Voting
document.getElementById('runway-next-theme-btn').addEventListener('click', async () => {
  const res = await fetch('/api/ella-world/runway', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ token: sessionToken, action: 'theme' })
  }).catch(() => null);
  if (res && res.ok) {
    const data = await res.json();
    applyRunwaySummary(data.runway);
  }
});

for (const starBtn of document.querySelectorAll('.star-vote-btn')) {
  starBtn.addEventListener('click', async () => {
    const stars = Number(starBtn.getAttribute('data-stars') || 5);
    const targetUsername = runwayModelSelect ? runwayModelSelect.value : selfUser?.username;
    if (!targetUsername) return;

    const res = await fetch('/api/ella-world/runway', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'vote', targetUsername, stars })
    }).catch(() => null);

    if (res && res.ok) {
      const data = await res.json();
      applyRunwaySummary(data.runway);
      if (stars === 5 && world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      showToast(`⭐ Gave ${stars} star${stars > 1 ? 's' : ''} for "${data.runway?.theme || 'Runway'}"!`);
    }
  });
}

// Build Toolbar Swatches
BLOCK_COLORS.forEach((c, idx) => {
  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = `color-swatch${idx === 0 ? ' selected' : ''}`;
  btn.style.backgroundColor = c.hex;
  btn.title = c.name;
  btn.setAttribute('aria-label', c.name);
  btn.addEventListener('click', () => {
    world3d.selectedBlockColor = c.hex;
    for (const s of colorSwatchBar.querySelectorAll('.color-swatch')) {
      s.classList.remove('selected');
    }
    btn.classList.add('selected');
  });
  colorSwatchBar.appendChild(btn);
});

function toggleBuildMode() {
  world3d.buildMode = !world3d.buildMode;
  buildToolbar.hidden = !world3d.buildMode;
  toggleBuildBtn.classList.toggle('active', world3d.buildMode);
  if (!world3d.buildMode) {
    world3d.ghostMesh.visible = false;
  } else {
    showToast('🔨 Build Mode ON! Click in the 3D world to place blocks, Right-Click to remove.');
  }
}

toggleBuildBtn.addEventListener('click', toggleBuildMode);

buildActionPlaceBtn.addEventListener('click', () => {
  world3d.buildTool = 'place';
  buildActionPlaceBtn.classList.add('active');
  buildActionRemoveBtn.classList.remove('active');
});

buildActionRemoveBtn.addEventListener('click', () => {
  world3d.buildTool = 'remove';
  buildActionRemoveBtn.classList.add('active');
  buildActionPlaceBtn.classList.remove('active');
});

// Keyboard shortcuts
window.addEventListener('keydown', (e) => {
  const tag = document.activeElement ? document.activeElement.tagName : '';
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
  if (document.querySelector('dialog:modal')) return;

  if (e.key === 'Escape' && isSalonOpen) {
    closeGlamSalon(false);
    return;
  }

  const k = e.key.toLowerCase();
  if (k === 'm') {
    pttVoiceSwitch.checked = false;
    voiceManager.toggleMute();
  } else if (k === 'v') {
    voiceManager.setPttKeyActive(true);
  } else if (k === 'c') {
    if (isSalonOpen) closeGlamSalon(false);
    else openGlamSalon();
  } else if (!isSalonOpen && k === 'b') {
    toggleBuildMode();
  } else if (!isSalonOpen && k === 'h') {
    world3d.warpToPlaza();
    showToast('⛲ Teleported to Central Hangout Plaza!');
  } else if (!isSalonOpen && k === 'f') {
    world3d.warpToRunway();
    showToast('💃 Teleported Backstage at the Fashion Runway!');
  } else if (!isSalonOpen && k === 'r') {
    world3d.warpToObbyCheckpoint();
    showToast('🏃‍♀️ Teleported to Sky Obby!');
  } else if (['1', '2', '3', '4', '5', '6', '7', '8'].includes(k)) {
    const emoteMap = {
      '1': 'pose',
      '2': 'peace',
      '3': 'dance',
      '4': 'spin',
      '5': 'wave',
      '6': 'cheer',
      '7': 'flip',
      '8': 'sit'
    };
    activateEmote(emoteMap[k]);
  }
});

window.addEventListener('keyup', (e) => {
  if (e.key.toLowerCase() === 'v') {
    voiceManager.setPttKeyActive(false);
  }
});

// On-screen Touch D-Pad Controls
for (const btn of document.querySelectorAll('[data-move]')) {
  const dir = btn.getAttribute('data-move');
  const start = (e) => {
    e.preventDefault();
    world3d.touchMove[dir] = true;
  };
  const end = (e) => {
    e.preventDefault();
    world3d.touchMove[dir] = false;
  };
  btn.addEventListener('pointerdown', start);
  btn.addEventListener('pointerup', end);
  btn.addEventListener('pointerleave', end);
}
document.getElementById('touch-jump-btn').addEventListener('pointerdown', (e) => {
  e.preventDefault();
  world3d.jump();
});

// 11. Accessible Glam Salon Closet Tabs & Live 3D Mirror Preview (modern-web-guidance: dialog + tabs)
const glamTablist = document.getElementById('glam-tablist');
const glamTabs = Array.from(glamTablist.querySelectorAll('[role="tab"]'));
const glamPanels = Array.from(avatarDialog.querySelectorAll('[role="tabpanel"]'));

const supportsUntilFound = 'onbeforematch' in HTMLElement.prototype;

function activateGlamTab(nextTab, moveFocus = false) {
  if (!nextTab) return;
  for (const tab of glamTabs) {
    const isSelected = tab === nextTab;
    tab.setAttribute('aria-selected', isSelected ? 'true' : 'false');
    tab.tabIndex = isSelected ? 0 : -1;
    tab.toggleAttribute('focusgroupstart', isSelected);
  }
  for (const panel of glamPanels) {
    const isTarget = panel.id === nextTab.getAttribute('aria-controls');
    if (isTarget) {
      panel.hidden = false;
      panel.removeAttribute('hidden');
    } else if (supportsUntilFound) {
      panel.setAttribute('hidden', 'until-found');
    } else {
      panel.hidden = true;
    }
  }
  try {
    nextTab.scrollIntoView({ block: 'nearest', inline: 'nearest', container: 'nearest' });
  } catch {
    nextTab.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }
  if (moveFocus) {
    nextTab.focus();
  }
}

for (const tab of glamTabs) {
  tab.addEventListener('click', () => activateGlamTab(tab, false));
}

// Sync selection if browser find-in-page reveals a `hidden="until-found"` panel
for (const panel of glamPanels) {
  panel.addEventListener('beforematch', () => {
    const matchingTab = glamTabs.find((t) => t.getAttribute('aria-controls') === panel.id);
    if (matchingTab) activateGlamTab(matchingTab, false);
  });
}

// Keyboard roving tabindex fallback for browsers without native CSS/HTML `focusgroup`
if (!Object.hasOwn(HTMLElement.prototype, 'focusGroup')) {
  glamTablist.addEventListener('keydown', (e) => {
    const currentIdx = glamTabs.indexOf(document.activeElement);
    if (currentIdx === -1) return;
    let nextIdx = -1;
    if (e.key === 'ArrowRight') nextIdx = (currentIdx + 1) % glamTabs.length;
    else if (e.key === 'ArrowLeft') nextIdx = (currentIdx - 1 + glamTabs.length) % glamTabs.length;
    else if (e.key === 'Home') nextIdx = 0;
    else if (e.key === 'End') nextIdx = glamTabs.length - 1;
    if (nextIdx !== -1) {
      e.preventDefault();
      activateGlamTab(glamTabs[nextIdx], true);
    }
  });
}

// Scroll-fade fallback via IntersectionObserver when scroll-driven animations aren't supported
if (!CSS.supports('animation-timeline', 'scroll()') && glamTabs.length > 1) {
  const fadeObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        const prop = entry.target === glamTabs[0] ? '--fade-start' : '--fade-end';
        glamTablist.style.setProperty(prop, entry.isIntersecting ? '0px' : '28px');
      }
    },
    { root: glamTablist, threshold: 0.95 }
  );
  fadeObserver.observe(glamTabs[0]);
  fadeObserver.observe(glamTabs[glamTabs.length - 1]);
}

function syncSalonControlsFromDraft() {
  if (!draftAvatar) return;

  const setPicker = (id, val) => {
    const el = document.getElementById(id);
    if (el && val) el.value = val;
  };
  setPicker('avatar-hair-color', draftAvatar.hairColor || '#fde047');
  setPicker('avatar-skin-color', draftAvatar.skinColor || '#ffdfc4');
  setPicker('avatar-eye-color', draftAvatar.eyeColor || '#60a5fa');
  setPicker('avatar-top-color', draftAvatar.topColor || draftAvatar.torsoColor || '#f472b6');
  setPicker('avatar-bottom-color', draftAvatar.bottomColor || draftAvatar.legsColor || '#6366f1');
  setPicker('avatar-shoe-color', draftAvatar.shoeColor || '#f472b6');
  setPicker('avatar-hat-color', draftAvatar.hatColor || '#fbbf24');
  setPicker('avatar-acc-color', draftAvatar.accessoryColor || '#f472b6');

  for (const card of avatarDialog.querySelectorAll('.glam-item-card[data-preset-id]')) {
    card.classList.toggle(
      'selected',
      card.getAttribute('data-preset-id') === draftAvatar.characterPreset
    );
  }
  for (const card of avatarDialog.querySelectorAll('.glam-item-card[data-field]')) {
    const f = card.getAttribute('data-field');
    const v = card.getAttribute('data-val');
    card.classList.toggle('selected', String(draftAvatar[f] || '') === v);
  }
  for (const seg of avatarDialog.querySelectorAll('.seg-btn[data-field]')) {
    const f = seg.getAttribute('data-field');
    const v = seg.getAttribute('data-val');
    seg.classList.toggle('active', String(draftAvatar[f] || '') === v);
  }

  const gownHint = document.getElementById('gown-override-hint');
  if (gownHint) {
    gownHint.hidden = draftAvatar.topStyle !== 'glam_gown';
  }
}

function previewDraftInMirror() {
  if (!draftAvatar) return;
  draftAvatar.torsoColor = draftAvatar.topColor;
  draftAvatar.legsColor = draftAvatar.bottomColor;
  syncSalonControlsFromDraft();
  if (world3d.selfAvatar) {
    world3d.selfAvatar.updateAvatarAppearance(draftAvatar);
  }
}

async function persistAvatarDraftNow(showSuccessToast = false) {
  if (!draftAvatar || !sessionToken) return false;
  if (avatarAutoSaveTimer) {
    clearTimeout(avatarAutoSaveTimer);
    avatarAutoSaveTimer = null;
  }

  const avatarPayload = {
    ...draftAvatar,
    torsoColor: draftAvatar.topColor || '#f472b6',
    legsColor: draftAvatar.bottomColor || '#6366f1'
  };

  try {
    const res = await fetch('/api/ella-world/avatar', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, avatar: avatarPayload })
    });
    if (!res.ok) return false;
    const data = await res.json();
    selfUser = data.user;
    if (selfId && playersMap.has(selfId)) {
      playersMap.get(selfId).user = selfUser;
    }
    if (world3d.selfAvatar) {
      world3d.selfAvatar.updateAvatarAppearance(selfUser.avatar);
    }
    if (showSuccessToast) {
      showToast('💖 Slay! Your new 3D Glam Runway Look is saved & live for all friends!');
    }
    return true;
  } catch {
    return false;
  }
}

function scheduleAutoSaveAvatar(delayMs = 240) {
  if (avatarAutoSaveTimer) clearTimeout(avatarAutoSaveTimer);
  avatarAutoSaveTimer = setTimeout(() => {
    avatarAutoSaveTimer = null;
    persistAvatarDraftNow(false);
  }, delayMs);
}

function openGlamSalon() {
  if (!selfUser) return;
  draftAvatar = {
    ...SIGNATURE_GLAM_PRESETS.ella_idol,
    ...(selfUser.avatar || {})
  };
  salonOriginalAvatar = JSON.parse(JSON.stringify(draftAvatar));
  syncSalonControlsFromDraft();

  if (typeof avatarDialog.showPopover === 'function') {
    try {
      avatarDialog.showPopover();
    } catch {
      avatarDialog.setAttribute('open', '');
    }
  } else {
    avatarDialog.show();
  }
  isSalonOpen = true;
  if (hudChatPanel) hudChatPanel.hidden = true;
  world3d.setSalonMirrorMode(true);
}

function closeGlamSalon(revertToOriginal = false) {
  if (revertToOriginal && salonOriginalAvatar) {
    draftAvatar = JSON.parse(JSON.stringify(salonOriginalAvatar));
    previewDraftInMirror();
    persistAvatarDraftNow(false);
  } else if (avatarAutoSaveTimer) {
    persistAvatarDraftNow(false);
  }
  if (typeof avatarDialog.hidePopover === 'function') {
    try {
      avatarDialog.hidePopover();
    } catch {
      avatarDialog.removeAttribute('open');
    }
  } else {
    avatarDialog.close();
  }
  isSalonOpen = false;
  if (hudChatPanel) hudChatPanel.hidden = false;
  world3d.setSalonMirrorMode(false);
}

document.getElementById('open-avatar-btn').addEventListener('click', () => {
  if (isSalonOpen) closeGlamSalon(false);
  else openGlamSalon();
});
document.getElementById('close-salon-btn').addEventListener('click', () => closeGlamSalon(false));
document.getElementById('cancel-salon-btn').addEventListener('click', () => {
  closeGlamSalon(true);
  showToast('↩️ Restored your previous outfit.');
});

// Color Pickers & Quick Swatches mapping
const COLOR_INPUT_TO_FIELD = {
  'avatar-hair-color': 'hairColor',
  'avatar-skin-color': 'skinColor',
  'avatar-eye-color': 'eyeColor',
  'avatar-top-color': 'topColor',
  'avatar-bottom-color': 'bottomColor',
  'avatar-shoe-color': 'shoeColor',
  'avatar-hat-color': 'hatColor',
  'avatar-acc-color': 'accessoryColor'
};

// Delegated click handler across all Glam Salon outfit buttons, presets, finishes & color swatches
avatarDialog.addEventListener('click', (e) => {
  const target = e.target instanceof Element ? e.target : null;
  if (!target || !draftAvatar) return;

  // 1. Signature Cast & DTI Preset Cards
  const presetCard = target.closest('.glam-item-card[data-preset-id]');
  if (presetCard) {
    const pid = presetCard.getAttribute('data-preset-id');
    const preset = SIGNATURE_GLAM_PRESETS[pid];
    if (preset) {
      draftAvatar = { ...draftAvatar, ...preset, avatarMode: 'stylized' };
      previewDraftInMirror();
      scheduleAutoSaveAvatar(180);
      showToast(`✨ Equipped ${presetCard.querySelector('.item-name')?.textContent || 'Signature Look'}!`);
    }
    return;
  }

  // 2. Modular Garment, Hair, Shoe, Hat & Accessory Cards
  const itemCard = target.closest('.glam-item-card[data-field]');
  if (itemCard) {
    const field = itemCard.getAttribute('data-field');
    const val = itemCard.getAttribute('data-val');
    if (field && val !== null) {
      // Ensure 3D Glam mode is active when selecting stylized wardrobe pieces
      if (draftAvatar.avatarMode === 'blocky' && field !== 'hat') {
        draftAvatar.avatarMode = 'stylized';
      }
      // Smart garment unblocking: picking a skirt/pants/overalls while wearing the full-length gown
      // automatically swaps the top to a matching blouse/tee so the bottom is immediately visible!
      if (field === 'bottomStyle' && draftAvatar.topStyle === 'glam_gown') {
        draftAvatar.topStyle = val === 'overalls' ? 'crew_tee' : 'puff_blouse';
      }
      // Picking a blouse/sweater/crop-top while wearing full overalls switches bottoms to jeans
      // unless crew_tee (which layers cleanly under overalls) or glam_gown is chosen.
      if (
        field === 'topStyle' &&
        val !== 'crew_tee' &&
        val !== 'glam_gown' &&
        draftAvatar.bottomStyle === 'overalls'
      ) {
        draftAvatar.bottomStyle = 'jeans';
      }

      draftAvatar[field] = val;
      draftAvatar.characterPreset = 'custom';
      previewDraftInMirror();
      scheduleAutoSaveAvatar(180);
    }
    return;
  }

  // 3. Segmented Buttons (Avatar Mode & Fabric Finish)
  const segBtn = target.closest('.seg-btn[data-field]');
  if (segBtn) {
    const field = segBtn.getAttribute('data-field');
    const val = segBtn.getAttribute('data-val');
    if (field && val !== null) {
      draftAvatar[field] = val;
      previewDraftInMirror();
      scheduleAutoSaveAvatar(180);
    }
    return;
  }

  // 4. Quick Color Swatches
  const swatchBtn = target.closest('.mini-swatch[data-hex]');
  if (swatchBtn) {
    const swatchGroup = swatchBtn.closest('.quick-swatches[data-target-input]');
    const targetId = swatchGroup ? swatchGroup.getAttribute('data-target-input') : '';
    const fieldName = COLOR_INPUT_TO_FIELD[targetId];
    const hex = swatchBtn.getAttribute('data-hex');
    if (fieldName && hex) {
      draftAvatar[fieldName] = hex;
      draftAvatar.characterPreset = 'custom';
      previewDraftInMirror();
      scheduleAutoSaveAvatar(180);
    }
  }
});

for (const [inputId, fieldName] of Object.entries(COLOR_INPUT_TO_FIELD)) {
  const inputEl = document.getElementById(inputId);
  if (inputEl) {
    inputEl.addEventListener('input', () => {
      if (!draftAvatar) return;
      draftAvatar[fieldName] = inputEl.value;
      draftAvatar.characterPreset = 'custom';
      previewDraftInMirror();
      scheduleAutoSaveAvatar(280);
    });
  }
}

document.getElementById('avatar-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  if (!draftAvatar) return;
  await persistAvatarDraftNow(true);
  closeGlamSalon(false);
});

// 12. Controls Help & Leave World
document.getElementById('open-help-btn').addEventListener('click', () => {
  helpDialog.showModal();
});

function leaveWorldToLogin() {
  if (isSalonOpen) closeGlamSalon(false);
  const oldTok = sessionToken;
  localStorage.removeItem('ella_world_token');
  sessionToken = '';
  if (oldTok) {
    fetch('/api/ella-world/logout', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: oldTok })
    }).catch(() => {});
  }
  if (udpTimer) {
    clearInterval(udpTimer);
    udpTimer = null;
  }
  spineClient.disconnect();
  voiceManager.disconnectSession();
  lastAnnouncedTransportState = '';
  wsTransportState = { state: 'connecting', rttMs: null };
  udpTransportState = { state: 'connecting', rttMs: null, port: 44444 };
  gameHud.hidden = true;
  loginScreen.hidden = false;
  loadPublicMeta();
}

document.getElementById('logout-btn').addEventListener('click', leaveWorldToLogin);

// 13. Parent Admin Panel Flow
function openAdminModal() {
  adminDialog.showModal();
  if (adminToken) {
    refreshAdminDashboard();
  }
}

document.getElementById('open-admin-from-login').addEventListener('click', openAdminModal);
document.getElementById('open-admin-hud-btn').addEventListener('click', openAdminModal);

const adminAuthForm = document.getElementById('admin-auth-form');
const adminAuthError = document.getElementById('admin-auth-error');
const adminDashboardContent = document.getElementById('admin-dashboard-content');

adminAuthForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  adminAuthError.hidden = true;
  const password = document.getElementById('admin-master-password').value;

  const res = await fetch('/api/ella-world/admin/login', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ password })
  });
  const data = await res.json();
  if (!res.ok) {
    adminAuthError.textContent = data.error || 'Incorrect password.';
    adminAuthError.hidden = false;
    return;
  }

  adminToken = data.adminToken;
  adminAuthForm.hidden = true;
  adminDashboardContent.hidden = false;
  renderAdminDashboard(data.dashboard);
});

async function refreshAdminDashboard() {
  if (!adminToken) return;
  const res = await fetch('/api/ella-world/admin/state', {
    headers: { Authorization: `Bearer ${adminToken}` }
  });
  if (res.ok) {
    const data = await res.json();
    renderAdminDashboard(data.dashboard);
  }
}

function renderAdminDashboard(dashboard) {
  if (!dashboard) return;
  document.getElementById('admin-user-count').textContent = String((dashboard.users || []).length);
  document.getElementById('admin-block-count').textContent = String(dashboard.blockCount || 0);
  document.getElementById('admin-world-name').value = dashboard.worldName || '';
  document.getElementById('admin-default-voice').value = dashboard.voiceDefaultMode || 'party';

  const filterCheckbox = document.getElementById('admin-chat-filter-enabled');
  if (filterCheckbox) {
    filterCheckbox.checked = dashboard.chatFilterEnabled !== false;
  }
  const blockedWordsBox = document.getElementById('admin-blocked-words');
  if (blockedWordsBox && Array.isArray(dashboard.blockedWords)) {
    blockedWordsBox.value = dashboard.blockedWords.join(', ');
  }
  const chatCountEl = document.getElementById('admin-chat-count');
  if (chatCountEl) {
    chatCountEl.textContent = String(dashboard.chatCount || 0);
  }
  const blockedCountEl = document.getElementById('admin-blocked-chat-count');
  if (blockedCountEl) {
    blockedCountEl.textContent = String(dashboard.blockedChatCount || 0);
  }

  const tbody = document.getElementById('admin-users-tbody');
  tbody.innerHTML = '';

  for (const u of dashboard.users || []) {
    const tr = document.createElement('tr');

    const tdUser = document.createElement('td');
    tdUser.innerHTML = `<strong>${u.username}</strong>`;

    const tdDisplay = document.createElement('td');
    tdDisplay.textContent = u.displayName;

    const tdPass = document.createElement('td');
    const code = document.createElement('code');
    code.textContent = u.password;
    tdPass.appendChild(code);

    const tdStatus = document.createElement('td');
    tdStatus.textContent = u.online ? '🟢 Online' : '⚪ Offline';

    const tdActions = document.createElement('td');
    const editBtn = document.createElement('button');
    editBtn.type = 'button';
    editBtn.className = 'small-action-btn';
    editBtn.textContent = 'Edit';
    editBtn.addEventListener('click', () => {
      document.getElementById('admin-friend-username').value = u.username;
      document.getElementById('admin-friend-display').value = u.displayName;
      document.getElementById('admin-friend-password').value = u.password;
      document.getElementById('admin-friend-hat').value = (u.avatar && u.avatar.hat) || 'crown';
    });

    const delBtn = document.createElement('button');
    delBtn.type = 'button';
    delBtn.className = 'small-action-btn danger-action';
    delBtn.style.marginLeft = '4px';
    delBtn.textContent = 'Remove';
    delBtn.addEventListener('click', async () => {
      const res = await fetch(`/api/ella-world/admin/users/${encodeURIComponent(u.username.toLowerCase())}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${adminToken}` }
      });
      if (res.ok) {
        const out = await res.json();
        renderAdminDashboard(out.dashboard);
      }
    });

    tdActions.append(editBtn, delBtn);
    tr.append(tdUser, tdDisplay, tdPass, tdStatus, tdActions);
    tbody.appendChild(tr);
  }

  refreshAdminChats();
}

async function refreshAdminChats() {
  if (!adminToken) return;
  const userFilter = (document.getElementById('admin-chat-user-filter')?.value || '').trim();
  const searchFilter = (document.getElementById('admin-chat-search')?.value || '').trim();
  const includeBlocked = Boolean(document.getElementById('admin-chat-include-blocked')?.checked);

  const params = new URLSearchParams({
    limit: '250',
    includeBlocked: String(includeBlocked)
  });
  if (userFilter) params.set('user', userFilter);
  if (searchFilter) params.set('search', searchFilter);

  const res = await fetch(`/api/ella-world/admin/chats?${params.toString()}`, {
    headers: { Authorization: `Bearer ${adminToken}` }
  });
  if (!res.ok) return;
  const data = await res.json();

  const chatCountEl = document.getElementById('admin-chat-count');
  if (chatCountEl) chatCountEl.textContent = String(data.total ?? 0);
  const blockedCountEl = document.getElementById('admin-blocked-chat-count');
  if (blockedCountEl) blockedCountEl.textContent = String(data.blockedTotal ?? 0);

  const combined = [
    ...(data.chats || []).map((c) => ({ ...c, _blocked: false })),
    ...(data.blockedChats || []).map((c) => ({ ...c, _blocked: true }))
  ].sort((a, b) => (b.ts || 0) - (a.ts || 0));

  const tbody = document.getElementById('admin-chats-tbody');
  if (!tbody) return;
  tbody.innerHTML = '';

  if (combined.length === 0) {
    const tr = document.createElement('tr');
    const td = document.createElement('td');
    td.colSpan = 4;
    td.textContent = 'No matching chat entries yet.';
    tr.appendChild(td);
    tbody.appendChild(tr);
    return;
  }

  for (const item of combined.slice(0, 250)) {
    const tr = document.createElement('tr');

    const tdTime = document.createElement('td');
    tdTime.textContent = item.ts ? new Date(item.ts).toLocaleString() : '—';

    const tdPlayer = document.createElement('td');
    tdPlayer.textContent = `${item.displayName || item.username} (@${item.username})`;

    const tdMsg = document.createElement('td');
    tdMsg.textContent = item.text || '';

    const tdStatus = document.createElement('td');
    const badge = document.createElement('span');
    if (item._blocked) {
      badge.className = 'admin-chat-badge-blocked';
      badge.textContent =
        item.reason === 'blocked_word'
          ? `Blocked (${item.matchedWord || 'filter'})`
          : `Blocked (${item.reason || 'rate'})`;
    } else {
      badge.className = 'admin-chat-badge-ok';
      badge.textContent = item.isSystem ? 'System' : 'Sent';
    }
    tdStatus.appendChild(badge);

    tr.append(tdTime, tdPlayer, tdMsg, tdStatus);
    tbody.appendChild(tr);
  }
}

document.getElementById('admin-chat-refresh-btn')?.addEventListener('click', refreshAdminChats);
document.getElementById('admin-chat-include-blocked')?.addEventListener('change', refreshAdminChats);
document.getElementById('admin-chat-user-filter')?.addEventListener('input', refreshAdminChats);
document.getElementById('admin-chat-search')?.addEventListener('input', refreshAdminChats);

document.getElementById('admin-upsert-user-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const feedback = document.getElementById('admin-user-feedback');
  feedback.hidden = true;

  const payload = {
    username: document.getElementById('admin-friend-username').value.trim(),
    displayName: document.getElementById('admin-friend-display').value.trim(),
    password: document.getElementById('admin-friend-password').value.trim(),
    avatar: {
      hat: document.getElementById('admin-friend-hat').value
    }
  };

  const res = await fetch('/api/ella-world/admin/users', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  const data = await res.json();
  if (res.ok) {
    renderAdminDashboard(data.dashboard);
    feedback.textContent = `✅ Saved friend account "${payload.username}" to data/world_users.json!`;
    feedback.hidden = false;
    document.getElementById('admin-friend-username').value = '';
    document.getElementById('admin-friend-display').value = '';
    document.getElementById('admin-friend-password').value = '';
  } else {
    feedback.textContent = `❌ ${data.error || 'Failed to save user'}`;
    feedback.hidden = false;
  }
});

document.getElementById('admin-settings-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const rawWords = document.getElementById('admin-blocked-words')?.value || '';
  const blockedWords = rawWords
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter((s) => s.length >= 2);
  const payload = {
    worldName: document.getElementById('admin-world-name').value.trim(),
    voiceDefaultMode: document.getElementById('admin-default-voice').value,
    newAdminPassword: document.getElementById('admin-new-master-password').value.trim(),
    chatFilterEnabled: Boolean(document.getElementById('admin-chat-filter-enabled')?.checked),
    blockedWords
  };

  const res = await fetch('/api/ella-world/admin/settings', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  if (res.ok) {
    const data = await res.json();
    renderAdminDashboard(data.dashboard);
    document.getElementById('admin-new-master-password').value = '';
    showToast('✅ World settings saved!');
  }
});

document.getElementById('admin-reset-starter-btn').addEventListener('click', async () => {
  const res = await fetch('/api/ella-world/admin/world/reset', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify({ mode: 'starter' })
  });
  if (res.ok) {
    const data = await res.json();
    renderAdminDashboard(data.dashboard);
  }
});

document.getElementById('admin-clear-blocks-btn').addEventListener('click', async () => {
  const res = await fetch('/api/ella-world/admin/world/reset', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify({ mode: 'clear' })
  });
  if (res.ok) {
    const data = await res.json();
    renderAdminDashboard(data.dashboard);
  }
});

function showToast(message, durationMs = 3800) {
  hudToast.textContent = message;
  hudToast.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    hudToast.hidden = true;
  }, durationMs);
}
