import { VoiceChatManager } from '/ella-world/js/voice.js';
import { WorldSpineClient } from '/ella-world/js/ws_spine.js';
import { BlockyWorld3D, BLOCK_COLORS } from '/ella-world/js/world3d.js';

// Signature Ella Forensics Cast & Dress-to-Impress Runway Presets
const SIGNATURE_GLAM_PRESETS = {
  ella_idol: {
    avatarMode: 'stylized',
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
    bodyType: 'female',
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
  },
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
    bottomStyle: 'tech_pants',
    bottomColor: '#64748b',
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
    bottomStyle: 'tech_pants',
    bottomColor: '#2c3e5a',
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

const ZONE_PLAZA = 0;
const ZONE_DUNGEON = 1;
const ZONE_PRISON = 2;

function nextOutboundSeq() {
  sharedOutboundSeq = (sharedOutboundSeq + 1) >>> 0;
  return sharedOutboundSeq;
}
let currentBlocksVersion = 0;
let currentZone = ZONE_PLAZA;
let latestMoveState = {
  x: 0,
  y: 0,
  z: 0,
  ry: 0,
  anim: 'idle',
  emote: null,
  zone: ZONE_PLAZA
};
let localVoiceMuted = false;
let localVoiceSpeaking = false;
let toastTimer = null;

// Crystal Cavern Dungeon, Persistent Economy, Parental Moderation & Debtor's Prison State
let playerEconomy = { coins: 0, inventory: {} };
let playerModeration = {
  violationStrikes: 0,
  inDebtorsPrison: false,
  prisonReason: null,
  prisonTasksRemaining: 0,
  voiceMutedByParent: false,
  chatMode: 'normal',
  suspended: false,
  canBuild: true,
  canTrade: true
};
let parentalWorldSettings = {
  globalVoiceEnabled: true,
  globalChatMode: 'normal',
  safeQuickChatPhrases: [],
  profanityFineBase: 15,
  debtPrisonThreshold: -50,
  prisonStrikeThreshold: 3
};
let prisonWorkInFlight = false;
let itemCatalog = [];
const catalogById = new Map();
let activeSpeedBoostUntilMs = 0;
let activeJumpBoostUntilMs = 0;
let pendingInviteTrade = null;
let activeTrade = null;
let draftTradeCoins = 0;
let draftTradeItems = {};
let tradeOfferDebounceTimer = null;
let zonePortalInFlight = false;
let pendingPostPortalWarp = null;
const pickupInFlightIds = new Set();

function isSelfEligibleForPrisonRelease() {
  return (
    Number(playerEconomy.coins || 0) >= 0 &&
    Number(playerModeration.prisonTasksRemaining || 0) === 0
  );
}

function effectiveChatModeForSelf() {
  const gMode = String(parentalWorldSettings.globalChatMode || 'normal');
  const uMode = String(playerModeration.chatMode || 'normal');
  if (gMode === 'muted' || uMode === 'muted') return 'muted';
  if (gMode === 'quick_only' || uMode === 'quick_only') return 'quick_only';
  return 'normal';
}

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
const hudZonePill = document.getElementById('hud-zone-pill');
const hudCoinsPill = document.getElementById('hud-coins-pill');
const hudCoinsCount = document.getElementById('hud-coins-count');
const hudBuffPill = document.getElementById('hud-buff-pill');
const hudItemCount = document.getElementById('hud-item-count');
const hudInteractPrompt = document.getElementById('hud-interact-prompt');
const udpStatusPill = document.getElementById('udp-status-pill');
const udpStatusSr = document.getElementById('udp-status-sr');
const hudToast = document.getElementById('hud-toast');
const hudChatPanel = document.querySelector('.hud-chat-panel');
const warpDungeonBtn = document.getElementById('warp-dungeon-btn');
const openBackpackBtn = document.getElementById('open-backpack-btn');
const touchActionBtn = document.getElementById('touch-action-btn');

const prisonSentenceBanner = document.getElementById('prison-sentence-banner');
const prisonBannerTitle = document.getElementById('prison-banner-title');
const prisonBannerReason = document.getElementById('prison-banner-reason');
const prisonDebtPill = document.getElementById('prison-debt-pill');
const prisonChoresPill = document.getElementById('prison-chores-pill');
const prisonGatePill = document.getElementById('prison-gate-pill');

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
const chatSendBtn = document.getElementById('chat-send-btn');

// Dialogs & Popover
const avatarDialog = document.getElementById('avatar-dialog');
const audioDialog = document.getElementById('audio-dialog');
const helpDialog = document.getElementById('help-dialog');
const adminDialog = document.getElementById('admin-dialog');
const backpackDialog = document.getElementById('backpack-dialog');
const backpackCoinsValue = document.getElementById('backpack-coins-value');
const backpackItemsGrid = document.getElementById('backpack-items-grid');
const backpackEnterDungeonBtn = document.getElementById('backpack-enter-dungeon-btn');
const tradeInvitePopover = document.getElementById('trade-invite-popover');
const tradeInviteText = document.getElementById('trade-invite-text');
const tradeInviteAcceptBtn = document.getElementById('trade-invite-accept-btn');
const tradeInviteDeclineBtn = document.getElementById('trade-invite-decline-btn');
const tradeDialog = document.getElementById('trade-dialog');
const tradeStatusNote = document.getElementById('trade-status-note');
const tradeCloseCancelBtn = document.getElementById('trade-close-cancel-btn');
const tradeSelfHeading = document.getElementById('trade-self-heading');
const tradeSelfStatePill = document.getElementById('trade-self-state-pill');
const tradeSelfWallet = document.getElementById('trade-self-wallet');
const tradeCoinsInput = document.getElementById('trade-coins-input');
const tradeSelfOfferList = document.getElementById('trade-self-offer-list');
const tradeInventoryPicker = document.getElementById('trade-inventory-picker');
const tradePartnerHeading = document.getElementById('trade-partner-heading');
const tradePartnerStatePill = document.getElementById('trade-partner-state-pill');
const tradePartnerCoins = document.getElementById('trade-partner-coins');
const tradePartnerOfferList = document.getElementById('trade-partner-offer-list');
const tradeCancelBtn = document.getElementById('trade-cancel-btn');
const tradeLockBtn = document.getElementById('trade-lock-btn');
const tradeConfirmBtn = document.getElementById('trade-confirm-btn');

let wsHandshakeProbeInFlight = false;
let lastWsHandshakeProbeAt = 0;

function forceSessionRevokedToLogin(reason) {
  const alertMessage =
    reason || 'Your session has expired or your password was updated. Please sign in again.';

  // Close player modals/popovers first so #login-screen is not inert when announcing
  if (isSalonOpen) closeGlamSalon(false);
  hideTradeInvitePopover();
  pendingInviteTrade = null;
  activeTrade = null;
  draftTradeCoins = 0;
  draftTradeItems = {};
  if (tradeDialog && tradeDialog.open) tradeDialog.close('cancelled');
  if (backpackDialog && backpackDialog.open) backpackDialog.close();
  if (audioDialog && audioDialog.open) audioDialog.close();
  if (helpDialog && helpDialog.open) helpDialog.close();

  localStorage.removeItem('ella_world_token');
  sessionToken = '';
  for (const [pid] of Array.from(playersMap.entries())) {
    if (pid !== selfId) {
      world3d.removeRemotePlayer(pid);
      voiceManager.removePeer(pid);
    }
  }
  selfId = null;
  selfUser = null;
  playersMap.clear();

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
  if (loginPasswordInput) {
    loginPasswordInput.value = '';
  }

  const showLoginAlertAndFocus = () => {
    if (loginErrorEl) {
      loginErrorEl.textContent = alertMessage;
      loginErrorEl.hidden = false;
    }
    if (loginPasswordInput) {
      loginPasswordInput.focus();
    }
  };

  // Coordinate live-region updates with modal dialog / inert state (modern-web-guidance: accessibility)
  if (adminDialog && adminDialog.open) {
    adminDialog.addEventListener('close', showLoginAlertAndFocus, { once: true });
  } else {
    showLoginAlertAndFocus();
  }
  loadPublicMeta();
}

async function handleHttpAuthCheck(res) {
  if (!res || (res.status !== 401 && res.status !== 403)) return false;
  let errMsg = 'Your session has expired or your password was updated. Please sign in again.';
  try {
    const body = await res.clone().json();
    if (body && body.error) errMsg = body.error;
  } catch {
    // ignore JSON parse failure
  }
  forceSessionRevokedToLogin(errMsg);
  return true;
}

async function probeSessionValidityAfterWsFailure() {
  if (!sessionToken || wsHandshakeProbeInFlight) return;
  const now = Date.now();
  if (now - lastWsHandshakeProbeAt < 1500) return;
  wsHandshakeProbeInFlight = true;
  lastWsHandshakeProbeAt = now;
  try {
    const res = await fetch('/api/ella-world/resume', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, clientId })
    });
    if (res.status === 401 || res.status === 403) {
      await handleHttpAuthCheck(res);
    }
  } catch {
    // network temporarily offline — keep normal WS reconnect backoff
  } finally {
    wsHandshakeProbeInFlight = false;
  }
}

