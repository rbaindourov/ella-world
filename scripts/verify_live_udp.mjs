#!/usr/bin/env node
/**
 * Permanent Live HTTP + Static Asset + Dual-Port UDP Verification Client (`verify_live_udp.mjs`)
 *
 * Usage:
 *   node scripts/verify_live_udp.mjs \
 *     --base-url https://your-domain.example.com \
 *     --lan-ip <SERVER_LAN_IP> \
 *     --wan-ip <SERVER_WAN_IP> \
 *     --username ella --password <PASSWORD>
 */

import dgram from 'node:dgram';
import crypto from 'node:crypto';

function parseArgs(argv) {
  const out = {
    baseUrl: 'http://127.0.0.1:8097',
    secondaryBaseUrl: '',
    lanIp: '',
    wanIp: '',
    webrtcPort: 44444,
    nativePort: 44445,
    username: '',
    password: ''
  };
  for (let i = 2; i < argv.length; i++) {
    const k = argv[i];
    const v = argv[i + 1];
    if (k === '--base-url' && v) { out.baseUrl = v; i++; }
    else if (k === '--secondary-url' && v) { out.secondaryBaseUrl = v; i++; }
    else if (k === '--lan-ip' && v) { out.lanIp = v; i++; }
    else if (k === '--wan-ip' && v) { out.wanIp = v; i++; }
    else if (k === '--username' && v) { out.username = v; i++; }
    else if (k === '--password' && v) { out.password = v; i++; }
  }
  return out;
}

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) {
      c = (c >>> 1) ^ (c & 1 ? 0xedb88320 : 0);
    }
  }
  return (c ^ 0xffffffff) >>> 0;
}

// Build a full RFC 5389 / RFC 8445 ICE STUN Binding Request (`USERNAME`, `PRIORITY`,
// `ICE-CONTROLLING`, `MESSAGE-INTEGRITY` HMAC-SHA1, and `FINGERPRINT` CRC32) so `webrtc-rs`
// UDPMuxDefault authenticates and replies with `0x0101` Binding Success Response on 44444/udp!
function buildMuxedStunBindingRequest(serverUfrag, serverPwd) {
  const usernameVal = Buffer.from(`${serverUfrag}:probeclient`, 'utf8');
  const uPad = (usernameVal.length + 3) & ~3;
  const attrUsername = Buffer.alloc(4 + uPad);
  attrUsername.writeUInt16BE(0x0006, 0);
  attrUsername.writeUInt16BE(usernameVal.length, 2);
  usernameVal.copy(attrUsername, 4);

  const attrPriority = Buffer.alloc(8);
  attrPriority.writeUInt16BE(0x0024, 0); // PRIORITY
  attrPriority.writeUInt16BE(4, 2);
  attrPriority.writeUInt32BE(0x6e001eff, 4);

  const attrControlling = Buffer.alloc(12);
  attrControlling.writeUInt16BE(0x802a, 0); // ICE-CONTROLLING
  attrControlling.writeUInt16BE(8, 2);
  crypto.randomBytes(8).copy(attrControlling, 4);

  const baseAttrs = Buffer.concat([attrUsername, attrPriority, attrControlling]);
  const header = Buffer.alloc(20);
  header.writeUInt16BE(0x0001, 0); // Binding Request
  // Length up to and including MESSAGE-INTEGRITY (24 bytes)
  header.writeUInt16BE(baseAttrs.length + 24, 2);
  header.writeUInt32BE(0x2112a442, 4);
  crypto.randomBytes(12).copy(header, 8);

  const hmacInput = Buffer.concat([header, baseAttrs]);
  const hmac = crypto.createHmac('sha1', Buffer.from(serverPwd, 'utf8')).update(hmacInput).digest();
  const attrMsgIntegrity = Buffer.alloc(24);
  attrMsgIntegrity.writeUInt16BE(0x0008, 0); // MESSAGE-INTEGRITY
  attrMsgIntegrity.writeUInt16BE(20, 2);
  hmac.copy(attrMsgIntegrity, 4);

  // Update header length to also include FINGERPRINT (8 bytes)
  header.writeUInt16BE(baseAttrs.length + 24 + 8, 2);
  const preFp = Buffer.concat([header, baseAttrs, attrMsgIntegrity]);
  const fpVal = (crc32(preFp) ^ 0x5354554e) >>> 0;
  const attrFingerprint = Buffer.alloc(8);
  attrFingerprint.writeUInt16BE(0x8028, 0); // FINGERPRINT
  attrFingerprint.writeUInt16BE(4, 2);
  attrFingerprint.writeUInt32BE(fpVal, 4);

  return Buffer.concat([preFp, attrFingerprint]);
}

function probeUdpPacket(host, port, payload, timeoutMs = 1800) {
  return new Promise((resolve) => {
    const sock = dgram.createSocket('udp4');
    const started = performance.now();
    let settled = false;
    const finish = (res) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      try { sock.close(); } catch {}
      resolve(res);
    };
    const timer = setTimeout(() => {
      finish({ reachable: false, reason: 'timeout', rttMs: null });
    }, timeoutMs);

    sock.once('error', (err) => {
      finish({ reachable: false, reason: err.message, rttMs: null });
    });
    sock.once('message', (msg) => {
      const rttMs = Math.round((performance.now() - started) * 100) / 100;
      finish({ reachable: true, rttMs, bytes: msg.length, headHex: msg.subarray(0, 8).toString('hex') });
    });
    sock.send(payload, port, host);
  });
}

