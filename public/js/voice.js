// Zero-Renegotiation Single-Port UDP Fast-Lane + 8-Slot Opus SFU Voice Client
// Connects to the standalone Rust `ella-world-server` on multiplexed UDP port (:44444/udp):
//   1. Voice Audio: 1 upstream Opus mic transceiver + 8 pre-allocated downlink speaker slots
//      (`ella_slot_0` .. `ella_slot_7`) negotiated ONCE at session start (zero mid-game SDP renegotiation!)
//   2. Real-Time Movement/Poses/Voice Flags: 28-byte binary datagrams over `ella-udp-state`
//      (`ordered: false, maxRetransmits: 0` SCTP/DTLS over UDP — zero head-of-line blocking)
//   3. Automatic Failover: exposes `isUdpStateHealthy()` verified by live inbound UDP pongs/ticks
//      so `WorldSpineClient` (`wss://`) seamlessly takes over 20 Hz state sync whenever UDP stalls.

const OP_C2S_STATE = 0x10;
const OP_S2C_WORLD_STATE = 0x11;
const OP_C2S_PING = 0x12;
const OP_S2C_PONG = 0x13;

const DEFAULT_DOWNLINK_VOICE_SLOTS = 8;

const ANIM_TO_CODE = { idle: 0, walk: 1, run: 2, jump: 3 };
const CODE_TO_ANIM = ['idle', 'walk', 'run', 'jump'];

const EMOTE_TO_CODE = {
  pose: 1,
  peace: 2,
  dance: 3,
  spin: 4,
  wave: 5,
  cheer: 6,
  flip: 7,
  sit: 8
};
const CODE_TO_EMOTE = [null, 'pose', 'peace', 'dance', 'spin', 'wave', 'cheer', 'flip', 'sit'];

const utf8Decoder = new TextDecoder();

export const ZONE_PLAZA = 0;
export const ZONE_DUNGEON = 1;
export const ZONE_PRISON = 2;
export const ZONE_ARENA = 3;

export function encodeBinaryC2SState(seq, clientTimeMs, state) {
  const buf = new ArrayBuffer(28);
  const dv = new DataView(buf);
  dv.setUint8(0, OP_C2S_STATE);
  dv.setUint32(1, seq >>> 0, true);
  dv.setUint32(5, clientTimeMs >>> 0, true);
  dv.setFloat32(9, Number(state.x) || 0, true);
  dv.setFloat32(13, Number(state.y) || 0, true);
  dv.setFloat32(17, Number(state.z) || 0, true);
  dv.setFloat32(21, Number(state.ry) || 0, true);
  dv.setUint8(25, ANIM_TO_CODE[state.anim] || 0);
  dv.setUint8(26, state.emote ? EMOTE_TO_CODE[state.emote] || 0 : 0);
  const zoneBits = ((Number(state.zone) || 0) & 0x3f) << 2;
  const flags = (state.muted ? 1 : 0) | (state.speaking ? 2 : 0) | zoneBits;
  dv.setUint8(27, flags);
  return buf;
}

export function decodeBinaryS2CWorldState(arrayBuffer) {
  if (!(arrayBuffer instanceof ArrayBuffer) || arrayBuffer.byteLength < 11) {
    return null;
  }
  const dv = new DataView(arrayBuffer);
  if (dv.getUint8(0) !== OP_S2C_WORLD_STATE) {
    return null;
  }
  const serverTick = dv.getUint32(1, true);
  const echoClientTimeMs = dv.getUint32(5, true);
  const count = dv.getUint16(9, true);

  let offset = 11;
  const players = [];
  for (let i = 0; i < count; i++) {
    if (offset >= arrayBuffer.byteLength) return null;
    const idLen = dv.getUint8(offset);
    offset += 1;
    if (offset + idLen + 19 > arrayBuffer.byteLength) return null;
    const id = utf8Decoder.decode(new Uint8Array(arrayBuffer, offset, idLen));
    offset += idLen;

    const x = dv.getFloat32(offset, true);
    const y = dv.getFloat32(offset + 4, true);
    const z = dv.getFloat32(offset + 8, true);
    const ry = dv.getFloat32(offset + 12, true);
    const anim = CODE_TO_ANIM[dv.getUint8(offset + 16)] || 'idle';
    const emote = CODE_TO_EMOTE[dv.getUint8(offset + 17)] || null;
    const flags = dv.getUint8(offset + 18);
    offset += 19;

    players.push({
      id,
      x,
      y,
      z,
      ry,
      anim,
      emote,
      muted: (flags & 1) !== 0,
      speaking: (flags & 2) !== 0,
      zone: (flags >>> 2) & 0x3f
    });
  }

  return { serverTick, echoClientTimeMs, players };
}