// 1. Initialize 3D World
const world3d = new BlockyWorld3D({
  container: document.getElementById('canvas-container'),
  onMove: (movePayload) => {
    latestMoveState = {
      ...latestMoveState,
      ...movePayload,
      zone: typeof movePayload.zone === 'number' ? movePayload.zone : currentZone
    };
    if (selfId && playersMap.has(selfId)) {
      const me = playersMap.get(selfId);
      me.x = movePayload.x;
      me.y = movePayload.y;
      me.z = movePayload.z;
      me.ry = movePayload.ry;
      me.anim = movePayload.anim;
      me.emote = movePayload.emote;
      me.zone = latestMoveState.zone;
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
    if (await handleHttpAuthCheck(res)) return;
    if (res && res.ok) {
      const data = await res.json();
      currentBlocksVersion = data.blocksVersion;
      updateBuildSaveStatus(Array.isArray(data.blocks) ? data.blocks.length : world3d.voxelMap.size, false);
    } else {
      world3d.removeBlockMesh({ x: block.x, y: block.y, z: block.z });
      updateBuildSaveStatus(world3d.voxelMap.size, false);
      if (res) {
        const err = await res.json().catch(() => ({}));
        if (err && err.error) showToast(`⚠️ ${err.error}`);
      }
    }
  },
  onRemoveBlock: async (coords) => {
    const prevEntry = world3d.voxelMap.get(`${coords.x},${coords.y},${coords.z}`);
    const prevBlock = prevEntry ? { ...prevEntry.data } : null;
    world3d.removeBlockMesh(coords);
    updateBuildSaveStatus(world3d.voxelMap.size, true);
    const res = await fetch('/api/ella-world/block', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'remove', ...coords })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    if (res && res.ok) {
      const data = await res.json();
      currentBlocksVersion = data.blocksVersion;
      updateBuildSaveStatus(Array.isArray(data.blocks) ? data.blocks.length : world3d.voxelMap.size, false);
    } else {
      if (prevBlock) world3d.addBlockMesh(prevBlock);
      updateBuildSaveStatus(world3d.voxelMap.size, false);
      if (res) {
        const err = await res.json().catch(() => ({}));
        if (err && err.error) showToast(`⚠️ ${err.error}`);
      }
    }
  },
  onObbyCheckpoint: (cpIndex) => {
    showToast(`⭐ Checkpoint ${cpIndex} Reached! Press R anytime to warp here.`);
  },
  onTrophyWin: async () => {
    showToast('🏆 YOU CONQUERED THE SKY OBBY! Celebration sent to all friends! 🎉', 6000);
    const res = await fetch('/api/ella-world/chat', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, isTrophyWin: true })
    }).catch(() => null);
    await handleHttpAuthCheck(res);
  },
  onRunwayProximity: (isNear) => {
    if (runwayHudBar) {
      runwayHudBar.classList.toggle('near-runway', Boolean(isNear));
    }
    if (isNear) {
      showToast('💃 Welcome to the Fashion Runway! Strike a pose (1–8) & rate outfits 1–5 ⭐!');
    }
  },
  onRequestZonePortal: (targetZone) => {
    requestZoneTransition(targetZone);
  },
  onInteractNode: (nodeId) => {
    interactDungeonNode(nodeId);
  },
  onPrisonWork: (nodeId) => {
    performPrisonWork(nodeId);
  },
  onAttackMob: (mobId, mode) => {
    attackDungeonMob(mobId, mode);
  },
  onPickupDrop: (dropId) => {
    pickupDungeonDrop(dropId);
  },
  onInteractionHint: (hintText) => {
    if (!hudInteractPrompt) return;
    if (hintText) {
      hudInteractPrompt.textContent = hintText;
      hudInteractPrompt.hidden = false;
    } else {
      hudInteractPrompt.hidden = true;
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
    const prevZone = existing.zone || ZONE_PLAZA;
    const nextZone = typeof frame.zone === 'number' ? frame.zone : prevZone;
    existing.x = frame.x;
    existing.y = frame.y;
    existing.z = frame.z;
    existing.ry = frame.ry;
    existing.anim = frame.anim;
    existing.emote = frame.emote;
    existing.zone = nextZone;
    world3d.upsertRemotePlayer(existing);
    if (prevZone !== nextZone) {
      renderFriendsList();
    }
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
  },
  onSessionRevoked: (reason) => {
    forceSessionRevokedToLogin(reason);
  },
  onHandshakeFailed: () => {
    probeSessionValidityAfterWsFailure();
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
  },
  onAuthRejected: (reason) => {
    forceSessionRevokedToLogin(reason);
  }
});

// Update 3D Spatial Proximity Audio at 20Hz
setInterval(() => {
  if (world3d.selfAvatar) {
    voiceManager.updateSpatialPositions(
      world3d.selfAvatar.group.position,
      world3d.cameraYaw,
      playersMap,
      currentZone
    );
  }
}, 50);

// Update active Swiftness / Cloud Leap potion timers at 4Hz
setInterval(() => {
  if (!hudBuffPill) return;
  const serverNow = Date.now() + (world3d.serverClockOffsetMs || 0);
  const speedRem = Math.ceil((activeSpeedBoostUntilMs - serverNow) / 1000);
  const jumpRem = Math.ceil((activeJumpBoostUntilMs - serverNow) / 1000);
  const parts = [];
  if (speedRem > 0) parts.push(`⚡ Swift ${speedRem}s`);
  if (jumpRem > 0) parts.push(`🪽 Leap ${jumpRem}s`);
  if (parts.length > 0) {
    hudBuffPill.textContent = parts.join(' • ');
    hudBuffPill.hidden = false;
  } else {
    hudBuffPill.hidden = true;
  }
}, 250);

// 3. Modern Web Guidance: Dialog Light-Dismiss Fallback (`pointerdown` + `pointerup` outside rect)
if (!('closedBy' in HTMLDialogElement.prototype)) {
  const outside = (e, dialog) => {
    const r = dialog.getBoundingClientRect();
    return e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
  };
  for (const dialog of document.querySelectorAll('dialog[closedby="any"]')) {
    let pressedOutside = false;
    dialog.addEventListener('pointerdown', (e) => {
      pressedOutside = outside(e, dialog);
    });
    dialog.addEventListener('pointerup', (e) => {
      if (pressedOutside && outside(e, dialog)) dialog.close();
    });
  }
}

for (const dialog of document.querySelectorAll('dialog')) {
  dialog.addEventListener('close', () => {
    dialog.returnValue = '';
  });
}

