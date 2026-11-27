/**
 * Pure Client-Side ISO/IEC 18004 QR Code SVG Generator (Versions 1–6, Byte Mode, EC Level L)
 * Zero external dependencies — renders scannable vector `<svg>` QR codes for:
 *   - Direct family Android APK download (`https://robertscreations.com:4444/download/ella-world.apk`)
 *   - Shareable `STAR-...` & `FAMILY-...` deep-link URLs (`https://robertscreations.com:4444/?invite=...`)
 */

// Version 1..6 capacities for EC Level L (single Reed-Solomon block for V1..V6 EC-L!)
const QR_VERSIONS_L = [
  null,
  { v: 1, size: 21, totalCodewords: 26, ecCodewords: 7, align: [] },
  { v: 2, size: 25, totalCodewords: 44, ecCodewords: 10, align: [6, 18] },
  { v: 3, size: 29, totalCodewords: 70, ecCodewords: 15, align: [6, 22] },
  { v: 4, size: 33, totalCodewords: 100, ecCodewords: 20, align: [6, 26] },
  { v: 5, size: 37, totalCodewords: 134, ecCodewords: 26, align: [6, 30] },
  { v: 6, size: 41, totalCodewords: 172, ecCodewords: 36, align: [] } // V6 uses 2 blocks; cap at V5 (106 UTF-8 bytes)
];

const GF_EXP = new Uint8Array(512);
const GF_LOG = new Uint8Array(256);
(() => {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    GF_EXP[i] = x;
    GF_LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i++) {
    GF_EXP[i] = GF_EXP[i - 255];
  }
})();

function gfMul(a, b) {
  if (a === 0 || b === 0) return 0;
  return GF_EXP[GF_LOG[a] + GF_LOG[b]];
}

function buildRsGenerator(degree) {
  let poly = [1];
  for (let i = 0; i < degree; i++) {
    const next = new Array(poly.length + 1).fill(0);
    const factor = GF_EXP[i];
    for (let j = 0; j < poly.length; j++) {
      next[j] ^= poly[j];
      next[j + 1] ^= gfMul(poly[j], factor);
    }
    poly = next;
  }
  return poly;
}

function computeRsRemainder(data, ecLen) {
  const gen = buildRsGenerator(ecLen);
  const rem = new Uint8Array(ecLen);
  for (let i = 0; i < data.length; i++) {
    const factor = data[i] ^ rem[0];
    rem.copyWithin(0, 1);
    rem[ecLen - 1] = 0;
    if (factor !== 0) {
      for (let j = 0; j < ecLen; j++) {
        rem[j] ^= gfMul(gen[j + 1], factor);
      }
    }
  }
  return rem;
}

function computeFormatBits(ecLevelBits, maskPattern) {
  const data = (ecLevelBits << 3) | maskPattern;
  let rem = data << 10;
  for (let i = 4; i >= 0; i--) {
    if ((rem >> (i + 10)) & 1) {
      rem ^= 0x537 << i;
    }
  }
  return ((data << 10) | rem) ^ 0x5412;
}

function maskBit(mask, r, c) {
  switch (mask) {
    case 0:
      return (r + c) % 2 === 0;
    case 1:
      return r % 2 === 0;
    case 2:
      return c % 3 === 0;
    case 3:
      return (r + c) % 3 === 0;
    case 4:
      return (Math.floor(r / 2) + Math.floor(c / 3)) % 2 === 0;
    case 5:
      return ((r * c) % 2) + ((r * c) % 3) === 0;
    case 6:
      return (((r * c) % 2) + ((r * c) % 3)) % 2 === 0;
    default:
      return (((r + c) % 2) + ((r * c) % 3)) % 2 === 0;
  }
}