export class VoiceChatManager {
  constructor({
    nextSeq,
    onLocalVoiceState,
    onPeerVoiceState,
    onWorldStateSnapshot,
    onWorldEvent,
    onUdpTransportStatus,
    onStatusChange,
    onAuthRejected
  }) {
    this.outboundSeq = 0;
    this.nextSeq =
      nextSeq ||
      (() => {
        this.outboundSeq = (this.outboundSeq + 1) >>> 0;
        return this.outboundSeq;
      });
    this.onLocalVoiceState = onLocalVoiceState || (() => {});
    this.onPeerVoiceState = onPeerVoiceState || (() => {});
    this.onWorldStateSnapshot = onWorldStateSnapshot || (() => {});
    this.onWorldEvent = onWorldEvent || (() => {});
    this.onUdpTransportStatus = onUdpTransportStatus || (() => {});
    this.onStatusChange = onStatusChange || (() => {});
    this.onAuthRejected = onAuthRejected || (() => {});

    this.selfId = null;
    this.sessionToken = '';
    this.rtcOfferUrl = '/api/ella-world/rtc/offer';
    this.iceServers = [{ urls: ['stun:stun.l.google.com:19302'] }];
    this.voiceSlotCount = DEFAULT_DOWNLINK_VOICE_SLOTS;

    this.localStream = null;
    this.audioCtx = null;
    this.localAnalyser = null;
    this.selectedDeviceId = '';

    this.isMuted = false;
    this.serverForceMuted = false;
    this.pushToTalk = false;
    this.pttKeyDown = false;
    this.proximityMode = false;
    this.localSpeaking = false;
    this.localLevel = 0;

    // Single UDP WebRTC connection to Rust SFU + State Engine
    this.serverPc = null;
    this.audioSender = null;
    this.udpStateChannel = null;
    this.reliableChannel = null;
    this.latestRttMs = null;
    this.lastUdpRxAt = 0;
    this.udpPort = 44444;
    this.activeSession = false;
    this._reconnectTimer = null;
    this._connectingPromise = null;

    // peerId -> { peerId, userVolume, proximityGain, proximityPan, speaking }
    this.remoteVoicePeers = new Map();
    // slotId ("ella_slot_0".."ella_slot_7") -> { slotId, stream, audioEl, sourceNode, analyser, assignedPeerId, speaking }
    this.slotStreams = new Map();
    // Latest server-authoritative `{ ella_slot_0: "peer_...", ... }` mapping
    this.latestSlotAssignments = {};

    this._vadTimer = setInterval(() => this._tickVad(), 85);
    this._pingTimer = setInterval(() => this._tickUdpPingAndStats(), 1200);
  }

  configure({
    selfId,
    sessionToken,
    rtcOfferUrl,
    iceServers,
    webrtcUdpPort,
    voiceSlotCount,
    defaultMode
  }) {
    this.selfId = selfId;
    this.sessionToken = sessionToken || '';
    this.rtcOfferUrl = rtcOfferUrl || '/api/ella-world/rtc/offer';
    if (Array.isArray(iceServers) && iceServers.length > 0) {
      this.iceServers = iceServers;
    }
    if (webrtcUdpPort) {
      this.udpPort = Number(webrtcUdpPort) || 44444;
    }
    if (voiceSlotCount && Number(voiceSlotCount) > 0) {
      this.voiceSlotCount = Math.min(16, Math.max(2, Number(voiceSlotCount)));
    }
    this.proximityMode = defaultMode === 'proximity';
  }