if (tradeDialog) {
  tradeDialog.addEventListener('cancel', (e) => {
    e.preventDefault();
    const mySide = getMyTradeSide(activeTrade);
    if (mySide && (mySide.locked || mySide.confirmed)) {
      if (tradeStatusNote) {
        tradeStatusNote.textContent =
          '🔒 Your trade offer is locked! Click "Unlock Offer" first or click "✕ Cancel Trade" to abort.';
      }
      if (tradeLockBtn) tradeLockBtn.focus();
      return;
    }
    cancelActiveTrade();
  });
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
    btn.setAttribute('aria-pressed', String(isPassword));
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
    if (data.worldLocked) {
      loginOnlineCount.textContent = '⏸️ World paused by parent';
    } else {
      loginOnlineCount.textContent =
        data.onlineCount === 1
          ? '1 friend in world right now'
          : `${data.onlineCount || 0} friends in world right now`;
    }
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
      const errData = await res.json().catch(() => ({}));
      localStorage.removeItem('ella_world_token');
      sessionToken = '';
      if ((res.status === 401 || res.status === 403) && loginErrorEl) {
        loginErrorEl.textContent =
          errData.error || 'Your session has expired or your password was updated. Please sign in again.';
        loginErrorEl.hidden = false;
      }
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
  currentZone = typeof loginData.self?.zone === 'number' ? loginData.self.zone : ZONE_PLAZA;
  latestMoveState = {
    x: loginData.self.x || 0,
    y: loginData.self.y || 0,
    z: loginData.self.z || 0,
    ry: loginData.self.ry || 0,
    anim: 'idle',
    emote: null,
    zone: currentZone
  };

  if (Array.isArray(loginData.itemCatalog)) {
    setItemCatalog(loginData.itemCatalog);
  } else if (itemCatalog.length === 0) {
    fetch('/api/ella-world/economy/catalog', { headers: { 'X-Cache-Warmup': 'true' } })
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        const list = d?.itemCatalog || d?.items;
        if (Array.isArray(list)) setItemCatalog(list);
      })
      .catch(() => {});
  }
  if (loginData.economy) {
    applyEconomySnapshot(loginData.economy);
  }
  if (loginData.parentalWorld) {
    applyParentalWorldSettings(loginData.parentalWorld);
  }
  if (loginData.moderation || loginData.user?.moderation) {
    applyModerationSnapshot(loginData.moderation || loginData.user.moderation);
  } else {
    applyModerationAndParentalUi();
  }

  world3d.initSelfPlayer(loginData.self);
  if (loginData.dungeon) {
    world3d.syncDungeonSnapshot(loginData.dungeon);
  }
  if (loginData.prison) {
    world3d.syncPrisonSnapshot(loginData.prison, isSelfEligibleForPrisonRelease());
  }
  world3d.setZone(currentZone, loginData.self);
  renderZoneHud(currentZone);
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
  applyModerationAndParentalUi();

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
    text:
      currentZone === ZONE_PRISON
        ? `⚖️ Welcome back, ${selfUser.displayName}. Complete glowing courtyard chores (E) to clear your debt and unlock the South Release Gate!`
        : `👑 Welcome to ${loginData.worldMeta?.worldName || "Ella's Starlight Glam Plaza"}, ${selfUser.displayName}! Enter the Crystal Cavern (G), open Backpack (I), Glam Closet (C), or Runway (F)!`
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
      zone: currentZone,
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
    applyModerationAndParentalUi();
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

  const prevZone = existing.zone || ZONE_PLAZA;
  const nextZone = typeof p.zone === 'number' ? p.zone : prevZone;
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
  existing.zone = nextZone;
  if (p.avatarHash) existing.avatarHash = p.avatarHash;
  if (p.user) existing.user = p.user;
  world3d.upsertRemotePlayer(existing);

  if (prevZone !== nextZone) {
    renderFriendsList();
  }
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
      if (Array.isArray(ev.itemCatalog)) {
        setItemCatalog(ev.itemCatalog);
      }
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      if (ev.parentalWorld) {
        applyParentalWorldSettings(ev.parentalWorld);
      }
      if (ev.moderation || ev.self?.user?.moderation) {
        applyModerationSnapshot(ev.moderation || ev.self.user.moderation);
      }
      if (ev.dungeon) {
        world3d.syncDungeonSnapshot(ev.dungeon);
      }
      if (ev.prison) {
        world3d.syncPrisonSnapshot(ev.prison, isSelfEligibleForPrisonRelease());
      }
      if (ev.self && typeof ev.self.zone === 'number' && ev.self.zone !== currentZone) {
        applyLocalZoneTransition({
          zone: ev.self.zone,
          x: ev.self.x,
          y: ev.self.y,
          z: ev.self.z,
          dungeon: ev.dungeon,
          prison: ev.prison
        });
      }
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
      if (ev.selfId === selfId && ev.user) {
        selfUser = ev.user;
        if (playersMap.has(selfId)) {
          playersMap.get(selfId).user = selfUser;
        }
        if (world3d.selfAvatar) {
          world3d.selfAvatar.updateAvatarAppearance(selfUser.avatar);
        }
        if (isSalonOpen && draftAvatar && selfUser.avatar) {
          draftAvatar.hat = selfUser.avatar.hat;
          draftAvatar.accessory = selfUser.avatar.accessory;
          syncSalonControlsFromDraft();
        }
      } else if (ev.selfId && ev.selfId !== selfId && ev.user) {
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
    case 'player_zone': {
      if (ev.id && ev.id !== selfId) {
        const existing = playersMap.get(ev.id);
        if (existing) {
          existing.zone = typeof ev.zone === 'number' ? ev.zone : ZONE_PLAZA;
          if (typeof ev.x === 'number') existing.x = ev.x;
          if (typeof ev.y === 'number') existing.y = ev.y;
          if (typeof ev.z === 'number') existing.z = ev.z;
        }
        world3d.setRemotePlayerZone(
          ev.id,
          typeof ev.zone === 'number' ? ev.zone : ZONE_PLAZA,
          ev
        );
        renderFriendsList();
      }
      break;
    }
    case 'zone_transitioned': {
      applyLocalZoneTransition(ev);
      break;
    }
    case 'dungeon_mobs': {
      if (Array.isArray(ev.mobs)) {
        world3d.syncDungeonMobs(ev.mobs);
      }
      break;
    }
    case 'dungeon_sync': {
      if (ev.dungeon) {
        world3d.syncDungeonSnapshot(ev.dungeon);
      }
      break;
    }
    case 'prison_sync': {
      if (ev.prison) {
        world3d.syncPrisonSnapshot(ev.prison, isSelfEligibleForPrisonRelease());
      }
      break;
    }
    case 'moderation_update': {
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      if (ev.moderation) {
        applyModerationSnapshot(ev.moderation);
      }
      if (ev.reason === 'admin:pardon_prison') {
        showToast('🕊️ You received a Parent Pardon! Debt cleared — welcome back to Starlight Plaza!', 5200);
      } else if (ev.reason === 'admin:update_controls') {
        showToast('🛡️ Parental safety settings updated for your account.');
      }
      break;
    }
    case 'parental_settings_update': {
      if (ev.parentalWorld) {
        applyParentalWorldSettings(ev.parentalWorld);
        showToast('🛡️ Parent updated global world safety settings.');
      }
      break;
    }
    case 'economy_update': {
      const prevCoins = playerEconomy.coins || 0;
      const prevInv = { ...(playerEconomy.inventory || {}) };
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      if (typeof ev.reason === 'string' && ev.reason.startsWith('pickup:') && ev.economy) {
        const coinGain = Math.max(0, (ev.economy.coins || 0) - prevCoins);
        const gainedParts = [];
        if (coinGain > 0) gainedParts.push(`🪙 +${coinGain} Coins`);
        for (const [iid, qty] of Object.entries(ev.economy.inventory || {})) {
          const delta = Number(qty) - Number(prevInv[iid] || 0);
          if (delta > 0) {
            const def = catalogById.get(iid);
            gainedParts.push(`${def ? def.icon : '✨'} +${delta}x ${def ? def.name : iid}`);
          }
        }
        if (gainedParts.length > 0) {
          showToast(`✨ Collected ${gainedParts.join(' • ')}!`, 3200);
        }
      }
      break;
    }
    case 'item_used': {
      if (typeof ev.speedBoostUntilMs === 'number') {
        activeSpeedBoostUntilMs = ev.speedBoostUntilMs;
      }
      if (typeof ev.jumpBoostUntilMs === 'number') {
        activeJumpBoostUntilMs = ev.jumpBoostUntilMs;
      }
      world3d.setBuffTimers({
        speedBoostUntilMs: activeSpeedBoostUntilMs,
        jumpBoostUntilMs: activeJumpBoostUntilMs
      });
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      showToast(`🧪 Used ${ev.itemName || 'item'}!`, 3600);
      break;
    }
    case 'trade_invite_received': {
      if (ev.trade) {
        pendingInviteTrade = ev.trade;
        showTradeInvitePopover(ev.trade);
      }
      break;
    }
    case 'trade_invite_sent': {
      if (ev.trade) {
        showToast(`🤝 Trade invite sent to ${ev.trade.partnerDisplayName}!`);
      }
      break;
    }
    case 'trade_updated': {
      if (ev.trade) {
        if (pendingInviteTrade && pendingInviteTrade.tradeId === ev.trade.tradeId) {
          hideTradeInvitePopover();
          pendingInviteTrade = null;
        }
        applyTradeSessionState(ev.trade);
      }
      break;
    }
    case 'trade_completed': {
      const partnerName =
        ev.trade && ev.trade.initiatorId === selfId
          ? ev.trade.partnerDisplayName
          : ev.trade?.initiatorDisplayName || 'your friend';
      activeTrade = null;
      draftTradeCoins = 0;
      draftTradeItems = {};
      if (tradeDialog && tradeDialog.open) {
        tradeDialog.close('completed');
      }
      if (world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      showToast(`🎉 Trade completed with ${partnerName}! Check your Backpack (I)!`, 5200);
      break;
    }
    case 'trade_cancelled': {
      if (pendingInviteTrade && pendingInviteTrade.tradeId === ev.tradeId) {
        hideTradeInvitePopover();
        pendingInviteTrade = null;
      }
      if (activeTrade && activeTrade.tradeId === ev.tradeId) {
        activeTrade = null;
        draftTradeCoins = 0;
        draftTradeItems = {};
        if (tradeDialog && tradeDialog.open) {
          tradeDialog.close('cancelled');
        }
        showToast(`🤝 ${ev.reason || 'Trade cancelled.'}`);
      } else if (ev.reason && ev.trade?.initiatorId === selfId) {
        showToast(`🤝 ${ev.reason}`);
      }
      break;
    }
    case 'action_error': {
      if (ev.error) {
        showToast(`⚠️ ${ev.error}`);
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

// ============================================================================
// Crystal Cavern Dungeon, Economy, Parental Controls & Debtor's Prison Client
// ============================================================================

function setItemCatalog(items) {
  itemCatalog = Array.isArray(items) ? items : [];
  catalogById.clear();
  for (const item of itemCatalog) {
    if (item && item.id) {
      catalogById.set(item.id, item);
    }
  }
  renderEconomyHud();
}

function applyEconomySnapshot(econ) {
  if (!econ || typeof econ !== 'object') return;
  playerEconomy = {
    coins: Number(econ.coins || 0),
    inventory: { ...(econ.inventory || {}) }
  };
  renderEconomyHud();
}

function applyModerationSnapshot(mod) {
  if (!mod || typeof mod !== 'object') return;
  playerModeration = {
    ...playerModeration,
    ...mod
  };
  if (selfUser) {
    selfUser.moderation = { ...playerModeration };
  }
  applyModerationAndParentalUi();
  renderPrisonBanner();
  renderFriendsList();
  world3d.setPrisonReleaseEligible(isSelfEligibleForPrisonRelease());
}

function applyParentalWorldSettings(pw) {
  if (!pw || typeof pw !== 'object') return;
  parentalWorldSettings = {
    ...parentalWorldSettings,
    ...pw
  };
  applyModerationAndParentalUi();
}

function applyModerationAndParentalUi() {
  const forceVoiceMuted =
    parentalWorldSettings.globalVoiceEnabled === false ||
    Boolean(playerModeration.voiceMutedByParent);
  voiceManager.setServerForceMuted(forceVoiceMuted);
  if (micToggleBtn) {
    micToggleBtn.classList.toggle('is-parent-locked', forceVoiceMuted);
    micToggleBtn.setAttribute('aria-disabled', String(forceVoiceMuted));
    if (!parentalWorldSettings.globalVoiceEnabled) {
      micToggleBtn.title = 'Global Voice Chat is currently turned off by a parent';
    } else if (playerModeration.voiceMutedByParent) {
      micToggleBtn.title = 'Your microphone is muted by a parent';
    } else {
      micToggleBtn.title = 'Toggle Microphone (Shortcut: M)';
    }
  }
  updateLocalMicUI(forceVoiceMuted ? true : localVoiceMuted, forceVoiceMuted ? false : localVoiceSpeaking);

  const effChat = effectiveChatModeForSelf();
  if (chatInput && chatSendBtn) {
    if (effChat === 'muted') {
      chatInput.disabled = true;
      chatInput.placeholder = '🔇 Text chat is muted by a parent';
      chatSendBtn.disabled = true;
    } else if (effChat === 'quick_only') {
      chatInput.disabled = true;
      chatInput.placeholder = '🛡️ Safe Quick-Chat Only — tap a preset phrase above!';
      chatSendBtn.disabled = true;
    } else {
      chatInput.disabled = false;
      chatInput.placeholder = 'Say something (shows bubble above your head)...';
      chatSendBtn.disabled = false;
    }
  }
  for (const rBtn of document.querySelectorAll('.reaction-btn')) {
    rBtn.disabled = effChat === 'muted';
  }

  const buildForbidden = !playerModeration.canBuild || currentZone !== ZONE_PLAZA;
  if (toggleBuildBtn) {
    toggleBuildBtn.classList.toggle('is-locked-chip', !playerModeration.canBuild);
  }
  if (buildForbidden && world3d.buildMode) {
    world3d.buildMode = false;
    world3d.ghostMesh.visible = false;
    if (buildToolbar) buildToolbar.hidden = true;
    if (toggleBuildBtn) toggleBuildBtn.classList.remove('active');
  }
}

function renderPrisonBanner() {
  if (!prisonSentenceBanner) return;
  const inPrison = currentZone === ZONE_PRISON;
  prisonSentenceBanner.hidden = !inPrison;
  if (!inPrison) return;

  const coins = Number(playerEconomy.coins || 0);
  const chores = Number(playerModeration.prisonTasksRemaining || 0);
  const ready = isSelfEligibleForPrisonRelease();

  if (prisonBannerTitle) {
    prisonBannerTitle.textContent = ready
      ? '🔓 Sentence Served! Walk Through the South Release Gate!'
      : `Community Service Active${playerModeration.violationStrikes ? ` • Strike #${playerModeration.violationStrikes}` : ''}`;
  }
  if (prisonBannerReason) {
    prisonBannerReason.textContent = ready
      ? 'Great job clearing your debt and chores! Step into the glowing green South Release Gate or click 🔓 Release Gate above.'
      : playerModeration.prisonReason ||
        'Keep ELLA World kind! Press E at glowing courtyard chores (+5 🪙 toward negative debt) or ask a parent for a pardon.';
  }
  if (prisonDebtPill) {
    prisonDebtPill.classList.toggle('is-debt', coins < 0);
    prisonDebtPill.classList.toggle('is-cleared', coins >= 0);
    prisonDebtPill.textContent =
      coins < 0 ? `🪙 Debt Balance: ${coins.toLocaleString()}` : `✅ Debt Cleared (${coins.toLocaleString()} 🪙)`;
  }
  if (prisonChoresPill) {
    prisonChoresPill.classList.toggle('is-cleared', chores === 0);
    prisonChoresPill.textContent =
      chores > 0 ? `🪨 Chores Left: ${chores}` : '✅ All Chores Complete';
  }
  if (prisonGatePill) {
    prisonGatePill.classList.toggle('is-cleared', ready);
    prisonGatePill.textContent = ready ? '🔓 South Gate Unlocked!' : '🔒 Gate Locked';
  }
}

function renderEconomyHud() {
  const coins = Number(playerEconomy.coins || 0);
  const inDebt = coins < 0;
  const inv = playerEconomy.inventory || {};
  let totalItems = 0;
  for (const qty of Object.values(inv)) {
    totalItems += Number(qty || 0);
  }

  if (hudCoinsPill) hudCoinsPill.classList.toggle('is-in-debt', inDebt);
  if (hudCoinsCount) hudCoinsCount.textContent = coins.toLocaleString();
  if (hudItemCount) hudItemCount.textContent = String(totalItems);
  if (backpackCoinsValue) {
    backpackCoinsValue.classList.toggle('is-in-debt', inDebt);
    backpackCoinsValue.textContent = `🪙 ${coins.toLocaleString()}`;
  }
  if (tradeSelfWallet) tradeSelfWallet.textContent = coins.toLocaleString();
  if (tradeCoinsInput) tradeCoinsInput.max = String(Math.max(0, coins));

  world3d.setPrisonReleaseEligible(isSelfEligibleForPrisonRelease());
  renderPrisonBanner();

  // Sync Glam Salon exclusive relic lock/unlock badges
  for (const card of avatarDialog.querySelectorAll('.glam-item-card[data-req-item]')) {
    const reqId = card.getAttribute('data-req-item');
    const qty = Number(inv[reqId] || 0);
    const owned = qty >= 1;
    card.classList.toggle('is-locked', !owned);
    const badge = card.querySelector('[data-lock-badge]');
    if (badge) {
      badge.classList.toggle('is-owned', owned);
      badge.textContent = owned ? `✅ Owned (${qty}x)` : '🔒 Farm / Trade';
    }
  }

  if (backpackDialog && backpackDialog.open) {
    renderBackpackGrid();
  }
  if (tradeDialog && tradeDialog.open && activeTrade) {
    renderTradeModal();
  }
}

function renderZoneHud(zone) {
  const inDungeon = zone === ZONE_DUNGEON;
  const inPrison = zone === ZONE_PRISON;
  if (hudZonePill) {
    hudZonePill.classList.toggle('is-dungeon', inDungeon);
    hudZonePill.classList.toggle('is-prison', inPrison);
    hudZonePill.textContent = inPrison
      ? "⚖️ Debtor's Prison"
      : inDungeon
        ? '🏰 Crystal Cavern'
        : '⛲ Starlight Plaza';
  }
  if (warpDungeonBtn) {
    warpDungeonBtn.textContent = inPrison
      ? '🔓 Release Gate'
      : inDungeon
        ? '⛲ Exit Dungeon'
        : '🌀 Dungeon';
  }
  if (touchActionBtn) {
    touchActionBtn.textContent = inPrison ? '🪨 WORK (E)' : '✨ BLAST (E)';
  }
  if ((inDungeon || inPrison) && world3d.buildMode) {
    world3d.buildMode = false;
    world3d.ghostMesh.visible = false;
    if (buildToolbar) buildToolbar.hidden = true;
    if (toggleBuildBtn) toggleBuildBtn.classList.remove('active');
  }
  if (runwayHudBar) {
    runwayHudBar.hidden = inPrison;
    if (inDungeon || inPrison) {
      runwayHudBar.classList.remove('near-runway');
    }
  }
  renderPrisonBanner();
}

function applyLocalZoneTransition(payload) {
  if (!payload || typeof payload.zone !== 'number') return;
  const prevZone = currentZone;
  const nextZone = payload.zone;
  const changed = nextZone !== currentZone;
  currentZone = nextZone;
  latestMoveState.zone = nextZone;
  if (typeof payload.x === 'number') latestMoveState.x = payload.x;
  if (typeof payload.y === 'number') latestMoveState.y = payload.y;
  if (typeof payload.z === 'number') latestMoveState.z = payload.z;

  if (selfId && playersMap.has(selfId)) {
    const me = playersMap.get(selfId);
    me.zone = nextZone;
    if (typeof payload.x === 'number') me.x = payload.x;
    if (typeof payload.y === 'number') me.y = payload.y;
    if (typeof payload.z === 'number') me.z = payload.z;
  }

  world3d.setZone(nextZone, payload);
  if (payload.dungeon) {
    world3d.syncDungeonSnapshot(payload.dungeon);
  }
  if (payload.prison) {
    world3d.syncPrisonSnapshot(payload.prison, isSelfEligibleForPrisonRelease());
  }
  if (payload.moderation) {
    playerModeration = {
      ...playerModeration,
      ...payload.moderation
    };
  } else if (prevZone === ZONE_PRISON && nextZone === ZONE_PLAZA) {
    playerModeration.inDebtorsPrison = false;
    playerModeration.prisonReason = null;
    playerModeration.prisonTasksRemaining = 0;
  }
  renderZoneHud(nextZone);
  applyModerationAndParentalUi();
  renderFriendsList();

  if (nextZone === ZONE_PLAZA && pendingPostPortalWarp) {
    const target = pendingPostPortalWarp;
    pendingPostPortalWarp = null;
    if (target === 'runway') {
      world3d.warpToRunway();
      showToast('💃 Returned to Starlight Plaza & teleported to the Fashion Runway!');
      return;
    }
    if (target === 'obby') {
      world3d.warpToObbyCheckpoint();
      showToast('🏃‍♀️ Returned to Starlight Plaza & teleported to the Sky Obby!');
      return;
    }
  }

  if (changed) {
    if (nextZone === ZONE_PRISON) {
      showToast(
        "⚖️ Sentenced to Debtor's Prison! Walk to glowing courtyard chores and press E (+5 🪙 toward debt) to earn your release!",
        6000
      );
    } else if (prevZone === ZONE_PRISON && nextZone === ZONE_PLAZA) {
      if (world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      showToast('🕊️ Released from Debtor\'s Prison! Welcome back to Starlight Glam Plaza!', 5200);
    } else {
      showToast(
        nextZone === ZONE_DUNGEON
          ? '🏰 Entered the Crystal Cavern! Press E or Jump-Stomp Slimes & Crystal Veins for loot!'
          : '⛲ Returned to Starlight Glam Plaza!'
      );
    }
  }
}

async function performPrisonWork(nodeId) {
  if (!sessionToken || !nodeId || prisonWorkInFlight) return;
  prisonWorkInFlight = true;
  try {
    const res = await fetch('/api/ella-world/prison/work', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, nodeId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.economy) applyEconomySnapshot(data.economy);
      if (data.moderation) applyModerationSnapshot(data.moderation);
      if (data.prison) {
        world3d.syncPrisonSnapshot(
          data.prison,
          Boolean(data.readyForRelease || isSelfEligibleForPrisonRelease())
        );
      }
      if (data.readyForRelease || isSelfEligibleForPrisonRelease()) {
        if (world3d.selfAvatar) {
          world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
        }
        showToast(
          '🔓 Debt cleared & community service complete! Walk through the glowing green South Release Gate!',
          5600
        );
      } else {
        const gainText =
          Number(data.earnedTowardDebt || 0) > 0 ? `+${data.earnedTowardDebt} 🪙 toward debt • ` : '';
        showToast(
          `🪨 Completed ${data.stationLabel || 'Chore'}! ${gainText}Balance: ${Number(playerEconomy.coins || 0)} 🪙 • Chores left: ${Number(playerModeration.prisonTasksRemaining || 0)}`,
          4200
        );
      }
    } else if (data.error) {
      showToast(`⚖️ ${data.error}`);
    }
  } catch {
    // ignore transient network error
  } finally {
    prisonWorkInFlight = false;
  }
}

async function requestZoneTransition(targetZone) {
  if (!sessionToken || zonePortalInFlight) return;
  zonePortalInFlight = true;
  try {
    const res = await fetch('/api/ella-world/dungeon/portal', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, zone: targetZone, targetZone })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok && typeof data.zone === 'number') {
      applyLocalZoneTransition(data);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    // fallback handled on next try
  } finally {
    zonePortalInFlight = false;
  }
}

async function interactDungeonNode(nodeId) {
  if (!sessionToken || !nodeId) return;
  try {
    const res = await fetch('/api/ella-world/dungeon/interact', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, nodeId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.dungeon) world3d.syncDungeonSnapshot(data.dungeon);
      if (data.economy) applyEconomySnapshot(data.economy);
    } else if (data.error) {
      showToast(`🗝️ ${data.error}`);
    }
  } catch {
    // ignore transient network error
  }
}

async function attackDungeonMob(mobId, mode = 'blast') {
  if (!sessionToken || !mobId) return;
  try {
    const res = await fetch('/api/ella-world/dungeon/attack', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, mobId, mode })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok && data.dungeon) {
      world3d.syncDungeonSnapshot(data.dungeon);
    } else if (!res.ok && data.error) {
      showToast(`⚔️ ${data.error}`);
    }
  } catch {
    // ignore transient network error
  }
}

