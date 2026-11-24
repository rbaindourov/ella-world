import { VoiceChatManager } from '/ella-world/js/voice.js';
import { WorldSpineClient } from '/ella-world/js/ws_spine.js';
import {
  collectMachineTelemetry,
  loadStoredMachineSeal,
  persistMachineSeal
} from '/ella-world/js/device_sig.js';
import {
  BlockyWorld3D,
  GlamSalonStage3D,
  BLOCK_COLORS,
  BUILD_CATALOG
} from '/ella-world/js/world3d.js';

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
let currentRunwayTheme = '';
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
const ZONE_ARENA = 3;

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
let arenaRespawnInFlight = false;
let pendingPostPortalWarp = null;
let arenaPlayerStatus = {
  hp: 100,
  maxHp: 100,
  alive: true,
  defeatedBy: null,
  respawnReadyAtMs: 0,
  invulnUntilMs: 0
};
let lastArenaSrState = '';
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
const warpArenaBtn = document.getElementById('warp-arena-btn');
const openBackpackBtn = document.getElementById('open-backpack-btn');
const touchActionBtn = document.getElementById('touch-action-btn');

const arenaHudBanner = document.getElementById('arena-hud-banner');
const arenaShieldPill = document.getElementById('arena-shield-pill');
const arenaHpBar = document.getElementById('arena-hp-bar');
const arenaHpText = document.getElementById('arena-hp-text');
const arenaHudSub = document.getElementById('arena-hud-sub');
const arenaStatusSr = document.getElementById('arena-status-sr');
const arenaDeathOverlay = document.getElementById('arena-death-overlay');
const arenaDeathCause = document.getElementById('arena-death-cause');
const arenaRespawnBtn = document.getElementById('arena-respawn-btn');
const arenaFleePlazaBtn = document.getElementById('arena-flee-plaza-btn');

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

const toggleHudModeBtn = document.getElementById('toggle-hud-mode-btn');
const toggleBrowserFullscreenBtn = document.getElementById('toggle-browser-fullscreen-btn');
const toggleFriendsPanelBtn = document.getElementById('toggle-friends-panel-btn');
const toggleRunwayBarBtn = document.getElementById('toggle-runway-bar-btn');
const toggleEmoteBarBtn = document.getElementById('toggle-emote-bar-btn');
const toggleTouchControlsBtn = document.getElementById('toggle-touch-controls-btn');
const toggleChatPanelBtn = document.getElementById('toggle-chat-panel-btn');
const closeChatPanelBtn = document.getElementById('close-chat-panel-btn');
const reopenChatBtn = document.getElementById('reopen-chat-btn');

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
const buildActionToolbar = document.getElementById('build-action-toolbar');
const toggleBuildBtn = document.getElementById('toggle-build-btn');
const buildActionPlaceBtn = document.getElementById('build-action-place');
const buildActionRemoveBtn = document.getElementById('build-action-remove');
const buildRotateBtn = document.getElementById('build-rotate-btn');
const buildFacingStatusEl = document.getElementById('build-facing-status');
const buildTablist = document.getElementById('build-tablist');
const buildPanelsStack = document.getElementById('build-panels-stack');
const buildHintTextEl = document.getElementById('build-hint-text');
const buildSaveStatusEl = document.getElementById('build-save-status');
const colorSwatchBar = document.getElementById('color-swatch-bar');

function formatFacingLabel(rot) {
  const labels = ['🧭 Facing S (0°)', '🧭 Facing E (90°)', '🧭 Facing N (180°)', '🧭 Facing W (270°)'];
  return labels[((Number(rot) || 0) % 4 + 4) % 4];
}

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
const subadminDialog = document.getElementById('subadmin-dialog');
const openSubadminHudBtn = document.getElementById('open-subadmin-hud-btn');
const subadminFriendsTbody = document.getElementById('subadmin-friends-tbody');
const subadminFeedback = document.getElementById('subadmin-feedback');
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

// Expansion Pack State & DOM Elements (Pets/Boards, Homes/Doorbells, Crafting, Fashion Show, Obby Speedruns, Sky/Dragon Boss)
let petCatalog = [];
let mountCatalog = [];
let craftingRecipes = [];
let playerCompanions = {
  ownedPets: ['cloud_puppy'],
  equippedPet: 'cloud_puppy',
  petNickname: '',
  ownedMounts: ['starlight_board'],
  equippedMount: null
};
let homesState = [];
let obbyLeaderboards = { easy: [], hard: [] };
let activeObbyRun = null;
let skyState = {
  mode: 'auto',
  phase: 'day',
  cycleDurationMs: 720000,
  cycleAnchorMs: 0,
  meteorShowerActive: false,
  meteorShowerUntilMs: 0,
  fallingStars: []
};
let runwayShowState = {
  showId: 0,
  phase: 'idle',
  theme: '',
  phaseStartedAt: 0,
  phaseEndsAt: 0,
  contestants: [],
  spotlightIndex: 0,
  spotlightUsername: null,
  podiumWinners: []
};
let lastInvitedShowId = 0;
let lastDoorbellPlotId = null;
let arenaBossStatus = null;

const hudSkyPill = document.getElementById('hud-sky-pill');
const openObbyBoardBtn = document.getElementById('open-obby-board-btn');
const openCompanionsBtn = document.getElementById('open-companions-btn');
const toggleHoverboardBtn = document.getElementById('toggle-hoverboard-btn');
const openHomesBtn = document.getElementById('open-homes-btn');
const openCraftBtn = document.getElementById('open-craft-btn');
const openSkyBtn = document.getElementById('open-sky-btn');

const runwayStartShowBtn = document.getElementById('runway-start-show-btn');
const runwayShowStatusEl = document.getElementById('runway-show-status');
const runwayShowPhaseText = document.getElementById('runway-show-phase-text');
const runwayShowProgress = document.getElementById('runway-show-progress');
const runwayReadyBtn = document.getElementById('runway-ready-btn');
const glamShowTimerPill = document.getElementById('glam-show-timer-pill');
const glamShowTimerText = document.getElementById('glam-show-timer-text');

const obbySpeedrunBanner = document.getElementById('obby-speedrun-banner');
const obbySpeedrunCourseLabel = document.getElementById('obby-speedrun-course-label');
const obbySpeedrunTime = document.getElementById('obby-speedrun-time');
const obbySpeedrunCancelBtn = document.getElementById('obby-speedrun-cancel-btn');

const arenaBossRow = document.getElementById('arena-boss-row');
const arenaBossBar = document.getElementById('arena-boss-bar');
const arenaBossHpText = document.getElementById('arena-boss-hp-text');

const srLiveAnnouncements = document.getElementById('sr-live-announcements');
const fashionInvitePopover = document.getElementById('fashion-invite-popover');
const fashionInviteText = document.getElementById('fashion-invite-text');
const fashionInviteJoinBtn = document.getElementById('fashion-invite-join-btn');
const doorbellPopover = document.getElementById('doorbell-popover');
const doorbellPopoverText = document.getElementById('doorbell-popover-text');
const doorbellUnlockBtn = document.getElementById('doorbell-unlock-btn');

const companionsDialog = document.getElementById('companions-dialog');
const homesDialog = document.getElementById('homes-dialog');
const craftDialog = document.getElementById('craft-dialog');
const obbyBoardDialog = document.getElementById('obby-board-dialog');
const skyDialog = document.getElementById('sky-dialog');

let wsHandshakeProbeInFlight = false;
let lastWsHandshakeProbeAt = 0;