function buildNativeEwldHello(token) {
  const tb = Buffer.from(token, 'utf8');
  const out = Buffer.alloc(6 + 2 + tb.length);
  out.write('EWLD', 0, 4, 'ascii');
  out.writeUInt8(1, 4); // version 1
  out.writeUInt8(0x01, 5); // OP_HELLO
  out.writeUInt16LE(tb.length, 6);
  tb.copy(out, 8);
  return out;
}

const VALID_MINIMAL_BROWSER_OFFER_SDP = [
  'v=0',
  'o=- 4611731400430051336 2 IN IP4 127.0.0.1',
  's=-',
  't=0 0',
  'a=group:BUNDLE 0 1',
  'a=msid-semantic: WMS',
  'm=audio 9 UDP/TLS/RTP/SAVPF 111',
  'c=IN IP4 0.0.0.0',
  'a=rtcp:9 IN IP4 0.0.0.0',
  'a=ice-ufrag:probeclient',
  'a=ice-pwd:probeclientpassword12345678',
  'a=fingerprint:sha-256 00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF',
  'a=setup:actpass',
  'a=mid:0',
  'a=sendonly',
  'a=rtcp-mux',
  'a=rtpmap:111 opus/48000/2',
  'm=application 9 UDP/DTLS/SCTP webrtc-datachannel',
  'c=IN IP4 0.0.0.0',
  'a=ice-ufrag:probeclient',
  'a=ice-pwd:probeclientpassword12345678',
  'a=fingerprint:sha-256 00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF',
  'a=setup:actpass',
  'a=mid:1',
  'a=sctp-port:5000',
  ''
].join('\r\n');

async function main() {
  const args = parseArgs(process.argv);
  const report = { ok: true, http: {}, assets: [], udp: {} };

  // 1. Primary & Secondary HTTPS Health Checks
  for (const base of [args.baseUrl, args.secondaryBaseUrl].filter(Boolean)) {
    const res = await fetch(`${base}/api/ella-world/health`, {
      headers: { 'X-Cache-Warmup': 'true' }
    });
    const json = await res.json();
    report.http[base] = { status: res.status, health: json };
    if (res.status !== 200 || json.engine !== 'ella-world-rust-udp' || json.sseEnabled !== false) {
      throw new Error(`Unexpected health payload on ${base}: ${JSON.stringify(json)}`);
    }
  }

  // 2. Static HTML & Critical Asset Verification (CSS, JS, Three.js Vendor, 3D GLB)
  const assetPaths = [
    '/ella-world/',
    '/ella-world/css/styles.css',
    '/ella-world/js/app.js',
    '/ella-world/js/voice.js',
    '/ella-world/js/world3d.js',
    '/ella-world/vendor/three.module.js',
    '/ella-world/vendor/addons/loaders/GLTFLoader.js',
    '/ella-world/models/ella_dti_wardrobe.glb'
  ];
  for (const path of assetPaths) {
    const res = await fetch(`${args.baseUrl}${path}`, {
      method: path.endsWith('.glb') ? 'HEAD' : 'GET',
      headers: { 'X-Cache-Warmup': 'true' }
    });
    report.assets.push({ path, status: res.status });
    if (res.status !== 200) {
      throw new Error(`Asset check failed for ${args.baseUrl}${path} -> HTTP ${res.status}`);
    }
  }

  // 3. Optional Authenticated UDP Handshake & LAN vs WAN Firewall Probe
  if (args.username && args.password) {
    const loginRes = await fetch(`${args.baseUrl}/api/ella-world/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Cache-Warmup': 'true' },
      body: JSON.stringify({
        username: args.username,
        password: args.password,
        clientId: 'udpprobe'
      })
    });
    const loginJson = await loginRes.json();
    if (!loginRes.ok || !loginJson.token) {
      throw new Error(`Login probe failed: ${JSON.stringify(loginJson)}`);
    }

    const offerRes = await fetch(`${args.baseUrl}/api/ella-world/rtc/offer`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Cache-Warmup': 'true' },
      body: JSON.stringify({
        token: loginJson.token,
        sdp: VALID_MINIMAL_BROWSER_OFFER_SDP
      })
    });
    const offerJson = await offerRes.json();
    if (!offerRes.ok || !offerJson.sdp) {
      throw new Error(`RTC Offer probe failed: ${JSON.stringify(offerJson)}`);
    }

    const ufragMatch = offerJson.sdp.match(/a=ice-ufrag:([^\r\n]+)/);
    const pwdMatch = offerJson.sdp.match(/a=ice-pwd:([^\r\n]+)/);
    const serverUfrag = ufragMatch ? ufragMatch[1].trim() : '';
    const serverPwd = pwdMatch ? pwdMatch[1].trim() : '';
    const candidateLines = offerJson.sdp
      .split(/\r?\n/)
      .filter((l) => l.startsWith('a=candidate:'));
    report.udp.advertisedIceCandidates = candidateLines;

    const stunReq = buildMuxedStunBindingRequest(serverUfrag, serverPwd);
    const ewldHello = buildNativeEwldHello(loginJson.token);

    if (args.lanIp) {
      report.udp.lan_44444_webrtc = await probeUdpPacket(args.lanIp, args.webrtcPort, stunReq);
      report.udp.lan_44445_native = await probeUdpPacket(args.lanIp, args.nativePort, ewldHello);
    }
    if (args.wanIp) {
      report.udp.wan_44444_webrtc = await probeUdpPacket(args.wanIp, args.webrtcPort, stunReq);
      report.udp.wan_44445_native = await probeUdpPacket(args.wanIp, args.nativePort, ewldHello);
    }

    // Clean up probe player
    await fetch(`${args.baseUrl}/api/ella-world/logout`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Cache-Warmup': 'true' },
      body: JSON.stringify({ token: loginJson.token })
    }).catch(() => {});
  }

  console.log(JSON.stringify(report, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