async function pickupDungeonDrop(dropId) {
  if (!sessionToken || !dropId || pickupInFlightIds.has(dropId)) return;
  pickupInFlightIds.add(dropId);
  try {
    const res = await fetch('/api/ella-world/dungeon/pickup', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, dropId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.dungeon) world3d.syncDungeonSnapshot(data.dungeon);
    }
  } catch {
    // ignore transient network error
  } finally {
    setTimeout(() => pickupInFlightIds.delete(dropId), 800);
  }
}

async function useInventoryItem(itemId) {
  if (!sessionToken || !itemId) return;
  try {
    const res = await fetch('/api/ella-world/economy/use', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, itemId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok && data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    showToast('⚠️ Could not use item right now.');
  }
}

function openBackpackModal() {
  renderBackpackGrid();
  if (backpackDialog && !backpackDialog.open) {
    backpackDialog.showModal();
  }
}

function renderBackpackGrid() {
  if (!backpackItemsGrid) return;
  backpackItemsGrid.innerHTML = '';
  const inv = playerEconomy.inventory || {};

  // Show owned items first, followed by unowned catalog items as a collection guide
  const ordered = [...itemCatalog].sort((a, b) => {
    const qa = Number(inv[a.id] || 0) > 0 ? 1 : 0;
    const qb = Number(inv[b.id] || 0) > 0 ? 1 : 0;
    if (qa !== qb) return qb - qa;
    return (b.suggestedCoins || 0) - (a.suggestedCoins || 0);
  });

  for (const item of ordered) {
    const qty = Number(inv[item.id] || 0);
    const card = document.createElement('article');
    card.className = `backpack-card rarity-${item.rarity || 'common'}${qty === 0 ? ' is-unowned' : ''}`;

    const top = document.createElement('div');
    top.className = 'backpack-card-top';

    const iconSpan = document.createElement('span');
    iconSpan.className = 'backpack-icon';
    iconSpan.textContent = item.icon || '✨';

    const qtyPill = document.createElement('span');
    qtyPill.className = 'backpack-qty-pill';
    qtyPill.textContent = qty > 0 ? `${qty}x Owned` : '0x (Unfound)';

    top.append(iconSpan, qtyPill);

    const mid = document.createElement('div');
    const rarityTag = document.createElement('div');
    rarityTag.className = 'backpack-rarity-tag';
    rarityTag.textContent = `${item.rarity || 'common'} • ${item.category || 'material'}`;
    const titleEl = document.createElement('h3');
    titleEl.className = 'backpack-card-title';
    titleEl.textContent = item.name;
    const descEl = document.createElement('p');
    descEl.className = 'backpack-desc';
    descEl.textContent = item.description || '';
    mid.append(rarityTag, titleEl, descEl);

    const actions = document.createElement('div');
    actions.className = 'backpack-card-actions';

    const valHint = document.createElement('small');
    valHint.className = 'trade-wallet-hint';
    valHint.textContent = `~${item.suggestedCoins || 10} 🪙 value`;
    actions.appendChild(valHint);

    if (qty > 0 && item.category === 'consumable') {
      const useBtn = document.createElement('button');
      useBtn.type = 'button';
      useBtn.className = 'small-action-btn';
      useBtn.textContent = item.id === 'prism_block' ? '🌈 Place Block' : '🧪 Drink Potion';
      useBtn.addEventListener('click', () => useInventoryItem(item.id));
      actions.appendChild(useBtn);
    } else if (qty > 0 && item.category === 'cosmetic') {
      const wearBtn = document.createElement('button');
      wearBtn.type = 'button';
      wearBtn.className = 'small-action-btn';
      wearBtn.textContent = '👗 Equip in Glam';
      wearBtn.addEventListener('click', () => {
        if (backpackDialog && backpackDialog.open) backpackDialog.close();
        openGlamSalon();
        const accTab = document.getElementById('glam-tab-acc');
        if (accTab) activateGlamTab(accTab, false);
        if (draftAvatar && item.cosmeticSlot && item.cosmeticValue) {
          draftAvatar[item.cosmeticSlot] = item.cosmeticValue;
          draftAvatar.characterPreset = 'custom';
          previewDraftInMirror();
          scheduleAutoSaveAvatar(120);
        }
      });
      actions.appendChild(wearBtn);
    }

    card.append(top, mid, actions);
    backpackItemsGrid.appendChild(card);
  }
}

function showTradeInvitePopover(trade) {
  if (!tradeInvitePopover || !trade) return;
  if (tradeInviteText) {
    tradeInviteText.textContent = `${trade.initiatorDisplayName} (@${trade.initiatorUsername}) wants to trade items & coins with you!`;
  }
  if (typeof tradeInvitePopover.showPopover === 'function') {
    try {
      tradeInvitePopover.showPopover();
    } catch {
      tradeInvitePopover.setAttribute('open', '');
    }
  } else {
    tradeInvitePopover.setAttribute('open', '');
  }
}

function hideTradeInvitePopover() {
  if (!tradeInvitePopover) return;
  if (typeof tradeInvitePopover.hidePopover === 'function') {
    try {
      tradeInvitePopover.hidePopover();
    } catch {
      tradeInvitePopover.removeAttribute('open');
    }
  } else {
    tradeInvitePopover.removeAttribute('open');
  }
}

async function sendTradeInviteToPeer(targetPeerId, displayName) {
  if (!sessionToken || !targetPeerId) return;
  try {
    const res = await fetch('/api/ella-world/trade/invite', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, targetPeerId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      showToast(`⚠️ ${data.error || `Could not invite ${displayName}.`}`);
    }
  } catch {
    showToast('⚠️ Could not reach trade server.');
  }
}

async function respondToPendingTradeInvite(accept) {
  if (!pendingInviteTrade || !sessionToken) {
    hideTradeInvitePopover();
    return;
  }
  const tradeId = pendingInviteTrade.tradeId;
  hideTradeInvitePopover();
  pendingInviteTrade = null;
  try {
    const res = await fetch('/api/ella-world/trade/respond', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, tradeId, accept: Boolean(accept) })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok && accept && data.trade) {
      applyTradeSessionState(data.trade);
    } else if (!res.ok && data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    showToast('⚠️ Could not respond to trade invite.');
  }
}