function forceSessionRevokedToLogin(reason) {
  const alertMessage =
    reason || 'Your session has expired or your password was updated. Please sign in again.';

  // Close player modals/popovers first so #login-screen is not inert when announcing
  if (isSalonOpen) closeGlamSalon(false);
  hideTradeInvitePopover();
  hideTopLayerToastPopover(fashionInvitePopover);
  hideTopLayerToastPopover(doorbellPopover);
  pendingInviteTrade = null;
  activeTrade = null;
  activeObbyRun = null;
  world3d.setActiveObbyRun(null);
  if (obbySpeedrunBanner) obbySpeedrunBanner.hidden = true;
  draftTradeCoins = 0;
  draftTradeItems = {};
  if (tradeDialog && tradeDialog.open) tradeDialog.close('cancelled');
  if (backpackDialog && backpackDialog.open) backpackDialog.close();
  if (companionsDialog && companionsDialog.open) companionsDialog.close();
  if (homesDialog && homesDialog.open) homesDialog.close();
  if (craftDialog && craftDialog.open) craftDialog.close();
  if (obbyBoardDialog && obbyBoardDialog.open) obbyBoardDialog.close();
  if (skyDialog && skyDialog.open) skyDialog.close();
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

async function buildMachineAuthFields() {
  const [machineSeal, deviceTelemetry] = await Promise.all([
    loadStoredMachineSeal(),
    collectMachineTelemetry()
  ]);
  return {
    machineSeal: machineSeal || undefined,
    deviceTelemetry
  };
}

async function probeSessionValidityAfterWsFailure() {
  if (!sessionToken || wsHandshakeProbeInFlight) return;
  const now = Date.now();
  if (now - lastWsHandshakeProbeAt < 1500) return;
  wsHandshakeProbeInFlight = true;
  lastWsHandshakeProbeAt = now;
  try {
    const machineFields = await buildMachineAuthFields();
    const res = await fetch('/api/ella-world/resume', {
      method: 'POST',
      credentials: 'same-origin',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, clientId, ...machineFields })
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
    if (block.kind === 'loot_decor') {
      const iid = block.lootItemId;
      if (!iid || Number((playerEconomy.inventory || {})[iid] || 0) < 1) {
        showToast('⚠️ Pick an owned treasure in 🏆 My Loot (or farm more in Crystal Cavern)!');
        return;
      }
    }
    const prevBlock = world3d.savedBlocksByKey.get(`${block.x},${block.y},${block.z}`) || null;
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
      if (Array.isArray(data.blocks)) {
        world3d.loadBlocks(data.blocks);
      }
      if (data.economy) {
        applyEconomySnapshot(data.economy);
      }
      updateBuildSaveStatus(Array.isArray(data.blocks) ? data.blocks.length : world3d.voxelMap.size, false);
    } else {
      world3d.removeBlockMesh({ x: block.x, y: block.y, z: block.z });
      if (prevBlock) world3d.addBlockMesh(prevBlock);
      updateBuildSaveStatus(world3d.voxelMap.size, false);
      if (res) {
        const err = await res.json().catch(() => ({}));
        if (err && err.error) showToast(`⚠️ ${err.error}`);
      }
    }
  },
  onRemoveBlock: async (coords) => {
    const prevBlock = world3d.savedBlocksByKey.get(`${coords.x},${coords.y},${coords.z}`) || null;
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
      if (Array.isArray(data.blocks)) {
        world3d.loadBlocks(data.blocks);
      }
      if (data.economy) {
        applyEconomySnapshot(data.economy);
      }
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
  onToggleBlock: async (coords) => {
    const res = await fetch('/api/ella-world/block', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'toggle', ...coords })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    if (res && res.ok) {
      const data = await res.json();
      currentBlocksVersion = data.blocksVersion;
      if (Array.isArray(data.blocks)) {
        world3d.loadBlocks(data.blocks);
        updateBuildSaveStatus(data.blocks.length, false);
      }
    } else if (res) {
      const err = await res.json().catch(() => ({}));
      if (err && err.error) showToast(`⚠️ ${err.error}`);
    }
  },
  onFireRayGun: async (targetArr) => {
    if (!sessionToken || !Array.isArray(targetArr)) return;
    const res = await fetch('/api/ella-world/raygun', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({
        token: sessionToken,
        x: targetArr[0],
        y: targetArr[1],
        z: targetArr[2]
      })
    }).catch(() => null);
    await handleHttpAuthCheck(res);
  },
  onRotateBuild: (rot) => {
    if (buildFacingStatusEl) {
      buildFacingStatusEl.textContent = formatFacingLabel(rot);
    }
  },
  onObbyCheckpoint: (cpIndex, tier = 'easy') => {
    if (tier === 'hard') {
      showToast(`🔥 Sky Gauntlet Checkpoint ${cpIndex}/5 Reached! Press T anytime to warp here.`);
    } else {
      showToast(`⭐ Sky Obby Checkpoint ${cpIndex} Reached! Press R anytime to warp here.`);
    }
  },
  onTrophyWin: async (tier = 'easy') => {
    const isHard = tier === 'hard';
    const res = await fetch('/api/ella-world/chat', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, isTrophyWin: true, trophyTier: tier })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    let coinsAwarded = 0;
    if (res && res.ok) {
      const data = await res.json().catch(() => null);
      coinsAwarded = Number(data?.coinsAwarded || 0);
    }
    if (isHard) {
      showToast(
        coinsAwarded > 0
          ? `💎👑 YOU CONQUERED THE STARLIGHT SKY GAUNTLET! +${coinsAwarded} 🪙 awarded! 🎉`
          : '💎👑 YOU CONQUERED THE STARLIGHT SKY GAUNTLET! Celebration sent to all friends! 🎉',
        6500
      );
    } else {
      showToast('🏆 YOU CONQUERED THE SKY OBBY! Celebration sent to all friends! 🎉', 6000);
    }
  },
  onRunwayProximity: (isNear) => {
    if (runwayHudBar) {
      runwayHudBar.classList.toggle('near-runway', Boolean(isNear));
    }
    if (isNear) {
      showToast('💃 Welcome to the Fashion Runway! Strike a pose (1–9), vote 1–5 ⭐, or click 🎬 Start Show!');
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
  },
  onManageHome: (plot) => {
    handleWorldMailboxInteraction(plot);
  },
  onOpenCrafting: () => {
    openCraftModal();
  },
  onObbyGateCrossed: (course, gateKind) => {
    handleObbyGateCrossed(course, gateKind);
  },
  onOpenObbyBoard: (course) => {
    openObbyBoardModal(course);
  },
  onCatchFallingStar: (starId) => {
    catchFallingStar(starId);
  },
  onSummonArenaBoss: () => {
    summonArenaBoss(false);
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

// Update active Swiftness / Cloud Leap timers, Sanctuary shield, Sky phase, Obby stopwatch & Fashion Show countdown at 10Hz
setInterval(() => {
  const serverNow = Date.now() + (world3d.serverClockOffsetMs || 0);
  if (arenaShieldPill && currentZone === ZONE_ARENA) {
    const pos = world3d.selfAvatar?.group?.position;
    const inSanct = pos && pos.z >= 19.5 && Math.abs(pos.x) <= 9.0;
    const invulnRem = Math.ceil(((arenaPlayerStatus.invulnUntilMs || 0) - serverNow) / 1000);
    if (inSanct || invulnRem > 0) {
      arenaShieldPill.textContent = invulnRem > 0 ? `🛡️ Spawn Shield ${invulnRem}s` : '🛡️ Safe Sanctuary';
      arenaShieldPill.hidden = false;
    } else {
      arenaShieldPill.hidden = true;
    }
  }

  if (hudSkyPill) {
    const sp = world3d.getSkyPhaseSummary(serverNow);
    const nextSkyText = `${sp.icon} ${sp.label}`;
    if (hudSkyPill.textContent !== nextSkyText) {
      hudSkyPill.textContent = nextSkyText;
    }
    hudSkyPill.classList.toggle('is-meteor', sp.phase === 'meteor');
  }

  if (obbySpeedrunBanner) {
    if (activeObbyRun && currentZone === ZONE_PLAZA) {
      const elapsedSec = Math.max(0, (serverNow - activeObbyRun.startedAtMs) / 1000);
      obbySpeedrunBanner.hidden = false;
      if (obbySpeedrunCourseLabel) {
        obbySpeedrunCourseLabel.textContent =
          activeObbyRun.course === 'hard' ? '🔥 Pro Gauntlet' : '⏱️ Sky Obby';
      }
      if (obbySpeedrunTime) {
        obbySpeedrunTime.textContent = `${elapsedSec.toFixed(2)}s`;
      }
    } else {
      obbySpeedrunBanner.hidden = true;
    }
  }

  renderFashionShowTicker(serverNow);

  if (!hudBuffPill) return;
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
}, 100);

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
  if (runway.theme) {
    currentRunwayTheme = runway.theme;
    if (runwayThemeLabel) {
      runwayThemeLabel.textContent = runway.theme;
    }
    const glamThemePill = document.getElementById('glam-runway-theme-pill');
    const glamThemeText = document.getElementById('glam-runway-theme-text');
    if (glamThemePill && glamThemeText) {
      glamThemeText.textContent = runway.theme;
      glamThemePill.hidden = false;
    }
  }
  if (runway.show) {
    runwayShowState = { ...runwayShowState, ...runway.show };
    world3d.syncRunwayShowState(runwayShowState);
    if (
      runwayShowState.phase === 'catwalk' &&
      runwayShowState.spotlightUsername &&
      runwayModelSelect &&
      String(selfUser?.username || '').toLowerCase() !==
        String(runwayShowState.spotlightUsername).toLowerCase()
    ) {
      const optExists = Array.from(runwayModelSelect.options).some(
        (o) => o.value.toLowerCase() === String(runwayShowState.spotlightUsername).toLowerCase()
      );
      if (optExists) {
        runwayModelSelect.value = runwayShowState.spotlightUsername;
      }
    }
    renderFashionShowTicker(Date.now() + (world3d.serverClockOffsetMs || 0));
  }
  if (runwayLeaderPill) {
    if (
      runway.show?.phase === 'podium' &&
      Array.isArray(runway.show.podiumWinners) &&
      runway.show.podiumWinners.length > 0
    ) {
      const w1 = runway.show.podiumWinners[0];
      runwayLeaderPill.textContent = `👑 1st: ${w1.displayName || w1.username} (${w1.avgStars}★)`;
    } else {
      const top = Array.isArray(runway.leaderboard) && runway.leaderboard[0];
      if (top && top.votes > 0) {
        const matchedPlayer = Array.from(playersMap.values()).find(
          (p) => String(p.user?.username || '').toLowerCase() === top.username
        );
        const disp = matchedPlayer ? matchedPlayer.user.displayName : top.username;
        runwayLeaderPill.textContent = `🏆 ${disp} (${top.avgStars}★)`;
      } else {
        runwayLeaderPill.textContent = '🏆 Rate 1–5★';
      }
    }
  }
}

// 6. Friend Sign-In & Automatic Persistent Session Resume
async function tryAutoResumeSession() {
  if (!sessionToken) return;
  try {
    const machineFields = await buildMachineAuthFields();
    const res = await fetch('/api/ella-world/resume', {
      method: 'POST',
      credentials: 'same-origin',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, clientId, ...machineFields })
    });
    if (!res.ok) {
      const errData = await res.json().catch(() => ({}));
      if (errData && errData.machineSeal) {
        await persistMachineSeal(errData.machineSeal);
      }
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
    if (data && data.machineSeal) {
      await persistMachineSeal(data.machineSeal);
    }
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
    const machineFields = await buildMachineAuthFields();
    const res = await fetch('/api/ella-world/login', {
      method: 'POST',
      credentials: 'same-origin',
      headers: JSON_HEADERS,
      body: JSON.stringify({ username, password, clientId, ...machineFields })
    });
    const data = await res.json();
    if (data && data.machineSeal) {
      await persistMachineSeal(data.machineSeal);
    }
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
  setHudFullscreenMode(true);
  sessionEnteredWallMs = Date.now();
  activeObbyRun = null;
  world3d.setActiveObbyRun(null);

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

  if (Array.isArray(loginData.petCatalog)) petCatalog = loginData.petCatalog;
  if (Array.isArray(loginData.mountCatalog)) mountCatalog = loginData.mountCatalog;
  if (Array.isArray(loginData.craftingRecipes)) craftingRecipes = loginData.craftingRecipes;

  if (Array.isArray(loginData.itemCatalog)) {
    setItemCatalog(loginData.itemCatalog);
  } else if (itemCatalog.length === 0 || petCatalog.length === 0 || craftingRecipes.length === 0) {
    fetch('/api/ella-world/economy/catalog', { headers: { 'X-Cache-Warmup': 'true' } })
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        const list = d?.itemCatalog || d?.items;
        if (Array.isArray(list)) setItemCatalog(list);
        if (Array.isArray(d?.petCatalog)) petCatalog = d.petCatalog;
        if (Array.isArray(d?.mountCatalog)) mountCatalog = d.mountCatalog;
        if (Array.isArray(d?.craftingRecipes)) craftingRecipes = d.craftingRecipes;
      })
      .catch(() => {});
  }
  if (loginData.economy) {
    applyEconomySnapshot(loginData.economy);
  }
  if (loginData.companions) {
    applyCompanionsSnapshot(loginData.companions);
  }
  if (loginData.homes) {
    applyHomesSnapshot(loginData.homes);
  }
  if (loginData.obbyLeaderboards) {
    applyObbyLeaderboards(loginData.obbyLeaderboards);
  }
  if (loginData.sky) {
    applySkySnapshot(loginData.sky);
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
  if (loginData.companions && selfUser) {
    world3d.updateUserCompanions(selfUser.username, loginData.companions);
  }
  if (loginData.dungeon) {
    world3d.syncDungeonSnapshot(loginData.dungeon);
  }
  if (loginData.arena) {
    world3d.syncArenaSnapshot(loginData.arena);
    applyArenaBossHud(loginData.arena.boss || null);
  }
  if (loginData.arenaStatus) {
    applyArenaPlayerStatus(loginData.arenaStatus);
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
      if (Array.isArray(ev.petCatalog)) petCatalog = ev.petCatalog;
      if (Array.isArray(ev.mountCatalog)) mountCatalog = ev.mountCatalog;
      if (Array.isArray(ev.craftingRecipes)) craftingRecipes = ev.craftingRecipes;
      if (Array.isArray(ev.itemCatalog)) {
        setItemCatalog(ev.itemCatalog);
      }
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      if (ev.companions) {
        applyCompanionsSnapshot(ev.companions);
        if (selfUser) {
          world3d.updateUserCompanions(selfUser.username, ev.companions);
        }
      }
      if (ev.homes) {
        applyHomesSnapshot(ev.homes);
      }
      if (ev.obbyLeaderboards) {
        applyObbyLeaderboards(ev.obbyLeaderboards);
      }
      if (ev.sky) {
        applySkySnapshot(ev.sky);
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
      if (ev.arena) {
        world3d.syncArenaSnapshot(ev.arena);
        applyArenaBossHud(ev.arena.boss || null);
      }
      if (ev.arenaStatus) {
        applyArenaPlayerStatus(ev.arenaStatus);
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
          arena: ev.arena,
          arenaStatus: ev.arenaStatus,
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
      if (ev.sky) {
        applySkySnapshot(ev.sky);
      }
      if (ev.homes) {
        applyHomesSnapshot(ev.homes);
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
          world3d.selfAvatar.user = selfUser;
          world3d.selfAvatar.updateAvatarAppearance(selfUser.avatar);
          world3d.selfAvatar.syncCompanions(selfUser);
        }
        if (isSalonOpen && draftAvatar && selfUser.avatar) {
          draftAvatar.hat = selfUser.avatar.hat;
          draftAvatar.accessory = selfUser.avatar.accessory;
          syncSalonControlsFromDraft();
        }
      } else if (ev.selfId && ev.selfId !== selfId && ev.user) {
        const existing = playersMap.get(ev.selfId);
        const av = world3d.remoteAvatars.get(ev.selfId);
        if (existing) {
          existing.user = ev.user;
        }
        if (av) {
          av.user = ev.user;
          av.syncCompanions(ev.user);
        }
        if (existing && ev.avatarHash && existing.avatarHash === ev.avatarHash) {
          break;
        }
        if (existing && ev.avatarHash) {
          existing.avatarHash = ev.avatarHash;
        }
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
    case 'arena_mobs': {
      if (Array.isArray(ev.mobs)) {
        world3d.syncArenaMobs(ev.mobs, ev.boss);
      }
      if ('boss' in ev) {
        applyArenaBossHud(ev.boss || null);
      }
      break;
    }
    case 'arena_sync': {
      if (ev.arena) {
        world3d.syncArenaSnapshot(ev.arena);
        applyArenaBossHud(ev.arena.boss || null);
      }
      break;
    }
    case 'arena_player_status': {
      if (ev.arena) {
        world3d.syncArenaSnapshot(ev.arena);
        applyArenaBossHud(ev.arena.boss || null);
      }
      if (ev.arenaStatus) {
        applyArenaPlayerStatus(ev.arenaStatus, ev);
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
      if (ev.arenaStatus) {
        applyArenaPlayerStatus(ev.arenaStatus, { event: 'healed' });
      }
      const healAmt = ev.itemId === 'starlight_potion' ? 75 : 35;
      showToast(
        `🧪 Used ${ev.itemName || 'item'}${currentZone === ZONE_ARENA ? ` (+${healAmt} HP Healed!)` : ''}!`,
        3600
      );
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
    case 'ray_gun_beam': {
      if (ev.zone === currentZone && Array.isArray(ev.from) && Array.isArray(ev.to)) {
        if (ev.actorId === selfId && performance.now() - world3d.lastLocalBeamAt < 280) {
          break;
        }
        world3d.spawnRayGunBeam(
          ev.from,
          ev.to,
          Boolean(ev.hitMobId || ev.zone === ZONE_DUNGEON || ev.zone === ZONE_ARENA),
          ev.style || 'ray_gun'
        );
      }
      break;
    }
    case 'runway': {
      if (ev.runway) {
        applyRunwaySummary(ev.runway);
      }
      if (ev.event === 'show_started' && ev.runway?.show) {
        const sh = ev.runway.show;
        const myU = String(selfUser?.username || '').toLowerCase();
        const alreadyIn = (sh.contestants || []).some(
          (c) => String(c.username || '').toLowerCase() === myU
        );
        if (!alreadyIn && sh.showId && sh.showId !== lastInvitedShowId && currentZone !== ZONE_PRISON) {
          lastInvitedShowId = sh.showId;
          const msg = `${sh.startedBy || 'A friend'} started a Runway Show! Theme: "${sh.theme || ev.runway.theme}" • 45s Dress-Up!`;
          if (fashionInviteText) fashionInviteText.textContent = msg;
          showTopLayerToastPopover(fashionInvitePopover, 16000);
          announceScreenReaderToast(msg);
        }
      } else if (ev.event === 'show_podium' && Array.isArray(ev.runway?.show?.podiumWinners)) {
        const winners = ev.runway.show.podiumWinners;
        if (winners.length > 0) {
          const topStr = winners
            .slice(0, 3)
            .map(
              (w, idx) =>
                `${idx === 0 ? '🥇' : idx === 1 ? '🥈' : '🥉'} ${w.displayName || w.username} (+${w.prizeCoins}🪙)`
            )
            .join(' • ');
          showToast(`👑 Fashion Show Podium: ${topStr}!`, 6800);
          if (world3d.selfAvatar) {
            world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
          }
        }
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
    case 'companions_update': {
      if (ev.user) {
        selfUser = ev.user;
        if (selfId && playersMap.has(selfId)) {
          playersMap.get(selfId).user = selfUser;
        }
      }
      if (ev.economy) {
        applyEconomySnapshot(ev.economy);
      }
      if (ev.companions) {
        applyCompanionsSnapshot(ev.companions);
        if (selfUser) {
          world3d.updateUserCompanions(selfUser.username, ev.companions);
        }
      }
      if (ev.action === 'auto_dismount_obby') {
        showToast('⏱️ Hoverboard auto-dismounted for fair Obby Speedrun timing!');
      }
      break;
    }
    case 'homes_sync': {
      if (Array.isArray(ev.plots)) {
        applyHomesSnapshot(ev.plots);
      }
      break;
    }
    case 'home_doorbell': {
      if (ev.plotId) {
        world3d.triggerDoorbellFx(ev.plotId);
      }
      const isMyHome =
        selfUser &&
        String(ev.ownerUsername || '').toLowerCase() === String(selfUser.username || '').toLowerCase();
      if (isMyHome || currentZone === ZONE_PLAZA) {
        playDoorbellChime();
      }
      if (isMyHome) {
        lastDoorbellPlotId = ev.plotId || null;
        const msg = `🔔 ${ev.ringerName || 'A friend'} rang the doorbell at ${ev.plotName || 'your home'}!`;
        if (doorbellPopoverText) doorbellPopoverText.textContent = msg;
        showTopLayerToastPopover(doorbellPopover, 14000);
        announceScreenReaderToast(msg);
      }
      break;
    }
    case 'obby_leaderboard': {
      if (ev.leaderboards) {
        applyObbyLeaderboards(ev.leaderboards);
      }
      if (ev.record) {
        const cLabel = ev.course === 'hard' ? 'Pro Obby Gauntlet' : 'Sky Obby';
        showToast(
          `⏱️🏆 ${ev.record.displayName || ev.record.username} set a Top-5 ${cLabel} speedrun: ${ev.record.timeFormatted}!`,
          5200
        );
      }
      break;
    }
    case 'sky_sync': {
      if (ev.sky) {
        applySkySnapshot(ev.sky);
      }
      if (ev.event === 'meteor_shower_started') {
        showToast('🌠 STARLIGHT METEOR SHOWER! Catch glowing Falling Stars across Starlight Plaza!', 5600);
        announceScreenReaderToast('Starlight Meteor Shower started in Starlight Plaza!');
      }
      break;
    }
    case 'boss_reward': {
      if (ev.reward) {
        const rw = ev.reward;
        const itemDef = rw.itemId ? catalogById.get(rw.itemId) : null;
        const parts = [`🪙 +${rw.coins || 0} Coins`];
        if (rw.itemId && rw.qty) {
          parts.push(`${itemDef ? itemDef.icon : '🔮'} +${rw.qty}x ${itemDef ? itemDef.name : rw.itemId}`);
        }
        if (rw.petEgg) {
          parts.push('🥚 +1x Starlight Companion Egg!');
        }
        if (world3d.selfAvatar) {
          world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
        }
        showToast(`🐉🏆 OBSIDIAN RIFT DRAGON DEFEATED! ${parts.join(' • ')}`, 7200);
        announceScreenReaderToast(`Obsidian Rift Dragon defeated! Earned ${parts.join(', ')}.`);
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
    inventory: { ...(econ.inventory || {}) },
    cloneableItems: Array.isArray(econ.cloneableItems) ? [...econ.cloneableItems] : []
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
  if (openSubadminHudBtn) {
    const canSubadmin =
      String(selfUser?.role || '').toLowerCase() === 'subadmin' &&
      !playerModeration.inDebtorsPrison;
    openSubadminHudBtn.hidden = !canSubadmin;
  }
  if (openSkyBtn) {
    const canControlSky =
      (String(selfUser?.role || '').toLowerCase() === 'host' &&
        !playerModeration.inDebtorsPrison) ||
      Boolean(adminToken);
    openSkyBtn.hidden = !canControlSky;
  }
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
  if (companionsDialog && companionsDialog.open) {
    renderCompanionsModal();
  }
  if (craftDialog && craftDialog.open) {
    renderCraftModal();
  }
  if (tradeDialog && tradeDialog.open && activeTrade) {
    renderTradeModal();
  }
  renderBuildLootItems();
}

function applyArenaPlayerStatus(status, meta = null) {
  if (!status || typeof status !== 'object') return;
  const maxHp = Math.max(1, Number(status.maxHp) || 100);
  const hp = Math.max(0, Math.min(maxHp, Number(status.hp ?? maxHp)));
  const alive = status.alive !== false && hp > 0;
  arenaPlayerStatus = {
    hp,
    maxHp,
    alive,
    defeatedBy: status.defeatedBy || null,
    respawnReadyAtMs: Number(status.respawnReadyAtMs || 0),
    invulnUntilMs: Number(status.invulnUntilMs || 0)
  };
  world3d.setArenaDead(currentZone === ZONE_ARENA && !alive);

  if (arenaHpBar) {
    arenaHpBar.max = maxHp;
    arenaHpBar.value = hp;
    arenaHpBar.classList.toggle('is-low', hp <= 28);
    arenaHpBar.classList.toggle('is-mid', hp > 28 && hp <= 58);
  }
  if (arenaHpText) {
    arenaHpText.textContent = `${hp} / ${maxHp} HP`;
  }

  if (meta && (meta.event === 'damaged' || meta.event === 'died')) {
    if (arenaHudBanner) {
      arenaHudBanner.classList.add('is-hurt-flash');
      setTimeout(() => arenaHudBanner.classList.remove('is-hurt-flash'), 360);
    }
    if (world3d.selfAvatar) {
      world3d.spawnSparkleBurst(world3d.selfAvatar.group.position, '#ef4444');
    }
    if (meta.event === 'damaged' && meta.damage) {
      showToast(`💥 -${meta.damage} HP from ${meta.source || 'Shadow Rift'}! (${hp}/${maxHp} HP)`, 2400);
    }
  }

  if (meta && meta.event === 'respawned') {
    if (typeof meta.x === 'number' && world3d.selfAvatar) {
      world3d.selfAvatar.group.position.set(meta.x, (meta.y || 0) + 0.15, meta.z || 25);
      world3d.velocity.set(0, 0, 0);
      world3d.selfAvatar.emote = null;
      world3d._emitMoveNow();
      world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
    }
    showToast('⚡ Respawned at the Sanctuary Circle with 100 / 100 HP!', 3600);
  }

  // Discrete screen-reader milestone announcements (modern-web-guidance: progress)
  if (arenaStatusSr && currentZone === ZONE_ARENA) {
    let srKey = 'healthy';
    let srText = '';
    if (!alive) {
      srKey = 'dead';
      srText = `Defeated in Shadow Rift${arenaPlayerStatus.defeatedBy ? ` by ${arenaPlayerStatus.defeatedBy}` : ''}. Press Respawn at Sanctuary to restore 100 health.`;
    } else if (meta && meta.event === 'respawned') {
      srKey = 'respawned';
      srText = 'Respawned at Safe Sanctuary with 100 health.';
    } else if (hp <= 28) {
      srKey = 'critical';
      srText = `Critical health: ${hp} out of ${maxHp}. Retreat to Sanctuary or drink a potion.`;
    }
    if (srKey !== lastArenaSrState) {
      lastArenaSrState = srKey;
      if (srText) arenaStatusSr.textContent = srText;
    }
  }

  renderArenaOverlays();
}

function renderArenaOverlays() {
  const inArena = currentZone === ZONE_ARENA;
  if (arenaHudBanner) {
    arenaHudBanner.hidden = !inArena;
  }
  if (arenaHudSub && inArena) {
    arenaHudSub.textContent = arenaPlayerStatus.alive
      ? 'Beware chasing Rift Spiders, Void Wraiths, Magma Colossi & molten lava! Drink potions to heal +35 HP.'
      : `💀 Defeated by ${arenaPlayerStatus.defeatedBy || 'the Shadow Rift'} — click Respawn below to rise at Sanctuary!`;
  }
  if (arenaDeathOverlay) {
    const showDeath = inArena && !arenaPlayerStatus.alive;
    const wasHidden = arenaDeathOverlay.hidden;
    arenaDeathOverlay.hidden = !showDeath;
    if (showDeath) {
      if (arenaDeathCause) {
        arenaDeathCause.textContent = arenaPlayerStatus.defeatedBy
          ? `Defeated by ${arenaPlayerStatus.defeatedBy} — your coins & Backpack treasures are completely safe!`
          : 'Defeated in battle — your coins & Backpack treasures are completely safe!';
      }
      if (wasHidden && arenaRespawnBtn) {
        arenaRespawnBtn.focus();
      }
    }
  }
}

async function respawnInArena() {
  if (!sessionToken || arenaRespawnInFlight) return;
  arenaRespawnInFlight = true;
  try {
    const res = await fetch('/api/ella-world/arena/respawn', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.arena) world3d.syncArenaSnapshot(data.arena);
      if (data.arenaStatus) applyArenaPlayerStatus(data.arenaStatus, data);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    showToast('⚠️ Could not respawn right now — try again!');
  } finally {
    arenaRespawnInFlight = false;
  }
}

function renderZoneHud(zone) {
  const inDungeon = zone === ZONE_DUNGEON;
  const inArena = zone === ZONE_ARENA;
  const inPrison = zone === ZONE_PRISON;
  if (hudZonePill) {
    hudZonePill.classList.toggle('is-dungeon', inDungeon);
    hudZonePill.classList.toggle('is-arena', inArena);
    hudZonePill.classList.toggle('is-prison', inPrison);
    hudZonePill.textContent = inPrison
      ? "⚖️ Debtor's Prison"
      : inArena
        ? '⚔️ Shadow Rift'
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
  if (warpArenaBtn) {
    warpArenaBtn.textContent = inArena ? '⛲ Exit Rift' : '⚔️ Shadow Rift';
  }
  if (touchActionBtn) {
    touchActionBtn.textContent = inPrison ? '🪨 WORK (E)' : '✨ BLAST (E)';
  }
  if ((inDungeon || inArena || inPrison) && world3d.buildMode) {
    world3d.buildMode = false;
    world3d.ghostMesh.visible = false;
    if (buildToolbar) buildToolbar.hidden = true;
    if (toggleBuildBtn) toggleBuildBtn.classList.remove('active');
  }
  if (runwayHudBar) {
    runwayHudBar.hidden = inPrison || inArena;
    if (inDungeon || inArena || inPrison) {
      runwayHudBar.classList.remove('near-runway');
    }
  }
  renderPrisonBanner();
  renderArenaOverlays();
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
  if (payload.arena) {
    world3d.syncArenaSnapshot(payload.arena);
  }
  if (payload.arenaStatus) {
    applyArenaPlayerStatus(payload.arenaStatus);
  } else if (nextZone !== ZONE_ARENA) {
    world3d.setArenaDead(false);
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
    if (target && typeof target === 'object' && target.type === 'decorate_loot') {
      openBuildModeForLootItem(target.itemId);
      return;
    }
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
    if (target === 'hard_obby') {
      world3d.warpToHardObbyCheckpoint();
      showToast('🔥 Returned to Starlight Plaza & teleported to the Starlight Sky Gauntlet!');
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
    } else if (nextZone === ZONE_ARENA) {
      showToast(
        '⚔️ Entered the Shadow Rift Arena! Stay on the bridges over molten lava — watch your 100 HP against chasing bosses!',
        5400
      );
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
      body: JSON.stringify({ token: sessionToken, zone: targetZone })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok && typeof data.zone === 'number') {
      applyLocalZoneTransition(data);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    } else if (!res.ok) {
      showToast(`⚠️ Portal transition failed (HTTP ${res.status}).`);
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
      if (data.arena) world3d.syncArenaSnapshot(data.arena);
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
    if (res.ok) {
      if (data.dungeon) world3d.syncDungeonSnapshot(data.dungeon);
      if (data.arena) world3d.syncArenaSnapshot(data.arena);
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
      if (data.arena) world3d.syncArenaSnapshot(data.arena);
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

async function cloneInventoryItem(itemId) {
  if (!sessionToken || !itemId) return;
  try {
    const res = await fetch('/api/ella-world/economy/clone', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, itemId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.economy) applyEconomySnapshot(data.economy);
      showToast(
        `🧬 Cloned +1 ${data.itemIcon || '✨'} ${data.itemName || itemId}! (Now ${data.newQty}x)`
      );
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  } catch {
    showToast('⚠️ Could not clone item right now.');
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

    if (qty > 0) {
      const btnRow = document.createElement('div');
      btnRow.className = 'backpack-action-btns';

      const weaponEquipMeta = {
        legendary_ray_gun: {
          acc: 'ray_gun',
          equipLabel: '🔫 Equip Ray Gun',
          equippedLabel: '✅ Ray Gun Equipped',
          equipToast:
            '🔫 Starlight Plasma Ray Gun equipped! Click or press E in Crystal Cavern / Shadow Rift or zap laser shows in Plaza!',
          unequipToast: '🎒 Holstered your Starlight Plasma Ray Gun.'
        },
        crystal_blade: {
          acc: 'crystal_blade',
          equipLabel: '⚔️ Equip Blade',
          equippedLabel: '✅ Blade Equipped',
          equipToast:
            '⚔️ Starlight Crystal Blade equipped! Deals 2 DMG melee cleave in Crystal Cavern & Shadow Rift!',
          unequipToast: '🎒 Sheathed your Starlight Crystal Blade.'
        },
        starlight_wand: {
          acc: 'starlight_wand',
          equipLabel: '🪄 Equip Wand',
          equippedLabel: '✅ Wand Equipped',
          equipToast:
            '🪄 Starlight Astral Wand equipped! Fires 16m star-bolts (2 DMG) & mends +8 HP per hit in Shadow Rift!',
          unequipToast: '🎒 Holstered your Starlight Astral Wand.'
        }
      }[item.id];

      if (weaponEquipMeta) {
        const isEquipped = selfUser?.avatar?.accessory === weaponEquipMeta.acc;
        const equipBtn = document.createElement('button');
        equipBtn.type = 'button';
        equipBtn.className = 'small-action-btn';
        equipBtn.textContent = isEquipped
          ? weaponEquipMeta.equippedLabel
          : weaponEquipMeta.equipLabel;
        equipBtn.addEventListener('click', async () => {
          if (!sessionToken || !selfUser) return;
          const nextAccessory =
            selfUser.avatar?.accessory === weaponEquipMeta.acc ? 'none' : weaponEquipMeta.acc;
          const nextAvatar = {
            ...SIGNATURE_GLAM_PRESETS.ella_idol,
            ...(selfUser.avatar || {}),
            accessory: nextAccessory,
            characterPreset: 'custom'
          };
          try {
            const res = await fetch('/api/ella-world/avatar', {
              method: 'POST',
              headers: JSON_HEADERS,
              body: JSON.stringify({ token: sessionToken, avatar: nextAvatar })
            });
            if (await handleHttpAuthCheck(res)) return;
            if (!res.ok) return;
            const data = await res.json();
            selfUser = data.user;
            if (selfId && playersMap.has(selfId)) {
              playersMap.get(selfId).user = selfUser;
            }
            if (world3d.selfAvatar) {
              world3d.selfAvatar.updateAvatarAppearance(selfUser.avatar);
            }
            renderBackpackGrid();
            showToast(
              nextAccessory === weaponEquipMeta.acc
                ? weaponEquipMeta.equipToast
                : weaponEquipMeta.unequipToast
            );
          } catch {
            showToast('⚠️ Could not update weapon equip state right now.');
          }
        });
        btnRow.appendChild(equipBtn);
      } else if (item.id === 'pet_egg') {
        const hatchBtn = document.createElement('button');
        hatchBtn.type = 'button';
        hatchBtn.className = 'small-action-btn';
        hatchBtn.textContent = '🥚 Hatch Egg!';
        hatchBtn.addEventListener('click', () => {
          if (backpackDialog && backpackDialog.open) backpackDialog.close();
          openCompanionsModal('pets');
          mutateCompanions({ action: 'hatch_egg' });
        });
        btnRow.appendChild(hatchBtn);
      } else if (item.id === 'rift_beacon') {
        const summonBtn = document.createElement('button');
        summonBtn.type = 'button';
        summonBtn.className = 'small-action-btn';
        summonBtn.textContent = '🔮 Summon Boss';
        summonBtn.addEventListener('click', () => {
          if (currentZone !== ZONE_ARENA) {
            showToast('⚔️ Enter Shadow Rift (K) first to unleash your Rift Beacon at the North Altar!');
            return;
          }
          if (backpackDialog && backpackDialog.open) backpackDialog.close();
          summonArenaBoss(true);
        });
        btnRow.appendChild(summonBtn);
      } else if (item.category === 'consumable') {
        const useBtn = document.createElement('button');
        useBtn.type = 'button';
        useBtn.className = 'small-action-btn';
        useBtn.textContent =
          item.id === 'prism_block'
            ? '🌈 Place Block'
            : item.id === 'starlight_potion'
              ? '✨ Drink Elixir'
              : '🧪 Drink Potion';
        useBtn.addEventListener('click', () => useInventoryItem(item.id));
        btnRow.appendChild(useBtn);
      } else if (item.category === 'cosmetic') {
        const wearBtn = document.createElement('button');
        wearBtn.type = 'button';
        wearBtn.className = 'small-action-btn';
        wearBtn.textContent = '👗 Equip in Glam';
        wearBtn.addEventListener('click', () => {
          if (backpackDialog && backpackDialog.open) backpackDialog.close();
          openGlamSalon();
          const accTab = document.getElementById('glam-tab-acc');
          if (accTab) activateGlamTab(accTab, false, false);
          if (draftAvatar && item.cosmeticSlot && item.cosmeticValue) {
            draftAvatar[item.cosmeticSlot] = item.cosmeticValue;
            draftAvatar.characterPreset = 'custom';
            previewDraftInMirror('#38bdf8');
            setGlamStageFraming(item.cosmeticSlot === 'hat' ? 'head' : 'torso', true);
            scheduleAutoSaveAvatar(120);
          }
        });
        btnRow.appendChild(wearBtn);
      }

      const decorBtn = document.createElement('button');
      decorBtn.type = 'button';
      decorBtn.className = 'small-action-btn';
      decorBtn.textContent = '🏠 Decorate Home';
      decorBtn.addEventListener('click', () => {
        if (backpackDialog && backpackDialog.open) backpackDialog.close();
        openBuildModeForLootItem(item.id);
      });
      btnRow.appendChild(decorBtn);

      const canClone =
        Array.isArray(playerEconomy.cloneableItems) &&
        playerEconomy.cloneableItems.includes(item.id);
      if (canClone) {
        const cloneBtn = document.createElement('button');
        cloneBtn.type = 'button';
        cloneBtn.className = 'small-action-btn';
        cloneBtn.textContent = '🧬 Clone (+1)';
        cloneBtn.title = 'Parent-authorized: make a clone of this gifted item!';
        cloneBtn.addEventListener('click', () => cloneInventoryItem(item.id));
        btnRow.appendChild(cloneBtn);
      }

      actions.appendChild(btnRow);
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
    const inArena = pZone === ZONE_ARENA;
    const inPrison = pZone === ZONE_PRISON;
    const li = document.createElement('li');
    li.className = `friend-card${p.speaking ? ' speaking' : ''}`;
    li.id = `friend-card-${pid}`;

    const topRow = document.createElement('div');
    topRow.className = 'friend-card-top';

    const nameSpan = document.createElement('span');
    nameSpan.className = 'friend-name';
    const roleBadge =
      p.user.role === 'host' ? '👑 ' : p.user.role === 'subadmin' ? '🛡️ ' : '🧒 ';
    nameSpan.textContent = `${roleBadge}${p.user.displayName}${isMe ? ' (You)' : ''}`;

    const statusTag = document.createElement('span');
    statusTag.className = 'friend-voice-tag';
    statusTag.id = `friend-voice-tag-${pid}`;
    statusTag.textContent = p.muted ? '🔇 Muted' : p.speaking ? '🔊 Talking' : '🎤 Ready';

    topRow.append(nameSpan, statusTag);
    li.appendChild(topRow);

    const metaRow = document.createElement('div');
    metaRow.className = 'friend-meta-row';

    const zoneBadge = document.createElement('span');
    zoneBadge.className = `friend-zone-tag${inDungeon ? ' in-dungeon' : ''}${inArena ? ' in-arena' : ''}${inPrison ? ' in-prison' : ''}`;
    zoneBadge.textContent = inPrison
      ? "⚖️ Debtor's Prison"
      : inArena
        ? '⚔️ Shadow Rift'
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

if (warpArenaBtn) {
  warpArenaBtn.addEventListener('click', () => {
    pendingPostPortalWarp = null;
    if (currentZone === ZONE_PRISON) {
      showToast("⚖️ Clear your Debtor's Prison chores & negative debt first!");
      return;
    }
    requestZoneTransition(currentZone === ZONE_ARENA ? ZONE_PLAZA : ZONE_ARENA);
  });
}

if (arenaRespawnBtn) {
  arenaRespawnBtn.addEventListener('click', () => {
    respawnInArena();
  });
}

if (arenaFleePlazaBtn) {
  arenaFleePlazaBtn.addEventListener('click', () => {
    pendingPostPortalWarp = null;
    requestZoneTransition(ZONE_PLAZA);
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
  cancelActiveObbyRun(true);
  if (currentZone !== ZONE_PLAZA) {
    pendingPostPortalWarp = 'obby';
    requestZoneTransition(ZONE_PLAZA);
  } else {
    world3d.warpToObbyCheckpoint();
    showToast('🏃‍♀️ Teleported to Sky Obby!');
  }
});

const warpHardObbyBtn = document.getElementById('warp-hard-obby-btn');
if (warpHardObbyBtn) {
  warpHardObbyBtn.addEventListener('click', () => {
    cancelActiveObbyRun(true);
    if (currentZone !== ZONE_PLAZA) {
      pendingPostPortalWarp = 'hard_obby';
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToHardObbyCheckpoint();
      showToast('🔥 Teleported to Starlight Sky Gauntlet (Pro Obby)!');
    }
  });
}

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

// Dress to Impress Runway Theme Spin, Timed Fashion Show & 1-5 Star Voting
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

if (runwayStartShowBtn) {
  runwayStartShowBtn.addEventListener('click', async () => {
    if (!sessionToken) return;
    const myU = String(selfUser?.username || '').toLowerCase();
    const phase = runwayShowState?.phase || 'idle';
    const inShow = (runwayShowState?.contestants || []).some(
      (c) => String(c.username || '').toLowerCase() === myU
    );
    const action = phase === 'dress_up' && !inShow ? 'join_show' : 'start_show';
    const res = await fetch('/api/ella-world/runway', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      if (data.runway) applyRunwaySummary(data.runway);
      showToast(
        action === 'join_show'
          ? '👗 Joined the Fashion Show! Open Glam Studio (C) — 45s Dress-Up!'
          : '🎬 Fashion Show Started! Friends got an invite — 45s Dress-Up!',
        4600
      );
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  });
}

if (runwayReadyBtn) {
  runwayReadyBtn.addEventListener('click', async () => {
    if (!sessionToken) return;
    const res = await fetch('/api/ella-world/runway', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'ready_show' })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      if (data.runway) applyRunwaySummary(data.runway);
      showToast('✅ Outfit locked & Ready for the Catwalk!');
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  });
}

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
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      applyRunwaySummary(data.runway);
      if (stars === 5 && world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      showToast(`⭐ Gave ${stars} star${stars > 1 ? 's' : ''} for "${data.runway?.theme || 'Runway'}"!`);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  });
}

// Build Toolbar: Action Toolbar, Single-Row Category Tabs, Furniture Catalog, Loot Pedestals & Swatches
const SUPPORTS_SCROLL_TIMELINE =
  typeof CSS !== 'undefined' && typeof CSS.supports === 'function' && CSS.supports('animation-timeline', 'scroll()');
const scrollFadeObservers = new WeakMap();

function refreshScrollFadeFallback(scroller) {
  if (!scroller || SUPPORTS_SCROLL_TIMELINE || typeof IntersectionObserver === 'undefined') return;
  const prev = scrollFadeObservers.get(scroller);
  if (prev) {
    prev.disconnect();
    scrollFadeObservers.delete(scroller);
  }
  scroller.classList.remove('more-start', 'more-end');
  const first = scroller.firstElementChild;
  const last = scroller.lastElementChild;
  if (!first || !last) return;
  const io = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.target === first) {
          scroller.classList.toggle('more-start', !entry.isIntersecting);
        }
        if (entry.target === last) {
          scroller.classList.toggle('more-end', !entry.isIntersecting);
        }
      }
    },
    { root: scroller, threshold: 1, rootMargin: '1px' }
  );
  io.observe(first);
  io.observe(last);
  scrollFadeObservers.set(scroller, io);
}

function setBuildTool(tool) {
  const nextTool = tool === 'remove' ? 'remove' : 'place';
  world3d.buildTool = nextTool;
  if (buildActionPlaceBtn) {
    buildActionPlaceBtn.setAttribute('aria-pressed', String(nextTool === 'place'));
  }
  if (buildActionRemoveBtn) {
    buildActionRemoveBtn.setAttribute('aria-pressed', String(nextTool === 'remove'));
  }
  if (world3d.buildMode) {
    world3d._updateBuildGhost();
  }
}

function syncBuildItemSelectionUi() {
  if (!buildPanelsStack) return;
  const isLoot = world3d.selectedBuildKind === 'loot_decor';
  for (const btn of buildPanelsStack.querySelectorAll('.build-item-btn')) {
    const btnKind = btn.getAttribute('data-build-kind');
    const btnLoot = btn.getAttribute('data-loot-id');
    const pressed = isLoot
      ? btnKind === 'loot_decor' && btnLoot === world3d.selectedLootItemId
      : btnKind === world3d.selectedBuildKind && !btnLoot;
    btn.setAttribute('aria-pressed', String(pressed));
  }
  if (buildHintTextEl) {
    if (isLoot) {
      buildHintTextEl.textContent = 'Pedestal tint:';
    } else if (world3d.selectedBuildKind === 'double_door') {
      buildHintTextEl.textContent = '2×2 Double Door tint (press Q to rotate, E to open/close):';
    } else if (world3d.selectedBuildKind === 'door' || world3d.selectedBuildKind === 'window') {
      buildHintTextEl.textContent = 'Frame tint (press E to open/close):';
    } else {
      buildHintTextEl.textContent = 'Paint tint:';
    }
  }
}

function selectBuildCatalogKind(kind) {
  world3d.selectedBuildKind = kind;
  world3d.selectedLootItemId = null;
  setBuildTool('place');
  syncBuildItemSelectionUi();
}

function selectBuildLootItem(itemId) {
  world3d.selectedBuildKind = 'loot_decor';
  world3d.selectedLootItemId = itemId;
  setBuildTool('place');
  syncBuildItemSelectionUi();
}

for (const entry of BUILD_CATALOG) {
  const rowEl = document.getElementById(`build-items-${entry.category}`);
  if (!rowEl) continue;
  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = 'build-item-btn';
  btn.setAttribute('data-build-kind', entry.kind);
  btn.setAttribute('aria-pressed', String(entry.kind === 'block'));
  btn.title =
    entry.kind === 'double_door'
      ? `${entry.label} (2×2 grand doorway • Press Q to rotate, E or Click either side to swing open!)`
      : entry.kind === 'door' || entry.kind === 'window'
        ? `${entry.label} — Click or press E after placing to swing open/shut!`
        : entry.height === 2
          ? `${entry.label} (2 blocks tall • Press Q to rotate)`
          : `${entry.label} (Press Q to rotate)`;
  btn.textContent = `${entry.icon} ${entry.label}`;
  btn.addEventListener('click', () => {
    selectBuildCatalogKind(entry.kind);
  });
  rowEl.appendChild(btn);
}

function renderBuildLootItems() {
  const lootRow = document.getElementById('build-items-loot');
  if (!lootRow) return;
  lootRow.innerHTML = '';

  const inv = playerEconomy.inventory || {};
  const ownedItems = [];
  if (itemCatalog.length > 0) {
    for (const item of itemCatalog) {
      const qty = Number(inv[item.id] || 0);
      if (qty > 0) {
        ownedItems.push({ item, qty });
      }
    }
  } else {
    for (const [id, rawQty] of Object.entries(inv)) {
      const qty = Number(rawQty || 0);
      if (qty > 0) {
        const fallback = catalogById.get(id) || { id, name: id, icon: '🏆', rarity: 'common' };
        ownedItems.push({ item: fallback, qty });
      }
    }
  }

  // Sort Legendary / Epic treasures first so star prizes stand out immediately
  const rarityWeight = { legendary: 5, epic: 4, rare: 3, uncommon: 2, common: 1 };
  ownedItems.sort(
    (a, b) => (rarityWeight[b.item.rarity] || 0) - (rarityWeight[a.item.rarity] || 0)
  );

  if (ownedItems.length === 0) {
    if (world3d.selectedBuildKind === 'loot_decor') {
      world3d.selectedLootItemId = null;
    }
    const emptySpan = document.createElement('span');
    emptySpan.className = 'build-empty-loot';
    emptySpan.textContent =
      '🗝️ Farm monsters & treasure chests in Crystal Cavern (G) to display 3D trophies in your house!';
    lootRow.appendChild(emptySpan);
    refreshScrollFadeFallback(lootRow);
    return;
  }

  if (
    world3d.selectedBuildKind === 'loot_decor' &&
    (!world3d.selectedLootItemId || Number(inv[world3d.selectedLootItemId] || 0) < 1)
  ) {
    world3d.selectedLootItemId = ownedItems[0].item.id;
  }

  for (const { item, qty } of ownedItems) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = `build-item-btn${item.rarity === 'legendary' ? ' is-legendary-loot' : ''}`;
    btn.setAttribute('data-build-kind', 'loot_decor');
    btn.setAttribute('data-loot-id', item.id);
    const isPressed =
      world3d.selectedBuildKind === 'loot_decor' && world3d.selectedLootItemId === item.id;
    btn.setAttribute('aria-pressed', String(isPressed));
    btn.title = `Display ${item.name} on a house pedestal (uses 1 from Backpack; pick up anytime to refund!)`;

    const labelSpan = document.createElement('span');
    labelSpan.textContent = `${item.icon || '🏆'} ${item.name}`;
    const qtyBadge = document.createElement('span');
    qtyBadge.className = 'build-loot-qty';
    qtyBadge.textContent = `${qty}x`;
    btn.append(labelSpan, qtyBadge);
    btn.addEventListener('click', () => {
      selectBuildLootItem(item.id);
    });
    lootRow.appendChild(btn);
  }
  refreshScrollFadeFallback(lootRow);
}

function activateBuildTab(nextTab, moveFocus = true) {
  if (!buildTablist || !buildPanelsStack || !nextTab) return;
  const tabs = [...buildTablist.querySelectorAll('[role="tab"]')];
  const panels = [...buildPanelsStack.querySelectorAll('[role="tabpanel"]')];

  for (const tab of tabs) {
    const active = tab === nextTab;
    tab.setAttribute('aria-selected', String(active));
    tab.tabIndex = active ? 0 : -1;
    tab.toggleAttribute('focusgroupstart', active);
  }

  for (const panel of panels) {
    const active = panel.getAttribute('aria-labelledby') === nextTab.id;
    if (active) {
      panel.removeAttribute('hidden');
      const scroller = panel.querySelector('.build-items-row');
      if (scroller) refreshScrollFadeFallback(scroller);
    } else {
      panel.setAttribute('hidden', 'until-found');
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

  if (nextTab.id === 'build-tab-loot') {
    renderBuildLootItems();
    const inv = playerEconomy.inventory || {};
    const firstOwned = Object.keys(inv).find((k) => Number(inv[k] || 0) > 0) || null;
    if (world3d.selectedBuildKind !== 'loot_decor' && firstOwned) {
      selectBuildLootItem(firstOwned);
    }
  }
}

if (buildTablist && buildPanelsStack) {
  const buildTabs = [...buildTablist.querySelectorAll('[role="tab"]')];
  const buildPanels = [...buildPanelsStack.querySelectorAll('[role="tabpanel"]')];

  buildTablist.addEventListener('click', (e) => {
    const tab = e.target instanceof Element ? e.target.closest('[role="tab"]') : null;
    if (tab && buildTablist.contains(tab)) {
      activateBuildTab(tab, false);
    }
  });

  // Reveal tab automatically when browser find-in-page matches inside a hidden="until-found" panel
  for (const panel of buildPanels) {
    panel.addEventListener('beforematch', () => {
      const tabId = panel.getAttribute('aria-labelledby');
      const tab = tabId ? document.getElementById(tabId) : null;
      if (tab) activateBuildTab(tab, false);
    });
  }

  // Fallback roving tabindex for engines without declarative HTML `focusgroup`
  if (!('focusGroup' in HTMLElement.prototype)) {
    buildTablist.addEventListener('keydown', (e) => {
      const current = e.target instanceof Element ? e.target.closest('[role="tab"]') : null;
      const idx = current ? buildTabs.indexOf(current) : -1;
      if (idx < 0) return;
      const dir = getComputedStyle(buildTablist).direction === 'rtl' ? -1 : 1;
      const move = {
        ArrowRight: dir,
        ArrowLeft: -dir,
        Home: -idx,
        End: buildTabs.length - 1 - idx
      }[e.key];
      if (move === undefined) return;
      e.preventDefault();
      const nextIdx = (idx + move + buildTabs.length) % buildTabs.length;
      activateBuildTab(buildTabs[nextIdx], true);
    });
  }
}

if (buildActionToolbar && !('focusGroup' in HTMLElement.prototype)) {
  const getToolbarBtns = () => [
    ...buildActionToolbar.querySelectorAll('button:not([disabled])')
  ];
  const initBtns = getToolbarBtns();
  initBtns.forEach((b, idx) => {
    b.tabIndex = idx === 0 ? 0 : -1;
  });
  buildActionToolbar.addEventListener('keydown', (e) => {
    const btns = getToolbarBtns();
    const idx = btns.indexOf(e.target);
    if (idx < 0) return;
    const dir = getComputedStyle(buildActionToolbar).direction === 'rtl' ? -1 : 1;
    let nextIdx = -1;
    if (e.key === 'ArrowRight') nextIdx = (idx + dir + btns.length) % btns.length;
    else if (e.key === 'ArrowLeft') nextIdx = (idx - dir + btns.length) % btns.length;
    else if (e.key === 'Home') nextIdx = 0;
    else if (e.key === 'End') nextIdx = btns.length - 1;
    if (nextIdx < 0) return;
    e.preventDefault();
    btns.forEach((b, bi) => {
      b.tabIndex = bi === nextIdx ? 0 : -1;
    });
    btns[nextIdx].focus();
  });
}

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
    setBuildTool('place');
  });
  colorSwatchBar.appendChild(btn);
});

function openBuildModeForLootItem(itemId) {
  if (currentZone === ZONE_PRISON) {
    showToast("⚖️ Block building is locked while serving a Debtor's Prison sentence!");
    return;
  }
  if (!playerModeration.canBuild) {
    showToast('🔒 Block building is currently locked for your account by a parent.');
    return;
  }
  if (currentZone !== ZONE_PLAZA) {
    pendingPostPortalWarp = { type: 'decorate_loot', itemId };
    requestZoneTransition(ZONE_PLAZA);
    return;
  }

  world3d.buildMode = true;
  if (buildToolbar) buildToolbar.hidden = false;
  if (toggleBuildBtn) toggleBuildBtn.classList.add('active');
  renderBuildLootItems();
  const lootTab = document.getElementById('build-tab-loot');
  if (lootTab) {
    activateBuildTab(lootTab, false);
  }
  selectBuildLootItem(itemId);
  if (buildTablist) refreshScrollFadeFallback(buildTablist);
  const itemDef = catalogById.get(itemId);
  showToast(
    `🏆 Ready to display ${itemDef?.name || 'your treasure'}! Click in your house to place on a pedestal (Q rotates).`
  );
}

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
    renderBuildLootItems();
    if (buildTablist) refreshScrollFadeFallback(buildTablist);
    for (const row of buildPanelsStack ? buildPanelsStack.querySelectorAll('.build-items-row') : []) {
      refreshScrollFadeFallback(row);
    }
    showToast(
      '🔨 Home Builder ON! Pick Structure, Kitchen, Bath, Bedroom, or My Loot • Press Q to rotate 90°!'
    );
  }
}

toggleBuildBtn.addEventListener('click', toggleBuildMode);

if (buildActionPlaceBtn) {
  buildActionPlaceBtn.addEventListener('click', () => {
    setBuildTool('place');
  });
}

if (buildActionRemoveBtn) {
  buildActionRemoveBtn.addEventListener('click', () => {
    setBuildTool('remove');
  });
}

if (buildRotateBtn) {
  buildRotateBtn.addEventListener('click', () => {
    world3d.rotateBuildSelection(1);
  });
}

renderBuildLootItems();

// Fullscreen HUD Mode (Default: Clean Fullscreen with Chat) & Per-Panel Toggles
const isCoarsePointerDevice =
  typeof window.matchMedia === 'function' && window.matchMedia('(pointer: coarse)').matches;

function setHudFullscreenMode(isFullscreen) {
  if (!gameHud) return;
  gameHud.classList.toggle('is-fullscreen-hud', Boolean(isFullscreen));
  if (toggleHudModeBtn) {
    toggleHudModeBtn.setAttribute('aria-expanded', String(!isFullscreen));
    toggleHudModeBtn.textContent = isFullscreen ? '🎛️ Controls' : '⛶ Fullscreen HUD';
    toggleHudModeBtn.title = isFullscreen
      ? 'Show HUD Controls & Panel Toggles (Shortcut: \\)'
      : 'Return to Clean Fullscreen HUD with Chat (Shortcut: \\)';
  }
}

function bindHudPanelToggle(btn, className, initialVisible, invertClass = false) {
  if (!btn || !gameHud) return;
  const applyState = (visible) => {
    gameHud.classList.toggle(className, invertClass ? !visible : visible);
    btn.setAttribute('aria-pressed', String(visible));
  };
  applyState(Boolean(initialVisible));
  btn.addEventListener('click', () => {
    const nextVisible = btn.getAttribute('aria-pressed') !== 'true';
    applyState(nextVisible);
  });
}

if (toggleHudModeBtn) {
  toggleHudModeBtn.addEventListener('click', () => {
    const currentlyFullscreen = gameHud.classList.contains('is-fullscreen-hud');
    setHudFullscreenMode(!currentlyFullscreen);
  });
}

bindHudPanelToggle(toggleFriendsPanelBtn, 'show-friends-panel', false);
bindHudPanelToggle(toggleRunwayBarBtn, 'show-runway-bar', false);
bindHudPanelToggle(toggleEmoteBarBtn, 'show-emote-bar', false);
bindHudPanelToggle(toggleTouchControlsBtn, 'show-touch-controls', isCoarsePointerDevice);

function setChatPanelVisible(visible) {
  if (!gameHud) return;
  const isVisible = Boolean(visible);
  gameHud.classList.toggle('hide-chat-panel', !isVisible);
  if (toggleChatPanelBtn) {
    toggleChatPanelBtn.setAttribute('aria-pressed', String(isVisible));
  }
  if (reopenChatBtn) {
    reopenChatBtn.hidden = isVisible;
    reopenChatBtn.setAttribute('aria-expanded', String(isVisible));
  }
}

setChatPanelVisible(true);
if (toggleChatPanelBtn) {
  toggleChatPanelBtn.addEventListener('click', () => {
    const currentlyVisible = !gameHud?.classList.contains('hide-chat-panel');
    setChatPanelVisible(!currentlyVisible);
  });
}
if (closeChatPanelBtn) {
  closeChatPanelBtn.addEventListener('click', () => {
    setChatPanelVisible(false);
    if (reopenChatBtn && !reopenChatBtn.hidden) {
      reopenChatBtn.focus();
    }
  });
}
if (reopenChatBtn) {
  reopenChatBtn.addEventListener('click', () => {
    setChatPanelVisible(true);
    chatInput?.focus();
  });
}

setHudFullscreenMode(true);

if (toggleBrowserFullscreenBtn) {
  const syncBrowserFullscreenBtn = () => {
    const active = Boolean(document.fullscreenElement);
    toggleBrowserFullscreenBtn.setAttribute('aria-pressed', String(active));
    toggleBrowserFullscreenBtn.title = active
      ? 'Exit Browser Fullscreen'
      : 'Enter Browser Fullscreen';
  };
  toggleBrowserFullscreenBtn.addEventListener('click', async () => {
    try {
      if (document.fullscreenElement) {
        await document.exitFullscreen();
      } else {
        await document.documentElement.requestFullscreen();
      }
    } catch {
      // Ignore browser fullscreen permission rejections
    }
    syncBrowserFullscreenBtn();
  });
  document.addEventListener('fullscreenchange', syncBrowserFullscreenBtn);
}

// Keyboard shortcuts
window.addEventListener('keydown', (e) => {
  const tag = document.activeElement ? document.activeElement.tagName : '';
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
  if (document.querySelector('dialog:modal')) return;

  if (e.key === 'Escape' && isSalonOpen) {
    closeGlamSalon(false);
    return;
  }

  if (e.key === '\\' && gameHud && !gameHud.hidden) {
    const currentlyFullscreen = gameHud.classList.contains('is-fullscreen-hud');
    setHudFullscreenMode(!currentlyFullscreen);
    return;
  }

  if (currentZone === ZONE_ARENA && !arenaPlayerStatus.alive && (e.key === 'Enter' || e.code === 'Space')) {
    e.preventDefault();
    respawnInArena();
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
  } else if (!isSalonOpen && k === 'p') {
    openCompanionsModal();
  } else if (!isSalonOpen && k === 'x') {
    toggleQuickHoverboard();
  } else if (!isSalonOpen && k === 'u') {
    openCraftModal();
  } else if (!isSalonOpen && k === 'g') {
    pendingPostPortalWarp = null;
    if (currentZone === ZONE_PRISON) {
      requestZoneTransition(ZONE_PLAZA);
    } else {
      requestZoneTransition(currentZone === ZONE_DUNGEON ? ZONE_PLAZA : ZONE_DUNGEON);
    }
  } else if (!isSalonOpen && k === 'k') {
    pendingPostPortalWarp = null;
    if (currentZone === ZONE_PRISON) {
      showToast("⚖️ Clear your Debtor's Prison chores & negative debt first!");
    } else {
      requestZoneTransition(currentZone === ZONE_ARENA ? ZONE_PLAZA : ZONE_ARENA);
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
    cancelActiveObbyRun(true);
    if (currentZone !== ZONE_PLAZA) {
      pendingPostPortalWarp = 'obby';
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToObbyCheckpoint();
      showToast('🏃‍♀️ Teleported to Sky Obby!');
    }
  } else if (!isSalonOpen && k === 't') {
    cancelActiveObbyRun(true);
    if (currentZone !== ZONE_PLAZA) {
      pendingPostPortalWarp = 'hard_obby';
      requestZoneTransition(ZONE_PLAZA);
    } else {
      world3d.warpToHardObbyCheckpoint();
      showToast('🔥 Teleported to Starlight Sky Gauntlet (Pro Obby)!');
    }
  } else if (['1', '2', '3', '4', '5', '6', '7', '8', '9'].includes(k)) {
    const emoteMap = {
      '1': 'pose',
      '2': 'peace',
      '3': 'dance',
      '4': 'spin',
      '5': 'wave',
      '6': 'cheer',
      '7': 'flip',
      '8': 'sit',
      '9': 'sleep'
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

// 11. Top-Layer Modal Glam Studio & 3D Fitting Stage (modern-web-guidance: dialog Rule 1 + tabs + toolbar)
const glamStage = new GlamSalonStage3D({
  container: document.getElementById('glam-stage-canvas-wrap')
});
const glamTablist = document.getElementById('glam-tablist');
const glamTabs = Array.from(glamTablist.querySelectorAll('[role="tab"]'));
const glamPanels = Array.from(avatarDialog.querySelectorAll('[role="tabpanel"]'));
const glamStageToolbar = document.getElementById('glam-stage-toolbar');
const glamPoseToolbar = document.getElementById('glam-pose-toolbar');
const glamSaveStatusEl = document.getElementById('glam-save-status');
const glamRunwayThemePill = document.getElementById('glam-runway-theme-pill');
const glamRunwayThemeText = document.getElementById('glam-runway-theme-text');
const glamAutoSpinBtn = document.getElementById('glam-autospin-btn');
const glamCompareBtn = document.getElementById('glam-compare-btn');
const glamSurpriseBtn = document.getElementById('glam-surprise-btn');

const supportsUntilFound = 'onbeforematch' in HTMLElement.prototype;
let isComparingOriginal = false;
let manualCameraFramingLocked = false;
let skipAutoFlushOnNextClose = false;

const TAB_ID_TO_STAGE_FRAME = {
  'glam-tab-looks': 'full',
  'glam-tab-hair': 'head',
  'glam-tab-tops': 'torso',
  'glam-tab-bottoms': 'shoes',
  'glam-tab-acc': 'full'
};

const HARMONIOUS_GLAM_PALETTES = [
  { name: 'Starlight Rose', top: '#f472b6', bottom: '#fce7f3', shoe: '#fbbf24', hat: '#fbbf24', acc: '#a5f3fc', hair: '#fde047', eye: '#60a5fa' },
  { name: 'Cyber Lagoon', top: '#38bdf8', bottom: '#1e293b', shoe: '#38bdf8', hat: '#22d3ee', acc: '#f472b6', hair: '#1e1b4b', eye: '#34d399' },
  { name: 'Lavender Coquette', top: '#c084fc', bottom: '#f8fafc', shoe: '#a855f7', hat: '#f472b6', acc: '#fde047', hair: '#78350f', eye: '#8b5cf6' },
  { name: 'Golden Gala', top: '#fbbf24', bottom: '#1e1b4b', shoe: '#fbbf24', hat: '#fde047', acc: '#f59e0b', hair: '#1e1b4b', eye: '#34d399' },
  { name: 'Mint Pop Idol', top: '#34d399', bottom: '#3b82f6', shoe: '#f8fafc', hat: '#fbbf24', acc: '#f472b6', hair: '#f472b6', eye: '#60a5fa' },
  { name: 'Tech-49 Horizon', top: '#e2e8f0', bottom: '#334155', shoe: '#1e293b', hat: '#334155', acc: '#38bdf8', hair: '#3b2516', eye: '#38bdf8' },
  { name: 'Velvet Cherry', top: '#1e1b4b', bottom: '#475569', shoe: '#ec4899', hat: '#a855f7', acc: '#f472b6', hair: '#dc2626', eye: '#ec4899' }
];

function setGlamStatusMessage(text, comparing = false) {
  if (!glamSaveStatusEl) return;
  glamSaveStatusEl.textContent = text;
  glamSaveStatusEl.classList.toggle('is-comparing', Boolean(comparing));
}

function setGlamStageFraming(presetName, lockManual = false) {
  const validPreset = glamStage.FRAMING_PRESETS[presetName] ? presetName : 'full';
  if (lockManual) {
    manualCameraFramingLocked = validPreset !== 'full';
  }
  glamStage.setFraming(validPreset, false);
  if (glamStageToolbar) {
    for (const btn of glamStageToolbar.querySelectorAll('[data-glam-frame]')) {
      const active = btn.getAttribute('data-glam-frame') === validPreset;
      btn.setAttribute('aria-pressed', String(active));
    }
  }
}

function setGlamStagePose(poseName, syncWorldEmote = false) {
  const nextPose = glamStage.setPose(poseName);
  if (glamPoseToolbar) {
    for (const btn of glamPoseToolbar.querySelectorAll('[data-glam-pose]')) {
      const active = btn.getAttribute('data-glam-pose') === nextPose;
      btn.setAttribute('aria-pressed', String(active));
    }
  }
  if (syncWorldEmote && world3d.selfAvatar) {
    world3d.selfAvatar.emote = nextPose === 'idle' ? null : nextPose;
    world3d._emitMoveNow();
    for (const b of document.querySelectorAll('.emote-btn')) {
      b.classList.toggle('active', b.getAttribute('data-emote') === world3d.selfAvatar.emote);
    }
  }
}

function exitGlamCompareIfActive() {
  if (!isComparingOriginal) return;
  isComparingOriginal = false;
  if (glamCompareBtn) {
    glamCompareBtn.setAttribute('aria-pressed', 'false');
  }
}

function activateGlamTab(nextTab, moveFocus = false, allowAutoFrame = true) {
  if (!nextTab) return;
  for (const tab of glamTabs) {
    const isSelected = tab === nextTab;
    tab.setAttribute('aria-selected', isSelected ? 'true' : 'false');
    tab.tabIndex = isSelected ? 0 : -1;
    tab.toggleAttribute('focusgroupstart', isSelected);
  }
  for (const panel of glamPanels) {
    const isTarget = panel.getAttribute('aria-labelledby') === nextTab.id;
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
  if (allowAutoFrame && isSalonOpen && !manualCameraFramingLocked) {
    const targetFrame = TAB_ID_TO_STAGE_FRAME[nextTab.id] || 'full';
    setGlamStageFraming(targetFrame, false);
  }
}

for (const tab of glamTabs) {
  tab.addEventListener('click', () => activateGlamTab(tab, false, true));
}

// Sync selection if browser find-in-page reveals a `hidden="until-found"` panel
for (const panel of glamPanels) {
  panel.addEventListener('beforematch', () => {
    const tabId = panel.getAttribute('aria-labelledby');
    const matchingTab = tabId ? document.getElementById(tabId) : null;
    if (matchingTab) activateGlamTab(matchingTab, false, false);
  });
}

// Keyboard roving tabindex fallback for browsers without native CSS/HTML `focusgroup`
if (!('focusGroup' in HTMLElement.prototype)) {
  glamTablist.addEventListener('keydown', (e) => {
    const currentIdx = glamTabs.indexOf(document.activeElement);
    if (currentIdx === -1) return;
    const rtl = getComputedStyle(glamTablist).direction === 'rtl';
    let nextIdx = -1;
    if (e.key === 'ArrowRight') {
      nextIdx = (currentIdx + (rtl ? -1 : 1) + glamTabs.length) % glamTabs.length;
    } else if (e.key === 'ArrowLeft') {
      nextIdx = (currentIdx + (rtl ? 1 : -1) + glamTabs.length) % glamTabs.length;
    } else if (e.key === 'Home') {
      nextIdx = 0;
    } else if (e.key === 'End') {
      nextIdx = glamTabs.length - 1;
    }
    if (nextIdx !== -1) {
      e.preventDefault();
      activateGlamTab(glamTabs[nextIdx], true, true);
    }
  });
}

function installHorizontalToolbarFocusFallback(toolbarEl) {
  if (!toolbarEl || 'focusGroup' in HTMLElement.prototype) return;
  const getItems = () => [
    ...toolbarEl.querySelectorAll('button:not([disabled]):not([aria-disabled="true"])')
  ];
  const initialItems = getItems();
  initialItems.forEach((btn, i) => {
    btn.tabIndex = i === 0 ? 0 : -1;
  });
  toolbarEl.addEventListener('focusin', ({ target }) => {
    const items = getItems();
    if (items.includes(target)) {
      items.forEach((btn) => {
        btn.tabIndex = btn === target ? 0 : -1;
      });
    }
  });
  toolbarEl.addEventListener('keydown', (e) => {
    const items = getItems();
    const idx = items.indexOf(e.target);
    if (idx < 0 || items.length === 0) return;
    const rtl = getComputedStyle(toolbarEl).direction === 'rtl';
    let nextIdx = -1;
    if (e.key === 'ArrowRight') {
      nextIdx = (idx + (rtl ? -1 : 1) + items.length) % items.length;
    } else if (e.key === 'ArrowLeft') {
      nextIdx = (idx + (rtl ? 1 : -1) + items.length) % items.length;
    } else if (e.key === 'Home') {
      nextIdx = 0;
    } else if (e.key === 'End') {
      nextIdx = items.length - 1;
    }
    if (nextIdx < 0) return;
    e.preventDefault();
    items.forEach((btn, bi) => {
      btn.tabIndex = bi === nextIdx ? 0 : -1;
    });
    items[nextIdx].focus();
  });
}

installHorizontalToolbarFocusFallback(glamStageToolbar);
installHorizontalToolbarFocusFallback(glamPoseToolbar);

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

function previewDraftInMirror(sparkleHex = null) {
  if (!draftAvatar) return;
  exitGlamCompareIfActive();
  draftAvatar.torsoColor = draftAvatar.topColor;
  draftAvatar.legsColor = draftAvatar.bottomColor;
  syncSalonControlsFromDraft();
  if (selfUser) {
    glamStage.setCharacter(selfUser, draftAvatar);
    if (sparkleHex) {
      glamStage.triggerSparkleBurst(sparkleHex);
    }
  }
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
    if (world3d.selfAvatar && !isComparingOriginal) {
      world3d.selfAvatar.updateAvatarAppearance(selfUser.avatar);
    }
    if (!isComparingOriginal) {
      setGlamStatusMessage('✨ Saved to World!', false);
    }
    if (showSuccessToast) {
      showToast('💖 Slay! Your new 3D Glam Runway Look is saved & live for all friends!');
    }
    return true;
  } catch {
    setGlamStatusMessage('⚠️ Offline — will retry on save', false);
    return false;
  }
}

function scheduleAutoSaveAvatar(delayMs = 240) {
  if (avatarAutoSaveTimer) clearTimeout(avatarAutoSaveTimer);
  if (!isComparingOriginal) {
    setGlamStatusMessage('⏳ Saving look…', false);
  }
  avatarAutoSaveTimer = setTimeout(() => {
    avatarAutoSaveTimer = null;
    persistAvatarDraftNow(false);
  }, delayMs);
}

function randomizeGlamOutfit() {
  if (!draftAvatar) return;
  const pick = (arr) => arr[Math.floor(Math.random() * arr.length)];
  const inv = playerEconomy.inventory || {};

  const isMale =
    draftAvatar.bodyType === 'male' ||
    draftAvatar.characterPreset === 'jack_tech49' ||
    draftAvatar.characterPreset === 'maverick_cabin';

  const palette = pick(HARMONIOUS_GLAM_PALETTES);
  const skinTones = ['#ffe4d0', '#ffdfc4', '#f3c6a5', '#e5b887', '#c68652', '#8d5524'];

  const hairPool = isMale
    ? ['tech_crop', 'maverick_quiff', 'lizzy_bob']
    : ['twin_pigtails', 'glam_ponytail', 'katey_long', 'amy_waves', 'lizzy_bob', 'mom_bun'];

  const topPool = isMale
    ? ['tech_jacket', 'hero_henley', 'crew_tee', 'sweater', 'star_tee']
    : ['glam_gown', 'puff_blouse', 'star_tee', 'sweater', 'crop_top', 'crew_tee'];

  const bottomPool = isMale
    ? ['tech_pants', 'cargo_pants', 'jeans']
    : ['pleated_skirt', 'ruffle_mini', 'jeans', 'cargo_pants', 'overalls'];

  const shoePool = isMale
    ? ['flight_boots', 'sneakers', 'platform_boots']
    : ['runway_heels', 'platform_boots', 'sneakers'];

  const hatPool = isMale
    ? ['none', 'recon_cap', 'astronaut', 'tophat']
    : ['crown', 'hair_bow', 'catears', 'beret', 'wizard', 'none'];
  if (Number(inv.cosmetic_hat_obsidian_crown || 0) > 0) hatPool.push('obsidian_crown');
  if (Number(inv.cosmetic_hat_shadow_hood || 0) > 0) hatPool.push('ninja');

  const accPool = isMale
    ? ['aviators', 'sunglasses', 'none']
    : ['star_purse', 'wings', 'sunglasses', 'choker', 'pearls', 'none'];
  if (Number(inv.cosmetic_acc_crystal_wings || 0) > 0) accPool.push('crystal_wings');
  if (Number(inv.cosmetic_acc_star_lantern || 0) > 0) accPool.push('star_lantern');
  if (Number(inv.legendary_ray_gun || 0) > 0) accPool.push('ray_gun');

  const chosenTop = pick(topPool);
  let chosenBottom = pick(bottomPool);
  if (chosenBottom === 'overalls' && chosenTop !== 'crew_tee' && chosenTop !== 'glam_gown') {
    chosenBottom = isMale ? 'tech_pants' : 'pleated_skirt';
  }

  draftAvatar = {
    ...draftAvatar,
    avatarMode: 'stylized',
    bodyType: isMale ? 'male' : 'female',
    characterPreset: 'custom',
    skinColor: draftAvatar.skinColor || pick(skinTones),
    eyeColor: palette.eye,
    hairStyle: pick(hairPool),
    hairColor: palette.hair,
    topStyle: chosenTop,
    topColor: palette.top,
    bottomStyle: chosenBottom,
    bottomColor: palette.bottom,
    shoeStyle: pick(shoePool),
    shoeColor: palette.shoe,
    hat: pick(hatPool),
    hatColor: palette.hat,
    accessory: pick(accPool),
    accessoryColor: palette.acc,
    fabricFinish: isMale ? pick(['matte', 'satin']) : pick(['matte', 'satin', 'glitter'])
  };

  previewDraftInMirror('#fde047');
  setGlamStatusMessage(`🎲 Surprise Look: ${palette.name}!`, false);
  scheduleAutoSaveAvatar(200);
}

function toggleGlamCompare() {
  if (!salonOriginalAvatar || !draftAvatar || !selfUser) return;
  isComparingOriginal = !isComparingOriginal;
  if (glamCompareBtn) {
    glamCompareBtn.setAttribute('aria-pressed', String(isComparingOriginal));
  }
  if (isComparingOriginal) {
    glamStage.setCharacter(selfUser, salonOriginalAvatar);
    setGlamStatusMessage('👀 Comparing previous outfit (pick any item to return)', true);
  } else {
    glamStage.setCharacter(selfUser, draftAvatar);
    setGlamStatusMessage('✨ Back to your new look', false);
  }
}

function openGlamSalon() {
  if (!selfUser) return;
  draftAvatar = {
    ...SIGNATURE_GLAM_PRESETS.ella_idol,
    ...(selfUser.avatar || {})
  };
  salonOriginalAvatar = JSON.parse(JSON.stringify(draftAvatar));
  isComparingOriginal = false;
  manualCameraFramingLocked = false;
  if (glamCompareBtn) {
    glamCompareBtn.setAttribute('aria-pressed', 'false');
  }
  if (glamRunwayThemePill && glamRunwayThemeText) {
    const themeText = currentRunwayTheme || runwayThemeLabel?.textContent?.trim() || '';
    if (themeText) {
      glamRunwayThemeText.textContent = themeText;
      glamRunwayThemePill.hidden = false;
    }
  }
  setGlamStatusMessage('✨ Live 3D Fitting Stage', false);
  syncSalonControlsFromDraft();

  if (!avatarDialog.open) {
    if (typeof avatarDialog.showModal === 'function') {
      avatarDialog.showModal();
    } else {
      avatarDialog.setAttribute('open', '');
    }
  }

  isSalonOpen = true;
  world3d.setSalonMirrorMode(true);

  glamStage.resetTurntable();
  glamStage.setCharacter(selfUser, draftAvatar);
  const currentWorldEmote = world3d.selfAvatar?.emote || 'idle';
  setGlamStagePose(currentWorldEmote, false);
  setGlamStageFraming('full', false);
  glamStage.setFraming('full', true);
  glamStage.setActive(true);
  glamStage.triggerSparkleBurst(draftAvatar.topColor || '#f472b6');
  requestAnimationFrame(() => {
    if (isSalonOpen) glamStage.handleResize();
  });
}

function finalizeGlamSalonClosed() {
  if (skipAutoFlushOnNextClose) {
    skipAutoFlushOnNextClose = false;
  } else if (avatarAutoSaveTimer) {
    persistAvatarDraftNow(false);
  }

  if (isComparingOriginal && draftAvatar && world3d.selfAvatar) {
    world3d.selfAvatar.updateAvatarAppearance(draftAvatar);
  }
  exitGlamCompareIfActive();
  avatarDialog.returnValue = '';
  glamStage.setActive(false);
  isSalonOpen = false;
  world3d.setSalonMirrorMode(false);
}

function closeGlamSalon(revertToOriginal = false) {
  if (revertToOriginal && salonOriginalAvatar) {
    skipAutoFlushOnNextClose = true;
    draftAvatar = JSON.parse(JSON.stringify(salonOriginalAvatar));
    previewDraftInMirror();
    persistAvatarDraftNow(false);
  }
  if (avatarDialog.open) {
    avatarDialog.close();
  } else {
    finalizeGlamSalonClosed();
  }
}

avatarDialog.addEventListener('close', () => {
  finalizeGlamSalonClosed();
});

document.getElementById('open-avatar-btn').addEventListener('click', () => {
  if (isSalonOpen) closeGlamSalon(false);
  else openGlamSalon();
});
document.getElementById('close-salon-btn').addEventListener('click', () => closeGlamSalon(false));
document.getElementById('cancel-salon-btn').addEventListener('click', () => {
  closeGlamSalon(true);
  showToast('↩️ Restored your previous outfit.');
});

if (glamStageToolbar) {
  glamStageToolbar.addEventListener('click', (e) => {
    const target = e.target instanceof Element ? e.target : null;
    if (!target) return;
    const frameBtn = target.closest('[data-glam-frame]');
    if (frameBtn) {
      const preset = frameBtn.getAttribute('data-glam-frame') || 'full';
      setGlamStageFraming(preset, true);
    }
  });
}

if (glamAutoSpinBtn) {
  glamAutoSpinBtn.addEventListener('click', () => {
    const enabled = glamStage.setAutoSpin(!glamStage.autoSpin);
    glamAutoSpinBtn.setAttribute('aria-pressed', String(enabled));
    setGlamStatusMessage(enabled ? '🔄 Turntable auto-spin ON' : '✨ Turntable auto-spin paused', isComparingOriginal);
  });
}

if (glamCompareBtn) {
  glamCompareBtn.addEventListener('click', () => {
    toggleGlamCompare();
  });
}

if (glamSurpriseBtn) {
  glamSurpriseBtn.addEventListener('click', () => {
    randomizeGlamOutfit();
  });
}

if (glamPoseToolbar) {
  glamPoseToolbar.addEventListener('click', (e) => {
    const target = e.target instanceof Element ? e.target : null;
    if (!target) return;
    const poseBtn = target.closest('[data-glam-pose]');
    if (poseBtn) {
      const pose = poseBtn.getAttribute('data-glam-pose') || 'idle';
      const nextPose = pose !== 'idle' && glamStage.currentPose === pose ? 'idle' : pose;
      setGlamStagePose(nextPose, true);
    }
  });
}

// In-modal keyboard shortcuts for poses (0-8) and quick salon toggle (C)
avatarDialog.addEventListener('keydown', (e) => {
  const tag = document.activeElement ? document.activeElement.tagName : '';
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
  if (e.ctrlKey || e.metaKey || e.altKey) return;

  const k = e.key.toLowerCase();
  if (k === 'c') {
    e.preventDefault();
    closeGlamSalon(false);
    return;
  }
  if (k === '0') {
    e.preventDefault();
    setGlamStagePose('idle', true);
    return;
  }
  const modalEmoteMap = {
    '1': 'pose',
    '2': 'peace',
    '3': 'dance',
    '4': 'spin',
    '5': 'wave',
    '6': 'cheer',
    '7': 'flip',
    '8': 'sit',
    '9': 'sleep'
  };
  if (modalEmoteMap[k]) {
    e.preventDefault();
    const desired = modalEmoteMap[k];
    const nextPose = glamStage.currentPose === desired ? 'idle' : desired;
    setGlamStagePose(nextPose, true);
  }
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
      previewDraftInMirror(draftAvatar.topColor || '#f472b6');
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
      if (draftAvatar.avatarMode === 'blocky' && field !== 'hat') {
        draftAvatar.avatarMode = 'stylized';
      }
      if (field === 'bottomStyle' && draftAvatar.topStyle === 'glam_gown') {
        draftAvatar.topStyle = val === 'overalls' ? 'crew_tee' : 'puff_blouse';
      }
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
      previewDraftInMirror('#f472b6');
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
      previewDraftInMirror(field === 'fabricFinish' ? '#fde047' : '#38bdf8');
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
      previewDraftInMirror(hex);
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
  skipAutoFlushOnNextClose = true;
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
  hideTopLayerToastPopover(fashionInvitePopover);
  hideTopLayerToastPopover(doorbellPopover);
  pendingInviteTrade = null;
  activeTrade = null;
  activeObbyRun = null;
  world3d.setActiveObbyRun(null);
  if (obbySpeedrunBanner) obbySpeedrunBanner.hidden = true;
  draftTradeCoins = 0;
  draftTradeItems = {};
  if (tradeDialog && tradeDialog.open) tradeDialog.close('cancelled');
  if (backpackDialog && backpackDialog.open) backpackDialog.close();
  if (companionsDialog && companionsDialog.open) companionsDialog.close();
  if (homesDialog && homesDialog.open) homesDialog.close();
  if (craftDialog && craftDialog.open) craftDialog.close();
  if (obbyBoardDialog && obbyBoardDialog.open) obbyBoardDialog.close();
  if (skyDialog && skyDialog.open) skyDialog.close();
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

function showSubadminFeedback(message) {
  if (!subadminFeedback) return;
  subadminFeedback.textContent = message;
  subadminFeedback.hidden = false;
}

function buildAvatarPresetSelect(currentPreset, username, onChoose) {
  const sel = document.createElement('select');
  sel.className = 'mod-chat-select';
  sel.setAttribute('aria-label', `Avatar look for ${username}`);
  const opts = [
    ['', '👗 Avatar Look...'],
    ['ella_idol', '👑 Ella Idol'],
    ['amy', '🐱 Amy Cat'],
    ['katey', '😎 Katey Cool'],
    ['jack_tech49', '🕶️ Jack Tech-49'],
    ['maverick_cabin', '🧢 Maverick Cabin']
  ];
  for (const [val, label] of opts) {
    const opt = document.createElement('option');
    opt.value = val;
    opt.textContent = label;
    if (val && currentPreset === val) opt.selected = true;
    sel.appendChild(opt);
  }
  sel.addEventListener('change', () => {
    if (!sel.value || !SIGNATURE_GLAM_PRESETS[sel.value]) return;
    onChoose(sel.value, SIGNATURE_GLAM_PRESETS[sel.value]);
  });
  return sel;
}

function renderSubadminFriends(friends, catalogOverride = null) {
  if (!subadminFriendsTbody) return;
  subadminFriendsTbody.replaceChildren();
  const list = Array.isArray(friends) ? friends : [];
  const activeCatalog = Array.isArray(catalogOverride) ? catalogOverride : itemCatalog;

  const giftUserSel = document.getElementById('subadmin-gift-username');
  if (giftUserSel) {
    const prevU = giftUserSel.value;
    giftUserSel.replaceChildren();
    for (const f of list) {
      const opt = document.createElement('option');
      opt.value = f.username;
      opt.textContent = `${f.displayName || f.username} (@${f.username})`;
      if (f.username === prevU) opt.selected = true;
      giftUserSel.appendChild(opt);
    }
  }
  const giftItemSel = document.getElementById('subadmin-gift-item');
  if (giftItemSel) {
    const prevI = giftItemSel.value;
    giftItemSel.replaceChildren();
    const sorted = [...activeCatalog].sort(
      (a, b) => Number(Boolean(b.parentExclusive)) - Number(Boolean(a.parentExclusive))
    );
    for (const it of sorted) {
      const opt = document.createElement('option');
      opt.value = it.id;
      opt.textContent = `${it.icon || '🎁'} ${it.name}${it.parentExclusive ? ' [Exclusive]' : ''}`;
      if (it.id === prevI) opt.selected = true;
      giftItemSel.appendChild(opt);
    }
  }

  if (list.length === 0) {
    const tr = document.createElement('tr');
    const td = document.createElement('td');
    td.colSpan = 4;
    td.textContent = 'No friend accounts available to moderate.';
    tr.appendChild(td);
    subadminFriendsTbody.appendChild(tr);
    return;
  }

  for (const f of list) {
    const tr = document.createElement('tr');
    const coins = Number(f.coins || 0);

    const tdName = document.createElement('td');
    const strong = document.createElement('strong');
    strong.textContent = `${f.displayName || f.username} (@${f.username})`;
    const stack = document.createElement('div');
    stack.className = 'admin-status-stack';
    const onlineSpan = document.createElement('span');
    onlineSpan.className = 'admin-online-pill';
    onlineSpan.textContent = f.online ? '🟢 Online' : '⚪ Offline';
    stack.appendChild(onlineSpan);
    if (f.inDebtorsPrison) {
      const pBadge = document.createElement('span');
      pBadge.className = 'prison-badge';
      pBadge.textContent = "⚖️ Debtor's Prison";
      stack.appendChild(pBadge);
    }
    if (f.suspended) {
      const sBadge = document.createElement('span');
      sBadge.className = 'suspended-badge';
      sBadge.textContent = '🚫 Paused by Parent';
      stack.appendChild(sBadge);
    }
    tdName.append(strong, stack);

    // Gold & Inventory Controls
    const tdEcon = document.createElement('td');
    const econTop = document.createElement('div');
    econTop.className = 'admin-econ-top';
    const coinBadge = document.createElement('span');
    coinBadge.className = `coin-balance-pill${coins < 0 ? ' is-in-debt' : ''}`;
    coinBadge.textContent = `🪙 ${coins.toLocaleString()}`;
    econTop.appendChild(coinBadge);

    const coinBtns = document.createElement('div');
    coinBtns.className = 'mod-coin-step-row';
    for (const [delta, label] of [
      [-25, '-25 🪙'],
      [25, '+25 🪙'],
      [100, '+100 🪙']
    ]) {
      const b = document.createElement('button');
      b.type = 'button';
      b.className = `mod-pill-btn${delta < 0 ? ' warn-pill' : ''}`;
      b.textContent = label;
      b.addEventListener('click', () =>
        subadminModerateFriend(
          { username: f.username, action: 'update_controls', coinDelta: delta },
          `🪙 Adjusted @${f.username}'s gold by ${delta > 0 ? `+${delta}` : delta}.`
        )
      );
      coinBtns.appendChild(b);
    }
    const setGoldBtn = document.createElement('button');
    setGoldBtn.type = 'button';
    setGoldBtn.className = 'mod-pill-btn';
    setGoldBtn.textContent = '✏️ Set Gold';
    setGoldBtn.addEventListener('click', () => {
      const raw = window.prompt(`Set exact gold balance for @${f.username}:`, String(coins));
      if (raw === null) return;
      const parsed = Math.trunc(Number(raw));
      if (!Number.isFinite(parsed)) return;
      subadminModerateFriend(
        { username: f.username, action: 'update_controls', setCoins: parsed },
        `🪙 Set @${f.username}'s gold to ${parsed.toLocaleString()}.`
      );
    });
    coinBtns.appendChild(setGoldBtn);
    tdEcon.append(econTop, coinBtns);

    const invEntries = Object.entries(f.inventory || {}).filter(([, q]) => Number(q) > 0);
    if (invEntries.length > 0) {
      const cloneSet = new Set(Array.isArray(f.cloneableItems) ? f.cloneableItems : []);
      const invWrap = document.createElement('div');
      invWrap.className = 'mod-coin-step-row';
      for (const [itemId, qty] of invEntries) {
        const def = catalogById.get(itemId);
        const allow = cloneSet.has(itemId);
        const addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.className = 'mod-pill-btn';
        addBtn.textContent = `${def?.icon || '🎁'} ${qty}x +1`;
        addBtn.title = `Give +1 ${def?.name || itemId} to @${f.username}`;
        addBtn.addEventListener('click', () =>
          subadminModerateFriend(
            { username: f.username, action: 'gift_item', itemId, qty: 1 },
            `🎁 Added +1 ${def?.name || itemId} to @${f.username}.`
          )
        );
        const cloneBtn = document.createElement('button');
        cloneBtn.type = 'button';
        cloneBtn.className = `mod-pill-btn${allow ? ' is-active-safe' : ''}`;
        cloneBtn.textContent = allow ? '🧬 On' : '🧬 Off';
        cloneBtn.title = `Toggle clone for ${def?.name || itemId}`;
        cloneBtn.addEventListener('click', () =>
          subadminModerateFriend(
            { username: f.username, action: 'gift_item', itemId, qty: 0, allowClone: !allow },
            `${!allow ? '🧬 Enabled' : '🔒 Disabled'} clone of ${def?.name || itemId} for @${f.username}.`
          )
        );
        const rmBtn = document.createElement('button');
        rmBtn.type = 'button';
        rmBtn.className = 'mod-pill-btn warn-pill';
        rmBtn.textContent = '🗑️';
        rmBtn.title = `Remove ${def?.name || itemId} from @${f.username}`;
        rmBtn.addEventListener('click', () =>
          subadminModerateFriend(
            { username: f.username, action: 'gift_item', itemId, setQty: 0 },
            `🗑️ Removed ${def?.name || itemId} from @${f.username}.`
          )
        );
        invWrap.append(addBtn, cloneBtn, rmBtn);
      }
      tdEcon.appendChild(invWrap);
    }

    // Customize & Moderation Controls
    const tdControls = document.createElement('td');
    const wrap = document.createElement('div');
    wrap.className = 'mod-controls-wrap';

    const presetSel = buildAvatarPresetSelect(
      f.avatar?.characterPreset || '',
      f.username,
      (presetKey, presetObj) =>
        subadminModerateFriend(
          { username: f.username, action: 'update_controls', avatar: presetObj },
          `👗 Changed @${f.username}'s avatar look to "${presetKey}".`
        )
    );
    const renameBtn = document.createElement('button');
    renameBtn.type = 'button';
    renameBtn.className = 'mod-pill-btn';
    renameBtn.textContent = '✏️ Name';
    renameBtn.addEventListener('click', () => {
      const next = window.prompt(
        `New display name for @${f.username}:`,
        f.displayName || f.username
      );
      if (!next || !next.trim()) return;
      subadminModerateFriend(
        { username: f.username, action: 'update_controls', displayName: next.trim() },
        `✏️ Renamed @${f.username} to "${next.trim()}".`
      );
    });

    const micBtn = document.createElement('button');
    micBtn.type = 'button';
    micBtn.className = `mod-pill-btn${f.voiceMutedByParent ? ' is-locked' : ''}`;
    micBtn.textContent = f.voiceMutedByParent ? '🔇 Muted' : '🎤 Mic';
    micBtn.addEventListener('click', () =>
      subadminModerateFriend(
        {
          username: f.username,
          action: 'update_controls',
          voiceMutedByParent: !f.voiceMutedByParent
        },
        `${!f.voiceMutedByParent ? '🔇 Muted' : '🎤 Unmuted'} @${f.username}'s microphone.`
      )
    );

    const chatSelect = document.createElement('select');
    chatSelect.className = 'mod-chat-select';
    chatSelect.setAttribute('aria-label', `Chat mode for ${f.username}`);
    [
      ['normal', '💬 Normal'],
      ['quick_only', '🛡️ Safe Quick'],
      ['muted', '🔇 Chat Off']
    ].forEach(([val, label]) => {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = label;
      if ((f.chatMode || 'normal') === val) opt.selected = true;
      chatSelect.appendChild(opt);
    });
    chatSelect.addEventListener('change', () =>
      subadminModerateFriend(
        {
          username: f.username,
          action: 'update_controls',
          chatMode: chatSelect.value
        },
        `💬 Set @${f.username}'s chat mode to "${chatSelect.value}".`
      )
    );

    const buildLocked = f.canBuild === false;
    const buildBtn = document.createElement('button');
    buildBtn.type = 'button';
    buildBtn.className = `mod-pill-btn${buildLocked ? ' is-locked' : ''}`;
    buildBtn.textContent = buildLocked ? '🚫 Build' : '🔨 Build';
    buildBtn.addEventListener('click', () =>
      subadminModerateFriend(
        {
          username: f.username,
          action: 'update_controls',
          canBuild: buildLocked
        },
        `${buildLocked ? '🔨 Allowed' : '🚫 Locked'} building for @${f.username}.`
      )
    );

    const tradeLocked = f.canTrade === false;
    const tradeBtn = document.createElement('button');
    tradeBtn.type = 'button';
    tradeBtn.className = `mod-pill-btn${tradeLocked ? ' is-locked' : ''}`;
    tradeBtn.textContent = tradeLocked ? '🚫 Trade' : '🤝 Trade';
    tradeBtn.addEventListener('click', () =>
      subadminModerateFriend(
        {
          username: f.username,
          action: 'update_controls',
          canTrade: tradeLocked
        },
        `${tradeLocked ? '🤝 Allowed' : '🚫 Locked'} trading for @${f.username}.`
      )
    );

    wrap.append(presetSel, renameBtn, micBtn, chatSelect, buildBtn, tradeBtn);
    tdControls.appendChild(wrap);

    const tdSession = document.createElement('td');
    const kickBtn = document.createElement('button');
    kickBtn.type = 'button';
    kickBtn.className = 'mod-pill-btn warn-pill';
    kickBtn.textContent = '👢 Kick';
    kickBtn.disabled = !f.online;
    kickBtn.addEventListener('click', () =>
      subadminModerateFriend(
        { username: f.username, action: 'kick_session' },
        `👢 Kicked @${f.username} from the current session.`
      )
    );
    tdSession.appendChild(kickBtn);

    tr.append(tdName, tdEcon, tdControls, tdSession);
    subadminFriendsTbody.appendChild(tr);
  }
}

async function refreshSubadminFriends() {
  if (!sessionToken) return;
  const res = await fetch('/api/ella-world/subadmin/state', {
    headers: { Authorization: `Bearer ${sessionToken}` }
  });
  const data = await res.json().catch(() => ({}));
  if (res.ok) {
    if (Array.isArray(data.itemCatalog) && data.itemCatalog.length > 0) {
      setItemCatalog(data.itemCatalog);
    }
    renderSubadminFriends(data.friends || [], data.itemCatalog || null);
  } else {
    showSubadminFeedback(`❌ ${data.error || 'Sub-Admin access unavailable.'}`);
  }
}

async function subadminModerateFriend(payload, successMsg) {
  if (!sessionToken) return;
  const res = await fetch('/api/ella-world/subadmin/moderate', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${sessionToken}`
    },
    body: JSON.stringify(payload)
  });
  const data = await res.json().catch(() => ({}));
  if (res.ok) {
    if (data.friends) {
      renderSubadminFriends(data.friends);
    } else {
      await refreshSubadminFriends();
    }
    if (successMsg) showSubadminFeedback(successMsg);
  } else {
    showSubadminFeedback(`❌ ${data.error || 'Moderation failed.'}`);
  }
}

document.getElementById('subadmin-gift-item-form')?.addEventListener('submit', async (e) => {
  e.preventDefault();
  const username = document.getElementById('subadmin-gift-username')?.value || '';
  const itemId = document.getElementById('subadmin-gift-item')?.value || '';
  const qty = Math.max(1, Number(document.getElementById('subadmin-gift-qty')?.value || 1));
  const allowClone = Boolean(document.getElementById('subadmin-gift-allow-clone')?.checked);
  if (!username || !itemId) return;
  await subadminModerateFriend(
    { username, action: 'gift_item', itemId, qty, allowClone },
    `🎁 Gifted ${qty}x ${itemId} to @${username}${allowClone ? ' (🧬 Clone Enabled)' : ''}!`
  );
});

if (openSubadminHudBtn && subadminDialog) {
  openSubadminHudBtn.addEventListener('click', () => {
    if (subadminFeedback) subadminFeedback.hidden = true;
    subadminDialog.showModal();
    refreshSubadminFriends();
  });
}
document.getElementById('subadmin-refresh-btn')?.addEventListener('click', refreshSubadminFriends);

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
  applyModerationAndParentalUi();
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
    applyModerationAndParentalUi();
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
  const enforceMachineSigCheckbox = document.getElementById('admin-enforce-machine-signatures');
  if (enforceMachineSigCheckbox) {
    enforceMachineSigCheckbox.checked = dashboard.enforceMachineSignatures !== false;
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

  const giftUserSelect = document.getElementById('admin-gift-username');
  if (giftUserSelect) {
    const prevUser = giftUserSelect.value;
    giftUserSelect.replaceChildren();
    for (const u of dashboard.users || []) {
      const opt = document.createElement('option');
      opt.value = u.username;
      opt.textContent = `${u.displayName || u.username} (@${u.username})`;
      if (u.username === prevUser) opt.selected = true;
      giftUserSelect.appendChild(opt);
    }
  }
  const giftItemSelect = document.getElementById('admin-gift-item');
  if (giftItemSelect) {
    const prevItem = giftItemSelect.value;
    giftItemSelect.replaceChildren();
    const catList = Array.isArray(dashboard.itemCatalog) ? dashboard.itemCatalog : itemCatalog;
    const sortedItems = [...catList].sort(
      (a, b) => Number(Boolean(b.parentExclusive)) - Number(Boolean(a.parentExclusive))
    );
    for (const it of sortedItems) {
      const opt = document.createElement('option');
      opt.value = it.id;
      opt.textContent = `${it.icon || '🎁'} ${it.name}${it.parentExclusive ? ' [Parent-Only]' : ''}`;
      if (it.id === prevItem) opt.selected = true;
      giftItemSelect.appendChild(opt);
    }
  }

  const tbody = document.getElementById('admin-users-tbody');
  tbody.innerHTML = '';

  for (const u of dashboard.users || []) {
    const tr = document.createElement('tr');
    const coins = Number(u.coins || 0);
    const strikes = Number(u.violationStrikes || 0);
    const inPrison = Boolean(u.inDebtorsPrison);

    // 1. Row Header (<th scope="row">): Friend Display Name + @username + Machine count
    const thFriend = document.createElement('th');
    thFriend.scope = 'row';
    thFriend.className = 'admin-friend-cell';
    const nameStrong = document.createElement('strong');
    nameStrong.textContent = u.displayName || u.username;
    const handleSub = document.createElement('span');
    handleSub.className = 'admin-handle-sub';
    const devCount = Number(u.deviceCount || 0);
    handleSub.textContent =
      devCount > 0 ? `@${u.username} • 🖥️ ${devCount}` : `@${u.username}`;
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
    const setGoldBtn = document.createElement('button');
    setGoldBtn.type = 'button';
    setGoldBtn.className = 'mod-pill-btn';
    setGoldBtn.textContent = '✏️ Set Gold';
    setGoldBtn.addEventListener('click', () => {
      const raw = window.prompt(`Set exact gold balance for @${u.username}:`, String(coins));
      if (raw === null) return;
      const parsed = Math.trunc(Number(raw));
      if (!Number.isFinite(parsed)) return;
      adminModerateUser(
        { username: u.username, action: 'update_controls', setCoins: parsed },
        `🪙 Set @${u.username}'s gold to ${parsed.toLocaleString()}.`
      );
    });
    stepRow.append(minusBtn, plusBtn, setGoldBtn);
    tdEcon.append(econTop, stepRow);

    const invEntries = Object.entries(u.inventory || {}).filter(([, q]) => Number(q) > 0);
    if (invEntries.length > 0) {
      const cloneSet = new Set(Array.isArray(u.cloneableItems) ? u.cloneableItems : []);
      const giftWrap = document.createElement('div');
      giftWrap.className = 'mod-coin-step-row';
      for (const [itemId, qty] of invEntries) {
        const def = catalogById.get(itemId);
        const allow = cloneSet.has(itemId);
        const addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.className = 'mod-pill-btn';
        addBtn.textContent = `${def?.icon || '🎁'} ${qty}x +1`;
        addBtn.title = `Give +1 ${def?.name || itemId} to @${u.username}`;
        addBtn.addEventListener('click', () =>
          adminGiftOrToggleClone(
            { username: u.username, itemId, qty: 1 },
            `🎁 Added +1 ${def?.name || itemId} to @${u.username}.`
          )
        );
        const cBtn = document.createElement('button');
        cBtn.type = 'button';
        cBtn.className = `mod-pill-btn${allow ? ' is-active-safe' : ''}`;
        cBtn.textContent = allow ? '🧬 On' : '🧬 Off';
        cBtn.title = `Toggle whether @${u.username} can clone ${def?.name || itemId}`;
        cBtn.addEventListener('click', () =>
          adminGiftOrToggleClone(
            { username: u.username, itemId, qty: 0, allowClone: !allow },
            `${!allow ? '🧬 Enabled' : '🔒 Disabled'} cloning of ${def?.name || itemId} for @${u.username}.`
          )
        );
        const rmBtn = document.createElement('button');
        rmBtn.type = 'button';
        rmBtn.className = 'mod-pill-btn warn-pill';
        rmBtn.textContent = '🗑️';
        rmBtn.title = `Remove ${def?.name || itemId} from @${u.username}`;
        rmBtn.addEventListener('click', () =>
          adminGiftOrToggleClone(
            { username: u.username, itemId, setQty: 0 },
            `🗑️ Removed ${def?.name || itemId} from @${u.username}.`
          )
        );
        giftWrap.append(addBtn, cBtn, rmBtn);
      }
      tdEcon.appendChild(giftWrap);
    }

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

    const presetSel = buildAvatarPresetSelect(
      u.avatar?.characterPreset || '',
      u.username,
      (presetKey, presetObj) =>
        adminModerateUser(
          { username: u.username, action: 'update_controls', avatar: presetObj },
          `👗 Updated @${u.username}'s avatar look to "${presetKey}".`
        )
    );
    const renameBtn = document.createElement('button');
    renameBtn.type = 'button';
    renameBtn.className = 'mod-pill-btn';
    renameBtn.textContent = '✏️ Name';
    renameBtn.addEventListener('click', () => {
      const next = window.prompt(
        `New display name for @${u.username}:`,
        u.displayName || u.username
      );
      if (!next || !next.trim()) return;
      adminModerateUser(
        { username: u.username, action: 'update_controls', displayName: next.trim() },
        `✏️ Renamed @${u.username} to "${next.trim()}".`
      );
    });

    const roleSelect = document.createElement('select');
    roleSelect.className = 'mod-chat-select';
    roleSelect.setAttribute('aria-label', `Role for ${u.username}`);
    [
      ['friend', '🧒 Friend'],
      ['subadmin', '🛡️ Sub-Admin'],
      ['host', '👑 Host']
    ].forEach(([val, label]) => {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = label;
      if ((u.role || 'friend').toLowerCase() === val) opt.selected = true;
      roleSelect.appendChild(opt);
    });
    roleSelect.addEventListener('change', () =>
      adminModerateUser(
        {
          username: u.username,
          action: 'update_controls',
          role: roleSelect.value
        },
        `🛡️ Updated role for @${u.username} to "${roleSelect.value}".`
      )
    );

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

    controlsWrap.append(presetSel, renameBtn, roleSelect, micBtn, chatSelect, buildBtn, tradeBtn, pauseBtn, sentenceBtn);

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

  renderAdminDevicesAndEvasions(dashboard);
  refreshAdminChats();
}

async function adminManageDevice(payload, successMsg) {
  if (!adminToken) return;
  const res = await fetch('/api/ella-world/admin/devices', {
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
    showAdminSessionFeedback(`❌ ${out.error || 'Machine update failed.'}`);
  }
}

function formatMatchLayerLabel(method) {
  switch (method) {
    case 'machine_seal_cookie':
      return '🔐 HttpOnly Cookie Seal';
    case 'machine_seal_vault':
      return '🔐 Client Vault Seal';
    case 'hardware_exact_sig':
      return '🧬 Exact Hardware Sig';
    case 'hardware_cross_browser_sig':
      return '🌐 Cross-Browser GPU/Screen';
    case 'network_ip_fallback':
      return '📡 Network + OS Fallback';
    case 'live_tab_eviction':
      return '⚡ Live Tab Eviction';
    default:
      return method || 'Signature';
  }
}

function renderAdminDevicesAndEvasions(dashboard) {
  const devices = Array.isArray(dashboard.devices) ? dashboard.devices : [];
  const evasions = Array.isArray(dashboard.recentDeviceEvasions)
    ? dashboard.recentDeviceEvasions
    : [];

  const devCountEl = document.getElementById('admin-device-count');
  if (devCountEl) devCountEl.textContent = String(devices.length);
  const evasionCountEl = document.getElementById('admin-evasion-count');
  if (evasionCountEl) {
    evasionCountEl.textContent = String(
      dashboard.deviceEvasionCount !== undefined ? dashboard.deviceEvasionCount : evasions.length
    );
  }

  const devTbody = document.getElementById('admin-devices-tbody');
  if (devTbody) {
    devTbody.textContent = '';
    if (devices.length === 0) {
      const tr = document.createElement('tr');
      const td = document.createElement('td');
      td.colSpan = 5;
      td.textContent =
        'No machines registered yet. Machines automatically register on the next browser sign-in.';
      tr.appendChild(td);
      devTbody.appendChild(tr);
    } else {
      for (const d of devices) {
        const tr = document.createElement('tr');

        // 1. Row Header (<th scope="row">): Machine Signature + Label
        const thSig = document.createElement('th');
        thSig.scope = 'row';
        thSig.className = 'admin-device-sig-cell';
        const sigCode = document.createElement('code');
        sigCode.className = 'machine-sig-code';
        sigCode.textContent = d.shortSig || d.deviceId;
        const labelSub = document.createElement('span');
        labelSub.className = 'admin-handle-sub';
        labelSub.textContent = d.label || d.osFamily || 'Device';
        thSig.append(sigCode, labelSub);

        // 2. Hardware & Network
        const tdHw = document.createElement('td');
        const hwPrimary = document.createElement('div');
        hwPrimary.className = 'device-hw-line';
        hwPrimary.textContent = [d.osFamily, d.screenSummary].filter(Boolean).join(' • ') || '—';
        const hwSub = document.createElement('div');
        hwSub.className = 'admin-handle-sub';
        const gpuClean = d.gpuSummary ? String(d.gpuSummary).slice(0, 44) : '';
        hwSub.textContent = [gpuClean, d.lastIp ? `IP ${d.lastIp}` : ''].filter(Boolean).join(' • ');
        tdHw.append(hwPrimary, hwSub);

        // 3. Linked Friends
        const tdUsers = document.createElement('td');
        const usersWrap = document.createElement('div');
        usersWrap.className = 'device-users-wrap';
        const assoc = Array.isArray(d.associatedUsers) ? d.associatedUsers : [];
        if (assoc.length === 0) {
          const noneSpan = document.createElement('span');
          noneSpan.className = 'admin-item-sub';
          noneSpan.textContent = 'Unlinked';
          usersWrap.appendChild(noneSpan);
        } else {
          for (const uname of assoc) {
            const pill = document.createElement('span');
            const isLockingOwner =
              d.lockedByUser && String(d.lockedByUser).toLowerCase() === String(uname).toLowerCase();
            pill.className = `device-user-pill${isLockingOwner ? ' is-penalized-owner' : ''}`;
            pill.textContent = `@${uname}`;
            usersWrap.appendChild(pill);
          }
        }
        tdUsers.appendChild(usersWrap);

        // 4. Lock Status
        const tdLock = document.createElement('td');
        const lockStack = document.createElement('div');
        lockStack.className = 'admin-status-stack';
        const statusBadge = document.createElement('span');
        if (d.banned) {
          statusBadge.className = 'suspended-badge';
          statusBadge.textContent = '🚫 Machine Banned';
        } else if (d.trustedShared) {
          statusBadge.className = 'device-shared-badge';
          statusBadge.textContent = '🏠 Trusted Shared PC';
        } else if (d.lockedByUser) {
          statusBadge.className = 'prison-badge';
          statusBadge.textContent = `⚖️ Locked to @${d.lockedByUser}`;
        } else {
          statusBadge.className = 'device-clean-badge';
          statusBadge.textContent =
            Number(d.onlineCount || 0) > 0
              ? `🟢 Clean (${d.onlineCount} online)`
              : '🛡️ Clean';
        }
        lockStack.appendChild(statusBadge);

        if (Array.isArray(d.activePenalties) && d.activePenalties.length > 0 && !d.trustedShared) {
          const penSub = document.createElement('span');
          penSub.className = 'admin-handle-sub';
          penSub.textContent = d.activePenalties.join(', ');
          lockStack.appendChild(penSub);
        }
        tdLock.appendChild(lockStack);

        // 5. Parent Controls
        const tdControls = document.createElement('td');
        const btnWrap = document.createElement('div');
        btnWrap.className = 'mod-controls-wrap';

        const sharedBtn = document.createElement('button');
        sharedBtn.type = 'button';
        sharedBtn.className = `mod-pill-btn${d.trustedShared ? ' pardon-pill' : ''}`;
        sharedBtn.textContent = d.trustedShared ? '🏠 Shared ON' : '🏠 Shared PC';
        sharedBtn.title = d.trustedShared
          ? 'Turn off Shared Family PC exemption (re-enable cross-account penalty lock)'
          : 'Exempt this family computer so siblings can sign in independently when one child has a penalty';
        sharedBtn.addEventListener('click', () =>
          adminManageDevice(
            {
              deviceId: d.deviceId,
              action: 'update',
              trustedShared: !d.trustedShared
            },
            `${!d.trustedShared ? '🏠 Marked' : '🔒 Unmarked'} machine ${d.shortSig} as a Trusted Shared Family PC.`
          )
        );

        const banBtn = document.createElement('button');
        banBtn.type = 'button';
        banBtn.className = `mod-pill-btn${d.banned ? ' is-locked' : ' warn-pill'}`;
        banBtn.textContent = d.banned ? '▶️ Unban PC' : '🚫 Ban PC';
        banBtn.title = d.banned
          ? 'Unban this machine so accounts can sign in again'
          : 'Block all logins & immediately sign out active sessions on this machine';
        banBtn.addEventListener('click', () =>
          adminManageDevice(
            {
              deviceId: d.deviceId,
              action: 'update',
              banned: !d.banned
            },
            `${!d.banned ? '🚫 Banned' : '▶️ Unbanned'} machine ${d.shortSig}.`
          )
        );

        const unlinkBtn = document.createElement('button');
        unlinkBtn.type = 'button';
        unlinkBtn.className = 'mod-pill-btn';
        unlinkBtn.textContent = '🧹 Unlink';
        unlinkBtn.title = 'Clear historical account associations from this machine';
        unlinkBtn.addEventListener('click', () =>
          adminManageDevice(
            {
              deviceId: d.deviceId,
              action: 'unlink_users'
            },
            `🧹 Cleared linked accounts from machine ${d.shortSig}.`
          )
        );

        btnWrap.append(sharedBtn, banBtn, unlinkBtn);
        tdControls.appendChild(btnWrap);

        tr.append(thSig, tdHw, tdUsers, tdLock, tdControls);
        devTbody.appendChild(tr);
      }
    }
  }

  const evTbody = document.getElementById('admin-evasions-tbody');
  if (evTbody) {
    evTbody.textContent = '';
    if (evasions.length === 0) {
      const tr = document.createElement('tr');
      const td = document.createElement('td');
      td.colSpan = 5;
      td.textContent = 'No alt-account penalty evasion attempts recorded.';
      tr.appendChild(td);
      evTbody.appendChild(tr);
    } else {
      for (const ev of evasions) {
        const tr = document.createElement('tr');

        const thTime = document.createElement('th');
        thTime.scope = 'row';
        thTime.textContent = ev.ts ? new Date(ev.ts).toLocaleString() : '—';

        const tdAttempt = document.createElement('td');
        const attemptBadge = document.createElement('span');
        attemptBadge.className = 'device-user-pill';
        attemptBadge.textContent = `@${ev.attemptedUsername || '?'}`;
        tdAttempt.appendChild(attemptBadge);

        const tdPenalized = document.createElement('td');
        const penBadge = document.createElement('span');
        penBadge.className = 'prison-badge';
        const blockingUser = ev.blockingUsername || ev.penalizedUsername || '?';
        penBadge.textContent =
          blockingUser === 'machine_banned' || blockingUser === 'parent'
            ? '🚫 Machine Banned'
            : `⚖️ @${blockingUser}`;
        tdPenalized.appendChild(penBadge);

        const tdMachine = document.createElement('td');
        const sigCode = document.createElement('code');
        sigCode.className = 'machine-sig-code';
        sigCode.textContent = ev.shortSig || ev.deviceId || '—';
        const layerSub = document.createElement('span');
        layerSub.className = 'admin-handle-sub';
        const peerIp = ev.peerIp || ev.ip || '';
        layerSub.textContent = `${formatMatchLayerLabel(ev.matchMethod)}${peerIp ? ` • IP ${peerIp}` : ''}`;
        tdMachine.append(sigCode, layerSub);

        const tdReason = document.createElement('td');
        tdReason.textContent =
          Array.isArray(ev.activePenalties) && ev.activePenalties.length > 0
            ? ev.activePenalties.join(', ')
            : 'Blocked';

        tr.append(thTime, tdAttempt, tdPenalized, tdMachine, tdReason);
        evTbody.appendChild(tr);
      }
    }
  }
}

document.getElementById('admin-devices-refresh-btn')?.addEventListener('click', refreshAdminDashboard);

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

async function adminGiftOrToggleClone(payload, successMsg) {
  if (!adminToken) return;
  const res = await fetch('/api/ella-world/admin/gift-item', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify(payload)
  });
  const data = await res.json().catch(() => ({}));
  if (res.ok) {
    if (data.dashboard) renderAdminDashboard(data.dashboard);
    if (successMsg) showAdminSessionFeedback(successMsg);
  } else {
    showAdminSessionFeedback(`❌ ${data.error || 'Gift update failed.'}`);
  }
}

document.getElementById('admin-gift-item-form')?.addEventListener('submit', async (e) => {
  e.preventDefault();
  if (!adminToken) return;
  const feedback = document.getElementById('admin-gift-feedback');
  if (feedback) feedback.hidden = true;
  const username = document.getElementById('admin-gift-username')?.value || '';
  const itemId = document.getElementById('admin-gift-item')?.value || '';
  const qty = Math.max(1, Number(document.getElementById('admin-gift-qty')?.value || 1));
  const allowClone = Boolean(document.getElementById('admin-gift-allow-clone')?.checked);

  const res = await fetch('/api/ella-world/admin/gift-item', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken}`
    },
    body: JSON.stringify({ username, itemId, qty, allowClone })
  });
  const data = await res.json().catch(() => ({}));
  if (res.ok) {
    if (data.dashboard) renderAdminDashboard(data.dashboard);
    if (feedback) {
      feedback.textContent = `🎁 Gifted ${data.qtyGranted}x ${data.itemName} to @${data.username}${data.allowClone ? ' (🧬 Clone Enabled)' : ''}!`;
      feedback.hidden = false;
    }
  } else if (feedback) {
    feedback.textContent = `❌ ${data.error || 'Could not gift item.'}`;
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
    enforceMachineSignatures: Boolean(
      document.getElementById('admin-enforce-machine-signatures')?.checked
    ),
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

// ============================================================================
// 14. Full Expansion Pack Controllers (Pets/Mounts, Homes/Doorbells, Crafting,
//     Timed Fashion Show, Obby Speedrun Boards, Sky/Meteor Shower & Dragon Boss)
// ============================================================================

const popoverDismissTimers = new WeakMap();
const starCatchInFlightIds = new Set();
let doorbellAudioCtx = null;
let lastFashionAdvanceProbeAt = 0;
let lastBossSrMilestone = '';

function announceScreenReaderToast(message) {
  if (!message) return;
  if ('ariaNotify' in document && typeof document.ariaNotify === 'function') {
    try {
      document.ariaNotify(message, { interrupt: 'none' });
      return;
    } catch {
      // fallback to live region below
    }
  }
  if (!srLiveAnnouncements) return;
  srLiveAnnouncements.textContent = '';
  requestAnimationFrame(() => {
    srLiveAnnouncements.textContent = message;
  });
}

function showTopLayerToastPopover(popoverEl, autoHideMs = 15000) {
  if (!popoverEl) return;
  if (typeof popoverEl.showPopover === 'function') {
    try {
      popoverEl.showPopover();
    } catch {
      popoverEl.setAttribute('open', '');
    }
  } else {
    popoverEl.setAttribute('open', '');
  }

  const prev = popoverDismissTimers.get(popoverEl);
  if (prev) clearTimeout(prev);

  const scheduleHide = (delayMs) => {
    const tid = setTimeout(() => {
      if (popoverEl.matches(':hover') || popoverEl.matches(':focus-within')) {
        scheduleHide(4000);
        return;
      }
      hideTopLayerToastPopover(popoverEl);
    }, delayMs);
    popoverDismissTimers.set(popoverEl, tid);
  };
  scheduleHide(autoHideMs);
}

function hideTopLayerToastPopover(popoverEl) {
  if (!popoverEl) return;
  const prev = popoverDismissTimers.get(popoverEl);
  if (prev) {
    clearTimeout(prev);
    popoverDismissTimers.delete(popoverEl);
  }
  if (typeof popoverEl.hidePopover === 'function') {
    try {
      popoverEl.hidePopover();
    } catch {
      popoverEl.removeAttribute('open');
    }
  } else {
    popoverEl.removeAttribute('open');
  }
}

function playDoorbellChime() {
  const AudioCtx = window.AudioContext || window.webkitAudioContext;
  if (!AudioCtx) return;
  try {
    if (!doorbellAudioCtx) {
      doorbellAudioCtx = new AudioCtx();
    }
    if (doorbellAudioCtx.state === 'suspended') {
      doorbellAudioCtx.resume().catch(() => {});
    }
    const now = doorbellAudioCtx.currentTime;
    const notes = [
      { freq: 659.25, start: 0, dur: 0.28 },
      { freq: 523.25, start: 0.24, dur: 0.44 }
    ];
    for (const n of notes) {
      const osc = doorbellAudioCtx.createOscillator();
      const gain = doorbellAudioCtx.createGain();
      osc.type = 'triangle';
      osc.frequency.setValueAtTime(n.freq, now + n.start);
      gain.gain.setValueAtTime(0.0001, now + n.start);
      gain.gain.exponentialRampToValueAtTime(0.16, now + n.start + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.0001, now + n.start + n.dur);
      osc.connect(gain);
      gain.connect(doorbellAudioCtx.destination);
      osc.start(now + n.start);
      osc.stop(now + n.start + n.dur + 0.02);
    }
  } catch {
    // ignore audio autoplay restriction
  }
}

function applyCompanionsSnapshot(comp) {
  if (!comp || typeof comp !== 'object') return;
  playerCompanions = {
    ownedPets: Array.isArray(comp.ownedPets) ? [...comp.ownedPets] : ['cloud_puppy'],
    equippedPet: comp.equippedPet ?? null,
    petNickname: String(comp.petNickname || ''),
    ownedMounts: Array.isArray(comp.ownedMounts) ? [...comp.ownedMounts] : ['starlight_board'],
    equippedMount: comp.equippedMount ?? null,
    firstBossDefeated: Boolean(comp.firstBossDefeated)
  };
  if (toggleHoverboardBtn) {
    const riding = Boolean(playerCompanions.equippedMount);
    toggleHoverboardBtn.classList.toggle('active', riding);
    toggleHoverboardBtn.setAttribute('aria-pressed', String(riding));
    toggleHoverboardBtn.textContent = riding ? '🛹 Board ON' : '🛹 Board (X)';
  }
  if (companionsDialog && companionsDialog.open) {
    renderCompanionsModal();
  }
}

function applyHomesSnapshot(homes) {
  const list = Array.isArray(homes)
    ? homes
    : Array.isArray(homes?.plots)
      ? homes.plots
      : null;
  if (!list) return;
  homesState = list;
  world3d.syncHomesSnapshot(homesState);
  if (homesDialog && homesDialog.open) {
    renderHomesModal();
  }
}

function applyObbyLeaderboards(lbs) {
  if (!lbs || typeof lbs !== 'object') return;
  obbyLeaderboards = {
    easy: Array.isArray(lbs.easy) ? lbs.easy : [],
    hard: Array.isArray(lbs.hard) ? lbs.hard : []
  };
  world3d.syncObbyLeaderboards(obbyLeaderboards);
  if (obbyBoardDialog && obbyBoardDialog.open) {
    renderObbyBoardModal();
  }
}

function applySkySnapshot(sky) {
  if (!sky || typeof sky !== 'object') return;
  skyState = {
    ...skyState,
    ...sky
  };
  world3d.syncSkySnapshot(skyState);
  if (skyDialog && skyDialog.open) {
    syncSkyModalButtons();
  }
}

function applyArenaBossHud(boss) {
  arenaBossStatus = boss || null;
  const active = Boolean(boss && boss.active && Number(boss.hp) > 0);
  if (arenaBossRow) {
    arenaBossRow.hidden = !active;
  }
  if (!active) {
    lastBossSrMilestone = '';
    return;
  }
  const maxHp = Math.max(1, Number(boss.maxHp) || 18);
  const hp = Math.max(0, Math.min(maxHp, Number(boss.hp) || 0));
  if (arenaBossBar) {
    arenaBossBar.max = maxHp;
    arenaBossBar.value = hp;
  }
  if (arenaBossHpText) {
    arenaBossHpText.textContent = `${hp} / ${maxHp} HP`;
  }
  if (currentZone === ZONE_ARENA) {
    let milestone = 'active';
    if (hp <= Math.ceil(maxHp * 0.25)) milestone = 'enraged';
    if (milestone !== lastBossSrMilestone) {
      lastBossSrMilestone = milestone;
      announceScreenReaderToast(
        milestone === 'enraged'
          ? `Obsidian Rift Dragon is below 25 percent health: ${hp} of ${maxHp} HP! Jump over purple shockwave rings!`
          : `Obsidian Rift Dragon awakened at the North Altar with ${maxHp} health! Jump over shockwave rings!`
      );
    }
  }
}

function renderFashionShowTicker(serverNow) {
  const sh = runwayShowState;
  const phase = sh?.phase || 'idle';
  const myU = String(selfUser?.username || '').toLowerCase();
  const meEntry = (sh?.contestants || []).find(
    (c) => String(c.username || '').toLowerCase() === myU
  );

  if (runwayStartShowBtn) {
    if (phase === 'idle' || phase === 'podium') {
      runwayStartShowBtn.textContent = '🎬 Start Show!';
      runwayStartShowBtn.disabled = false;
    } else if (phase === 'dress_up' && !meEntry) {
      runwayStartShowBtn.textContent = '👗 Join Show!';
      runwayStartShowBtn.disabled = false;
    } else {
      runwayStartShowBtn.textContent =
        phase === 'dress_up' ? `👗 In Show (${(sh.contestants || []).length})` : '🔦 Catwalk Live';
      runwayStartShowBtn.disabled = phase === 'catwalk' || Boolean(meEntry);
    }
  }

  if (phase === 'idle') {
    if (runwayShowStatusEl) runwayShowStatusEl.hidden = true;
    if (glamShowTimerPill) glamShowTimerPill.hidden = true;
    return;
  }

  const totalMs = Math.max(1000, Number(sh.phaseEndsAt || 0) - Number(sh.phaseStartedAt || 0));
  const remMs = Math.max(0, Number(sh.phaseEndsAt || 0) - serverNow);
  const remSec = Math.ceil(remMs / 1000);

  if (runwayShowStatusEl) runwayShowStatusEl.hidden = false;
  if (runwayShowProgress) {
    runwayShowProgress.max = Math.round(totalMs / 100);
    runwayShowProgress.value = Math.min(
      Math.round(totalMs / 100),
      Math.max(0, Math.round(remMs / 100))
    );
  }

  if (phase === 'dress_up') {
    if (runwayShowPhaseText) {
      runwayShowPhaseText.textContent = `👗 Dress-Up: ${remSec}s • ${(sh.contestants || []).length} models`;
    }
    if (runwayReadyBtn) {
      runwayReadyBtn.hidden = !meEntry;
      runwayReadyBtn.disabled = Boolean(meEntry?.ready);
      runwayReadyBtn.textContent = meEntry?.ready ? '✅ Ready!' : '✅ Ready';
    }
    if (glamShowTimerPill && glamShowTimerText) {
      glamShowTimerPill.hidden = false;
      glamShowTimerText.textContent = `⏱️ Runway Dress-Up: ${remSec}s`;
    }
  } else if (phase === 'catwalk') {
    const spotUser = sh.spotlightUsername || 'Model';
    const matched = Array.from(playersMap.values()).find(
      (p) => String(p.user?.username || '').toLowerCase() === String(spotUser).toLowerCase()
    );
    const disp =
      String(selfUser?.username || '').toLowerCase() === String(spotUser).toLowerCase()
        ? selfUser.displayName
        : matched?.user?.displayName || spotUser;
    if (runwayShowPhaseText) {
      runwayShowPhaseText.textContent = `🔦 Spotlight: ${disp} (${remSec}s) — Vote 1–5★!`;
    }
    if (runwayReadyBtn) runwayReadyBtn.hidden = true;
    if (glamShowTimerPill && glamShowTimerText) {
      glamShowTimerPill.hidden = false;
      glamShowTimerText.textContent = `🔦 Catwalk Live: ${disp} (${remSec}s)`;
    }
  } else if (phase === 'podium') {
    const w1 = (sh.podiumWinners || [])[0];
    if (runwayShowPhaseText) {
      runwayShowPhaseText.textContent = w1
        ? `👑 Winner: ${w1.displayName || w1.username} (${w1.avgStars}★ • +${w1.prizeCoins}🪙) • ${remSec}s`
        : `👑 Podium Finale (${remSec}s)`;
    }
    if (runwayReadyBtn) runwayReadyBtn.hidden = true;
    if (glamShowTimerPill) glamShowTimerPill.hidden = true;
  }

  // Backup phase-advance nudge if server timer crossed zero while idle
  if (
    sessionToken &&
    remMs <= 0 &&
    serverNow >= Number(sh.phaseEndsAt || 0) + 450 &&
    performance.now() - lastFashionAdvanceProbeAt > 2000
  ) {
    lastFashionAdvanceProbeAt = performance.now();
    fetch('/api/ella-world/runway', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'advance_phase' })
    }).catch(() => {});
  }
}

// ----------------------------------------------------------------------------
// 14A. Adoptable 3D Pets & Plaza Hoverboards (#companions-dialog)
// ----------------------------------------------------------------------------

async function mutateCompanions(payload) {
  if (!sessionToken) return null;
  try {
    const res = await fetch('/api/ella-world/companions', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, ...payload })
    });
    if (await handleHttpAuthCheck(res)) return null;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      if (data.error) showToast(`⚠️ ${data.error}`);
      return null;
    }
    if (data.user) {
      selfUser = data.user;
      if (selfId && playersMap.has(selfId)) {
        playersMap.get(selfId).user = selfUser;
      }
    }
    if (data.economy) applyEconomySnapshot(data.economy);
    if (data.companions) {
      applyCompanionsSnapshot(data.companions);
      if (selfUser) {
        world3d.updateUserCompanions(selfUser.username, data.companions);
      }
    }
    if (data.action === 'hatch_egg' && data.hatchedPetId) {
      const petDef = petCatalog.find((p) => p.id === data.hatchedPetId);
      const pName = petDef ? `${petDef.icon} ${petDef.name}` : data.hatchedPetId;
      if (world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      showToast(
        data.duplicateBonusCoins > 0
          ? `🥚 Hatched ${pName}! (Already owned — +${data.duplicateBonusCoins} 🪙 bonus!)`
          : `🥚✨ HATCHED NEW 3D PET: ${pName}!`,
        5600
      );
    } else if (data.action === 'adopt_pet') {
      showToast('🐾 Adopted & equipped your new 3D Companion Pet!');
    } else if (data.action === 'rename_pet') {
      showToast(
        data.companions?.petNickname
          ? `🏷️ Named your pet "${data.companions.petNickname}"!`
          : '🏷️ Cleared custom pet nickname.'
      );
    } else if (data.action === 'buy_mount') {
      showToast('🛹 Unlocked & mounted your new Plaza Hoverboard!');
    } else if (data.action === 'equip_mount') {
      const mId = data.companions?.equippedMount;
      const mDef = mId ? mountCatalog.find((m) => m.id === mId) : null;
      showToast(
        mId
          ? `🛹 Riding ${mDef ? mDef.name : mId}! (${mDef ? mDef.speedMultiplier : 1.62}x Plaza Speed)`
          : '👟 Stepped off your Hoverboard.'
      );
    }
    return data;
  } catch {
    showToast('⚠️ Could not update companions right now.');
    return null;
  }
}

function toggleQuickHoverboard() {
  if (currentZone === ZONE_PRISON) {
    showToast("⚖️ Hoverboards are disabled inside Debtor's Prison!");
    return;
  }
  if (currentZone !== ZONE_PLAZA) {
    showToast('🛹 Hoverboards & Neon Skates are designed for Starlight Plaza cruising!');
    return;
  }
  if (activeObbyRun) {
    cancelActiveObbyRun(false);
  }
  mutateCompanions({ action: 'equip_mount', mountId: 'toggle' });
}

function activateCompanionsTab(tabId, moveFocus = false) {
  const isMounts = tabId === 'mounts' || tabId === 'comp-tab-mounts';
  const petsTab = document.getElementById('comp-tab-pets');
  const mountsTab = document.getElementById('comp-tab-mounts');
  const petsPanel = document.getElementById('comp-panel-pets');
  const mountsPanel = document.getElementById('comp-panel-mounts');
  if (!petsTab || !mountsTab || !petsPanel || !mountsPanel) return;

  petsTab.setAttribute('aria-selected', String(!isMounts));
  mountsTab.setAttribute('aria-selected', String(isMounts));
  petsTab.tabIndex = isMounts ? -1 : 0;
  mountsTab.tabIndex = isMounts ? 0 : -1;

  if (isMounts) {
    mountsPanel.hidden = false;
    petsPanel.setAttribute('hidden', 'until-found');
    if (moveFocus) mountsTab.focus();
  } else {
    petsPanel.hidden = false;
    mountsPanel.setAttribute('hidden', 'until-found');
    if (moveFocus) petsTab.focus();
  }
}

function openCompanionsModal(initialTab = 'pets') {
  activateCompanionsTab(initialTab, false);
  renderCompanionsModal();
  if (companionsDialog && !companionsDialog.open) {
    companionsDialog.showModal();
  }
}

function renderCompanionsModal() {
  const coinsEl = document.getElementById('companions-coins-value');
  if (coinsEl) {
    coinsEl.textContent = `🪙 ${Number(playerEconomy.coins || 0).toLocaleString()}`;
  }
  const eggCount = Number((playerEconomy.inventory || {}).pet_egg || 0);
  const eggSpan = document.getElementById('companions-egg-count');
  if (eggSpan) eggSpan.textContent = String(eggCount);
  const hatchBtn = document.getElementById('hatch-pet-egg-btn');
  if (hatchBtn) hatchBtn.disabled = eggCount < 1;

  const nickInput = document.getElementById('pet-nickname-input');
  if (nickInput && document.activeElement !== nickInput) {
    nickInput.value = playerCompanions.petNickname || '';
  }

  const petsGrid = document.getElementById('companions-pets-grid');
  if (petsGrid) {
    petsGrid.replaceChildren();
    for (const pet of petCatalog) {
      const owned = (playerCompanions.ownedPets || []).includes(pet.id);
      const equipped = playerCompanions.equippedPet === pet.id;
      const card = document.createElement('article');
      card.className = `backpack-card rarity-${pet.rarity || 'common'}${!owned ? ' is-unowned' : ''}`;

      const top = document.createElement('div');
      top.className = 'backpack-card-top';
      const iconSpan = document.createElement('span');
      iconSpan.className = 'backpack-icon';
      iconSpan.textContent = pet.icon || '🐾';
      const statePill = document.createElement('span');
      statePill.className = 'backpack-qty-pill';
      statePill.textContent = equipped ? '✨ Following You' : owned ? '✅ Adopted' : '🔒 Unadopted';
      top.append(iconSpan, statePill);

      const mid = document.createElement('div');
      const rarityTag = document.createElement('div');
      rarityTag.className = 'backpack-rarity-tag';
      const perks = [];
      if (pet.magnetBonusM > 0) perks.push(`+${pet.magnetBonusM}m Magnet`);
      if (pet.coinBonusPct > 0) perks.push(`+${pet.coinBonusPct}% Coins`);
      if (pet.bonusDamage > 0) perks.push(`+${pet.bonusDamage} Combat DMG`);
      rarityTag.textContent = `${pet.rarity || 'common'} • ${perks.join(' • ') || 'Companion'}`;
      const titleEl = document.createElement('h3');
      titleEl.className = 'backpack-card-title';
      titleEl.textContent = pet.name;
      const descEl = document.createElement('p');
      descEl.className = 'backpack-desc';
      descEl.textContent = pet.description || '';
      mid.append(rarityTag, titleEl, descEl);

      const actions = document.createElement('div');
      actions.className = 'backpack-card-actions';
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = equipped ? 'secondary-outline-btn' : 'small-action-btn';
      if (owned) {
        btn.textContent = equipped ? '💤 Rest Pet' : '🐾 Equip Pet';
        btn.addEventListener('click', () =>
          mutateCompanions({ action: 'equip_pet', petId: equipped ? 'none' : pet.id })
        );
      } else if (pet.coinPrice && pet.coinPrice > 0) {
        btn.textContent = `🪙 Adopt (${pet.coinPrice} Coins)`;
        btn.disabled = Number(playerEconomy.coins || 0) < pet.coinPrice;
        btn.addEventListener('click', () =>
          mutateCompanions({ action: 'adopt_pet', petId: pet.id })
        );
      } else {
        btn.textContent = '🥚 Hatch from Companion Egg';
        btn.disabled = eggCount < 1;
        btn.addEventListener('click', () => mutateCompanions({ action: 'hatch_egg' }));
      }
      actions.appendChild(btn);

      card.append(top, mid, actions);
      petsGrid.appendChild(card);
    }
  }

  const mountsGrid = document.getElementById('companions-mounts-grid');
  if (mountsGrid) {
    mountsGrid.replaceChildren();
    for (const mount of mountCatalog) {
      const owned = (playerCompanions.ownedMounts || []).includes(mount.id);
      const riding = playerCompanions.equippedMount === mount.id;
      const card = document.createElement('article');
      card.className = `backpack-card rarity-${mount.rarity || 'common'}${!owned ? ' is-unowned' : ''}`;

      const top = document.createElement('div');
      top.className = 'backpack-card-top';
      const iconSpan = document.createElement('span');
      iconSpan.className = 'backpack-icon';
      iconSpan.textContent = mount.icon || '🛹';
      const statePill = document.createElement('span');
      statePill.className = 'backpack-qty-pill';
      statePill.textContent = riding ? '⚡ Riding Now' : owned ? '✅ Owned' : `🪙 ${mount.coinPrice} Coins`;
      top.append(iconSpan, statePill);

      const mid = document.createElement('div');
      const rarityTag = document.createElement('div');
      rarityTag.className = 'backpack-rarity-tag';
      rarityTag.textContent = `${mount.rarity || 'common'} • ${mount.speedMultiplier}x Plaza Speed`;
      const titleEl = document.createElement('h3');
      titleEl.className = 'backpack-card-title';
      titleEl.textContent = mount.name;
      const descEl = document.createElement('p');
      descEl.className = 'backpack-desc';
      descEl.textContent = mount.description || '';
      mid.append(rarityTag, titleEl, descEl);

      const actions = document.createElement('div');
      actions.className = 'backpack-card-actions';
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = riding ? 'secondary-outline-btn' : 'small-action-btn';
      if (owned) {
        btn.textContent = riding ? '👟 Dismount' : '🛹 Ride in Plaza';
        btn.addEventListener('click', () => {
          if (!riding && activeObbyRun) cancelActiveObbyRun(false);
          mutateCompanions({ action: 'equip_mount', mountId: riding ? 'none' : mount.id });
        });
      } else {
        btn.textContent = `🪙 Unlock (${mount.coinPrice} Coins)`;
        btn.disabled = Number(playerEconomy.coins || 0) < (mount.coinPrice || 0);
        btn.addEventListener('click', () =>
          mutateCompanions({ action: 'buy_mount', mountId: mount.id })
        );
      }
      actions.appendChild(btn);

      card.append(top, mid, actions);
      mountsGrid.appendChild(card);
    }
  }
}

// ----------------------------------------------------------------------------
// 14B. Claimable Homes, 3D Nameplate Mailboxes & Doorbells (#homes-dialog)
// ----------------------------------------------------------------------------

async function mutateHomes(payload) {
  const headers = { ...JSON_HEADERS };
  if (adminToken) {
    headers.Authorization = `Bearer ${adminToken}`;
  }
  try {
    const res = await fetch('/api/ella-world/homes', {
      method: 'POST',
      headers,
      body: JSON.stringify({ token: sessionToken || undefined, ...payload })
    });
    if (sessionToken && !adminToken && (await handleHttpAuthCheck(res))) return null;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      if (data.error) showToast(`⚠️ ${data.error}`);
      return null;
    }
    if (Array.isArray(data.plots)) {
      applyHomesSnapshot(data.plots);
    }
    return data;
  } catch {
    showToast('⚠️ Could not update Neighborhood Home right now.');
    return null;
  }
}

function handleWorldMailboxInteraction(plot) {
  if (!plot) {
    openHomesModal();
    return;
  }
  const myU = String(selfUser?.username || '').toLowerCase();
  const ownerU = String(plot.ownerUsername || '').toLowerCase();
  const isCoBuilder = (plot.coBuilders || []).some(
    (u) => String(u || '').toLowerCase() === myU
  );
  const isMod = String(selfUser?.role || '').toLowerCase() === 'host' || Boolean(adminToken);

  if (!ownerU || ownerU === myU || isCoBuilder || isMod) {
    openHomesModal();
    return;
  }

  playDoorbellChime();
  world3d.triggerDoorbellFx(plot.plotId);
  mutateHomes({ action: 'ring_doorbell', plotId: plot.plotId }).then((res) => {
    if (res) {
      showToast(
        `🔔 Ding-Dong! Rang the doorbell at ${plot.customTitle || plot.defaultName} (@${plot.ownerUsername})!`
      );
    }
  });
}

function openHomesModal() {
  renderHomesModal();
  if (homesDialog && !homesDialog.open) {
    homesDialog.showModal();
  }
}

function renderHomesModal() {
  const myU = String(selfUser?.username || '').toLowerCase();
  const isHostOrAdmin =
    String(selfUser?.role || '').toLowerCase() === 'host' || Boolean(adminToken);
  const myPlot = homesState.find(
    (p) => String(p.ownerUsername || '').toLowerCase() === myU
  );

  const ownerForm = document.getElementById('home-owner-settings-form');
  if (ownerForm) {
    ownerForm.hidden = !myPlot;
    if (myPlot) {
      const heading = document.getElementById('home-owner-settings-heading');
      if (heading) {
        heading.textContent = `🏷️ My Claimed Home: ${myPlot.customTitle || myPlot.defaultName}`;
      }
      const titleIn = document.getElementById('home-custom-title-input');
      if (titleIn && document.activeElement !== titleIn) {
        titleIn.value = myPlot.customTitle || '';
      }
      const coIn = document.getElementById('home-cobuilders-input');
      if (coIn && document.activeElement !== coIn) {
        coIn.value = (myPlot.coBuilders || []).join(', ');
      }
      const lockChk = document.getElementById('home-door-locked-checkbox');
      if (lockChk) {
        lockChk.checked = Boolean(myPlot.doorLocked);
      }
    }
  }

  const grid = document.getElementById('homes-plots-grid');
  if (!grid) return;
  grid.replaceChildren();

  for (const plot of homesState) {
    const ownerU = String(plot.ownerUsername || '').toLowerCase();
    const isMine = Boolean(myU && ownerU === myU);
    const isUnclaimed = !ownerU;
    const card = document.createElement('article');
    card.className = `backpack-card ${isMine ? 'rarity-legendary' : isUnclaimed ? 'rarity-rare' : 'rarity-epic'}`;

    const top = document.createElement('div');
    top.className = 'backpack-card-top';
    const iconSpan = document.createElement('span');
    iconSpan.className = 'backpack-icon';
    iconSpan.textContent = plot.prebuilt ? '🏡' : '🏗️';
    const statusPill = document.createElement('span');
    statusPill.className = 'backpack-qty-pill';
    statusPill.textContent = isMine
      ? '👑 Your Home'
      : isUnclaimed
        ? '✨ Available Free!'
        : plot.doorLocked
          ? `🔒 Locked (@${plot.ownerUsername})`
          : `🔓 Open (@${plot.ownerUsername})`;
    top.append(iconSpan, statusPill);

    const mid = document.createElement('div');
    const tag = document.createElement('div');
    tag.className = 'backpack-rarity-tag';
    tag.textContent = plot.prebuilt ? 'Furnished Starter Villa' : 'Open Custom Architect Lot';
    const titleEl = document.createElement('h3');
    titleEl.className = 'backpack-card-title';
    titleEl.textContent = plot.customTitle || plot.defaultName;
    const descEl = document.createElement('p');
    descEl.className = 'backpack-desc';
    descEl.textContent = isUnclaimed
      ? 'Unclaimed neighborhood plot — claim for free to personalize its 3D Mailbox & build!'
      : `Owner: ${plot.ownerDisplayName || plot.ownerUsername}${plot.coBuilders?.length ? ` • Co-Builders: ${plot.coBuilders.join(', ')}` : ''}`;
    mid.append(tag, titleEl, descEl);

    const actions = document.createElement('div');
    actions.className = 'backpack-card-actions';
    const btnRow = document.createElement('div');
    btnRow.className = 'backpack-action-btns';

    if (isUnclaimed && sessionToken) {
      const claimBtn = document.createElement('button');
      claimBtn.type = 'button';
      claimBtn.className = 'small-action-btn';
      claimBtn.textContent = myPlot ? '🔄 Move Home Here' : '🏡 Claim Free Plot!';
      claimBtn.addEventListener('click', async () => {
        const out = await mutateHomes({ action: 'claim', plotId: plot.plotId });
        if (out) {
          showToast(`🏡 Claimed ${plot.defaultName}! Customize your 3D Mailbox & Co-Builders above!`);
        }
      });
      btnRow.appendChild(claimBtn);
    }

    if (!isUnclaimed && !isMine && sessionToken) {
      const bellBtn = document.createElement('button');
      bellBtn.type = 'button';
      bellBtn.className = 'small-action-btn';
      bellBtn.textContent = '🔔 Ring Doorbell';
      bellBtn.addEventListener('click', async () => {
        playDoorbellChime();
        world3d.triggerDoorbellFx(plot.plotId);
        const out = await mutateHomes({ action: 'ring_doorbell', plotId: plot.plotId });
        if (out) {
          showToast(`🔔 Rang the doorbell at ${plot.customTitle || plot.defaultName}!`);
        }
      });
      btnRow.appendChild(bellBtn);
    }

    const visitBtn = document.createElement('button');
    visitBtn.type = 'button';
    visitBtn.className = 'secondary-outline-btn';
    visitBtn.textContent = '🚀 Visit Mailbox';
    visitBtn.addEventListener('click', () => {
      if (homesDialog && homesDialog.open) homesDialog.close();
      if (currentZone !== ZONE_PLAZA) {
        requestZoneTransition(ZONE_PLAZA);
      } else if (world3d.selfAvatar && Array.isArray(plot.mailboxPos)) {
        world3d.selfAvatar.group.position.set(
          plot.mailboxPos[0],
          0.15,
          plot.mailboxPos[2] + (plot.mailboxPos[2] >= 0 ? -1.6 : 1.6)
        );
        world3d.velocity.set(0, 0, 0);
        world3d._emitMoveNow();
        showToast(`🏡 Warped to ${plot.customTitle || plot.defaultName}!`);
      }
    });
    btnRow.appendChild(visitBtn);

    if (!isUnclaimed && (isMine || isHostOrAdmin)) {
      const relBtn = document.createElement('button');
      relBtn.type = 'button';
      relBtn.className = 'secondary-outline-btn danger-action';
      relBtn.textContent = isMine ? '🔓 Release' : '🛡️ Mod Release';
      relBtn.addEventListener('click', async () => {
        const out = await mutateHomes({ action: 'release', plotId: plot.plotId });
        if (out) {
          showToast(`🔓 Released ${plot.defaultName}.`);
        }
      });
      btnRow.appendChild(relBtn);
    }

    actions.appendChild(btnRow);
    card.append(top, mid, actions);
    grid.appendChild(card);
  }
}

// ----------------------------------------------------------------------------
// 14C. Starlight Workshop Crafting Bench (#craft-dialog)
// ----------------------------------------------------------------------------

function openCraftModal() {
  renderCraftModal();
  if (craftDialog && !craftDialog.open) {
    craftDialog.showModal();
  }
}

function renderCraftModal() {
  const matBar = document.getElementById('craft-materials-bar');
  const inv = playerEconomy.inventory || {};
  const coins = Number(playerEconomy.coins || 0);
  if (matBar) {
    matBar.replaceChildren();
    const coinChip = document.createElement('span');
    coinChip.className = 'craft-mat-chip';
    coinChip.textContent = `🪙 ${coins.toLocaleString()} Coins`;
    matBar.appendChild(coinChip);

    const keyMatIds = [
      'star_shard',
      'cavern_relic',
      'void_crystal',
      'dragon_scale',
      'ancient_key',
      'solar_citrine'
    ];
    for (const mid of keyMatIds) {
      const def = catalogById.get(mid);
      const qty = Number(inv[mid] || 0);
      const chip = document.createElement('span');
      chip.className = 'craft-mat-chip';
      chip.textContent = `${def ? def.icon : '✨'} ${def ? def.name : mid}: ${qty}x`;
      matBar.appendChild(chip);
    }
  }

  const grid = document.getElementById('craft-recipes-grid');
  if (!grid) return;
  grid.replaceChildren();

  for (const rec of craftingRecipes) {
    const outDef = catalogById.get(rec.outputItemId) || {
      id: rec.outputItemId,
      name: rec.outputItemId,
      icon: '⚒️',
      rarity: 'epic'
    };
    const canAffordCoins = coins >= Number(rec.coinCost || 0);
    let hasAllMats = true;
    const ingParts = [`${canAffordCoins ? '✅' : '❌'} 🪙 ${rec.coinCost} Coins`];
    for (const ing of rec.ingredients || []) {
      const have = Number(inv[ing.itemId] || 0);
      const ok = have >= ing.qty;
      if (!ok) hasAllMats = false;
      const iDef = catalogById.get(ing.itemId);
      ingParts.push(
        `${ok ? '✅' : '❌'} ${iDef ? iDef.icon : '✨'} ${have}/${ing.qty} ${iDef ? iDef.name : ing.itemId}`
      );
    }
    const craftable = canAffordCoins && hasAllMats;

    const card = document.createElement('article');
    card.className = `backpack-card rarity-${outDef.rarity || 'epic'}${!craftable ? ' is-unowned' : ''}`;

    const top = document.createElement('div');
    top.className = 'backpack-card-top';
    const iconSpan = document.createElement('span');
    iconSpan.className = 'backpack-icon';
    iconSpan.textContent = outDef.icon || '⚒️';
    const ownedPill = document.createElement('span');
    ownedPill.className = 'backpack-qty-pill';
    const ownedQty = Number(inv[rec.outputItemId] || 0);
    ownedPill.textContent = ownedQty > 0 ? `${ownedQty}x in Backpack` : 'New Recipe';
    top.append(iconSpan, ownedPill);

    const mid = document.createElement('div');
    const tag = document.createElement('div');
    tag.className = 'backpack-rarity-tag';
    tag.textContent = `${outDef.rarity || 'epic'} • Forges +${rec.outputQty || 1}x`;
    const titleEl = document.createElement('h3');
    titleEl.className = 'backpack-card-title';
    titleEl.textContent = outDef.name;
    const descEl = document.createElement('p');
    descEl.className = 'backpack-desc';
    descEl.textContent = rec.summary || outDef.description || '';
    const reqList = document.createElement('p');
    reqList.className = 'craft-ing-list';
    reqList.textContent = ingParts.join('  •  ');
    mid.append(tag, titleEl, descEl, reqList);

    const actions = document.createElement('div');
    actions.className = 'backpack-card-actions';
    const craftBtn = document.createElement('button');
    craftBtn.type = 'button';
    craftBtn.className = 'primary-cta-btn';
    craftBtn.disabled = !craftable;
    craftBtn.textContent = craftable ? '⚒️ Forge Item!' : '🔒 Gather Materials';
    craftBtn.addEventListener('click', () => craftWorkshopRecipe(rec.id));
    actions.appendChild(craftBtn);

    card.append(top, mid, actions);
    grid.appendChild(card);
  }
}

async function craftWorkshopRecipe(recipeId) {
  if (!sessionToken || !recipeId) return;
  try {
    const res = await fetch('/api/ella-world/craft', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, recipeId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      if (data.error) showToast(`⚠️ ${data.error}`);
      return;
    }
    if (data.economy) applyEconomySnapshot(data.economy);
    if (data.arenaStatus) applyArenaPlayerStatus(data.arenaStatus);
    if (world3d.selfAvatar) {
      world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
    }
    showToast(
      `⚒️✨ Forged +${data.qty || 1}x ${data.itemIcon || '✨'} ${data.itemName || recipeId}! Check Backpack (I)!`,
      4800
    );
  } catch {
    showToast('⚠️ Could not forge item right now.');
  }
}

// ----------------------------------------------------------------------------
// 14D. Obby Speedrun Gates & Persistent Top-5 Leaderboards (#obby-board-dialog)
// ----------------------------------------------------------------------------

async function handleObbyGateCrossed(course, gateKind) {
  if (!sessionToken || currentZone !== ZONE_PLAZA) return;
  const serverNow = Date.now() + (world3d.serverClockOffsetMs || 0);

  if (gateKind === 'start') {
    activeObbyRun = { course, startedAtMs: serverNow };
    world3d.setActiveObbyRun(course);
    const res = await fetch('/api/ella-world/obby', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'start', course })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      if (typeof data.startedAtMs === 'number') {
        activeObbyRun = { course, startedAtMs: data.startedAtMs };
      }
      showToast(
        course === 'hard'
          ? '⏱️🔥 Pro Obby Gauntlet Speedrun STARTED! Sprint to the Summit Finish Gate!'
          : '⏱️🏃‍♀️ Sky Obby Speedrun STARTED! Sprint to the Summit Finish Gate!',
        3200
      );
    } else {
      activeObbyRun = null;
      world3d.setActiveObbyRun(null);
      if (data.error) showToast(`⚠️ ${data.error}`);
    }
    return;
  }

  if (gateKind === 'finish' && activeObbyRun && activeObbyRun.course === course) {
    activeObbyRun = null;
    world3d.setActiveObbyRun(null);
    const res = await fetch('/api/ella-world/obby', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'finish', course })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      if (data.leaderboards) applyObbyLeaderboards(data.leaderboards);
      if (data.economy) applyEconomySnapshot(data.economy);
      if (world3d.selfAvatar) {
        world3d.spawnCelebrationConfetti(world3d.selfAvatar.group.position);
      }
      const rankBadge = data.rank ? ` • #${data.rank} All-Time!` : '';
      const coinBadge = data.awardedCoins ? ` • +${data.awardedCoins} 🪙` : '';
      showToast(
        `⏱️🏆 FINISHED in ${data.timeFormatted || 'record time'}${rankBadge}${coinBadge}!`,
        6200
      );
      announceScreenReaderToast(
        `Finished ${course === 'hard' ? 'Pro Obby Gauntlet' : 'Sky Obby'} in ${data.timeFormatted || 'record time'}.`
      );
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  }
}

function cancelActiveObbyRun(silent = false) {
  if (!activeObbyRun) return;
  activeObbyRun = null;
  world3d.setActiveObbyRun(null);
  if (obbySpeedrunBanner) obbySpeedrunBanner.hidden = true;
  if (sessionToken) {
    fetch('/api/ella-world/obby', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'cancel' })
    }).catch(() => {});
  }
  if (!silent) {
    showToast('⏱️ Obby Speedrun cancelled.');
  }
}

function openObbyBoardModal() {
  renderObbyBoardModal();
  if (obbyBoardDialog && !obbyBoardDialog.open) {
    obbyBoardDialog.showModal();
  }
}

function renderObbyBoardModal() {
  const renderCourseList = (listEl, records) => {
    if (!listEl) return;
    listEl.replaceChildren();
    if (!Array.isArray(records) || records.length === 0) {
      const li = document.createElement('li');
      li.className = 'obby-leaderboard-empty';
      li.textContent = 'No timed speedruns recorded yet — cross the green Start Gate to set #1!';
      listEl.appendChild(li);
      return;
    }
    records.slice(0, 5).forEach((rec, idx) => {
      const li = document.createElement('li');
      li.className = 'obby-leaderboard-row';
      const medal = idx === 0 ? '🥇' : idx === 1 ? '🥈' : idx === 2 ? '🥉' : `#${idx + 1}`;
      const left = document.createElement('span');
      left.textContent = `${medal} ${rec.displayName || rec.username}`;
      const right = document.createElement('strong');
      right.textContent = rec.timeFormatted || `${(Number(rec.elapsedMs || 0) / 1000).toFixed(2)}s`;
      li.append(left, right);
      listEl.appendChild(li);
    });
  };
  renderCourseList(document.getElementById('obby-board-easy-list'), obbyLeaderboards.easy);
  renderCourseList(document.getElementById('obby-board-hard-list'), obbyLeaderboards.hard);
}

// ----------------------------------------------------------------------------
// 14E. Dynamic Sky Cycle, Meteor Shower & Catchable Falling Stars (#sky-dialog)
// ----------------------------------------------------------------------------

async function catchFallingStar(starId) {
  if (!sessionToken || !starId || starCatchInFlightIds.has(starId)) return;
  starCatchInFlightIds.add(starId);
  try {
    const res = await fetch('/api/ella-world/sky', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'catch_star', starId })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (res.ok) {
      if (data.sky) applySkySnapshot(data.sky);
      if (data.economy) applyEconomySnapshot(data.economy);
      if (world3d.selfAvatar) {
        world3d.spawnSparkleBurst(world3d.selfAvatar.group.position, '#fde047');
      }
      showToast(
        `🌠 Caught a Falling Star! +${data.coins || 12} 🪙${data.bonusShard ? ' • +1x ⭐ Starlight Shard!' : '!'}`,
        4200
      );
    }
  } catch {
    // ignore transient race if another friend caught it first
  } finally {
    setTimeout(() => starCatchInFlightIds.delete(starId), 1200);
  }
}

function syncSkyModalButtons() {
  if (!skyDialog) return;
  const activeMode = skyState.mode || 'auto';
  for (const btn of skyDialog.querySelectorAll('[data-sky-mode]')) {
    const m = btn.getAttribute('data-sky-mode');
    btn.classList.toggle('selected', m === activeMode);
    btn.setAttribute('aria-pressed', String(m === activeMode));
  }
}

function openSkyModal() {
  syncSkyModalButtons();
  if (skyDialog && !skyDialog.open) {
    skyDialog.showModal();
  }
}

async function mutateSky(payload) {
  const headers = { ...JSON_HEADERS };
  if (adminToken) {
    headers.Authorization = `Bearer ${adminToken}`;
  }
  try {
    const res = await fetch('/api/ella-world/sky', {
      method: 'POST',
      headers,
      body: JSON.stringify({ token: sessionToken || undefined, ...payload })
    });
    if (sessionToken && !adminToken && (await handleHttpAuthCheck(res))) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      if (data.error) showToast(`⚠️ ${data.error}`);
      return;
    }
    if (data.sky) applySkySnapshot(data.sky);
    if (payload.action === 'set_sky') {
      showToast(`🌅 Plaza Sky switched to ${payload.mode}!`);
    } else if (payload.action === 'start_meteor_shower') {
      showToast('🌠 Meteor Shower launched across Starlight Plaza!');
      if (skyDialog && skyDialog.open) skyDialog.close();
    }
  } catch {
    showToast('⚠️ Could not update Plaza Sky right now.');
  }
}

// ----------------------------------------------------------------------------
// 14F. Shadow Rift Co-Op Obsidian Dragon Boss Summon
// ----------------------------------------------------------------------------

async function summonArenaBoss(useBeacon = false) {
  if (!sessionToken) return;
  try {
    const res = await fetch('/api/ella-world/arena/boss', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, useBeacon: Boolean(useBeacon) })
    });
    if (await handleHttpAuthCheck(res)) return;
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      if (data.error) showToast(`🐉 ${data.error}`);
      return;
    }
    if (data.arena) world3d.syncArenaSnapshot(data.arena);
    if (data.boss) applyArenaBossHud(data.boss);
    if (data.economy) applyEconomySnapshot(data.economy);
    showToast(
      '🐉🔥 THE OBSIDIAN RIFT DRAGON HAS AWAKENED! Jump (Space) over purple Shockwave Rings!',
      6000
    );
  } catch {
    showToast('⚠️ Could not summon the Obsidian Rift Dragon right now.');
  }
}

// ----------------------------------------------------------------------------
// 14G. Wire Expansion Pack HUD Buttons, Modals, Tabs & Popover Listeners
// ----------------------------------------------------------------------------

if (openCompanionsBtn) {
  openCompanionsBtn.addEventListener('click', () => openCompanionsModal('pets'));
}
if (toggleHoverboardBtn) {
  toggleHoverboardBtn.addEventListener('click', () => toggleQuickHoverboard());
}
if (openHomesBtn) {
  openHomesBtn.addEventListener('click', () => openHomesModal());
}
if (openCraftBtn) {
  openCraftBtn.addEventListener('click', () => openCraftModal());
}
if (openObbyBoardBtn) {
  openObbyBoardBtn.addEventListener('click', () => openObbyBoardModal());
}
if (hudSkyPill) {
  hudSkyPill.addEventListener('click', () => {
    const canControlSky =
      (String(selfUser?.role || '').toLowerCase() === 'host' &&
        !playerModeration.inDebtorsPrison) ||
      Boolean(adminToken);
    if (canControlSky) {
      openSkyModal();
    } else {
      const sp = world3d.getSkyPhaseSummary(Date.now() + (world3d.serverClockOffsetMs || 0));
      showToast(`${sp.icon} Starlight Plaza Sky: ${sp.label} (12-minute live cycle)`);
    }
  });
}
if (openSkyBtn) {
  openSkyBtn.addEventListener('click', () => openSkyModal());
}
if (obbySpeedrunCancelBtn) {
  obbySpeedrunCancelBtn.addEventListener('click', () => cancelActiveObbyRun(false));
}

// Companions Dialog Tabs (modern-web-guidance: tabs — focusgroup roving fallback + beforematch)
const companionsTablist = document.getElementById('companions-tablist');
if (companionsTablist) {
  companionsTablist.addEventListener('click', (e) => {
    const tabBtn = e.target instanceof Element ? e.target.closest('[role="tab"]') : null;
    if (tabBtn) {
      activateCompanionsTab(tabBtn.id, false);
    }
  });
  if (!('focusgroup' in HTMLElement.prototype)) {
    companionsTablist.addEventListener('keydown', (e) => {
      const tabs = Array.from(companionsTablist.querySelectorAll('[role="tab"]'));
      const curIdx = tabs.indexOf(document.activeElement);
      if (curIdx === -1) return;
      let nextIdx = -1;
      if (e.key === 'ArrowRight') nextIdx = (curIdx + 1) % tabs.length;
      else if (e.key === 'ArrowLeft') nextIdx = (curIdx - 1 + tabs.length) % tabs.length;
      else if (e.key === 'Home') nextIdx = 0;
      else if (e.key === 'End') nextIdx = tabs.length - 1;
      if (nextIdx !== -1) {
        e.preventDefault();
        activateCompanionsTab(tabs[nextIdx].id, true);
      }
    });
  }
}
document.getElementById('comp-panel-pets')?.addEventListener('beforematch', () => {
  activateCompanionsTab('comp-tab-pets', false);
});
document.getElementById('comp-panel-mounts')?.addEventListener('beforematch', () => {
  activateCompanionsTab('comp-tab-mounts', false);
});

document.getElementById('pet-nickname-form')?.addEventListener('submit', (e) => {
  e.preventDefault();
  const val = document.getElementById('pet-nickname-input')?.value.trim() || '';
  mutateCompanions({ action: 'rename_pet', nickname: val });
});

document.getElementById('hatch-pet-egg-btn')?.addEventListener('click', () => {
  mutateCompanions({ action: 'hatch_egg' });
});

// Homes Owner Settings Form
document.getElementById('home-owner-settings-form')?.addEventListener('submit', async (e) => {
  e.preventDefault();
  const myU = String(selfUser?.username || '').toLowerCase();
  const myPlot = homesState.find((p) => String(p.ownerUsername || '').toLowerCase() === myU);
  if (!myPlot) return;
  const customTitle = document.getElementById('home-custom-title-input')?.value.trim() || '';
  const rawCo = document.getElementById('home-cobuilders-input')?.value || '';
  const coBuilders = rawCo
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean);
  const doorLocked = Boolean(document.getElementById('home-door-locked-checkbox')?.checked);
  const out = await mutateHomes({
    action: 'update_settings',
    plotId: myPlot.plotId,
    customTitle,
    coBuilders,
    doorLocked
  });
  if (out) {
    showToast('✅ Saved 3D Mailbox Nameplate, Co-Builders & Door Lock settings!');
  }
});

document.getElementById('home-release-mine-btn')?.addEventListener('click', async () => {
  const myU = String(selfUser?.username || '').toLowerCase();
  const myPlot = homesState.find((p) => String(p.ownerUsername || '').toLowerCase() === myU);
  if (!myPlot) return;
  const out = await mutateHomes({ action: 'release', plotId: myPlot.plotId });
  if (out) {
    showToast(`🔓 Released ${myPlot.defaultName}. You can claim any open plot anytime!`);
  }
});

// Obby Leaderboard Warp-to-Start Buttons
document.getElementById('obby-board-warp-easy-btn')?.addEventListener('click', () => {
  if (obbyBoardDialog && obbyBoardDialog.open) obbyBoardDialog.close();
  cancelActiveObbyRun(true);
  if (currentZone !== ZONE_PLAZA) {
    requestZoneTransition(ZONE_PLAZA);
  } else if (world3d.selfAvatar) {
    world3d.selfAvatar.group.position.set(17.5, 0.15, 7.2);
    world3d.velocity.set(0, 0, 0);
    world3d._emitMoveNow();
    showToast('⏱️ Ready at the Sky Obby Start Gate — cross the glowing green laser on foot!');
  }
});

document.getElementById('obby-board-warp-hard-btn')?.addEventListener('click', () => {
  if (obbyBoardDialog && obbyBoardDialog.open) obbyBoardDialog.close();
  cancelActiveObbyRun(true);
  if (currentZone !== ZONE_PLAZA) {
    requestZoneTransition(ZONE_PLAZA);
  } else if (world3d.selfAvatar) {
    world3d.selfAvatar.group.position.set(-22.5, 0.15, 19.8);
    world3d.velocity.set(0, 0, 0);
    world3d._emitMoveNow();
    showToast('⏱️🔥 Ready at the Pro Obby Gauntlet Start Gate — cross the laser on foot!');
  }
});

// Sky Dialog Controls
if (skyDialog) {
  skyDialog.addEventListener('click', (e) => {
    const modeBtn = e.target instanceof Element ? e.target.closest('[data-sky-mode]') : null;
    if (modeBtn) {
      const mode = modeBtn.getAttribute('data-sky-mode');
      if (mode) mutateSky({ action: 'set_sky', mode });
    }
  });
}
document.getElementById('launch-meteor-shower-btn')?.addEventListener('click', () => {
  mutateSky({ action: 'start_meteor_shower' });
});

// Fashion Show Invite Popover Join CTA (Opt-in — warps to Runway & opens Glam Studio!)
if (fashionInviteJoinBtn) {
  fashionInviteJoinBtn.addEventListener('click', async () => {
    hideTopLayerToastPopover(fashionInvitePopover);
    if (!sessionToken) return;
    const res = await fetch('/api/ella-world/runway', {
      method: 'POST',
      headers: JSON_HEADERS,
      body: JSON.stringify({ token: sessionToken, action: 'join_show' })
    }).catch(() => null);
    if (await handleHttpAuthCheck(res)) return;
    const data = res ? await res.json().catch(() => ({})) : {};
    if (res && res.ok) {
      if (data.runway) applyRunwaySummary(data.runway);
      if (currentZone !== ZONE_PLAZA) {
        pendingPostPortalWarp = 'runway';
        await requestZoneTransition(ZONE_PLAZA);
      } else {
        world3d.warpToRunway();
      }
      if (!isSalonOpen) openGlamSalon();
      showToast('👗 Joined the Fashion Show! Style your look before the Dress-Up timer hits 0s!', 4600);
    } else if (data.error) {
      showToast(`⚠️ ${data.error}`);
    }
  });
}

// Doorbell Popover Quick-Unlock CTA
if (doorbellUnlockBtn) {
  doorbellUnlockBtn.addEventListener('click', async () => {
    hideTopLayerToastPopover(doorbellPopover);
    const myU = String(selfUser?.username || '').toLowerCase();
    const myPlot =
      homesState.find((p) => p.plotId === lastDoorbellPlotId) ||
      homesState.find((p) => String(p.ownerUsername || '').toLowerCase() === myU);
    if (!myPlot) return;
    const out = await mutateHomes({
      action: 'update_settings',
      plotId: myPlot.plotId,
      customTitle: myPlot.customTitle || '',
      coBuilders: myPlot.coBuilders || [],
      doorLocked: false
    });
    if (out) {
      showToast(`🔓 Unlocked ${myPlot.customTitle || myPlot.defaultName} for your visitors!`);
    }
  });
}
