/**
 * ELLA World — Cryptographic Machine Seal & Minimized Hardware Digital Signature (`device_sig.js`)
 *
 * Defense-in-depth client module paired with `src/device_sig.rs`:
 * 1. Dual-redundant client seal vault (`localStorage` + `IndexedDB`) complementing the server's
 *    `HttpOnly` `__Host-ewld_machine_seal` / `ewld_machine_seal` cookie. If a player wipes one
 *    client storage area in DevTools, the surviving vault automatically heals it on the next request.
 * 2. Privacy-minimized hardware telemetry (`collectMachineTelemetry`) collecting local irreversible
 *    digests of WebGL GPU capabilities, 2D Canvas rasterization, and OfflineAudioContext DSP output
 *    alongside display/CPU/OS traits — without third-party trackers or raw media/pixel uploads.
 */

const LS_SEAL_KEY = 'ella_world_machine_seal';
const IDB_NAME = 'ella_world_device_vault';
const IDB_STORE = 'seals';
const IDB_PRIMARY_KEY = 'primary';

let cachedTelemetryPromise = null;

function withTimeout(promise, ms, fallbackVal = null) {
  let timer = null;
  const timeoutPromise = new Promise((resolve) => {
    timer = setTimeout(() => resolve(fallbackVal), ms);
  });
  return Promise.race([promise, timeoutPromise]).finally(() => {
    if (timer) clearTimeout(timer);
  });
}

/**
 * Deterministic 64-bit dual-lane FNV-1a hex digest (16 hex chars), plus SHA-256 upgrade when
 * `crypto.subtle` is available in Secure Contexts (`https:` or `localhost`).
 */
function fnv64Hex(str) {
  let h1 = 0x811c9dc5;
  let h2 = 0x01000193;
  for (let i = 0; i < str.length; i += 1) {
    const c = str.charCodeAt(i);
    h1 ^= c;
    h1 = Math.imul(h1, 0x01000193) >>> 0;
    h2 ^= c ^ ((i & 0xff) + 17);
    h2 = Math.imul(h2, 0x85ebca6b) >>> 0;
  }
  return h1.toString(16).padStart(8, '0') + h2.toString(16).padStart(8, '0');
}

async function digestStringHex(input) {
  const raw = String(input || '');
  if (typeof crypto !== 'undefined' && crypto.subtle && typeof TextEncoder !== 'undefined') {
    try {
      const bytes = new TextEncoder().encode(raw);
      const buf = await crypto.subtle.digest('SHA-256', bytes);
      const view = new Uint8Array(buf);
      let hex = '';
      for (let i = 0; i < 16; i += 1) {
        hex += view[i].toString(16).padStart(2, '0');
      }
      return hex;
    } catch {
      // Fall back to deterministic FNV-64 on non-secure HTTP LAN origins
    }
  }
  return fnv64Hex(raw);
}