function getMyTradeSide(trade) {
  if (!trade) return null;
  return trade.initiatorId === selfId ? trade.initiatorOffer : trade.partnerOffer;
}

function getPartnerTradeSide(trade) {
  if (!trade) return null;
  return trade.initiatorId === selfId ? trade.partnerOffer : trade.initiatorOffer;
}

function applyTradeSessionState(trade) {
  if (!trade || trade.status !== 'active') return;
  const isNewSession = !activeTrade || activeTrade.tradeId !== trade.tradeId;
  activeTrade = trade;

  const mySide = getMyTradeSide(trade);
  if (isNewSession || !tradeOfferDebounceTimer) {
    draftTradeCoins = Number(mySide?.coins || 0);
    draftTradeItems = { ...(mySide?.items || {}) };
  }

  if (isSalonOpen) {
    closeGlamSalon(false);
  }
  if (backpackDialog && backpackDialog.open) {
    backpackDialog.close();
  }
  if (tradeDialog && !tradeDialog.open) {
    tradeDialog.showModal();
  }
  renderTradeModal();
}

function formatStatePill(pillEl, cardEl, side) {
  if (!pillEl || !side) return;
  pillEl.classList.remove('locked', 'confirmed');
  if (cardEl) cardEl.classList.remove('is-locked', 'is-confirmed');
  if (side.confirmed) {
    pillEl.classList.add('confirmed');
    pillEl.textContent = '✅ Confirmed';
    if (cardEl) cardEl.classList.add('is-confirmed');
  } else if (side.locked) {
    pillEl.classList.add('locked');
    pillEl.textContent = '🔒 Locked In';
    if (cardEl) cardEl.classList.add('is-locked');
  } else {
    pillEl.textContent = '✏️ Editing...';
  }
}

function renderTradeModal() {
  if (!activeTrade || !tradeDialog) return;
  const isInitiator = activeTrade.initiatorId === selfId;
  const partnerDisplay = isInitiator
    ? activeTrade.partnerDisplayName
    : activeTrade.initiatorDisplayName;
  const partnerUser = isInitiator ? activeTrade.partnerUsername : activeTrade.initiatorUsername;
  const mySide = getMyTradeSide(activeTrade);
  const partnerSide = getPartnerTradeSide(activeTrade);

  if (tradeSelfHeading) {
    tradeSelfHeading.textContent = `Your Offer (@${selfUser?.username || 'you'})`;
  }
  if (tradePartnerHeading) {
    tradePartnerHeading.textContent = `${partnerDisplay}'s Offer (@${partnerUser})`;
  }
  if (tradeStatusNote) {
    tradeStatusNote.textContent =
      activeTrade.statusNote ||
      'Add items or coins, then both friends click Lock Offer before Confirming!';
  }

  const selfCard = tradeDialog.querySelector('.trade-side-card.is-self');
  const partnerCard = tradeDialog.querySelector('.trade-side-card.is-partner');
  formatStatePill(tradeSelfStatePill, selfCard, mySide);
  formatStatePill(tradePartnerStatePill, partnerCard, partnerSide);

  if (tradeSelfWallet) {
    tradeSelfWallet.textContent = Number(playerEconomy.coins || 0).toLocaleString();
  }
  if (tradeCoinsInput && document.activeElement !== tradeCoinsInput) {
    tradeCoinsInput.value = String(draftTradeCoins || 0);
  }
  if (tradeCoinsInput) {
    tradeCoinsInput.max = String(Math.max(0, Number(playerEconomy.coins || 0)));
  }

  // Render Self Offered Items
  if (tradeSelfOfferList) {
    tradeSelfOfferList.innerHTML = '';
    const entries = Object.entries(draftTradeItems).filter(([, q]) => Number(q) > 0);
    if (entries.length === 0) {
      const empty = document.createElement('span');
      empty.className = 'small-note';
      empty.textContent = 'No items added yet.';
      tradeSelfOfferList.appendChild(empty);
    } else {
      for (const [itemId, qty] of entries) {
        const def = catalogById.get(itemId);
        const chip = document.createElement('span');
        chip.className = 'trade-offer-chip';
        chip.textContent = `${def ? def.icon : '✨'} ${qty}x ${def ? def.name : itemId} `;
        const minusBtn = document.createElement('button');
        minusBtn.type = 'button';
        minusBtn.className = 'trade-chip-remove-btn';
        minusBtn.title = 'Remove 1 from offer';
        minusBtn.textContent = '−1';
        minusBtn.addEventListener('click', () => {
          const next = Math.max(0, Number(draftTradeItems[itemId] || 0) - 1);
          if (next === 0) delete draftTradeItems[itemId];
          else draftTradeItems[itemId] = next;
          renderTradeModal();
          schedulePushTradeOffer(80);
        });
        chip.appendChild(minusBtn);
        tradeSelfOfferList.appendChild(chip);
      }
    }
  }

  // Render Self Inventory Picker
  if (tradeInventoryPicker) {
    tradeInventoryPicker.innerHTML = '';
    const inv = playerEconomy.inventory || {};
    const ownedIds = Object.keys(inv).filter((k) => Number(inv[k]) > 0);
    if (ownedIds.length === 0) {
      const hint = document.createElement('span');
      hint.className = 'small-note';
      hint.textContent = 'Your backpack has no items yet (you can still offer coins!).';
      tradeInventoryPicker.appendChild(hint);
    } else {
      for (const itemId of ownedIds) {
        const ownedQty = Number(inv[itemId] || 0);
        const offeredQty = Number(draftTradeItems[itemId] || 0);
        const remQty = Math.max(0, ownedQty - offeredQty);
        const def = catalogById.get(itemId);
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'trade-pick-btn';
        btn.disabled = remQty <= 0;
        btn.textContent = `${def ? def.icon : '✨'} +1 ${def ? def.name : itemId} (${remQty} left)`;
        btn.addEventListener('click', () => {
          if (remQty <= 0) return;
          draftTradeItems[itemId] = offeredQty + 1;
          renderTradeModal();
          schedulePushTradeOffer(80);
        });
        tradeInventoryPicker.appendChild(btn);
      }
    }
  }

  // Render Partner Offer
  if (tradePartnerCoins) {
    tradePartnerCoins.textContent = `🪙 ${Number(partnerSide?.coins || 0).toLocaleString()}`;
  }
  if (tradePartnerOfferList) {
    tradePartnerOfferList.innerHTML = '';
    const pEntries = Object.entries(partnerSide?.items || {}).filter(([, q]) => Number(q) > 0);
    if (pEntries.length === 0) {
      const empty = document.createElement('span');
      empty.className = 'small-note';
      empty.textContent = 'No items offered by friend yet.';
      tradePartnerOfferList.appendChild(empty);
    } else {
      for (const [itemId, qty] of pEntries) {
        const def = catalogById.get(itemId);
        const chip = document.createElement('span');
        chip.className = 'trade-offer-chip';
        chip.textContent = `${def ? def.icon : '✨'} ${qty}x ${def ? def.name : itemId}`;
        tradePartnerOfferList.appendChild(chip);
      }
    }
  }

  const bothLocked = Boolean(mySide?.locked && partnerSide?.locked);
  if (tradeLockBtn) {
    tradeLockBtn.textContent = mySide?.locked ? '🔓 Unlock Offer' : '🔒 Lock Offer';
  }
  if (tradeConfirmBtn) {
    tradeConfirmBtn.disabled = !bothLocked || Boolean(mySide?.confirmed);
    tradeConfirmBtn.textContent = mySide?.confirmed
      ? '⏳ Waiting for Friend...'
      : '✅ Confirm Trade';
  }
}

function schedulePushTradeOffer(delayMs = 160) {
  if (tradeOfferDebounceTimer) clearTimeout(tradeOfferDebounceTimer);
  tradeOfferDebounceTimer = setTimeout(() => {
    tradeOfferDebounceTimer = null;
    pushTradeOfferNow();
  }, delayMs);
}

async function pushTradeOfferNow() {
  if (!activeTrade || !sessionToken) return false;
  if (tradeOfferDebounceTimer) {
    clearTimeout(tradeOfferDebounceTimer);
    tradeOfferDebounceTimer = null;
  }
  const cleanCoins = Math.max(
    0,
    Math.min(Number(playerEconomy.coins || 0), Math.floor(Number(draftTradeCoins) || 0))
  );
  draftTradeCoins = cleanCoins;
  try {
    const res = await fetch('/api/ella-world/trade/offer', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({
        token: sessionToken,
        tradeId: activeTrade.tradeId,
        coins: cleanCoins,
        items: draftTradeItems
      })
    });
    if (await handleHttpAuthCheck(res)) return false;
    const data = await res.json().catch(() => ({}));
    if (res.ok && data.trade) {
      applyTradeSessionState(data.trade);
      return true;
    }
    if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
    return false;
  } catch {
    return false;
  }
}