export function encodeQrMatrix(text) {
  const bytes = new TextEncoder().encode(String(text || ''));
  if (bytes.length === 0) {
    throw new Error('QR payload cannot be empty.');
  }
  let spec = null;
  for (let v = 1; v <= 5; v++) {
    const candidate = QR_VERSIONS_L[v];
    const dataCap = candidate.totalCodewords - candidate.ecCodewords;
    // 4-bit mode + 8-bit length + 8*N + 4-bit terminator -> bytes + 2 <= dataCap
    if (bytes.length + 2 <= dataCap) {
      spec = candidate;
      break;
    }
  }
  if (!spec) {
    throw new Error('QR payload exceeds Version 5 EC-L capacity (106 bytes).');
  }

  const dataCodewordsLen = spec.totalCodewords - spec.ecCodewords;
  const bits = [];
  const pushBits = (val, count) => {
    for (let i = count - 1; i >= 0; i--) {
      bits.push((val >> i) & 1);
    }
  };

  // Byte mode indicator (0100) + 8-bit byte count (Versions 1..9)
  pushBits(0b0100, 4);
  pushBits(bytes.length, 8);
  for (const b of bytes) {
    pushBits(b, 8);
  }
  const maxDataBits = dataCodewordsLen * 8;
  const termLen = Math.min(4, maxDataBits - bits.length);
  pushBits(0, termLen);
  while (bits.length % 8 !== 0) bits.push(0);

  const dataCodewords = new Uint8Array(dataCodewordsLen);
  for (let i = 0; i < bits.length / 8; i++) {
    let byte = 0;
    for (let b = 0; b < 8; b++) {
      byte = (byte << 1) | bits[i * 8 + b];
    }
    dataCodewords[i] = byte;
  }
  const padBytes = [0xec, 0x11];
  for (let i = bits.length / 8, p = 0; i < dataCodewordsLen; i++, p++) {
    dataCodewords[i] = padBytes[p % 2];
  }

  const ecCodewords = computeRsRemainder(dataCodewords, spec.ecCodewords);
  const allCodewords = new Uint8Array(spec.totalCodewords);
  allCodewords.set(dataCodewords, 0);
  allCodewords.set(ecCodewords, dataCodewordsLen);

  const n = spec.size;
  const grid = Array.from({ length: n }, () => new Uint8Array(n));
  const reserved = Array.from({ length: n }, () => new Uint8Array(n));

  const setFunctionModule = (r, c, dark) => {
    if (r < 0 || r >= n || c < 0 || c >= n) return;
    grid[r][c] = dark ? 1 : 0;
    reserved[r][c] = 1;
  };

  const placeFinder = (top, left) => {
    for (let dr = -1; dr <= 7; dr++) {
      for (let dc = -1; dc <= 7; dc++) {
        const inside =
          dr >= 0 &&
          dr <= 6 &&
          dc >= 0 &&
          dc <= 6 &&
          (dr === 0 ||
            dr === 6 ||
            dc === 0 ||
            dc === 6 ||
            (dr >= 2 && dr <= 4 && dc >= 2 && dc <= 4));
        setFunctionModule(top + dr, left + dc, inside);
      }
    }
  };

  placeFinder(0, 0);
  placeFinder(0, n - 7);
  placeFinder(n - 7, 0);

  // Timing patterns
  for (let i = 8; i < n - 8; i++) {
    setFunctionModule(6, i, i % 2 === 0);
    setFunctionModule(i, 6, i % 2 === 0);
  }

  // Alignment patterns (V2..V5)
  for (const r of spec.align) {
    for (const c of spec.align) {
      if (reserved[r][c]) continue;
      for (let dr = -2; dr <= 2; dr++) {
        for (let dc = -2; dc <= 2; dc++) {
          const dark = Math.max(Math.abs(dr), Math.abs(dc)) !== 1;
          setFunctionModule(r + dr, c + dc, dark);
        }
      }
    }
  }

  // Dark module + reserve format info strips
  setFunctionModule(4 * spec.v + 9, 8, true);
  for (let i = 0; i <= 8; i++) {
    if (!reserved[8][i]) reserved[8][i] = 1;
    if (!reserved[i][8]) reserved[i][8] = 1;
  }
  for (let i = 0; i < 8; i++) {
    reserved[8][n - 1 - i] = 1;
    reserved[n - 1 - i][8] = 1;
  }

  // Zigzag data placement
  let bitIdx = 0;
  const totalBits = allCodewords.length * 8;
  let upward = true;
  for (let right = n - 1; right >= 1; right -= 2) {
    if (right === 6) right = 5;
    for (let vert = 0; vert < n; vert++) {
      const r = upward ? n - 1 - vert : vert;
      for (let j = 0; j < 2; j++) {
        const c = right - j;
        if (!reserved[r][c]) {
          let dark = false;
          if (bitIdx < totalBits) {
            dark = ((allCodewords[bitIdx >> 3] >> (7 - (bitIdx & 7))) & 1) === 1;
            bitIdx++;
          }
          grid[r][c] = dark ? 1 : 0;
        }
      }
    }
    upward = !upward;
  }

  // Pick lowest-penalty mask among all 8 ISO mask patterns (EC Level L => indicator bits 0b01)
  const applyFormat = (target, mask) => {
    const fmt = computeFormatBits(0b01, mask);
    for (let i = 0; i <= 5; i++) target[8][i] = (fmt >> (14 - i)) & 1;
    target[8][7] = (fmt >> 8) & 1;
    target[8][8] = (fmt >> 7) & 1;
    target[7][8] = (fmt >> 6) & 1;
    for (let i = 9; i < 15; i++) target[14 - i][8] = (fmt >> (14 - i)) & 1;

    for (let i = 0; i < 8; i++) target[8][n - 1 - i] = (fmt >> i) & 1;
    for (let i = 8; i < 15; i++) target[n - 15 + i][8] = (fmt >> i) & 1;
  };

  const scorePenalty = (mGrid) => {
    let score = 0;
    for (let r = 0; r < n; r++) {
      let runLen = 1;
      for (let c = 1; c < n; c++) {
        if (mGrid[r][c] === mGrid[r][c - 1]) {
          runLen++;
          if (runLen === 5) score += 3;
          else if (runLen > 5) score += 1;
        } else {
          runLen = 1;
        }
      }
    }
    for (let c = 0; c < n; c++) {
      let runLen = 1;
      for (let r = 1; r < n; r++) {
        if (mGrid[r][c] === mGrid[r - 1][c]) {
          runLen++;
          if (runLen === 5) score += 3;
          else if (runLen > 5) score += 1;
        } else {
          runLen = 1;
        }
      }
    }
    return score;
  };

  let bestGrid = null;
  let bestScore = Infinity;
  for (let mask = 0; mask < 8; mask++) {
    const candidate = grid.map((row, r) => {
      const copy = new Uint8Array(row);
      for (let c = 0; c < n; c++) {
        if (!reserved[r][c] && maskBit(mask, r, c)) {
          copy[c] ^= 1;
        }
      }
      return copy;
    });
    applyFormat(candidate, mask);
    const p = scorePenalty(candidate);
    if (p < bestScore) {
      bestScore = p;
      bestGrid = candidate;
    }
  }

  return bestGrid;
}

export function renderQrSvgString(url, label = 'QR Code') {
  const matrix = encodeQrMatrix(url);
  const n = matrix.length;
  const quiet = 4;
  const total = n + quiet * 2;
  const rects = [];
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      if (matrix[r][c]) {
        rects.push(`M${c + quiet},${r + quiet}h1v1h-1z`);
      }
    }
  }
  const safeLabel = String(label).replace(/[<>&"]/g, '');
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${total} ${total}" role="img" aria-label="${safeLabel}" shape-rendering="crispEdges"><rect width="${total}" height="${total}" rx="2" fill="#ffffff"/><path d="${rects.join('')}" fill="#0f172a"/></svg>`;
}