  nowRelMs() {
    return (Math.floor(performance.now()) & 0x7fffffff) >>> 0;
  }

  async initLocalMicrophone(deviceId = '') {
    try {
      if (!this.audioCtx) {
        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        if (AudioContextClass) {
          this.audioCtx = new AudioContextClass();
        }
      }
      if (this.audioCtx && this.audioCtx.state === 'suspended') {
        await this.audioCtx.resume().catch(() => {});
      }

      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        this.onStatusChange({
          ok: false,
          message: 'Voice requires HTTPS or localhost for browser microphone access.'
        });
        return false;
      }

      const audioConstraints = {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true
      };
      if (deviceId) {
        audioConstraints.deviceId = { exact: deviceId };
      }

      const newStream = await navigator.mediaDevices.getUserMedia({
        audio: audioConstraints,
        video: false
      });

      const oldStream = this.localStream;
      this.localStream = newStream;
      this.selectedDeviceId = deviceId;

      this._applyMicTrackEnabledState();
      this._setupLocalAnalyser(newStream);

      const newTrack = newStream.getAudioTracks()[0];
      if (newTrack && this.audioSender) {
        await this.audioSender.replaceTrack(newTrack).catch(() => {});
      }

      if (oldStream) {
        for (const t of oldStream.getTracks()) t.stop();
      }

      // Ensure WebAudio analysers for any already-open remote speaker slots are attached
      for (const slotObj of this.slotStreams.values()) {
        this._ensureSlotAnalyser(slotObj);
      }

      this.onStatusChange({
        ok: true,
        message: 'UDP Opus voice active! Green ring glows when you or friends talk.'
      });
      return true;
    } catch (err) {
      this.onStatusChange({
        ok: false,
        message: `Mic not active (${err.name || 'permission needed'}). Hearing friends over UDP!`
      });
      return false;
    }
  }

  async connectServerUdp() {
    if (this._connectingPromise) {
      return this._connectingPromise;
    }
    this._connectingPromise = this._connectServerUdpInner().finally(() => {
      this._connectingPromise = null;
    });
    return this._connectingPromise;
  }

  async _connectServerUdpInner() {
    this.activeSession = true;
    clearTimeout(this._reconnectTimer);
    this._closeServerPcOnly();

    if (typeof RTCPeerConnection === 'undefined' || !this.sessionToken) {
      this.onUdpTransportStatus({
        state: 'unavailable',
        rttMs: null,
        port: this.udpPort,
        protocol: 'udp'
      });
      return false;
    }

    this.onUdpTransportStatus({
      state: 'connecting',
      rttMs: null,
      port: this.udpPort,
      protocol: 'udp'
    });

    try {
      const pc = new RTCPeerConnection({ iceServers: this.iceServers });
      this.serverPc = pc;
      let recvSlotIdx = 0;

      // 1. Upstream Opus Voice RTP transceiver over UDP (:44444/udp)
      const micTrack = this.localStream ? this.localStream.getAudioTracks()[0] : null;
      if (micTrack) {
        const tx = pc.addTransceiver(micTrack, {
          direction: 'sendonly',
          streams: [this.localStream]
        });
        this.audioSender = tx.sender;
      } else {
        const tx = pc.addTransceiver('audio', { direction: 'sendonly' });
        this.audioSender = tx.sender;
      }

      // 2. Pre-allocate 8 downlink Opus speaker slots ONCE (`ella_slot_0`..`ella_slot_7`)
      //    so players joining/leaving never trigger mid-game SDP renegotiation or ICE resets!
      for (let i = 0; i < this.voiceSlotCount; i++) {
        pc.addTransceiver('audio', { direction: 'recvonly' });
      }

      // 3. Unreliable, unordered binary UDP game-state channel (`0x10` / `0x11` at 20 Hz)
      const stateDc = pc.createDataChannel('ella-udp-state', {
        ordered: false,
        maxRetransmits: 0
      });
      stateDc.binaryType = 'arraybuffer';
      stateDc.onopen = () => {
        this._sendUdpPing();
      };
      stateDc.onmessage = (ev) => {
        if (!(ev.data instanceof ArrayBuffer) || ev.data.byteLength < 1) return;
        const dv = new DataView(ev.data);
        const op = dv.getUint8(0);
        if (op === OP_S2C_WORLD_STATE) {
          const parsed = decodeBinaryS2CWorldState(ev.data);
          if (!parsed) return;
          this.lastUdpRxAt = performance.now();
          if (parsed.echoClientTimeMs > 0) {
            const delta = (this.nowRelMs() - parsed.echoClientTimeMs) >>> 0;
            if (delta < 10000) {
              this.latestRttMs = delta;
            }
          }
          this.onWorldStateSnapshot({
            serverTick: parsed.serverTick,
            rttMs: this.latestRttMs,
            transport: 'udp',
            players: parsed.players
          });
        } else if (op === OP_S2C_PONG && ev.data.byteLength >= 9) {
          this.lastUdpRxAt = performance.now();
          const sentAt = dv.getUint32(1, true);
          const delta = (this.nowRelMs() - sentAt) >>> 0;
          if (delta < 10000) {
            this.latestRttMs = delta;
            this.onUdpTransportStatus({
              state: 'connected',
              rttMs: this.latestRttMs,
              port: this.udpPort,
              protocol: 'udp'
            });
          }
        }
      };
      this.udpStateChannel = stateDc;

      // 4. Ordered UDP DataChannel for `voice_slot_map` & supplementary low-latency events
      const eventsDc = pc.createDataChannel('ella-reliable-events', {
        ordered: true
      });
      eventsDc.onmessage = (ev) => {
        if (typeof ev.data !== 'string') return;
        let msg;
        try {
          msg = JSON.parse(ev.data);
        } catch {
          return;
        }
        if (!msg || typeof msg !== 'object') return;
        if (msg.type === 'session_revoked') {
          const reason = msg.reason || 'Your session has expired. Please sign in again.';
          this.disconnectSession();
          this.onAuthRejected(reason);
          return;
        }
        if (msg.type === 'voice_slot_map' && msg.slots) {
          this.applyVoiceSlotMap(msg.slots);
          return;
        }
        this.onWorldEvent(msg);
      };
      this.reliableChannel = eventsDc;

      // 5. Bind each pre-allocated SFU downlink Opus speaker slot (`ella_slot_0`..`ella_slot_7`)
      pc.ontrack = (event) => {
        const stream =
          event.streams && event.streams[0]
            ? event.streams[0]
            : new MediaStream([event.track]);
        let slotId = '';
        if (stream.id && stream.id.startsWith('ella_slot_')) {
          slotId = stream.id;
        } else if (event.track.id && event.track.id.startsWith('voice_ella_slot_')) {
          slotId = event.track.id.slice('voice_'.length);
        } else {
          slotId = `ella_slot_${recvSlotIdx}`;
        }
        recvSlotIdx += 1;
        this._attachSpeakerSlotStream(slotId, stream);
      };

      pc.onconnectionstatechange = () => {
        if (pc !== this.serverPc) return;
        const st = pc.connectionState;
        if (st === 'connected') {
          this._sendUdpPing();
          this.onUdpTransportStatus({
            state: 'connected',
            rttMs: this.latestRttMs,
            port: this.udpPort,
            protocol: 'udp'
          });
        } else if (st === 'failed' || st === 'closed') {
          this.lastUdpRxAt = 0;
          this.onUdpTransportStatus({
            state: 'reconnecting',
            rttMs: null,
            port: this.udpPort,
            protocol: 'udp'
          });
          this._scheduleUdpReconnect(2200);
        } else if (st === 'disconnected') {
          // Give WebRTC ICE 3.5 seconds to self-recover before tearing down the PeerConnection
          clearTimeout(this._reconnectTimer);
          this._reconnectTimer = setTimeout(() => {
            if (
              this.activeSession &&
              pc === this.serverPc &&
              (pc.connectionState === 'disconnected' || pc.connectionState === 'failed')
            ) {
              this.lastUdpRxAt = 0;
              this.connectServerUdp();
            }
          }, 3500);
        }
      };

      // 6. Gather ICE candidates and complete single-shot SDP offer/answer with Rust SFU
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      await this._waitForIceGathering(pc, 900);

      const res = await fetch(this.rtcOfferUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Cache-Warmup': 'true'
        },
        body: JSON.stringify({
          token: this.sessionToken,
          sdp: pc.localDescription ? pc.localDescription.sdp : offer.sdp
        })
      });

      if (res.status === 401) {
        let errMsg = 'Your session has expired. Please sign in again.';
        try {
          const errJson = await res.json();
          if (errJson && errJson.error) errMsg = errJson.error;
        } catch {
          // ignore parse failure
        }
        this.disconnectSession();
        this.onAuthRejected(errMsg);
        return false;
      }
      if (!res.ok) {
        throw new Error(`RTC offer HTTP ${res.status}`);
      }
      const answerData = await res.json();
      if (answerData.udpPort) {
        this.udpPort = Number(answerData.udpPort) || this.udpPort;
      }
      if (pc !== this.serverPc) return false;
      await pc.setRemoteDescription({
        type: 'answer',
        sdp: answerData.sdp
      });
      return true;
    } catch {
      this.lastUdpRxAt = 0;
      this.onUdpTransportStatus({
        state: 'reconnecting',
        rttMs: null,
        port: this.udpPort,
        protocol: 'udp'
      });
      this._scheduleUdpReconnect(2500);
      return false;
    }
  }

  _scheduleUdpReconnect(delayMs = 2200) {
    if (!this.activeSession) return;
    clearTimeout(this._reconnectTimer);
    this._reconnectTimer = setTimeout(() => {
      this._reconnectTimer = null;
      if (this.activeSession) {
        this.connectServerUdp();
      }
    }, delayMs);
  }

  _waitForIceGathering(pc, timeoutMs = 900) {
    if (pc.iceGatheringState === 'complete') {
      return Promise.resolve();
    }
    return new Promise((resolve) => {
      const done = () => {
        clearTimeout(timer);
        pc.removeEventListener('icegatheringstatechange', onState);
        resolve();
      };
      const onState = () => {
        if (pc.iceGatheringState === 'complete') done();
      };
      const timer = setTimeout(done, timeoutMs);
      pc.addEventListener('icegatheringstatechange', onState);
    });
  }

  isUdpConnected() {
    return Boolean(this.udpStateChannel && this.udpStateChannel.readyState === 'open');
  }

  isUdpStateHealthy() {
    return Boolean(
      this.udpStateChannel &&
      this.udpStateChannel.readyState === 'open' &&
      this.lastUdpRxAt > 0 &&
      performance.now() - this.lastUdpRxAt < 2200
    );
  }

  sendBinaryState(state) {
    if (!this.udpStateChannel || this.udpStateChannel.readyState !== 'open') {
      return false;
    }
    if (this.udpStateChannel.bufferedAmount > 65536) {
      return false;
    }
    const seq = this.nextSeq();
    const pkt = encodeBinaryC2SState(seq, this.nowRelMs(), state);
    try {
      this.udpStateChannel.send(pkt);
      return true;
    } catch {
      return false;
    }
  }

  sendReliableCommand(cmdObj) {
    if (!this.reliableChannel || this.reliableChannel.readyState !== 'open') {
      return false;
    }
    try {
      this.reliableChannel.send(JSON.stringify(cmdObj));
      return true;
    } catch {
      return false;
    }
  }

  _sendUdpPing() {
    if (!this.udpStateChannel || this.udpStateChannel.readyState !== 'open') return;
    const buf = new ArrayBuffer(5);
    const dv = new DataView(buf);
    dv.setUint8(0, OP_C2S_PING);
    dv.setUint32(1, this.nowRelMs(), true);
    try {
      this.udpStateChannel.send(buf);
    } catch {
      // ignore closed channel
    }
  }

  async _tickUdpPingAndStats() {
    if (!this.activeSession || !this.serverPc) return;
    this._sendUdpPing();
    if (typeof this.serverPc.getStats !== 'function') return;
    try {
      const stats = await this.serverPc.getStats();
      let activePair = null;
      stats.forEach((report) => {
        if (report.type === 'candidate-pair' && report.state === 'succeeded' && report.nominated) {
          activePair = report;
        }
      });
      if (activePair) {
        const remoteCand = stats.get(activePair.remoteCandidateId);
        const rttFromIce =
          typeof activePair.currentRoundTripTime === 'number'
            ? Math.round(activePair.currentRoundTripTime * 1000)
            : this.latestRttMs;
        if (typeof rttFromIce === 'number') {
          this.latestRttMs = rttFromIce;
        }
        this.onUdpTransportStatus({
          state: this.isUdpStateHealthy() ? 'connected' : 'connecting',
          rttMs: this.latestRttMs,
          port: (remoteCand && remoteCand.port) || this.udpPort,
          protocol: (remoteCand && remoteCand.protocol) || 'udp',
          candidateType: remoteCand ? remoteCand.candidateType : undefined
        });
      }
    } catch {
      // ignore stats error
    }
  }

  _setupLocalAnalyser(stream) {
    if (!this.audioCtx) return;
    try {
      const source = this.audioCtx.createMediaStreamSource(stream);
      const analyser = this.audioCtx.createAnalyser();
      analyser.fftSize = 256;
      analyser.smoothingTimeConstant = 0.5;
      source.connect(analyser);
      this.localAnalyser = analyser;
    } catch {
      // ignore analyser errors
    }
  }

  _attachSpeakerSlotStream(slotId, stream) {
    let slotObj = this.slotStreams.get(slotId);
    if (!slotObj) {
      slotObj = {
        slotId,
        stream,
        audioEl: null,
        sourceNode: null,
        analyser: null,
        assignedPeerId: null,
        speaking: false
      };
      this.slotStreams.set(slotId, slotObj);
    } else {
      slotObj.stream = stream;
    }

    const container = document.getElementById('remote-audio-container');
    if (!slotObj.audioEl && container) {
      const audio = document.createElement('audio');
      audio.autoplay = true;
      audio.playsInline = true;
      audio.muted = false;
      audio.dataset.voiceSlot = slotId;
      audio.srcObject = stream;
      container.appendChild(audio);
      slotObj.audioEl = audio;
      audio.play().catch(() => {});
    } else if (slotObj.audioEl) {
      slotObj.audioEl.srcObject = stream;
      slotObj.audioEl.muted = false;
      slotObj.audioEl.play().catch(() => {});
    }

    this._ensureSlotAnalyser(slotObj);
    this._reconcileSlotBindings();
  }

  _ensureSlotAnalyser(slotObj) {
    if (!this.audioCtx || !slotObj || !slotObj.stream) return;
    if (this.audioCtx.state === 'suspended') {
      this.audioCtx.resume().catch(() => {});
    }
    try {
      if (slotObj.sourceNode) slotObj.sourceNode.disconnect();
      const source = this.audioCtx.createMediaStreamSource(slotObj.stream);
      const analyser = this.audioCtx.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);
      slotObj.sourceNode = source;
      slotObj.analyser = analyser;
    } catch {
      // Native <audio> element still plays even if WebAudio graph fails
    }
  }

  applyVoiceSlotMap(slotsObj) {
    if (!slotsObj || typeof slotsObj !== 'object') return;
    this.latestSlotAssignments = { ...slotsObj };
    this._reconcileSlotBindings();
  }

  _reconcileSlotBindings() {
    for (const [slotId, slotObj] of this.slotStreams.entries()) {
      const rawTarget = this.latestSlotAssignments[slotId];
      const nextPeerId = rawTarget && rawTarget !== this.selfId ? String(rawTarget) : null;

      if (slotObj.assignedPeerId !== nextPeerId) {
        const prevPeerId = slotObj.assignedPeerId;
        if (prevPeerId && slotObj.speaking) {
          this.onPeerVoiceState(prevPeerId, { speaking: false });
        }
        slotObj.assignedPeerId = nextPeerId;
        slotObj.speaking = false;
        if (nextPeerId) {
          this.ensurePeer(nextPeerId);
        }
      }
      this._syncSlotVolume(slotObj);
    }
  }

  _syncSlotVolume(slotObj) {
    if (!slotObj || !slotObj.audioEl) return;
    if (!slotObj.assignedPeerId) {
      slotObj.audioEl.volume = 0;
      return;
    }
    const peerObj = this.ensurePeer(slotObj.assignedPeerId);
    const gain = peerObj ? Math.max(0, Math.min(1, peerObj.userVolume * peerObj.proximityGain)) : 1.0;
    slotObj.audioEl.volume = gain;
  }

  _syncAllSlotVolumes() {
    for (const slotObj of this.slotStreams.values()) {
      this._syncSlotVolume(slotObj);
    }
  }

  ensurePeer(peerId) {
    if (!peerId || peerId === this.selfId) return null;
    if (!this.remoteVoicePeers.has(peerId)) {
      this.remoteVoicePeers.set(peerId, {
        peerId,
        userVolume: 1.0,
        proximityGain: 1.0,
        proximityPan: 0.0,
        speaking: false
      });
    }
    return this.remoteVoicePeers.get(peerId);
  }

  removePeer(peerId) {
    if (!peerId) return;
    for (const [slotId, assigned] of Object.entries(this.latestSlotAssignments)) {
      if (assigned === peerId) {
        delete this.latestSlotAssignments[slotId];
      }
    }
    this._reconcileSlotBindings();
    this.remoteVoicePeers.delete(peerId);
  }

  async getAudioInputDevices() {
    if (!navigator.mediaDevices || !navigator.mediaDevices.enumerateDevices) return [];
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      return devices.filter((d) => d.kind === 'audioinput');
    } catch {
      return [];
    }
  }

  setMuted(muted) {
    this.isMuted = Boolean(muted);
    this._applyMicTrackEnabledState();
    this._notifyLocalState();
  }

  setServerForceMuted(forceMuted) {
    const next = Boolean(forceMuted);
    if (this.serverForceMuted === next) return;
    this.serverForceMuted = next;
    this._applyMicTrackEnabledState();
    this._notifyLocalState();
  }

  toggleMute() {
    if (this.serverForceMuted) {
      return true;
    }
    if (this.pushToTalk) {
      this.pushToTalk = false;
    }
    this.setMuted(!this.isMuted);
    return this.isMuted;
  }

  setPushToTalk(enabled) {
    this.pushToTalk = Boolean(enabled);
    this.pttKeyDown = false;
    this._applyMicTrackEnabledState();
    this._notifyLocalState();
  }

  setPttKeyActive(down) {
    if (this.serverForceMuted || !this.pushToTalk) return;
    if (this.pttKeyDown === down) return;
    this.pttKeyDown = Boolean(down);
    this._applyMicTrackEnabledState();
    this._notifyLocalState();
  }

  setProximityMode(enabled) {
    this.proximityMode = Boolean(enabled);
    this._syncAllSlotVolumes();
  }

  isEffectivelyMuted() {
    if (this.serverForceMuted) {
      return true;
    }
    if (this.pushToTalk) {
      return !this.pttKeyDown;
    }
    return this.isMuted;
  }

  _applyMicTrackEnabledState() {
    const enabled = !this.isEffectivelyMuted();
    if (this.localStream) {
      for (const track of this.localStream.getAudioTracks()) {
        track.enabled = enabled;
      }
    }
    if (!enabled) {
      this.localSpeaking = false;
      this.localLevel = 0;
    }
  }

  _notifyLocalState() {
    this.onLocalVoiceState({
      muted: this.isEffectivelyMuted(),
      speaking: this.localSpeaking,
      level: this.localLevel
    });
  }

  setPeerUserVolume(peerId, volumePercent) {
    const peerObj = this.ensurePeer(peerId);
    if (!peerObj) return;
    peerObj.userVolume = Math.max(0, Math.min(1, Number(volumePercent) / 100));
    this._syncAllSlotVolumes();
  }

  updateSpatialPositions(selfPos, cameraYaw, remotePlayersMap, selfZone = 0) {
    for (const [peerId, peerObj] of this.remoteVoicePeers.entries()) {
      if (!this.proximityMode) {
        peerObj.proximityGain = 1.0;
        peerObj.proximityPan = 0.0;
      } else {
        const remote = remotePlayersMap.get(peerId);
        if (!remote) {
          peerObj.proximityGain = 1.0;
          peerObj.proximityPan = 0.0;
        } else if ((remote.zone || 0) !== selfZone) {
          peerObj.proximityGain = 0.04;
          peerObj.proximityPan = 0.0;
        } else {
          const dx = remote.x - selfPos.x;
          const dy = remote.y - selfPos.y;
          const dz = remote.z - selfPos.z;
          const dist = Math.hypot(dx, dy, dz);

          const fullRadius = 6.0;
          const maxRadius = 35.0;
          if (dist <= fullRadius) {
            peerObj.proximityGain = 1.0;
          } else if (dist >= maxRadius) {
            peerObj.proximityGain = 0.04;
          } else {
            const t = (dist - fullRadius) / (maxRadius - fullRadius);
            peerObj.proximityGain = Math.max(0.04, Math.pow(1 - t, 1.65));
          }

          const angleToPeer = Math.atan2(dx, dz);
          const relAngle = angleToPeer - cameraYaw;
          peerObj.proximityPan = Math.max(-0.85, Math.min(0.85, -Math.sin(relAngle) * 0.75));
        }
      }
    }
    this._syncAllSlotVolumes();
  }

  _computeAnalyserLevel(analyser) {
    if (!analyser) return 0;
    const bins = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteFrequencyData(bins);
    let sum = 0;
    for (let i = 0; i < bins.length; i++) {
      sum += bins[i];
    }
    return sum / (bins.length * 255);
  }

  _tickVad() {
    if (this.localAnalyser && !this.isEffectivelyMuted()) {
      const level = this._computeAnalyserLevel(this.localAnalyser);
      this.localLevel = level;
      const nowSpeaking = level > 0.045;
      if (nowSpeaking !== this.localSpeaking) {
        this.localSpeaking = nowSpeaking;
        this._notifyLocalState();
      } else {
        this.onLocalVoiceState({
          muted: false,
          speaking: this.localSpeaking,
          level: this.localLevel,
          silentTick: true
        });
      }
    } else if (this.localSpeaking || this.localLevel > 0) {
      this.localSpeaking = false;
      this.localLevel = 0;
      this._notifyLocalState();
    }

    for (const slotObj of this.slotStreams.values()) {
      if (slotObj.analyser && slotObj.assignedPeerId) {
        const level = this._computeAnalyserLevel(slotObj.analyser);
        const nowSpeaking = level > 0.04;
        if (nowSpeaking !== slotObj.speaking) {
          slotObj.speaking = nowSpeaking;
          this.onPeerVoiceState(slotObj.assignedPeerId, { speaking: nowSpeaking });
        }
      }
    }
  }

  _clearSlotStreams() {
    for (const slotObj of this.slotStreams.values()) {
      try {
        if (slotObj.audioEl) slotObj.audioEl.remove();
        if (slotObj.sourceNode) slotObj.sourceNode.disconnect();
      } catch {
        // ignore
      }
    }
    this.slotStreams.clear();
  }

  _closeServerPcOnly() {
    this.lastUdpRxAt = 0;
    this._clearSlotStreams();
    try {
      if (this.udpStateChannel) this.udpStateChannel.close();
      if (this.reliableChannel) this.reliableChannel.close();
      if (this.serverPc) this.serverPc.close();
    } catch {
      // ignore
    }
    this.udpStateChannel = null;
    this.reliableChannel = null;
    this.serverPc = null;
    this.audioSender = null;
  }

  disconnectSession() {
    this.activeSession = false;
    clearTimeout(this._reconnectTimer);
    this._reconnectTimer = null;
    this.latestSlotAssignments = {};
    this._closeServerPcOnly();
    this.remoteVoicePeers.clear();
  }

  destroy() {
    clearInterval(this._vadTimer);
    clearInterval(this._pingTimer);
    this.disconnectSession();
    if (this.localStream) {
      for (const t of this.localStream.getTracks()) t.stop();
    }
  }
}