async function toggleMyTradeLock() {
  if (!activeTrade || !sessionToken) return;
  if (tradeOfferDebounceTimer) {
    const ok = await pushTradeOfferNow();
    if (!ok) return;
  }
  const mySide = getMyTradeSide(activeTrade);
  const nextLocked = !mySide?.locked;
  try {
    const res = await fetch('/api/ella-world/trade/lock', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({
        token: sessionToken,
        tradeId: activeTrade.tradeId,
        locked: nextLocked
      })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok && data.trade) {
      applyTradeSessionState(data.trade);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    showToast('⚠️ Could not update lock state.');
  }
}

async function confirmActiveTrade() {
  if (!activeTrade || !sessionToken) return;
  try {
    const res = await fetch('/api/ella-world/trade/confirm', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, tradeId: activeTrade.tradeId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok && data.error) {
      showToast(`⚠️ ${data.error}`);
    } else if (res.ok && !data.completed && data.trade) {
      applyTradeSessionState(data.trade);
    }
  } catch {
    showToast('⚠️ Could not confirm trade right now.');
  }
}

async function cancelActiveTrade() {
  if (!activeTrade) {
    if (tradeDialog && tradeDialog.open) tradeDialog.close('cancelled');
    return;
  }
  const tradeId = activeTrade.tradeId;
  activeTrade = null;
  draftTradeCoins = 0;
  draftTradeItems = {};
  if (tradeDialog && tradeDialog.open) {
    tradeDialog.close('cancelled');
  }
  if (!sessionToken) return;
  await fetch('/api/ella-world/trade/cancel', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ token: sessionToken, tradeId })
  }).catch(() => {});
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
    const pZone = p.zone || ZONE_PLAZA;
    const inDungeon = pZone === ZONE_DUNGEON;
    const inPrison = pZone === ZONE_PRISON;
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

    const metaRow = document.createElement('div');
    metaRow.className = 'friend-meta-row';

    const zoneBadge = document.createElement('span');
    zoneBadge.className = `friend-zone-tag${inDungeon ? ' in-dungeon' : ''}${inPrison ? ' in-prison' : ''}`;
    zoneBadge.textContent = inPrison
      ? "⚖️ Debtor's Prison"
      : inDungeon
        ? '🏰 Crystal Cavern'
        : '⛲ Plaza';
    metaRow.appendChild(zoneBadge);

    const sameAccount =
      selfUser &&
      p.user?.username &&
      String(p.user.username).toLowerCase() === String(selfUser.username).toLowerCase();
    if (!isMe && !sameAccount) {
      const tradeBtn = document.createElement('button');
      tradeBtn.type = 'button';
      tradeBtn.className = 'friend-trade-btn';
      const tradeBlocked =
        currentZone === ZONE_PRISON ||
        inPrison ||
        !playerModeration.canTrade;
      tradeBtn.disabled = tradeBlocked;
      tradeBtn.textContent = tradeBlocked ? '🔒 Trade' : '🤝 Trade';
      tradeBtn.title =
        currentZone === ZONE_PRISON || inPrison
          ? "Trading is locked during a Debtor's Prison sentence"
          : !playerModeration.canTrade
            ? 'Trading is locked by a parent'
            : `Invite ${p.user.displayName} to swap items & coins`;
      tradeBtn.addEventListener('click', () => {
        sendTradeInviteToPeer(pid, p.user.displayName);
      });
      metaRow.appendChild(tradeBtn);
    }
    li.appendChild(metaRow);

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
  const parentLocked =
    parentalWorldSettings.globalVoiceEnabled === false ||
    Boolean(playerModeration.voiceMutedByParent);
  const effectiveMuted = parentLocked || muted;
  micToggleBtn.classList.toggle('mic-muted', effectiveMuted);
  micToggleBtn.classList.toggle('mic-unmuted', !effectiveMuted);
  micToggleBtn.classList.toggle('is-speaking', Boolean(speaking && !effectiveMuted));
  micIcon.textContent = parentLocked ? '🔒' : effectiveMuted ? '🔇' : '🎤';
  if (parentalWorldSettings.globalVoiceEnabled === false) {
    micLabel.textContent = 'Voice Off (Parent)';
  } else if (playerModeration.voiceMutedByParent) {
    micLabel.textContent = 'Muted by Parent';
  } else if (voiceManager.pushToTalk) {
    micLabel.textContent = effectiveMuted ? 'Hold V to Talk' : 'Talking (PTT)';
  } else {
    micLabel.textContent = effectiveMuted ? 'Mic Muted' : 'Mic On';
  }
  const scale = 1 + Math.min(1.6, level * 5);
  localSpeakingMeter.style.transform = `scale(${effectiveMuted ? 0.6 : scale})`;

  const dialogFill = document.getElementById('mic-dialog-meter-fill');
  if (dialogFill) {
    dialogFill.style.width = `${effectiveMuted ? 4 : Math.min(100, Math.round(level * 260) + 5)}%`;
  }
}

function updateVoiceModeBadge() {
  const isProx = proximityVoiceSwitch.checked;
  voiceModeBadge.textContent = isProx ? '3D Proximity' : 'Party Voice';
}

function notifyVoiceLockIfNeeded() {
  if (parentalWorldSettings.globalVoiceEnabled === false) {
    showToast('🔒 Live Voice Chat is currently turned off globally by a parent.');
    return true;
  }
  if (playerModeration.voiceMutedByParent) {
    showToast('🔒 Your microphone is currently muted by a parent.');
    return true;
  }
  return false;
}

// 8. Voice Controls Events
micToggleBtn.addEventListener('click', () => {
  if (notifyVoiceLockIfNeeded()) return;
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
    if (await handleHttpAuthCheck(res)) return;
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
  pendingPostPortalWarp = null;
  if (currentZone !== ZONE_PLAZA) {
    requestZoneTransition(ZONE_PLAZA);
  } else {
    world3d.warpToPlaza();
    showToast('⛲ Teleported to Central Hangout Plaza!');
  }
});

if (warpDungeonBtn) {
  warpDungeonBtn.addEventListener('click', () => {
    pendingPostPortalWarp = null;
    if (currentZone === ZONE_PRISON) {
      requestZoneTransition(ZONE_PLAZA);
      return;
    }
    requestZoneTransition(currentZone === ZONE_DUNGEON ? ZONE_PLAZA : ZONE_DUNGEON);
  });
}

document.getElementById('warp-runway-btn').addEventListener('click', () => {
  if (currentZone !== ZONE_PLAZA) {
    pendingPostPortalWarp = 'runway';
    requestZoneTransition(ZONE_PLAZA);
  } else {
    world3d.warpToRunway();
    showToast('💃 Teleported Backstage at the Dress to Impress Fashion Runway!');
  }
});

document.getElementById('warp-obby-btn').addEventListener('click', () => {
  if (currentZone !== ZONE_PLAZA) {
    pendingPostPortalWarp = 'obby';
    requestZoneTransition(ZONE_PLAZA);
  } else {
    world3d.warpToObbyCheckpoint();
    showToast('🏃‍♀️ Teleported to Sky Obby!');
  }
});

if (openBackpackBtn) {
  openBackpackBtn.addEventListener('click', () => {
    openBackpackModal();
  });
}

if (backpackEnterDungeonBtn) {
  backpackEnterDungeonBtn.addEventListener('click', () => {
    if (currentZone === ZONE_PRISON) {
      showToast("⚖️ Clear your Debtor's Prison chores & negative debt before warping to Crystal Cavern!");
      return;
    }
    if (backpackDialog && backpackDialog.open) backpackDialog.close();
    if (currentZone !== ZONE_DUNGEON) {
      requestZoneTransition(ZONE_DUNGEON);
    }
  });
}

if (tradeInviteAcceptBtn) {
  tradeInviteAcceptBtn.addEventListener('click', () => respondToPendingTradeInvite(true));
}
if (tradeInviteDeclineBtn) {
  tradeInviteDeclineBtn.addEventListener('click', () => respondToPendingTradeInvite(false));
}
if (tradeCoinsInput) {
  tradeCoinsInput.addEventListener('input', () => {
    draftTradeCoins = Math.max(0, Math.floor(Number(tradeCoinsInput.value) || 0));
    schedulePushTradeOffer(180);
  });
}
if (tradeLockBtn) {
  tradeLockBtn.addEventListener('click', () => toggleMyTradeLock());
}
if (tradeConfirmBtn) {
  tradeConfirmBtn.addEventListener('click', () => confirmActiveTrade());
}
if (tradeCancelBtn) {
  tradeCancelBtn.addEventListener('click', () => cancelActiveTrade());
}
if (tradeCloseCancelBtn) {
  tradeCloseCancelBtn.addEventListener('click', () => cancelActiveTrade());
}