function openSealDb() {
  return new Promise((resolve) => {
    if (typeof indexedDB === 'undefined') {
      resolve(null);
      return;
    }
    try {
      const req = indexedDB.open(IDB_NAME, 1);
      req.onupgradeneeded = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains(IDB_STORE)) {
          db.createObjectStore(IDB_STORE);
        }
      };
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => resolve(null);
      req.onblocked = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

async function readIndexedDbSeal() {
  const db = await withTimeout(openSealDb(), 220, null);
  if (!db) return '';
  return withTimeout(
    new Promise((resolve) => {
      try {
        const tx = db.transaction(IDB_STORE, 'readonly');
        const store = tx.objectStore(IDB_STORE);
        const getReq = store.get(IDB_PRIMARY_KEY);
        getReq.onsuccess = () => {
          const val = typeof getReq.result === 'string' ? getReq.result.trim() : '';
          resolve(val);
        };
        getReq.onerror = () => resolve('');
      } catch {
        resolve('');
      }
    }),
    220,
    ''
  ).finally(() => {
    try {
      db.close();
    } catch {
      // ignore close error
    }
  });
}

async function writeIndexedDbSeal(seal) {
  const db = await withTimeout(openSealDb(), 220, null);
  if (!db) return;
  await withTimeout(
    new Promise((resolve) => {
      try {
        const tx = db.transaction(IDB_STORE, 'readwrite');
        tx.objectStore(IDB_STORE).put(seal, IDB_PRIMARY_KEY);
        tx.oncomplete = () => resolve(true);
        tx.onerror = () => resolve(false);
        tx.onabort = () => resolve(false);
      } catch {
        resolve(false);
      }
    }),
    220,
    false
  );
  try {
    db.close();
  } catch {
    // ignore close error
  }
}

/**
 * Loads the server-signed `ewld_machine_seal` from `localStorage` and `IndexedDB`,
 * automatically healing whichever store was cleared.
 */
export async function loadStoredMachineSeal() {
  let lsVal = '';
  try {
    lsVal = (localStorage.getItem(LS_SEAL_KEY) || '').trim();
  } catch {
    lsVal = '';
  }

  const idbVal = await readIndexedDbSeal();
  const chosen = lsVal || idbVal || '';

  if (chosen) {
    if (!lsVal) {
      try {
        localStorage.setItem(LS_SEAL_KEY, chosen);
      } catch {
        // ignore storage quota / private mode
      }
    }
    if (!idbVal) {
      writeIndexedDbSeal(chosen).catch(() => {});
    }
  }
  return chosen;
}

/**
 * Persists a newly issued or refreshed server-signed `ewld_machine_seal` into both
 * `localStorage` and `IndexedDB`.
 */
export async function persistMachineSeal(seal) {
  const clean = typeof seal === 'string' ? seal.trim() : '';
  if (!clean) return;
  try {
    localStorage.setItem(LS_SEAL_KEY, clean);
  } catch {
    // ignore storage quota / private mode
  }
  await writeIndexedDbSeal(clean);
}

function collectWebGlTraits() {
  try {
    const canvas = document.createElement('canvas');
    canvas.width = 16;
    canvas.height = 16;
    const gl =
      canvas.getContext('webgl2', { powerPreference: 'default' }) ||
      canvas.getContext('webgl') ||
      canvas.getContext('experimental-webgl');
    if (!gl) {
      return { gpuVendor: '', gpuRenderer: '', glCaps: '' };
    }

    let gpuVendor = String(gl.getParameter(gl.VENDOR) || '');
    let gpuRenderer = String(gl.getParameter(gl.RENDERER) || '');
    const dbg = gl.getExtension('WEBGL_debug_renderer_info');
    if (dbg) {
      const unmaskedVendor = gl.getParameter(dbg.UNMASKED_VENDOR_WEBGL);
      const unmaskedRenderer = gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL);
      if (unmaskedVendor) gpuVendor = String(unmaskedVendor);
      if (unmaskedRenderer) gpuRenderer = String(unmaskedRenderer);
    }

    const viewport = gl.getParameter(gl.MAX_VIEWPORT_DIMS);
    const vpStr = viewport ? `${viewport[0]}x${viewport[1]}` : '';
    const caps = [
      gl.getParameter(gl.MAX_TEXTURE_SIZE),
      gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE),
      gl.getParameter(gl.MAX_RENDERBUFFER_SIZE),
      gl.getParameter(gl.MAX_VERTEX_ATTRIBS),
      gl.getParameter(gl.MAX_VERTEX_UNIFORM_VECTORS),
      gl.getParameter(gl.MAX_FRAGMENT_UNIFORM_VECTORS),
      gl.getParameter(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS),
      vpStr
    ].join('|');

    // Release temporary WebGL context immediately so Three.js keeps full GPU context budget
    const loseExt = gl.getExtension('WEBGL_lose_context');
    if (loseExt) loseExt.loseContext();

    return { gpuVendor, gpuRenderer, glCaps: caps };
  } catch {
    return { gpuVendor: '', gpuRenderer: '', glCaps: '' };
  }
}

async function collectCanvasDigest() {
  try {
    const canvas = document.createElement('canvas');
    canvas.width = 240;
    canvas.height = 60;
    const ctx = canvas.getContext('2d');
    if (!ctx) return '';

    ctx.textBaseline = 'alphabetic';
    const grad = ctx.createLinearGradient(0, 0, 240, 60);
    grad.addColorStop(0, '#ec4899');
    grad.addColorStop(0.5, '#38bdf8');
    grad.addColorStop(1, '#fde047');
    ctx.fillStyle = grad;
    ctx.fillRect(4, 4, 232, 52);

    ctx.globalCompositeOperation = 'multiply';
    ctx.fillStyle = 'rgba(99, 102, 241, 0.78)';
    ctx.beginPath();
    ctx.arc(76, 30, 22, 0, Math.PI * 2, true);
    ctx.closePath();
    ctx.fill();

    ctx.globalCompositeOperation = 'source-over';
    ctx.fillStyle = '#0f172a';
    ctx.font = '15px sans-serif';
    ctx.fillText('EllaWorld-Sig-v1 ★ ⚖️', 12, 34);
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.85)';
    ctx.beginPath();
    ctx.arc(188, 30, 16, 0.25, Math.PI * 1.65, false);
    ctx.stroke();

    const dataUrl = canvas.toDataURL('image/png');
    return digestStringHex(dataUrl);
  } catch {
    return '';
  }
}

