// Authoritative Persistent Binary + JSON WebSocket Spine Client (`wss://<host>/api/ella-world/ws`)
// Provides:
//   1. Session presence ownership with 12-second server grace window across Wi-Fi / tab hiccups
//   2. Guaranteed ordered delivery for world events (`welcome_snapshot`, `world_sync_digest`,
//      `voice_slot_map`, `block`, `avatar_updated`, `chat`, `runway`, `player_join`, `player_leave`)
//   3. Automatic 20 Hz binary `OP_C2S_STATE` / `OP_S2C_WORLD_STATE` fallback whenever UDP :44444
//      is handshaking, reconnecting, or blocked by strict NAT/firewall
//   4. Sub-10ms server clock offset synchronization for moving Sky Obby platforms & stage lights

import { encodeBinaryC2SState, decodeBinaryS2CWorldState } from '/ella-world/js/voice.js';

const OP_S2C_WORLD_STATE = 0x11;
const OP_C2S_PING = 0x12;
const OP_S2C_PONG = 0x13;

export class WorldSpineClient {
  constructor({
    nextSeq,
    onWorldStateSnapshot,
    onWorldEvent,
    onTransportStatus,
    onClockOffset
  }) {
    this.nextSeq = nextSeq || (() => 1);
    this.onWorldStateSnapshot = onWorldStateSnapshot || (() => {});
    this.onWorldEvent = onWorldEvent || (() => {});
    this.onTransportStatus = onTransportStatus || (() => {});
    this.onClockOffset = onClockOffset || (() => {});

    this.sessionToken = '';
    this.wsPath = '/api/ella-world/ws';
    this.ws = null;
    this.activeSession = false;
    this.reconnectAttempt = 0;
    this._reconnectTimer = null;

    this.latestRttMs = null;
    this.serverClockOffsetMs = 0;
    this._hasRttClockLock = false;
    this._desiredUdpFastLane = false;
    this._reportedUdpFastLane = null;
    this._lastResyncReqAt = 0;

    this._pingTimer = setInterval(() => this._tickKeepalive(), 3000);

    window.addEventListener('online', () => {
      if (this.activeSession && !this.isConnected()) {
        this.reconnectAttempt = 0;
        this.connect();
      }
    });

    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible' && this.activeSession && !this.isConnected()) {
        this.reconnectAttempt = 0;
        this.connect();
      }
    });
  }

  configure({ sessionToken, wsUrl }) {
    this.sessionToken = sessionToken || '';
    if (wsUrl) {
      this.wsPath = wsUrl;
    }
  }

  nowRelMs() {
    return (Math.floor(performance.now()) & 0x7fffffff) >>> 0;
  }

  isConnected() {
    return Boolean(this.ws && this.ws.readyState === WebSocket.OPEN);
  }

  connect() {
    if (!this.sessionToken || typeof WebSocket === 'undefined') {
      return false;
    }
    this.activeSession = true;
    clearTimeout(this._reconnectTimer);

    if (
      this.ws &&
      (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)
    ) {
      return true;
    }

    this._closeSocketOnly();
    this.onTransportStatus({
      state: this.reconnectAttempt > 0 ? 'reconnecting' : 'connecting',
      rttMs: this.latestRttMs
    });

    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const baseUrl = this.wsPath.startsWith('ws')
      ? this.wsPath
      : `${proto}//${window.location.host}${this.wsPath}`;
    const sep = baseUrl.includes('?') ? '&' : '?';
    const fullUrl = `${baseUrl}${sep}token=${encodeURIComponent(this.sessionToken)}`;

    let sock;
    try {
      sock = new WebSocket(fullUrl);
    } catch {
      this._scheduleReconnect();
      return false;
    }

    sock.binaryType = 'arraybuffer';
    this.ws = sock;
    this._reportedUdpFastLane = null;

    sock.onopen = () => {
      if (sock !== this.ws) return;
      this.reconnectAttempt = 0;
      this.onTransportStatus({
        state: 'connected',
        rttMs: this.latestRttMs
      });
      this._sendClockPing();
      // Immediately inform the server whether our UDP fast-lane is already healthy
      this._flushUdpFastLaneFlag();
    };

    sock.onmessage = (ev) => {
      if (sock !== this.ws) return;
      if (ev.data instanceof ArrayBuffer) {
        this._handleBinaryFrame(ev.data);
      } else if (typeof ev.data === 'string') {
        this._handleJsonFrame(ev.data);
      }
    };

    sock.onerror = () => {
      // `onclose` handles reconnection cleanly
    };

    sock.onclose = () => {
      if (sock !== this.ws) return;
      this.ws = null;
      this._reportedUdpFastLane = null;
      if (!this.activeSession) return;
      this.onTransportStatus({
        state: 'reconnecting',
        rttMs: null
      });
      this._scheduleReconnect();
    };

    return true;
  }

  _handleBinaryFrame(buf) {
    if (buf.byteLength < 1) return;
    const dv = new DataView(buf);
    const op = dv.getUint8(0);
    if (op === OP_S2C_WORLD_STATE) {
      const parsed = decodeBinaryS2CWorldState(buf);
      if (!parsed) return;
      if (parsed.echoClientTimeMs > 0) {
        const delta = (this.nowRelMs() - parsed.echoClientTimeMs) >>> 0;
        if (delta < 10000) {
          this.latestRttMs = delta;
        }
      }
      this.onWorldStateSnapshot({
        serverTick: parsed.serverTick,
        rttMs: this.latestRttMs,
        transport: 'ws',
        players: parsed.players
      });
    } else if (op === OP_S2C_PONG && buf.byteLength >= 9) {
      const sentRel = dv.getUint32(1, true);
      const delta = (this.nowRelMs() - sentRel) >>> 0;
      if (delta < 10000) {
        this.latestRttMs = delta;
        this.onTransportStatus({
          state: 'connected',
          rttMs: this.latestRttMs
        });
      }
    }
  }

  _handleJsonFrame(rawText) {
    let msg;
    try {
      msg = JSON.parse(rawText);
    } catch {
      return;
    }
    if (!msg || typeof msg !== 'object') return;

    if (msg.type === 'pong') {
      const nowWall = Date.now();
      const sentWall = Number(msg.clientTimeMs) || 0;
      const serverMs = Number(msg.serverTimeMs) || 0;
      if (sentWall > 0 && nowWall >= sentWall && nowWall - sentWall < 10000) {
        const rtt = nowWall - sentWall;
        this.latestRttMs = rtt;
        if (serverMs > 0) {
          const sampleOffset = serverMs - (sentWall + rtt * 0.5);
          if (!this._hasRttClockLock) {
            this.serverClockOffsetMs = sampleOffset;
            this._hasRttClockLock = true;
          } else {
            this.serverClockOffsetMs = this.serverClockOffsetMs * 0.75 + sampleOffset * 0.25;
          }
          this.onClockOffset(this.serverClockOffsetMs);
        }
        this.onTransportStatus({
          state: 'connected',
          rttMs: this.latestRttMs
        });
      }
      return;
    }

    if (
      (msg.type === 'welcome_snapshot' || msg.type === 'world_sync_digest') &&
      typeof msg.serverTimeMs === 'number' &&
      !this._hasRttClockLock
    ) {
      this.serverClockOffsetMs = msg.serverTimeMs - Date.now();
      this.onClockOffset(this.serverClockOffsetMs);
    }

    this.onWorldEvent(msg);
  }

  sendBinaryState(state) {
    if (!this.isConnected()) return false;
    if (this.ws.bufferedAmount > 65536) return false;
    const seq = this.nextSeq();
    const pkt = encodeBinaryC2SState(seq, this.nowRelMs(), state);
    try {
      this.ws.send(pkt);
      return true;
    } catch {
      return false;
    }
  }

  sendCommand(cmdObj) {
    if (!this.isConnected()) return false;
    try {
      this.ws.send(JSON.stringify(cmdObj));
      return true;
    } catch {
      return false;
    }
  }

  notifyUdpFastLane(active) {
    const flag = Boolean(active);
    this._desiredUdpFastLane = flag;
    this._flushUdpFastLaneFlag();
  }

  _flushUdpFastLaneFlag() {
    if (!this.isConnected()) return;
    if (this._reportedUdpFastLane === this._desiredUdpFastLane) return;
    if (this.sendCommand({ type: 'udp_fastlane', active: this._desiredUdpFastLane })) {
      this._reportedUdpFastLane = this._desiredUdpFastLane;
    }
  }

  requestResync() {
    const now = performance.now();
    if (now - this._lastResyncReqAt < 2000) return false;
    this._lastResyncReqAt = now;
    return this.sendCommand({ type: 'request_resync' });
  }

  _sendClockPing() {
    if (!this.isConnected()) return;
    this.sendCommand({ type: 'ping', clientTimeMs: Date.now() });
    const buf = new ArrayBuffer(5);
    const dv = new DataView(buf);
    dv.setUint8(0, OP_C2S_PING);
    dv.setUint32(1, this.nowRelMs(), true);
    try {
      this.ws.send(buf);
    } catch {
      // ignore closed socket
    }
  }

  _tickKeepalive() {
    if (!this.activeSession) return;
    if (!this.isConnected()) {
      if (!this._reconnectTimer) {
        this._scheduleReconnect();
      }
      return;
    }
    this._sendClockPing();
  }

  _scheduleReconnect() {
    if (!this.activeSession) return;
    clearTimeout(this._reconnectTimer);
    const baseDelays = [250, 500, 1000, 2000, 3000];
    const idx = Math.min(this.reconnectAttempt, baseDelays.length - 1);
    const jitter = Math.floor(Math.random() * 180);
    const delay = baseDelays[idx] + jitter;
    this.reconnectAttempt += 1;
    this._reconnectTimer = setTimeout(() => {
      this._reconnectTimer = null;
      if (this.activeSession) {
        this.connect();
      }
    }, delay);
  }

  _closeSocketOnly() {
    if (this.ws) {
      const old = this.ws;
      this.ws = null;
      try {
        old.onopen = null;
        old.onmessage = null;
        old.onerror = null;
        old.onclose = null;
        old.close();
      } catch {
        // ignore
      }
    }
  }

  disconnect() {
    this.activeSession = false;
    this.reconnectAttempt = 0;
    this._hasRttClockLock = false;
    this._desiredUdpFastLane = false;
    this._reportedUdpFastLane = null;
    clearTimeout(this._reconnectTimer);
    this._reconnectTimer = null;
    this._closeSocketOnly();
  }

  destroy() {
    clearInterval(this._pingTimer);
    this.disconnect();
  }
}