// Dress to Impress Runway Theme Spin & 1-5 Star Voting
document.getElementById('runway-next-theme-btn').addEventListener('click', async () => {
  const res = await fetch('/api/ella-world/runway', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ token: sessionToken, action: 'theme' })
  }).catch(() => null);
  if (await handleHttpAuthCheck(res)) return;
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

    if (await handleHttpAuthCheck(res)) return;
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
  if (currentZone === ZONE_PRISON) {
    showToast("⚖️ Block building is locked while serving a Debtor's Prison sentence!");
    return;
  }
  if (!playerModeration.canBuild) {
    showToast('🔒 Block building is currently locked for your account by a parent.');
    return;
  }
  if (currentZone !== ZONE_PLAZA) {
    showToast('🔨 Co-Op Block Building is in Starlight Plaza! Exit the Dungeon (G) to build.');
    return;
  }
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
    if (notifyVoiceLockIfNeeded()) return;
    pttVoiceSwitch.checked = false;
    voiceManager.toggleMute();
  } else if (k === 'v') {
    if (notifyVoiceLockIfNeeded()) return;
    voiceManager.setPttKeyActive(true);
  } else if (k === 'c') {
    if (isSalonOpen) closeGlamSalon(false);
    else openGlamSalon();
  } else if (!isSalonOpen && k === 'i') {
    openBackpackModal();
  } else if (!isSalonOpen && k === 'g') {
    pendingPostPortalWarp = null;
    if (currentZone === ZONE_PRISON) {
      requestZoneTransition(ZONE_PLAZA);
    } else {
      requestZoneTransition(currentZone === ZONE_DUNGEON ? ZONE_PLAZA : ZONE_DUNGEON);
    }
  } else if (!isSalonOpen && k === 'b') {
    toggleBuildMode();
  } else if (!isSalonOpen && k === 'h') {
    pendingPostPortalWarp = null;
    if (currentZone !== ZONE_PLAZA) {
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToPlaza();
      showToast('⛲ Teleported to Central Hangout Plaza!');
    }
  } else if (!isSalonOpen && k === 'f') {
    if (currentZone !== ZONE_PLAZA) {
      pendingPostPortalWarp = 'runway';
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToRunway();
      showToast('💃 Teleported Backstage at the Fashion Runway!');
    }
  } else if (!isSalonOpen && k === 'r') {
    if (currentZone !== ZONE_PLAZA) {
      pendingPostPortalWarp = 'obby';
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToObbyCheckpoint();
      showToast('🏃‍♀️ Teleported to Sky Obby!');
    }
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
if (touchActionBtn) {
  touchActionBtn.addEventListener('pointerdown', (e) => {
    e.preventDefault();
    world3d.triggerAction();
  });
}

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

  const effectiveBodyType =
    draftAvatar.bodyType ||
    (draftAvatar.characterPreset === 'jack_tech49' || draftAvatar.characterPreset === 'maverick_cabin'
      ? 'male'
      : 'female');

  for (const card of avatarDialog.querySelectorAll('.glam-item-card[data-preset-id]')) {
    const isSel = card.getAttribute('data-preset-id') === draftAvatar.characterPreset;
    card.classList.toggle('selected', isSel);
    card.setAttribute('aria-pressed', String(isSel));
  }
  for (const card of avatarDialog.querySelectorAll('.glam-item-card[data-field]')) {
    const f = card.getAttribute('data-field');
    const v = card.getAttribute('data-val');
    const isSel = String(draftAvatar[f] || '') === v;
    card.classList.toggle('selected', isSel);
    card.setAttribute('aria-pressed', String(isSel));
  }
  for (const seg of avatarDialog.querySelectorAll('.seg-btn[data-field]')) {
    const f = seg.getAttribute('data-field');
    const v = seg.getAttribute('data-val');
    const curVal = f === 'bodyType' ? effectiveBodyType : String(draftAvatar[f] || '');
    const isAct = curVal === v;
    seg.classList.toggle('active', isAct);
    seg.setAttribute('aria-pressed', String(isAct));
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
    if (await handleHttpAuthCheck(res)) return false;
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
      showToast(`✨ Equipped ${presetCard.querySelector('strong')?.textContent || 'Signature Look'}!`);
    }
    return;
  }

  // 2. Modular Garment, Hair, Shoe, Hat & Accessory Cards
  const itemCard = target.closest('.glam-item-card[data-field]');
  if (itemCard) {
    const reqItem = itemCard.getAttribute('data-req-item');
    if (reqItem && Number((playerEconomy.inventory || {})[reqItem] || 0) < 1) {
      const def = catalogById.get(reqItem);
      showToast(
        `🔒 Farm or trade for "${def ? def.name : reqItem}" in the Crystal Cavern (G) to unlock this look!`
      );
      return;
    }
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

  // 3. Segmented Buttons (Character Build, Avatar Mode & Fabric Finish)
  const segBtn = target.closest('.seg-btn[data-field]');
  if (segBtn) {
    const field = segBtn.getAttribute('data-field');
    const val = segBtn.getAttribute('data-val');
    if (field && val !== null) {
      if (field === 'bodyType' && val === 'male' && draftAvatar.topStyle === 'glam_gown') {
        draftAvatar = {
          ...draftAvatar,
          ...SIGNATURE_GLAM_PRESETS.jack_tech49,
          avatarMode: draftAvatar.avatarMode || 'stylized'
        };
      } else if (
        field === 'bodyType' &&
        val === 'female' &&
        draftAvatar.topStyle === 'tech_jacket' &&
        draftAvatar.characterPreset === 'jack_tech49'
      ) {
        draftAvatar = {
          ...draftAvatar,
          ...SIGNATURE_GLAM_PRESETS.ella_idol,
          avatarMode: draftAvatar.avatarMode || 'stylized'
        };
      } else {
        draftAvatar[field] = val;
        if (field === 'bodyType') {
          draftAvatar.characterPreset = 'custom';
        }
      }
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
  hideTradeInvitePopover();
  pendingInviteTrade = null;
  activeTrade = null;
  draftTradeCoins = 0;
  draftTradeItems = {};
  if (tradeDialog && tradeDialog.open) tradeDialog.close('cancelled');
  if (backpackDialog && backpackDialog.open) backpackDialog.close();
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
  if (res.status === 401) {
    adminToken = '';
    adminAuthForm.hidden = false;
    adminDashboardContent.hidden = true;
    return;
  }
  if (res.ok) {
    const data = await res.json();
    renderAdminDashboard(data.dashboard);
  }
}

function showAdminSessionFeedback(message) {
  const banner = document.getElementById('admin-session-feedback');
  if (!banner) return;
  banner.textContent = message;
  banner.hidden = false;
}

async function adminModerateUser(payload, successMsg) {
  if (!adminToken) return;
  const res = await fetch('/api/ella-world/admin/moderate', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  if (res.status === 401) {
    adminToken = '';
    adminAuthForm.hidden = false;
    adminDashboardContent.hidden = true;
    return;
  }
  const out = await res.json().catch(() => ({}));
  if (res.ok) {
    if (out.dashboard) {
      renderAdminDashboard(out.dashboard);
    } else {
      await refreshAdminDashboard();
    }
    if (successMsg) {
      showAdminSessionFeedback(successMsg);
    }
  } else {
    showAdminSessionFeedback(`❌ ${out.error || 'Moderation update failed.'}`);
  }
}

function renderAdminDashboard(dashboard) {
  if (!dashboard) return;
  document.getElementById('admin-user-count').textContent = String((dashboard.users || []).length);
  document.getElementById('admin-block-count').textContent = String(dashboard.blockCount || 0);
  document.getElementById('admin-world-name').value = dashboard.worldName || '';
  document.getElementById('admin-default-voice').value = dashboard.voiceDefaultMode || 'party';

  const epochEl = document.getElementById('admin-session-epoch');
  if (epochEl) {
    epochEl.textContent = String(dashboard.sessionEpoch || 1);
  }

  const worldLockedEl = document.getElementById('admin-world-locked');
  if (worldLockedEl) {
    worldLockedEl.checked = Boolean(dashboard.worldLocked);
  }
  const globalVoiceEl = document.getElementById('admin-global-voice-enabled');
  if (globalVoiceEl) {
    globalVoiceEl.checked = dashboard.globalVoiceEnabled !== false;
  }
  const globalChatEl = document.getElementById('admin-global-chat-mode');
  if (globalChatEl) {
    globalChatEl.value = dashboard.globalChatMode || 'normal';
  }
  const curfewEnabledEl = document.getElementById('admin-curfew-enabled');
  if (curfewEnabledEl) {
    curfewEnabledEl.checked = Boolean(dashboard.curfewEnabled);
  }
  const curfewStartEl = document.getElementById('admin-curfew-start');
  if (curfewStartEl && dashboard.curfewStart) {
    curfewStartEl.value = dashboard.curfewStart;
  }
  const curfewEndEl = document.getElementById('admin-curfew-end');
  if (curfewEndEl && dashboard.curfewEnd) {
    curfewEndEl.value = dashboard.curfewEnd;
  }

  const filterCheckbox = document.getElementById('admin-chat-filter-enabled');
  if (filterCheckbox) {
    filterCheckbox.checked = dashboard.chatFilterEnabled !== false;
  }
  const fineBaseEl = document.getElementById('admin-profanity-fine-base');
  if (fineBaseEl && dashboard.profanityFineBase !== undefined) {
    fineBaseEl.value = String(dashboard.profanityFineBase);
  }
  const debtThreshEl = document.getElementById('admin-debt-prison-threshold');
  if (debtThreshEl && dashboard.debtPrisonThreshold !== undefined) {
    debtThreshEl.value = String(dashboard.debtPrisonThreshold);
  }
  const strikeThreshEl = document.getElementById('admin-prison-strike-threshold');
  if (strikeThreshEl && dashboard.prisonStrikeThreshold !== undefined) {
    strikeThreshEl.value = String(dashboard.prisonStrikeThreshold);
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
    const coins = Number(u.coins || 0);
    const strikes = Number(u.violationStrikes || 0);
    const inPrison = Boolean(u.inDebtorsPrison);

    // 1. Row Header (<th scope="row">): Friend Display Name + @username
    const thFriend = document.createElement('th');
    thFriend.scope = 'row';
    thFriend.className = 'admin-friend-cell';
    const nameStrong = document.createElement('strong');
    nameStrong.textContent = u.displayName || u.username;
    const handleSub = document.createElement('span');
    handleSub.className = 'admin-handle-sub';
    handleSub.textContent = `@${u.username}`;
    thFriend.append(nameStrong, handleSub);

    // 2. Coins / Negative Debt + +/-25 Quick Fine/Bonus Buttons
    const tdEcon = document.createElement('td');
    const econTop = document.createElement('div');
    econTop.className = 'admin-econ-top';
    const coinBadge = document.createElement('span');
    coinBadge.className = `coin-balance-pill${coins < 0 ? ' is-in-debt' : ''}`;
    coinBadge.textContent = `🪙 ${coins.toLocaleString()}`;
    const itemSub = document.createElement('span');
    itemSub.className = 'admin-item-sub';
    itemSub.textContent = `🎒 ${u.itemCount || 0}`;
    econTop.append(coinBadge, itemSub);

    const stepRow = document.createElement('div');
    stepRow.className = 'mod-coin-step-row';
    const minusBtn = document.createElement('button');
    minusBtn.type = 'button';
    minusBtn.className = 'mod-pill-btn warn-pill';
    minusBtn.textContent = '-25 🪙';
    minusBtn.title = `Deduct 25 coins from @${u.username}`;
    minusBtn.addEventListener('click', () =>
      adminModerateUser(
        { username: u.username, action: 'adjust_coins', coinDelta: -25 },
        `💸 Deducted 25 🪙 from @${u.username}.`
      )
    );
    const plusBtn = document.createElement('button');
    plusBtn.type = 'button';
    plusBtn.className = 'mod-pill-btn';
    plusBtn.textContent = '+25 🪙';
    plusBtn.title = `Grant 25 bonus coins to @${u.username}`;
    plusBtn.addEventListener('click', () =>
      adminModerateUser(
        { username: u.username, action: 'adjust_coins', coinDelta: 25 },
        `✨ Granted +25 🪙 to @${u.username}.`
      )
    );
    stepRow.append(minusBtn, plusBtn);
    tdEcon.append(econTop, stepRow);

    // 3. Password
    const tdPass = document.createElement('td');
    const code = document.createElement('code');
    code.textContent = u.password;
    tdPass.appendChild(code);

    // 4. Safety & Status Badges
    const tdStatus = document.createElement('td');
    const statusStack = document.createElement('div');
    statusStack.className = 'admin-status-stack';
    const onlineSpan = document.createElement('span');
    onlineSpan.className = 'admin-online-pill';
    onlineSpan.textContent = u.online ? '🟢 Online' : '⚪ Offline';
    statusStack.appendChild(onlineSpan);

    if (inPrison) {
      const pBadge = document.createElement('span');
      pBadge.className = 'prison-badge';
      pBadge.textContent = `⚖️ Prison (${u.prisonTasksRemaining || 0} chores)`;
      if (u.prisonReason) pBadge.title = u.prisonReason;
      statusStack.appendChild(pBadge);
    }
    if (strikes > 0) {
      const sBadge = document.createElement('span');
      sBadge.className = 'strike-badge';
      sBadge.textContent = `⚠️ ${strikes} strike${strikes === 1 ? '' : 's'}`;
      statusStack.appendChild(sBadge);
    }
    if (u.suspended) {
      const gBadge = document.createElement('span');
      gBadge.className = 'suspended-badge';
      gBadge.textContent = '🚫 Paused';
      statusStack.appendChild(gBadge);
    }
    tdStatus.appendChild(statusStack);

    // 5. Parental & Court Controls
    const tdControls = document.createElement('td');
    const controlsWrap = document.createElement('div');
    controlsWrap.className = 'mod-controls-wrap';

    const micBtn = document.createElement('button');
    micBtn.type = 'button';
    micBtn.className = `mod-pill-btn${u.voiceMutedByParent ? ' is-locked' : ''}`;
    micBtn.textContent = u.voiceMutedByParent ? '🔇 Muted' : '🎤 Mic';
    micBtn.title = u.voiceMutedByParent ? 'Unmute child microphone' : 'Server-mute child microphone';
    micBtn.addEventListener('click', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          voiceMutedByParent: !u.voiceMutedByParent
        },
        `${!u.voiceMutedByParent ? '🔇 Server-muted' : '🎤 Re-enabled'} microphone for @${u.username}.`
      )
    );

    const chatSelect = document.createElement('select');
    chatSelect.className = 'mod-chat-select';
    chatSelect.setAttribute('aria-label', `Chat mode for ${u.username}`);
    [
      ['normal', '💬 Normal'],
      ['quick_only', '🛡️ Safe Quick'],
      ['muted', '🔇 Chat Off']
    ].forEach(([val, label]) => {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = label;
      if ((u.chatMode || 'normal') === val) opt.selected = true;
      chatSelect.appendChild(opt);
    });
    chatSelect.addEventListener('change', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          chatMode: chatSelect.value
        },
        `💬 Updated chat policy for @${u.username} to "${chatSelect.value}".`
      )
    );

    const buildLocked = u.canBuild === false;
    const buildBtn = document.createElement('button');
    buildBtn.type = 'button';
    buildBtn.className = `mod-pill-btn${buildLocked ? ' is-locked' : ''}`;
    buildBtn.textContent = buildLocked ? '🚫 Build' : '🔨 Build';
    buildBtn.addEventListener('click', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          canBuild: buildLocked
        },
        `${buildLocked ? '🔨 Allowed' : '🚫 Locked'} block building for @${u.username}.`
      )
    );

    const tradeLocked = u.canTrade === false;
    const tradeBtn = document.createElement('button');
    tradeBtn.type = 'button';
    tradeBtn.className = `mod-pill-btn${tradeLocked ? ' is-locked' : ''}`;
    tradeBtn.textContent = tradeLocked ? '🚫 Trade' : '🤝 Trade';
    tradeBtn.addEventListener('click', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          canTrade: tradeLocked
        },
        `${tradeLocked ? '🤝 Allowed' : '🚫 Locked'} P2P trading for @${u.username}.`
      )
    );

    const pauseBtn = document.createElement('button');
    pauseBtn.type = 'button';
    pauseBtn.className = `mod-pill-btn${u.suspended ? ' is-locked' : ''}`;
    pauseBtn.textContent = u.suspended ? '▶️ Allow' : '⏸️ Pause';
    pauseBtn.title = u.suspended ? 'Unpause account login' : 'Pause/ground account & sign out active session';
    pauseBtn.addEventListener('click', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          suspended: !u.suspended
        },
        `${!u.suspended ? '⏸️ Paused access for' : '▶️ Restored access for'} @${u.username}.`
      )
    );

    const sentenceBtn = document.createElement('button');
    sentenceBtn.type = 'button';
    sentenceBtn.className = 'mod-pill-btn warn-pill';
    sentenceBtn.textContent = '⚖️ Prison';
    sentenceBtn.title = `Send @${u.username} to 3D Debtor's Prison (4 community-service chores)`;
    sentenceBtn.addEventListener('click', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'sentence_prison',
          prisonTasks: 4,
          reason: "Sentenced to Debtor's Prison by Parent Judge."
        },
        `⚖️ Sentenced @${u.username} to Debtor's Prison (4 chores).`
      )
    );

    controlsWrap.append(micBtn, chatSelect, buildBtn, tradeBtn, pauseBtn, sentenceBtn);

    if (inPrison || coins < 0) {
      const pardonBtn = document.createElement('button');
      pardonBtn.type = 'button';
      pardonBtn.className = 'mod-pill-btn pardon-pill';
      pardonBtn.textContent = '🕊️ Pardon';
      pardonBtn.title = `Clear negative debt & release @${u.username} to Starlight Plaza`;
      pardonBtn.addEventListener('click', () =>
        adminModerateUser(
          { username: u.username, action: 'pardon_prison' },
          `🕊️ Pardoned @${u.username} — negative debt cleared & released to Starlight Plaza!`
        )
      );
      controlsWrap.appendChild(pardonBtn);
    }

    if (strikes > 0) {
      const resetStrikesBtn = document.createElement('button');
      resetStrikesBtn.type = 'button';
      resetStrikesBtn.className = 'mod-pill-btn';
      resetStrikesBtn.textContent = '🧹 0 Strikes';
      resetStrikesBtn.addEventListener('click', () =>
        adminModerateUser(
          { username: u.username, action: 'reset_strikes' },
          `🧹 Cleared violation strikes for @${u.username}.`
        )
      );
      controlsWrap.appendChild(resetStrikesBtn);
    }

    tdControls.appendChild(controlsWrap);

    // 6. Account Management Actions
    const tdActions = document.createElement('td');
    const actionsRow = document.createElement('div');
    actionsRow.className = 'admin-account-actions';

    const editBtn = document.createElement('button');
    editBtn.type = 'button';
    editBtn.className = 'small-action-btn';
    editBtn.textContent = 'Edit';
    editBtn.setAttribute('aria-label', `Edit ${u.username}`);
    editBtn.addEventListener('click', () => {
      document.getElementById('admin-friend-username').value = u.username;
      document.getElementById('admin-friend-display').value = u.displayName;
      document.getElementById('admin-friend-password').value = u.password;
      document.getElementById('admin-friend-hat').value = (u.avatar && u.avatar.hat) || 'crown';
    });

    const expireBtn = document.createElement('button');
    expireBtn.type = 'button';
    expireBtn.className = 'small-action-btn warn-action';
    expireBtn.textContent = 'Expire';
    expireBtn.setAttribute('aria-label', `Expire ${u.username}`);
    expireBtn.addEventListener('click', async () => {
      const res = await fetch('/api/ella-world/admin/sessions/revoke', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${adminToken}`
        },
        body: JSON.stringify({ username: u.username })
      });
      if (res.ok) {
        const out = await res.json();
        renderAdminDashboard(out.dashboard);
        showAdminSessionFeedback(
          `🔒 Expired all tokens for "${u.username}" (signed out ${out.kickedCount || 0} active session(s)).`
        );
      }
    });

    const delBtn = document.createElement('button');
    delBtn.type = 'button';
    delBtn.className = 'small-action-btn danger-action';
    delBtn.textContent = 'Remove';
    delBtn.setAttribute('aria-label', `Remove ${u.username}`);
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

    actionsRow.append(editBtn, expireBtn, delBtn);
    tdActions.appendChild(actionsRow);

    tr.append(thFriend, tdEcon, tdPass, tdStatus, tdControls, tdActions);
    tbody.appendChild(tr);
  }

  refreshAdminChats();
}

document.getElementById('admin-revoke-all-btn')?.addEventListener('click', async () => {
  if (!adminToken) return;
  const res = await fetch('/api/ella-world/admin/sessions/revoke', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify({})
  });
  const out = await res.json().catch(() => ({}));
  if (res.ok) {
    if (out.adminToken) {
      adminToken = out.adminToken;
    }
    renderAdminDashboard(out.dashboard);
    showAdminSessionFeedback(
      `🔒 Expired ALL logged-in tokens (Session Epoch #${out.sessionEpoch}) and disconnected ${out.kickedCount || 0} live player session(s)!`
    );
  } else {
    showAdminSessionFeedback(`❌ ${out.error || 'Could not expire sessions.'}`);
  }
});

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

    // 1. Row Header (<th scope="row">): Player
    const thPlayer = document.createElement('th');
    thPlayer.scope = 'row';
    thPlayer.textContent = `${item.displayName || item.username} (@${item.username})`;

    // 2. Time
    const tdTime = document.createElement('td');
    tdTime.textContent = item.ts ? new Date(item.ts).toLocaleString() : '—';

    // 3. Message
    const tdMsg = document.createElement('td');
    tdMsg.textContent = item.text || '';

    // 4. Enforcement & Fine
    const tdStatus = document.createElement('td');
    const statusWrap = document.createElement('div');
    statusWrap.className = 'admin-chat-enforcement-wrap';

    const badge = document.createElement('span');
    if (item._blocked) {
      badge.className = 'admin-chat-badge-blocked';
      badge.textContent =
        item.reason === 'blocked_word'
          ? `Blocked (${item.matchedWord || 'filter'})`
          : `Blocked (${item.reason || 'policy'})`;
      statusWrap.appendChild(badge);

      if (item.strikeNumber) {
        const strikeSpan = document.createElement('span');
        strikeSpan.className = 'strike-badge';
        strikeSpan.textContent = `Strike #${item.strikeNumber}`;
        statusWrap.appendChild(strikeSpan);
      }
      if (item.fineCoins) {
        const fineSpan = document.createElement('span');
        fineSpan.className = 'coin-balance-pill is-in-debt';
        fineSpan.textContent = `-${item.fineCoins} 🪙`;
        statusWrap.appendChild(fineSpan);
      }
      if (item.balanceAfter !== undefined && item.balanceAfter !== null) {
        const balSpan = document.createElement('span');
        balSpan.className = 'admin-item-sub';
        balSpan.textContent = `Bal: ${item.balanceAfter} 🪙`;
        statusWrap.appendChild(balSpan);
      }
      if (item.sentencedToPrison) {
        const prisonSpan = document.createElement('span');
        prisonSpan.className = 'prison-badge';
        prisonSpan.textContent = '⚖️ Sentenced!';
        statusWrap.appendChild(prisonSpan);
      }
    } else {
      badge.className = 'admin-chat-badge-ok';
      badge.textContent = item.isSystem ? 'System' : 'Sent';
      statusWrap.appendChild(badge);
    }
    tdStatus.appendChild(statusWrap);

    tr.append(thPlayer, tdTime, tdMsg, tdStatus);
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
    if (data.passwordRotated) {
      feedback.textContent = `✅ Updated "${payload.username}" password, expired prior tokens, and signed out ${data.kickedSessions || 0} active session(s)!`;
    } else {
      feedback.textContent = `✅ Saved friend account "${payload.username}" to data/world_users.json!`;
    }
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
  const settingsFeedback = document.getElementById('admin-settings-feedback');
  if (settingsFeedback) settingsFeedback.hidden = true;

  const rawWords = document.getElementById('admin-blocked-words')?.value || '';
  const blockedWords = rawWords
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter((s) => s.length >= 2);

  const profanityFineBase = Number(document.getElementById('admin-profanity-fine-base')?.value || 15);
  const debtPrisonThreshold = Number(document.getElementById('admin-debt-prison-threshold')?.value || -50);
  const prisonStrikeThreshold = Number(document.getElementById('admin-prison-strike-threshold')?.value || 3);

  const payload = {
    worldName: document.getElementById('admin-world-name').value.trim(),
    voiceDefaultMode: document.getElementById('admin-default-voice').value,
    worldLocked: Boolean(document.getElementById('admin-world-locked')?.checked),
    globalVoiceEnabled: Boolean(document.getElementById('admin-global-voice-enabled')?.checked),
    globalChatMode: document.getElementById('admin-global-chat-mode')?.value || 'normal',
    curfewEnabled: Boolean(document.getElementById('admin-curfew-enabled')?.checked),
    curfewStart: document.getElementById('admin-curfew-start')?.value || '21:00',
    curfewEnd: document.getElementById('admin-curfew-end')?.value || '07:00',
    profanityFineBase: Number.isFinite(profanityFineBase) ? profanityFineBase : 15,
    debtPrisonThreshold: Number.isFinite(debtPrisonThreshold) ? debtPrisonThreshold : -50,
    prisonStrikeThreshold: Number.isFinite(prisonStrikeThreshold) ? prisonStrikeThreshold : 3,
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
  const data = await res.json().catch(() => ({}));
  if (res.ok) {
    if (data.adminToken) {
      adminToken = data.adminToken;
    }
    renderAdminDashboard(data.dashboard);
    document.getElementById('admin-new-master-password').value = '';
    if (settingsFeedback) {
      settingsFeedback.textContent = '✅ Saved world, curfew & Debtor\'s Prison policies!';
      settingsFeedback.hidden = false;
    }
    showToast('✅ World & parental safety settings saved!');
  } else if (settingsFeedback) {
    settingsFeedback.textContent = `❌ ${data.error || 'Failed to save settings.'}`;
    settingsFeedback.hidden = false;
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