async function collectAudioDigest() {
  const OfflineCtx =
    typeof window !== 'undefined'
      ? window.OfflineAudioContext || window.webkitOfflineAudioContext
      : null;
  if (!OfflineCtx) return '';

  const runProbe = async () => {
    const ctx = new OfflineCtx(1, 4500, 44100);
    const osc = ctx.createOscillator();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(10000, ctx.currentTime);

    const comp = ctx.createDynamicsCompressor();
    comp.threshold.setValueAtTime(-50, ctx.currentTime);
    comp.knee.setValueAtTime(40, ctx.currentTime);
    comp.ratio.setValueAtTime(12, ctx.currentTime);
    comp.attack.setValueAtTime(0, ctx.currentTime);
    comp.release.setValueAtTime(0.25, ctx.currentTime);

    osc.connect(comp);
    comp.connect(ctx.destination);
    osc.start(0);

    const rendered = await ctx.startRendering();
    const samples = rendered.getChannelData(0);
    let acc = '';
    for (let i = 4000; i < Math.min(4500, samples.length); i += 4) {
      acc += Math.round(samples[i] * 1e6).toString(36) + ',';
    }
    return digestStringHex(acc);
  };

  try {
    return (await withTimeout(runProbe(), 280, '')) || '';
  } catch {
    return '';
  }
}

async function resolveOsFamily() {
  try {
    if (navigator.userAgentData) {
      if (typeof navigator.userAgentData.platform === 'string' && navigator.userAgentData.platform) {
        return navigator.userAgentData.platform;
      }
      if (typeof navigator.userAgentData.getHighEntropyValues === 'function') {
        const hi = await withTimeout(
          navigator.userAgentData.getHighEntropyValues(['platform']),
          120,
          null
        );
        if (hi && hi.platform) return String(hi.platform);
      }
    }
  } catch {
    // fall through
  }
  return String(navigator.platform || navigator.userAgent || '');
}

/**
 * Collects minimized hardware, GPU, canvas, audio, display, and OS traits once per page load.
 */
export async function collectMachineTelemetry() {
  if (!cachedTelemetryPromise) {
    cachedTelemetryPromise = (async () => {
      const gl = collectWebGlTraits();
      const [canvasHash, audioHash, osFamily] = await Promise.all([
        collectCanvasDigest(),
        collectAudioDigest(),
        resolveOsFamily()
      ]);

      const screenW = Math.max(0, Math.round(Number(window.screen?.width || 0)));
      const screenH = Math.max(0, Math.round(Number(window.screen?.height || 0)));
      const colorDepth = Math.max(0, Math.round(Number(window.screen?.colorDepth || 24)));
      const pixelRatioX100 = Math.max(
        50,
        Math.round(Number(window.devicePixelRatio || 1) * 100)
      );
      const cpuCores = Math.max(0, Math.round(Number(navigator.hardwareConcurrency || 0)));
      const deviceMemoryGb = Math.max(0, Math.round(Number(navigator.deviceMemory || 0)));
      const maxTouchPoints = Math.max(0, Math.round(Number(navigator.maxTouchPoints || 0)));

      let timezone = '';
      try {
        timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || '';
      } catch {
        timezone = '';
      }

      return {
        gpuVendor: gl.gpuVendor,
        gpuRenderer: gl.gpuRenderer,
        glCaps: gl.glCaps,
        canvasHash,
        audioHash,
        screenW,
        screenH,
        colorDepth,
        pixelRatioX100,
        cpuCores,
        deviceMemoryGb,
        maxTouchPoints,
        osFamily,
        timezone,
        language: String(navigator.language || '')
      };
    })();
  }
  return cachedTelemetryPromise;
}
