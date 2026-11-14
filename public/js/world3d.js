import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import * as SkeletonUtils from 'three/addons/utils/SkeletonUtils.js';

export const BLOCK_COLORS = [
  { name: 'Bubblegum Pink', hex: '#f472b6' },
  { name: 'Sky Blue', hex: '#38bdf8' },
  { name: 'Mint Emerald', hex: '#34d399' },
  { name: 'Sunshine Gold', hex: '#fbbf24' },
  { name: 'Lavender', hex: '#a78bfa' },
  { name: 'Coral Red', hex: '#fb7185' },
  { name: 'Orange Pop', hex: '#fb923c' },
  { name: 'Lime Glow', hex: '#a3e635' },
  { name: 'Snow White', hex: '#f8fafc' },
  { name: 'Warm Wood', hex: '#b45309' },
  { name: 'Royal Indigo', hex: '#6366f1' },
  { name: 'Obsidian Slate', hex: '#334155' }
];

// Shared singleton loader for the Blender 5.2 Ella Forensics + Dress-to-Impress Wardrobe Pack
let dtiPackPromise = null;
let dtiPackAsset = null;

export function loadDtiWardrobePack() {
  if (dtiPackAsset) return Promise.resolve(dtiPackAsset);
  if (!dtiPackPromise) {
    const loader = new GLTFLoader();
    dtiPackPromise = new Promise((resolve, reject) => {
      loader.load(
        '/ella-world/models/ella_dti_wardrobe.glb',
        (gltf) => {
          dtiPackAsset = gltf;
          resolve(gltf);
        },
        undefined,
        (err) => {
          console.warn('[EllaWorld] Could not load ella_dti_wardrobe.glb, using fallback rig:', err);
          reject(err);
        }
      );
    });
  }
  return dtiPackPromise;
}

const EMOTE_TO_CLIP = {
  wave: 'Emote_Wave',
  dance: 'Emote_Dance',
  spin: 'Emote_Spin',
  cheer: 'Emote_Cheer',
  pose: 'Emote_Pose',
  peace: 'Emote_Peace',
  sit: 'Emote_Sit',
  flip: 'Emote_Flip'
};

const ANIM_TO_CLIP = {
  idle: 'Idle_Glam',
  walk: 'Walk_Runway',
  run: 'Run_Obby',
  jump: 'Jump_Air'
};

function createFaceTexture(faceType = 'happy', skinColor = '#ffdfc4') {
  const canvas = document.createElement('canvas');
  canvas.width = 256;
  canvas.height = 256;
  const ctx = canvas.getContext('2d');

  ctx.fillStyle = skinColor;
  ctx.fillRect(0, 0, 256, 256);

  ctx.fillStyle = 'rgba(244, 114, 182, 0.38)';
  ctx.beginPath();
  ctx.arc(58, 152, 20, 0, Math.PI * 2);
  ctx.arc(198, 152, 20, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = '#0f172a';
  ctx.strokeStyle = '#0f172a';
  ctx.lineWidth = 10;
  ctx.lineCap = 'round';

  if (faceType === 'cool') {
    ctx.fillStyle = '#0f172a';
    ctx.fillRect(36, 84, 78, 44);
    ctx.fillRect(142, 84, 78, 44);
    ctx.fillRect(24, 92, 208, 10);
    ctx.fillStyle = '#38bdf8';
    ctx.fillRect(48, 92, 16, 26);
    ctx.fillRect(154, 92, 16, 26);
    ctx.beginPath();
    ctx.arc(128, 164, 34, 0.15 * Math.PI, 0.85 * Math.PI);
    ctx.stroke();
  } else if (faceType === 'star') {
    const drawStar = (cx, cy, r) => {
      ctx.fillStyle = '#0f172a';
      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#fde047';
      ctx.beginPath();
      for (let i = 0; i < 5; i++) {
        const a = (i * 4 * Math.PI) / 5 - Math.PI / 2;
        const x = cx + Math.cos(a) * (r * 0.72);
        const y = cy + Math.sin(a) * (r * 0.72);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.closePath();
      ctx.fill();
    };
    drawStar(82, 108, 22);
    drawStar(174, 108, 22);
    ctx.beginPath();
    ctx.arc(128, 158, 36, 0.1 * Math.PI, 0.9 * Math.PI);
    ctx.stroke();
  } else if (faceType === 'cat') {
    ctx.beginPath();
    ctx.arc(82, 108, 15, 0, Math.PI * 2);
    ctx.arc(174, 108, 15, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(77, 102, 5, 0, Math.PI * 2);
    ctx.arc(169, 102, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ec4899';
    ctx.beginPath();
    ctx.moveTo(118, 134);
    ctx.lineTo(138, 134);
    ctx.lineTo(128, 146);
    ctx.closePath();
    ctx.fill();
    ctx.lineWidth = 7;
    ctx.beginPath();
    ctx.arc(114, 154, 14, 0, Math.PI);
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(142, 154, 14, 0, Math.PI);
    ctx.stroke();
    ctx.lineWidth = 5;
    ctx.beginPath();
    ctx.moveTo(22, 136);
    ctx.lineTo(56, 142);
    ctx.moveTo(22, 156);
    ctx.lineTo(56, 154);
    ctx.moveTo(234, 136);
    ctx.lineTo(200, 142);
    ctx.moveTo(234, 156);
    ctx.lineTo(200, 154);
    ctx.stroke();
  } else if (faceType === 'wink') {
    ctx.beginPath();
    ctx.arc(82, 108, 15, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(77, 102, 5, 0, Math.PI * 2);
    ctx.arc(169, 102, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.beginPath();
    ctx.arc(174, 112, 16, 1.15 * Math.PI, 1.85 * Math.PI);
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(128, 158, 34, 0.15 * Math.PI, 0.85 * Math.PI);
    ctx.stroke();
  } else if (faceType === 'silly') {
    ctx.beginPath();
    ctx.arc(82, 108, 15, 0, Math.PI * 2);
    ctx.arc(174, 108, 15, 0, Math.PI * 2);
    ctx.fill();
    ctx.beginPath();
    ctx.arc(128, 156, 34, 0.1 * Math.PI, 0.9 * Math.PI);
    ctx.stroke();
    ctx.fillStyle = '#fb7185';
    ctx.beginPath();
    ctx.arc(142, 184, 16, 0, Math.PI);
    ctx.fill();
  } else {
    ctx.beginPath();
    ctx.arc(82, 108, 15, 0, Math.PI * 2);
    ctx.arc(174, 108, 15, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(77, 102, 5, 0, Math.PI * 2);
    ctx.arc(169, 102, 5, 0, Math.PI * 2);
    ctx.fill();
    ctx.beginPath();
    ctx.arc(128, 158, 34, 0.15 * Math.PI, 0.85 * Math.PI);
    ctx.stroke();
  }

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}

function buildHatMesh(hatType, hatColor = '#fbbf24') {
  const group = new THREE.Group();
  if (!hatType || hatType === 'none') return group;

  if (hatType === 'crown') {
    const goldMat = new THREE.MeshStandardMaterial({ color: hatColor || '#fbbf24', metalness: 0.45, roughness: 0.25 });
    const base = new THREE.Mesh(new THREE.BoxGeometry(0.96, 0.22, 0.96), goldMat);
    base.position.y = 0.54;
    group.add(base);

    const gemMat = new THREE.MeshStandardMaterial({ color: '#ec4899', emissive: '#ec4899', emissiveIntensity: 0.4 });
    const positions = [
      [-0.36, 0.72, -0.36],
      [0, 0.76, -0.38],
      [0.36, 0.72, -0.36],
      [-0.36, 0.72, 0.36],
      [0.36, 0.72, 0.36]
    ];
    for (const [px, py, pz] of positions) {
      const spike = new THREE.Mesh(new THREE.BoxGeometry(0.18, 0.22, 0.18), goldMat);
      spike.position.set(px, py, pz);
      group.add(spike);
    }
    const frontGem = new THREE.Mesh(new THREE.BoxGeometry(0.16, 0.16, 0.08), gemMat);
    frontGem.position.set(0, 0.56, 0.49);
    group.add(frontGem);
  } else if (hatType === 'catears' || hatType === 'hair_bow') {
    const outerMat = new THREE.MeshStandardMaterial({ color: hatColor || '#f472b6', roughness: 0.4 });
    const innerMat = new THREE.MeshStandardMaterial({ color: '#fce7f3', roughness: 0.4 });
    for (const side of [-1, 1]) {
      const ear = new THREE.Mesh(new THREE.ConeGeometry(0.24, 0.45, 4), outerMat);
      ear.position.set(side * 0.28, 0.65, 0);
      ear.rotation.y = Math.PI / 4;
      group.add(ear);

      const inner = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.22, 0.05), innerMat);
      inner.position.set(side * 0.28, 0.6, 0.12);
      group.add(inner);
    }
  } else if (hatType === 'wizard' || hatType === 'beret') {
    const hatMat = new THREE.MeshStandardMaterial({ color: hatColor || '#7c3aed', roughness: 0.4 });
    const bandMat = new THREE.MeshStandardMaterial({ color: '#fbbf24', roughness: 0.3 });
    const brim = new THREE.Mesh(new THREE.CylinderGeometry(0.78, 0.78, 0.08, 16), hatMat);
    brim.position.y = 0.48;
    group.add(brim);

    if (hatType === 'wizard') {
      const band = new THREE.Mesh(new THREE.CylinderGeometry(0.46, 0.48, 0.12, 16), bandMat);
      band.position.y = 0.55;
      group.add(band);

      const cone = new THREE.Mesh(new THREE.ConeGeometry(0.46, 0.95, 16), hatMat);
      cone.position.y = 1.04;
      group.add(cone);
    }
  } else if (hatType === 'astronaut') {
    const glassMat = new THREE.MeshStandardMaterial({
      color: '#e0f2fe',
      transparent: true,
      opacity: 0.42,
      roughness: 0.15
    });
    const helmet = new THREE.Mesh(new THREE.BoxGeometry(1.12, 1.12, 1.12), glassMat);
    group.add(helmet);

    const ringMat = new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.3 });
    const collar = new THREE.Mesh(new THREE.BoxGeometry(1.16, 0.14, 1.16), ringMat);
    collar.position.y = -0.5;
    group.add(collar);
  } else if (hatType === 'tophat') {
    const blackMat = new THREE.MeshStandardMaterial({ color: hatColor || '#1e293b', roughness: 0.4 });
    const redMat = new THREE.MeshStandardMaterial({ color: '#ec4899', roughness: 0.3 });
    const brim = new THREE.Mesh(new THREE.BoxGeometry(1.22, 0.09, 1.22), blackMat);
    brim.position.y = 0.48;
    group.add(brim);
    const ribbon = new THREE.Mesh(new THREE.BoxGeometry(0.84, 0.12, 0.84), redMat);
    ribbon.position.y = 0.56;
    group.add(ribbon);
    const tall = new THREE.Mesh(new THREE.BoxGeometry(0.8, 0.65, 0.8), blackMat);
    tall.position.y = 0.9;
    group.add(tall);
  } else if (hatType === 'ninja') {
    const hoodMat = new THREE.MeshStandardMaterial({ color: '#1e1b4b', roughness: 0.45, metalness: 0.15 });
    const bandMat = new THREE.MeshStandardMaterial({
      color: hatColor || '#ec4899',
      emissive: hatColor || '#ec4899',
      emissiveIntensity: 0.28,
      roughness: 0.35
    });
    const crestMat = new THREE.MeshStandardMaterial({ color: '#fbbf24', metalness: 0.7, roughness: 0.2 });
    const cowl = new THREE.Mesh(new THREE.BoxGeometry(0.98, 0.36, 0.98), hoodMat);
    cowl.position.y = 0.36;
    group.add(cowl);
    const band = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.18, 1.0), bandMat);
    band.position.y = 0.24;
    group.add(band);
    const plate = new THREE.Mesh(new THREE.BoxGeometry(0.32, 0.12, 0.06), crestMat);
    plate.position.set(0, 0.24, 0.52);
    group.add(plate);
    for (const side of [-1, 1]) {
      const tail = new THREE.Mesh(new THREE.BoxGeometry(0.16, 0.1, 0.48), bandMat);
      tail.position.set(side * 0.12, 0.2, -0.64);
      tail.rotation.x = -0.32;
      tail.rotation.y = side * 0.18;
      group.add(tail);
    }
  } else if (hatType === 'obsidian_crown') {
    const obsidianMat = new THREE.MeshStandardMaterial({
      color: '#1e1b4b',
      metalness: 0.72,
      roughness: 0.2
    });
    const crystalMat = new THREE.MeshStandardMaterial({
      color: hatColor || '#c084fc',
      emissive: '#a855f7',
      emissiveIntensity: 0.65,
      metalness: 0.35,
      roughness: 0.15
    });
    const base = new THREE.Mesh(new THREE.CylinderGeometry(0.52, 0.48, 0.22, 8), obsidianMat);
    base.position.y = 0.54;
    group.add(base);

    const spireAngles = [0, (Math.PI * 2) / 5, (Math.PI * 4) / 5, (Math.PI * 6) / 5, (Math.PI * 8) / 5];
    spireAngles.forEach((ang, idx) => {
      const isFront = idx === 0;
      const spire = new THREE.Mesh(
        new THREE.ConeGeometry(isFront ? 0.14 : 0.1, isFront ? 0.48 : 0.34, 5),
        idx % 2 === 0 ? crystalMat : obsidianMat
      );
      spire.position.set(Math.sin(ang) * 0.42, isFront ? 0.84 : 0.75, Math.cos(ang) * 0.42);
      group.add(spire);
    });
    const centerGem = new THREE.Mesh(new THREE.OctahedronGeometry(0.14, 0), crystalMat);
    centerGem.position.set(0, 0.58, 0.5);
    group.add(centerGem);
  }

  return group;
}

function buildCrystalWingsAttachment(accentHex = '#38bdf8') {
  const group = new THREE.Group();
  const prismPrimary = new THREE.MeshStandardMaterial({
    color: accentHex || '#38bdf8',
    emissive: '#a855f7',
    emissiveIntensity: 0.55,
    transparent: true,
    opacity: 0.84,
    roughness: 0.14,
    metalness: 0.35
  });
  const prismSecondary = new THREE.MeshStandardMaterial({
    color: '#f472b6',
    emissive: '#38bdf8',
    emissiveIntensity: 0.48,
    transparent: true,
    opacity: 0.8,
    roughness: 0.16,
    metalness: 0.3
  });

  const leftWing = new THREE.Group();
  const rightWing = new THREE.Group();
  for (const [side, wing] of [[-1, leftWing], [1, rightWing]]) {
    const feathers = [
      { x: side * 0.24, y: 0.16, rotZ: side * -0.52, len: 0.54, r: 0.085, mat: prismPrimary },
      { x: side * 0.32, y: 0.02, rotZ: side * -0.92, len: 0.48, r: 0.075, mat: prismSecondary },
      { x: side * 0.24, y: -0.13, rotZ: side * -1.32, len: 0.38, r: 0.065, mat: prismPrimary }
    ];
    for (const f of feathers) {
      const shard = new THREE.Mesh(new THREE.OctahedronGeometry(f.r, 0), f.mat);
      shard.scale.set(1.0, f.len / (f.r * 2), 0.42);
      shard.position.set(f.x, f.y, 0);
      shard.rotation.z = f.rotZ;
      wing.add(shard);
    }
    wing.rotation.y = side * -0.32;
  }
  const coreGem = new THREE.Mesh(new THREE.OctahedronGeometry(0.08, 0), prismSecondary);
  group.add(leftWing, rightWing, coreGem);
  group.userData.leftWing = leftWing;
  group.userData.rightWing = rightWing;
  return group;
}

function buildStarLanternAttachment(accentHex = '#fbbf24') {
  const group = new THREE.Group();
  const frameMat = new THREE.MeshStandardMaterial({ color: '#1e1b4b', metalness: 0.72, roughness: 0.25 });
  const glowMat = new THREE.MeshStandardMaterial({
    color: accentHex || '#fde047',
    emissive: '#f59e0b',
    emissiveIntensity: 0.95,
    roughness: 0.15
  });
  const glassMat = new THREE.MeshStandardMaterial({
    color: '#c084fc',
    emissive: '#a855f7',
    emissiveIntensity: 0.35,
    transparent: true,
    opacity: 0.46,
    roughness: 0.12
  });

  const handleRing = new THREE.Mesh(new THREE.TorusGeometry(0.055, 0.012, 8, 16), frameMat);
  handleRing.position.y = 0.02;
  const capTop = new THREE.Mesh(new THREE.ConeGeometry(0.095, 0.07, 6), frameMat);
  capTop.position.y = -0.05;
  const glassCage = new THREE.Mesh(new THREE.CylinderGeometry(0.075, 0.065, 0.16, 6), glassMat);
  glassCage.position.y = -0.15;
  const starCore = new THREE.Mesh(new THREE.OctahedronGeometry(0.05, 0), glowMat);
  starCore.position.y = -0.15;
  const capBottom = new THREE.Mesh(new THREE.CylinderGeometry(0.07, 0.045, 0.04, 6), frameMat);
  capBottom.position.y = -0.24;

  group.add(handleRing, capTop, glassCage, starCore, capBottom);
  group.userData.starCore = starCore;
  return group;
}

function createOverheadLabelSprite({ displayName, role, speaking, muted, speechText }) {
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = 220;
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  if (speechText) {
    ctx.fillStyle = 'rgba(255, 255, 255, 0.96)';
    ctx.strokeStyle = '#ec4899';
    ctx.lineWidth = 5;
    const bx = 26;
    const by = 12;
    const bw = 460;
    const bh = 96;
    ctx.beginPath();
    ctx.roundRect(bx, by, bw, bh, 22);
    ctx.fill();
    ctx.stroke();

    ctx.fillStyle = '#0f172a';
    ctx.font = 'bold 26px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const clipped = speechText.length > 34 ? speechText.slice(0, 32) + '…' : speechText;
    ctx.fillText(clipped, 256, by + bh / 2);
  }

  const ny = 126;
  const nw = 372;
  const nh = 62;
  const nx = (512 - nw) / 2;

  ctx.fillStyle = speaking ? 'rgba(22, 163, 74, 0.92)' : 'rgba(15, 23, 42, 0.85)';
  ctx.strokeStyle = speaking ? '#4ade80' : role === 'host' ? '#f472b6' : '#38bdf8';
  ctx.lineWidth = 4;
  ctx.beginPath();
  ctx.roundRect(nx, ny, nw, nh, 999);
  ctx.fill();
  ctx.stroke();

  const micBadge = muted ? '🔇 ' : speaking ? '🔊 ' : '🎤 ';
  const crown = role === 'host' ? '👑 ' : '';
  ctx.fillStyle = '#ffffff';
  ctx.font = 'bold 28px Inter, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(`${micBadge}${crown}${displayName}`, 256, ny + nh / 2);

  const texture = new THREE.CanvasTexture(canvas);
  texture.colorSpace = THREE.SRGBColorSpace;
  return texture;
}

/**
 * Stylized 3D Blender Character Rig (Ella Forensics + Dress to Impress Modular Wardrobe)
 */
class StylizedCharacterRig {
  constructor(gltf, avatarConfig) {
    this.root = new THREE.Group();
    this.model = SkeletonUtils.clone(gltf.scene);
    this.model.scale.setScalar(1.55);
    this.root.add(this.model);

    this.slotsByName = new Map();
    this.glitterMaterials = [];
    this.wingsNode = null;
    this.headBone = null;

    const isCanonicalSlotName = (n) =>
      n === 'Body_Stylized' ||
      n === 'Face_Brows' ||
      n === 'Face_SmileTeeth' ||
      n === 'Eye_L' ||
      n === 'Eye_R' ||
      n.startsWith('Hair_') ||
      n.startsWith('Top_') ||
      n.startsWith('Bottom_') ||
      n.startsWith('Shoe_') ||
      n.startsWith('Hat_') ||
      n.startsWith('Acc_');

    // Pass 1: clone per-character materials on every mesh primitive so colors never bleed across players
    this.chestBone = null;
    this.leftHandBone = null;
    this.model.traverse((node) => {
      if (node.isBone) {
        if (node.name === 'head') this.headBone = node;
        else if (node.name === 'chest') this.chestBone = node;
        else if (node.name === 'hand.L') this.leftHandBone = node;
      }
      if (node.isMesh) {
        node.castShadow = true;
        node.receiveShadow = true;
        node.frustumCulled = false;

        if (Array.isArray(node.material)) {
          node.material = node.material.map((m) => m.clone());
        } else if (node.material) {
          node.material = node.material.clone();
        }
      }
    });

    // Procedural dungeon-exclusive bone mounts for StylizedCharacterRig
    this.dungeonHatMount = new THREE.Group();
    this.dungeonHatMount.position.set(0, -0.02, 0.02);
    this.dungeonHatMount.scale.setScalar(0.36);
    if (this.headBone) {
      this.headBone.add(this.dungeonHatMount);
    } else {
      this.dungeonHatMount.position.set(0, 1.48, 0.02);
      this.model.add(this.dungeonHatMount);
    }

    this.dungeonBackMount = new THREE.Group();
    this.dungeonBackMount.position.set(0, 0.08, -0.13);
    if (this.chestBone) {
      this.chestBone.add(this.dungeonBackMount);
    } else {
      this.dungeonBackMount.position.set(0, 1.24, -0.14);
      this.model.add(this.dungeonBackMount);
    }

    this.dungeonHandMount = new THREE.Group();
    this.dungeonHandMount.position.set(0, 0.09, 0.03);
    if (this.leftHandBone) {
      this.leftHandBone.add(this.dungeonHandMount);
    } else {
      this.dungeonHandMount.position.set(-0.28, 0.96, 0.16);
      this.model.add(this.dungeonHandMount);
    }

    this.customCrystalWings = null;
    this.customStarLantern = null;

    // Pass 2: index top-level wardrobe slots (handles BOTH 1-primitive SkinnedMesh AND multi-primitive Group!)
    this.model.traverse((node) => {
      if (node.isBone || !node.name || !isCanonicalSlotName(node.name)) return;

      // Skip child primitive meshes whose parent Group is already registered as the slot node
      let anc = node.parent;
      while (anc && anc !== this.model) {
        if (this.slotsByName.has(anc.name)) return;
        anc = anc.parent;
      }

      const childMeshes = [];
      node.traverse((sub) => {
        if (sub.isMesh) childMeshes.push(sub);
      });
      if (childMeshes.length === 0) return;

      this.slotsByName.set(node.name, { node, meshes: childMeshes });
      if (node.name === 'Acc_wings') {
        this.wingsNode = node;
      }
    });

    this.mixer = new THREE.AnimationMixer(this.model);
    this.actions = new Map();
    for (const clip of gltf.animations || []) {
      const action = this.mixer.clipAction(clip);
      this.actions.set(clip.name, action);
    }
    this.activeClipName = null;
    this._playClip('Idle_Glam', true);

    this.applyAppearance(avatarConfig || {});
  }

  _playClip(clipName, immediate = false) {
    if (!clipName || this.activeClipName === clipName) return;
    const nextAction = this.actions.get(clipName) || this.actions.get('Idle_Glam');
    if (!nextAction) return;

    const prevAction = this.activeClipName ? this.actions.get(this.activeClipName) : null;
    nextAction.reset();
    nextAction.enabled = true;
    nextAction.setEffectiveTimeScale(1);
    nextAction.setEffectiveWeight(1);
    nextAction.play();

    if (prevAction && prevAction !== nextAction && !immediate) {
      prevAction.crossFadeTo(nextAction, 0.18, true);
    } else if (prevAction && prevAction !== nextAction) {
      prevAction.stop();
    }
    this.activeClipName = clipName;
  }

  applyAppearance(av = {}) {
    const hairStyle = av.hairStyle || 'twin_pigtails';
    const topStyle = av.topStyle || 'glam_gown';
    const bottomStyle = av.bottomStyle || 'pleated_skirt';
    const shoeStyle = av.shoeStyle || 'runway_heels';
    const hat = av.hat || 'crown';
    const accessory = av.accessory || (hat === 'wings' ? 'wings' : 'none');

    const skinColor = av.skinColor || '#ffdfc4';
    const eyeColor = av.eyeColor || '#60a5fa';
    const hairColor = av.hairColor || '#fde047';
    const topColor = av.topColor || av.torsoColor || '#f472b6';
    const bottomColor = av.bottomColor || av.legsColor || '#6366f1';
    const shoeColor = av.shoeColor || '#f472b6';
    const hatColor = av.hatColor || '#fbbf24';
    const accessoryColor = av.accessoryColor || '#f472b6';
    const fabricFinish = av.fabricFinish || 'satin';

    const isFullGown = topStyle === 'glam_gown';
    this.glitterMaterials = [];

    for (const [slotName, slot] of this.slotsByName.entries()) {
      let isVisible = false;
      if (
        slotName === 'Body_Stylized' ||
        slotName === 'Face_Brows' ||
        slotName === 'Face_SmileTeeth' ||
        slotName === 'Eye_L' ||
        slotName === 'Eye_R'
      ) {
        isVisible = true;
      } else if (slotName.startsWith('Hair_')) {
        isVisible = slotName === `Hair_${hairStyle}`;
      } else if (slotName.startsWith('Top_')) {
        isVisible = slotName === `Top_${topStyle}`;
      } else if (slotName.startsWith('Bottom_')) {
        isVisible = !isFullGown && bottomStyle !== 'none' && slotName === `Bottom_${bottomStyle}`;
      } else if (slotName.startsWith('Shoe_')) {
        isVisible = slotName === `Shoe_${shoeStyle}`;
      } else if (slotName.startsWith('Hat_')) {
        isVisible = hat !== 'none' && hat !== 'wings' && slotName === `Hat_${hat}`;
      } else if (slotName === 'Acc_wings') {
        isVisible = accessory === 'wings' || hat === 'wings';
      } else if (slotName.startsWith('Acc_')) {
        isVisible = accessory !== 'none' && slotName === `Acc_${accessory}`;
      }

      slot.node.visible = isVisible;
      for (const mesh of slot.meshes) {
        mesh.visible = isVisible;
        const mats = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
        for (const mat of mats) {
          if (!mat) continue;
          const mName = mat.name || '';

          if (mName.startsWith('Mat_Avatar_Skin')) {
            mat.color.set(skinColor);
            mat.roughness = 0.46;
            mat.metalness = 0.02;
          } else if (mName.startsWith('Mat_Avatar_EyeIris')) {
            mat.color.set(eyeColor);
            mat.roughness = 0.08;
          } else if (mName.startsWith('Mat_Avatar_Brow')) {
            const browCol = new THREE.Color(hairColor).multiplyScalar(0.58);
            mat.color.copy(browCol);
          } else if (mName.startsWith('Mat_Avatar_Hair')) {
            mat.color.set(hairColor);
            mat.roughness = 0.34;
            mat.metalness = 0.06;
          } else if (mName.startsWith('Mat_Avatar_Top')) {
            mat.color.set(topColor);
            this._applyFabricFinish(mat, topColor, fabricFinish);
          } else if (mName.startsWith('Mat_Avatar_Bottom')) {
            mat.color.set(bottomColor);
            this._applyFabricFinish(mat, bottomColor, fabricFinish);
          } else if (mName.startsWith('Mat_Avatar_Shoe') && !mName.startsWith('Mat_Avatar_ShoeSole')) {
            mat.color.set(shoeColor);
            mat.roughness = fabricFinish === 'glitter' ? 0.18 : 0.34;
            mat.metalness = fabricFinish === 'glitter' ? 0.45 : 0.12;
          } else if (mName.startsWith('Mat_Avatar_Hat')) {
            mat.color.set(hatColor);
            mat.roughness = 0.28;
            mat.metalness = 0.32;
          } else if (mName.startsWith('Mat_Avatar_Accent')) {
            if (slotName.startsWith('Hair_')) {
              mat.color.set(accessory !== 'none' ? accessoryColor : topColor);
            } else if (slotName.startsWith('Top_') || slotName.startsWith('Bottom_')) {
              mat.color.set(accessory !== 'none' ? accessoryColor : hatColor);
              this._applyFabricFinish(mat, accessory !== 'none' ? accessoryColor : hatColor, fabricFinish);
            } else {
              mat.color.set(accessoryColor);
              mat.roughness = 0.26;
              mat.metalness = 0.25;
            }
          } else if (mName.startsWith('Mat_Avatar_Wings')) {
            mat.color.set(accessory === 'wings' ? accessoryColor : '#a5f3fc');
            mat.emissive.set(accessory === 'wings' ? accessoryColor : '#38bdf8');
            mat.emissiveIntensity = 0.38;
          }
        }
      }
    }

    // Mount procedural Crystal Cavern exclusive hats & accessories on StylizedCharacterRig
    while (this.dungeonHatMount.children.length > 0) {
      this.dungeonHatMount.remove(this.dungeonHatMount.children[0]);
    }
    if (hat === 'obsidian_crown' || hat === 'ninja') {
      const customHat = buildHatMesh(hat, hatColor);
      this.dungeonHatMount.add(customHat);
    }

    while (this.dungeonBackMount.children.length > 0) {
      this.dungeonBackMount.remove(this.dungeonBackMount.children[0]);
    }
    this.customCrystalWings = null;
    if (accessory === 'crystal_wings') {
      this.customCrystalWings = buildCrystalWingsAttachment(accessoryColor);
      this.dungeonBackMount.add(this.customCrystalWings);
    }

    while (this.dungeonHandMount.children.length > 0) {
      this.dungeonHandMount.remove(this.dungeonHandMount.children[0]);
    }
    this.customStarLantern = null;
    if (accessory === 'star_lantern') {
      this.customStarLantern = buildStarLanternAttachment(accessoryColor);
      this.dungeonHandMount.add(this.customStarLantern);
    }
  }

  _applyFabricFinish(mat, hexColor, finish) {
    if (finish === 'glitter') {
      mat.roughness = 0.14;
      mat.metalness = 0.56;
      mat.emissive.set(hexColor);
      mat.emissiveIntensity = 0.14;
      this.glitterMaterials.push(mat);
    } else if (finish === 'satin') {
      mat.roughness = 0.25;
      mat.metalness = 0.18;
      mat.emissive.set(hexColor);
      mat.emissiveIntensity = 0.04;
    } else {
      mat.roughness = 0.66;
      mat.metalness = 0.04;
      mat.emissive.set('#000000');
      mat.emissiveIntensity = 0.0;
    }
  }

  update(dt, animClock, animState, emoteState, speaking) {
    const desiredClip =
      (emoteState && EMOTE_TO_CLIP[emoteState]) ||
      ANIM_TO_CLIP[animState] ||
      'Idle_Glam';
    this._playClip(desiredClip, false);
    this.mixer.update(dt);

    if (this.glitterMaterials.length > 0) {
      const shimmer = 0.13 + Math.sin(animClock * 6.5) * 0.05;
      for (const gm of this.glitterMaterials) {
        gm.emissiveIntensity = shimmer;
      }
    }

    if (this.wingsNode && this.wingsNode.visible) {
      this.wingsNode.scale.x = 1.0 + Math.sin(animClock * 9.0) * 0.08;
    }

    if (this.customCrystalWings) {
      const flap = Math.sin(animClock * 8.5) * 0.28;
      if (this.customCrystalWings.userData.leftWing) {
        this.customCrystalWings.userData.leftWing.rotation.y = 0.32 + flap;
      }
      if (this.customCrystalWings.userData.rightWing) {
        this.customCrystalWings.userData.rightWing.rotation.y = -0.32 - flap;
      }
    }

    if (this.customStarLantern && this.customStarLantern.userData.starCore) {
      this.customStarLantern.rotation.z = Math.sin(animClock * 3.8) * 0.14;
      this.customStarLantern.userData.starCore.rotation.y = animClock * 3.2;
    }

    if (speaking && this.headBone) {
      this.headBone.rotation.x += Math.sin(animClock * 13.0) * 0.035;
    }
  }
}

/**
 * Classic Retro Blocky Rig (preserved as optional 'blocky' nostalgia mode)
 */
class ClassicBlockyRig {
  constructor(avatarConfig) {
    this.root = new THREE.Group();
    this.bodyGroup = new THREE.Group();
    this.root.add(this.bodyGroup);
    this.applyAppearance(avatarConfig || {});
  }

  applyAppearance(av = {}) {
    while (this.bodyGroup.children.length > 0) {
      this.bodyGroup.remove(this.bodyGroup.children[0]);
    }

    const skinColor = av.skinColor || '#ffdfc4';
    const torsoColor = av.topColor || av.torsoColor || '#f472b6';
    const legsColor = av.bottomColor || av.legsColor || '#6366f1';
    const hatType = av.hat || 'crown';
    const hatColor = av.hatColor || '#fbbf24';
    const faceType = av.face || 'happy';

    const skinMat = new THREE.MeshStandardMaterial({ color: skinColor, roughness: 0.5 });
    const torsoMat = new THREE.MeshStandardMaterial({ color: torsoColor, roughness: 0.45 });
    const legsMat = new THREE.MeshStandardMaterial({ color: legsColor, roughness: 0.55 });
    const faceTex = createFaceTexture(faceType, skinColor);
    const faceMat = new THREE.MeshStandardMaterial({ map: faceTex, roughness: 0.45 });

    const headMaterials = [skinMat, skinMat, skinMat, skinMat, faceMat, skinMat];
    this.headMesh = new THREE.Mesh(new THREE.BoxGeometry(0.92, 0.92, 0.92), headMaterials);
    this.headMesh.position.set(0, 2.48, 0);
    this.headMesh.castShadow = true;

    const hatGroup = buildHatMesh(hatType, hatColor);
    this.headMesh.add(hatGroup);
    this.bodyGroup.add(this.headMesh);

    this.torsoMesh = new THREE.Mesh(new THREE.BoxGeometry(1.05, 1.08, 0.52), torsoMat);
    this.torsoMesh.position.set(0, 1.48, 0);
    this.torsoMesh.castShadow = true;
    this.bodyGroup.add(this.torsoMesh);

    this.crystalWingsGroup = null;
    this.starLanternGroup = null;

    if (hatType === 'wings' || av.accessory === 'wings') {
      const wingMat = new THREE.MeshStandardMaterial({
        color: av.accessoryColor || '#a5f3fc',
        emissive: '#38bdf8',
        emissiveIntensity: 0.35,
        transparent: true,
        opacity: 0.72,
        side: THREE.DoubleSide
      });
      this.leftWing = new THREE.Mesh(new THREE.BoxGeometry(0.95, 0.85, 0.06), wingMat);
      this.leftWing.position.set(-0.55, 0.1, -0.32);
      this.leftWing.rotation.y = 0.4;
      this.rightWing = new THREE.Mesh(new THREE.BoxGeometry(0.95, 0.85, 0.06), wingMat);
      this.rightWing.position.set(0.55, 0.1, -0.32);
      this.rightWing.rotation.y = -0.4;
      this.torsoMesh.add(this.leftWing, this.rightWing);
    } else if (av.accessory === 'crystal_wings') {
      this.leftWing = null;
      this.rightWing = null;
      this.crystalWingsGroup = buildCrystalWingsAttachment(av.accessoryColor || '#38bdf8');
      this.crystalWingsGroup.scale.setScalar(1.9);
      this.crystalWingsGroup.position.set(0, 0.12, -0.3);
      this.torsoMesh.add(this.crystalWingsGroup);
    } else {
      this.leftWing = null;
      this.rightWing = null;
    }

    this.leftArmPivot = new THREE.Group();
    this.leftArmPivot.position.set(-0.74, 1.95, 0);
    const leftArm = new THREE.Mesh(new THREE.BoxGeometry(0.4, 1.02, 0.42), skinMat);
    leftArm.position.y = -0.44;
    leftArm.castShadow = true;
    this.leftArmPivot.add(leftArm);
    if (av.accessory === 'star_lantern') {
      this.starLanternGroup = buildStarLanternAttachment(av.accessoryColor || '#fbbf24');
      this.starLanternGroup.scale.setScalar(2.1);
      this.starLanternGroup.position.set(-0.04, -0.92, 0.24);
      this.leftArmPivot.add(this.starLanternGroup);
    }

    this.rightArmPivot = new THREE.Group();
    this.rightArmPivot.position.set(0.74, 1.95, 0);
    const rightArm = new THREE.Mesh(new THREE.BoxGeometry(0.4, 1.02, 0.42), skinMat);
    rightArm.position.y = -0.44;
    rightArm.castShadow = true;
    this.rightArmPivot.add(rightArm);
    this.bodyGroup.add(this.leftArmPivot, this.rightArmPivot);

    this.leftLegPivot = new THREE.Group();
    this.leftLegPivot.position.set(-0.27, 0.94, 0);
    const leftLeg = new THREE.Mesh(new THREE.BoxGeometry(0.46, 0.94, 0.48), legsMat);
    leftLeg.position.y = -0.47;
    leftLeg.castShadow = true;
    this.leftLegPivot.add(leftLeg);

    this.rightLegPivot = new THREE.Group();
    this.rightLegPivot.position.set(0.27, 0.94, 0);
    const rightLeg = new THREE.Mesh(new THREE.BoxGeometry(0.46, 0.94, 0.48), legsMat);
    rightLeg.position.y = -0.47;
    rightLeg.castShadow = true;
    this.rightLegPivot.add(rightLeg);
    this.bodyGroup.add(this.leftLegPivot, this.rightLegPivot);
  }

  update(_dt, t, anim, emote) {
    if (this.leftWing && this.rightWing) {
      const flap = Math.sin(t * 9) * 0.25;
      this.leftWing.rotation.y = 0.4 + flap;
      this.rightWing.rotation.y = -0.4 - flap;
    }
    if (this.crystalWingsGroup) {
      const flap = Math.sin(t * 8.5) * 0.28;
      if (this.crystalWingsGroup.userData.leftWing) {
        this.crystalWingsGroup.userData.leftWing.rotation.y = 0.32 + flap;
      }
      if (this.crystalWingsGroup.userData.rightWing) {
        this.crystalWingsGroup.userData.rightWing.rotation.y = -0.32 - flap;
      }
    }
    if (this.starLanternGroup && this.starLanternGroup.userData.starCore) {
      this.starLanternGroup.rotation.z = Math.sin(t * 3.8) * 0.14;
      this.starLanternGroup.userData.starCore.rotation.y = t * 3.2;
    }

    this.bodyGroup.position.set(0, 0, 0);
    this.bodyGroup.rotation.set(0, 0, 0);
    this.leftArmPivot.rotation.set(0, 0, 0);
    this.rightArmPivot.rotation.set(0, 0, 0);
    this.leftLegPivot.rotation.set(0, 0, 0);
    this.rightLegPivot.rotation.set(0, 0, 0);
    this.headMesh.rotation.set(0, 0, 0);

    if (emote === 'wave' || emote === 'peace') {
      this.rightArmPivot.rotation.x = Math.PI * 0.88;
      this.rightArmPivot.rotation.z = Math.sin(t * 10) * 0.35 - 0.2;
      this.headMesh.rotation.z = Math.sin(t * 5) * 0.08;
    } else if (emote === 'pose') {
      this.bodyGroup.rotation.y = 0.32;
      this.leftArmPivot.rotation.z = 0.55;
      this.rightArmPivot.rotation.x = Math.PI * 0.78;
      this.headMesh.rotation.z = -0.14;
    } else if (emote === 'dance') {
      this.bodyGroup.position.y = Math.abs(Math.sin(t * 8)) * 0.28;
      this.bodyGroup.rotation.y = Math.sin(t * 4) * 0.35;
      this.leftArmPivot.rotation.z = 0.7 + Math.sin(t * 8) * 0.5;
      this.rightArmPivot.rotation.z = -0.7 + Math.cos(t * 8) * 0.5;
      this.leftLegPivot.rotation.x = Math.sin(t * 8) * 0.45;
      this.rightLegPivot.rotation.x = -Math.sin(t * 8) * 0.45;
    } else if (emote === 'spin') {
      this.bodyGroup.rotation.y = t * 10;
      this.leftArmPivot.rotation.z = 1.35;
      this.rightArmPivot.rotation.z = -1.35;
    } else if (emote === 'cheer') {
      this.bodyGroup.position.y = Math.abs(Math.sin(t * 9)) * 0.55;
      this.leftArmPivot.rotation.x = Math.PI * 0.88;
      this.rightArmPivot.rotation.x = Math.PI * 0.88;
      this.leftArmPivot.rotation.z = 0.25;
      this.rightArmPivot.rotation.z = -0.25;
    } else if (emote === 'flip') {
      const cycle = (t * 3.2) % (Math.PI * 2);
      this.bodyGroup.position.y = Math.max(0, Math.sin(cycle / 2) * 1.35);
      this.bodyGroup.rotation.x = -cycle;
      this.leftArmPivot.rotation.x = 1.2;
      this.rightArmPivot.rotation.x = 1.2;
    } else if (emote === 'sit') {
      this.bodyGroup.position.y = -0.45;
      this.leftLegPivot.rotation.x = -Math.PI / 2;
      this.rightLegPivot.rotation.x = -Math.PI / 2;
      this.leftArmPivot.rotation.x = -0.25;
      this.rightArmPivot.rotation.x = -0.25;
    } else if (anim === 'jump') {
      this.leftArmPivot.rotation.x = Math.PI * 0.72;
      this.rightArmPivot.rotation.x = Math.PI * 0.72;
      this.leftLegPivot.rotation.x = -0.35;
      this.rightLegPivot.rotation.x = 0.25;
    } else if (anim === 'walk' || anim === 'run') {
      const freq = anim === 'run' ? 15 : 11;
      const swing = Math.sin(t * freq) * 0.72;
      this.leftArmPivot.rotation.x = swing;
      this.rightArmPivot.rotation.x = -swing;
      this.leftLegPivot.rotation.x = -swing;
      this.rightLegPivot.rotation.x = swing;
      this.bodyGroup.position.y = Math.abs(Math.cos(t * freq)) * 0.08;
    } else {
      this.leftArmPivot.rotation.z = 0.05 + Math.sin(t * 2.2) * 0.03;
      this.rightArmPivot.rotation.z = -0.05 - Math.sin(t * 2.2) * 0.03;
      this.headMesh.rotation.y = Math.sin(t * 1.3) * 0.06;
    }
  }
}

function createWorldBannerSprite(title, subtitle = '', accentHex = '#c084fc') {
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = subtitle ? 156 : 96;
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = 'rgba(15, 23, 42, 0.88)';
  ctx.strokeStyle = accentHex;
  ctx.lineWidth = 4;
  ctx.beginPath();
  ctx.roundRect(14, 10, canvas.width - 28, canvas.height - 20, 26);
  ctx.fill();
  ctx.stroke();

  ctx.fillStyle = '#ffffff';
  ctx.font = 'bold 28px Inter, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(String(title || '').slice(0, 34), canvas.width / 2, subtitle ? 52 : canvas.height / 2);

  if (subtitle) {
    ctx.fillStyle = '#e2e8f0';
    ctx.font = '600 20px Inter, sans-serif';
    ctx.fillText(String(subtitle).slice(0, 42), canvas.width / 2, 104);
  }

  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  const mat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(subtitle ? 3.8 : 2.6, subtitle ? 1.16 : 0.56, 1);
  return sprite;
}

/**
 * Unified WorldAvatar (Defaults to 3D Rigged Forensics + DTI Character, with live Blocky toggle)
 */
class WorldAvatar {
  constructor(playerData) {
    this.id = playerData.id;
    this.user = playerData.user || { username: 'friend', displayName: 'Friend', avatar: {} };
    this.zone = Number(playerData.zone) || 0;
    this.group = new THREE.Group();
    this.rigContainer = new THREE.Group();
    this.group.add(this.rigContainer);

    this.targetPos = new THREE.Vector3(playerData.x || 0, playerData.y || 0, playerData.z || 0);
    this.group.position.copy(this.targetPos);
    this.targetRy = playerData.ry || 0;
    this.group.rotation.y = this.targetRy;

    this.anim = playerData.anim || 'idle';
    this.emote = playerData.emote || null;
    this.speaking = Boolean(playerData.speaking);
    this.muted = Boolean(playerData.muted);
    this.speechText = '';
    this.speechExpiresAt = 0;
    this.animClock = Math.random() * 10;

    this.activeMode = null;
    this.stylizedRig = null;
    this.blockyRig = null;

    const ringGeo = new THREE.RingGeometry(0.75, 1.02, 32);
    ringGeo.rotateX(-Math.PI / 2);
    const ringMat = new THREE.MeshBasicMaterial({
      color: '#22c55e',
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.0
    });
    this.voiceRing = new THREE.Mesh(ringGeo, ringMat);
    this.voiceRing.position.y = 0.04;
    this.group.add(this.voiceRing);

    this._refreshLabelSprite();
    this._syncRigMode();
  }

  _syncRigMode() {
    const av = this.user.avatar || {};
    const wantMode = av.avatarMode === 'blocky' ? 'blocky' : 'stylized';

    if (wantMode === 'blocky') {
      this._activateBlocky(av);
      return;
    }

    if (dtiPackAsset) {
      this._activateStylized(dtiPackAsset, av);
    } else {
      this._activateBlocky(av);
      loadDtiWardrobePack()
        .then((gltf) => {
          const latestAv = this.user.avatar || {};
          if (latestAv.avatarMode !== 'blocky') {
            this._activateStylized(gltf, latestAv);
          }
        })
        .catch(() => {});
    }
  }

  _activateStylized(gltf, av) {
    while (this.rigContainer.children.length > 0) {
      this.rigContainer.remove(this.rigContainer.children[0]);
    }
    if (!this.stylizedRig) {
      this.stylizedRig = new StylizedCharacterRig(gltf, av);
    } else {
      this.stylizedRig.applyAppearance(av);
    }
    this.rigContainer.add(this.stylizedRig.root);
    this.activeMode = 'stylized';
    if (this.labelSprite) {
      this.labelSprite.position.set(0, 3.92, 0);
    }
  }

  _activateBlocky(av) {
    while (this.rigContainer.children.length > 0) {
      this.rigContainer.remove(this.rigContainer.children[0]);
    }
    if (!this.blockyRig) {
      this.blockyRig = new ClassicBlockyRig(av);
    } else {
      this.blockyRig.applyAppearance(av);
    }
    this.rigContainer.add(this.blockyRig.root);
    this.activeMode = 'blocky';
    if (this.labelSprite) {
      this.labelSprite.position.set(0, 3.85, 0);
    }
  }

  _refreshLabelSprite() {
    const tex = createOverheadLabelSprite({
      displayName: this.user.displayName || this.user.username,
      role: this.user.role,
      speaking: this.speaking,
      muted: this.muted,
      speechText: this.speechText
    });

    if (!this.labelSprite) {
      const mat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false });
      this.labelSprite = new THREE.Sprite(mat);
      this.labelSprite.scale.set(3.4, 1.46, 1);
      this.labelSprite.position.set(0, 3.92, 0);
      this.group.add(this.labelSprite);
    } else {
      if (this.labelSprite.material.map) {
        this.labelSprite.material.map.dispose();
      }
      this.labelSprite.material.map = tex;
      this.labelSprite.material.needsUpdate = true;
    }
  }

  updateAvatarAppearance(newAvatar) {
    this.user.avatar = { ...(this.user.avatar || {}), ...newAvatar };
    const wantMode = this.user.avatar.avatarMode === 'blocky' ? 'blocky' : 'stylized';
    if (wantMode !== this.activeMode) {
      this._syncRigMode();
    } else if (this.activeMode === 'stylized' && this.stylizedRig) {
      this.stylizedRig.applyAppearance(this.user.avatar);
    } else if (this.activeMode === 'blocky' && this.blockyRig) {
      this.blockyRig.applyAppearance(this.user.avatar);
    }
  }

  setVoiceState({ muted, speaking }) {
    const changed = this.muted !== Boolean(muted) || this.speaking !== Boolean(speaking);
    this.muted = Boolean(muted);
    this.speaking = Boolean(speaking);
    if (changed) {
      this._refreshLabelSprite();
    }
  }

  showSpeechBubble(text) {
    this.speechText = String(text || '').slice(0, 80);
    this.speechExpiresAt = performance.now() + 6500;
    this._refreshLabelSprite();
  }

  update(dt, isSelf = false) {
    this.animClock += dt;

    if (this.speechText && performance.now() > this.speechExpiresAt) {
      this.speechText = '';
      this._refreshLabelSprite();
    }

    if (!isSelf) {
      if (this.group.position.distanceTo(this.targetPos) > 10.0) {
        this.group.position.copy(this.targetPos);
      } else {
        this.group.position.lerp(this.targetPos, Math.min(1, dt * 14));
      }
      let diff = this.targetRy - this.group.rotation.y;
      while (diff > Math.PI) diff -= Math.PI * 2;
      while (diff < -Math.PI) diff += Math.PI * 2;
      this.group.rotation.y += diff * Math.min(1, dt * 14);
    }

    if (this.voiceRing) {
      if (this.speaking) {
        this.voiceRing.material.opacity = 0.72 + Math.sin(this.animClock * 12) * 0.22;
        const s = 1.05 + Math.sin(this.animClock * 10) * 0.12;
        this.voiceRing.scale.set(s, 1, s);
      } else {
        this.voiceRing.material.opacity = 0;
      }
    }

    if (this.activeMode === 'stylized' && this.stylizedRig) {
      this.stylizedRig.update(dt, this.animClock, this.anim, this.emote, this.speaking);
    } else if (this.blockyRig) {
      this.blockyRig.update(dt, this.animClock, this.anim, this.emote);
    }
  }
}

export class BlockyWorld3D {
  constructor({
    container,
    onMove,
    onPlaceBlock,
    onRemoveBlock,
    onObbyCheckpoint,
    onTrophyWin,
    onRunwayProximity,
    onRequestZonePortal,
    onInteractNode,
    onAttackMob,
    onPickupDrop,
    onInteractionHint
  }) {
    this.container = container;
    this.onMove = onMove || (() => {});
    this.onPlaceBlock = onPlaceBlock || (() => {});
    this.onRemoveBlock = onRemoveBlock || (() => {});
    this.onObbyCheckpoint = onObbyCheckpoint || (() => {});
    this.onTrophyWin = onTrophyWin || (() => {});
    this.onRunwayProximity = onRunwayProximity || (() => {});
    this.onRequestZonePortal = onRequestZonePortal || (() => {});
    this.onInteractNode = onInteractNode || (() => {});
    this.onAttackMob = onAttackMob || (() => {});
    this.onPickupDrop = onPickupDrop || (() => {});
    this.onInteractionHint = onInteractionHint || (() => {});

    loadDtiWardrobePack().catch(() => {});

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color('#7dd3fc');
    this.scene.fog = new THREE.FogExp2('#bae6fd', 0.0078);

    this.camera = new THREE.PerspectiveCamera(58, window.innerWidth / window.innerHeight, 0.1, 240);
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.08;
    this.container.appendChild(this.renderer.domElement);

    this.currentZone = 0; // 0 = Starlight Plaza, 1 = Crystal Cavern Dungeon
    this.plazaGroup = new THREE.Group();
    this.scene.add(this.plazaGroup);
    this.dungeonGroup = new THREE.Group();
    this.dungeonGroup.visible = false;
    this.scene.add(this.dungeonGroup);
    this.dungeonEntitiesGroup = new THREE.Group();
    this.dungeonGroup.add(this.dungeonEntitiesGroup);

    this.plazaColliders = [];
    this.voxelColliders = [];
    this.dungeonColliders = [];
    this.colliders = [];
    this.movingPlatforms = [];
    this.discoTiles = [];
    this.runwayStripLights = [];
    this.jumpPads = [];
    this.trophyMesh = null;
    this.runwayStarCrest = null;
    this.confettiParticles = [];
    this.sparkleRings = [];
    this.portalVortices = [];

    this.plazaPortalPos = { x: 18, y: 0, z: 16 };
    this.dungeonReturnPortalPos = { x: 0, y: 0, z: 28 };
    this.lastPortalTriggerAt = 0;
    this.lastHintText = '';

    this.dungeonNodesMap = new Map();
    this.dungeonMobsMap = new Map();
    this.dungeonDropsMap = new Map();

    this.speedBoostUntilMs = 0;
    this.jumpBoostUntilMs = 0;

    this.voxelMap = new Map();
    this.voxelGroup = new THREE.Group();
    this.scene.add(this.voxelGroup);

    this.buildMode = false;
    this.buildTool = 'place';
    this.selectedBlockColor = BLOCK_COLORS[0].hex;
    this.raycaster = new THREE.Raycaster();
    this.pointerNdc = new THREE.Vector2();
    this.ghostMesh = new THREE.Mesh(
      new THREE.BoxGeometry(1.02, 1.02, 1.02),
      new THREE.MeshBasicMaterial({ color: '#38bdf8', wireframe: true })
    );
    this.ghostMesh.visible = false;
    this.scene.add(this.ghostMesh);

    this.selfAvatar = null;
    this.remoteAvatars = new Map();

    this.keys = new Set();
    this.touchMove = { forward: false, backward: false, left: false, right: false };
    this.velocity = new THREE.Vector3();
    this.isGrounded = true;
    this.currentCheckpoint = { x: 0, y: 0.5, z: -12, index: 0 };
    this.lastTrophyWinAt = 0;
    this.nearRunway = false;

    this.cameraYaw = 0;
    this.cameraPitch = 0.34;
    this.cameraDist = 9.2;
    this.salonMirrorMode = false;
    this.savedCameraBeforeSalon = null;
    this.isDraggingCamera = false;
    this.dragMoved = false;
    this.lastPointer = { x: 0, y: 0 };
    this.serverClockOffsetMs = 0;

    this._setupLights();
    this._buildEnvironment();
    this._buildDungeonEnvironment();
    this._rebuildActiveColliders();
    this._bindEvents();

    this.lastFrameTime = performance.now();
    this.lastMoveSentAt = 0;
    this._animate = this._animate.bind(this);
    requestAnimationFrame(this._animate);
  }

  _rebuildActiveColliders() {
    if (this.currentZone === 1) {
      this.colliders = this.dungeonColliders;
    } else {
      this.colliders = [...this.plazaColliders, ...this.voxelColliders];
    }
  }

  _setupLights() {
    this.hemiLight = new THREE.HemisphereLight('#ffffff', '#fbcfe8', 0.92);
    this.scene.add(this.hemiLight);

    this.sunLight = new THREE.DirectionalLight('#fffbeb', 1.28);
    this.sunLight.position.set(28, 48, 24);
    this.sunLight.castShadow = true;
    this.sunLight.shadow.mapSize.width = 2048;
    this.sunLight.shadow.mapSize.height = 2048;
    this.sunLight.shadow.camera.near = 1;
    this.sunLight.shadow.camera.far = 140;
    const d = 48;
    this.sunLight.shadow.camera.left = -d;
    this.sunLight.shadow.camera.right = d;
    this.sunLight.shadow.camera.top = d;
    this.sunLight.shadow.camera.bottom = -d;
    this.scene.add(this.sunLight);

    this.fillLight = new THREE.DirectionalLight('#fdf2f8', 0.55);
    this.fillLight.position.set(-28, 18, 26);
    this.scene.add(this.fillLight);
  }

  _addBoxPlatform(
    { x, y, z, w, h, d, color, type = 'solid', bounceVel = 0, checkpointIndex = 0, emissive = null, emissiveIntensity = 0 },
    targetGroup = this.plazaGroup,
    targetColliders = this.plazaColliders
  ) {
    const mat = new THREE.MeshStandardMaterial({
      color,
      roughness: type === 'ice' || type === 'runway' ? 0.18 : 0.55,
      metalness: type === 'trophy-pad' || type === 'runway' ? 0.28 : 0.08
    });
    if (emissive) {
      mat.emissive = new THREE.Color(emissive);
      mat.emissiveIntensity = emissiveIntensity || 0.3;
    }
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), mat);
    mesh.position.set(x, y, z);
    mesh.receiveShadow = true;
    mesh.castShadow = y > 0.5;
    targetGroup.add(mesh);

    const entry = {
      mesh,
      w,
      h,
      d,
      minX: x - w / 2,
      maxX: x + w / 2,
      minZ: z - d / 2,
      maxZ: z + d / 2,
      topY: y + h / 2,
      bottomY: y - h / 2,
      type,
      bounceVel,
      checkpointIndex
    };
    targetColliders.push(entry);
    return entry;
  }

  _createPortalArch({ x, y, z, title, subtitle, primaryHex, secondaryHex }, targetGroup) {
    const portal = new THREE.Group();
    portal.position.set(x, y, z);

    const stoneMat = new THREE.MeshStandardMaterial({ color: '#1e1b4b', roughness: 0.35, metalness: 0.45 });
    const runeMat = new THREE.MeshStandardMaterial({
      color: primaryHex,
      emissive: primaryHex,
      emissiveIntensity: 0.72,
      roughness: 0.2,
      metalness: 0.35
    });

    const pad = new THREE.Mesh(new THREE.CylinderGeometry(2.75, 3.05, 0.2, 28), stoneMat);
    pad.position.y = 0.1;
    const runeRing = new THREE.Mesh(new THREE.TorusGeometry(2.45, 0.09, 10, 32), runeMat);
    runeRing.rotation.x = Math.PI / 2;
    runeRing.position.y = 0.22;
    portal.add(pad, runeRing);

    for (const side of [-1, 1]) {
      const pillar = new THREE.Mesh(new THREE.BoxGeometry(0.68, 4.1, 0.68), stoneMat);
      pillar.position.set(side * 1.85, 2.15, 0);
      pillar.castShadow = true;
      const runeStrip = new THREE.Mesh(new THREE.BoxGeometry(0.2, 3.5, 0.74), runeMat);
      runeStrip.position.set(side * 1.85, 2.15, 0);
      portal.add(pillar, runeStrip);
    }

    const lintel = new THREE.Mesh(new THREE.BoxGeometry(4.65, 0.72, 0.88), stoneMat);
    lintel.position.set(0, 4.35, 0);
    const keystone = new THREE.Mesh(new THREE.OctahedronGeometry(0.46, 0), runeMat);
    keystone.position.set(0, 4.95, 0);
    portal.add(lintel, keystone);

    const vortexMat = new THREE.MeshBasicMaterial({
      color: secondaryHex,
      transparent: true,
      opacity: 0.68,
      side: THREE.DoubleSide
    });
    const vortexDisc = new THREE.Mesh(new THREE.CircleGeometry(1.75, 28), vortexMat);
    vortexDisc.position.set(0, 2.28, 0);
    const swirlRing = new THREE.Mesh(
      new THREE.TorusGeometry(1.25, 0.08, 10, 28),
      new THREE.MeshBasicMaterial({ color: '#ffffff', transparent: true, opacity: 0.75 })
    );
    swirlRing.position.set(0, 2.28, 0.04);
    portal.add(vortexDisc, swirlRing);
    this.portalVortices.push({ disc: vortexDisc, ring: swirlRing, keystone });

    const banner = createWorldBannerSprite(title, subtitle, primaryHex);
    banner.position.set(0, 6.05, 0);
    portal.add(banner);

    targetGroup.add(portal);
    return portal;
  }

  _buildEnvironment() {
    const groundMat = new THREE.MeshStandardMaterial({ color: '#4ade80', roughness: 0.75 });
    this.groundMesh = new THREE.Mesh(new THREE.BoxGeometry(84, 2, 84), groundMat);
    this.groundMesh.position.set(0, -1, 0);
    this.groundMesh.receiveShadow = true;
    this.plazaGroup.add(this.groundMesh);
    this.plazaColliders.push({
      mesh: this.groundMesh,
      w: 84,
      h: 2,
      d: 84,
      minX: -42,
      maxX: 42,
      minZ: -42,
      maxZ: 42,
      topY: 0,
      bottomY: -2,
      type: 'ground'
    });

    this._addBoxPlatform({ x: 0, y: 0.04, z: 0, w: 18, h: 0.08, d: 18, color: '#e2e8f0' });
    const spawnPad = new THREE.Mesh(
      new THREE.CylinderGeometry(3.5, 3.7, 0.14, 32),
      new THREE.MeshStandardMaterial({ color: '#f472b6', emissive: '#db2777', emissiveIntensity: 0.25 })
    );
    spawnPad.position.set(0, 0.08, 0);
    this.plazaGroup.add(spawnPad);

    const discoColors = ['#f472b6', '#38bdf8', '#fbbf24', '#a78bfa', '#34d399'];
    for (let gx = 0; gx < 4; gx++) {
      for (let gz = 0; gz < 4; gz++) {
        const tileMat = new THREE.MeshStandardMaterial({
          color: discoColors[(gx + gz) % discoColors.length],
          emissive: discoColors[(gx + gz) % discoColors.length],
          emissiveIntensity: 0.35
        });
        const tile = new THREE.Mesh(new THREE.BoxGeometry(1.9, 0.14, 1.9), tileMat);
        tile.position.set(-13 + gx * 2, 0.08, -2 + gz * 2);
        this.plazaGroup.add(tile);
        this.discoTiles.push(tile);
      }
    }

    this._buildDtiFashionRunway();

    const logMat = new THREE.MeshStandardMaterial({ color: '#78350f', roughness: 0.8 });
    const fireMat = new THREE.MeshStandardMaterial({
      color: '#fb923c',
      emissive: '#ef4444',
      emissiveIntensity: 0.75
    });
    const fireCore = new THREE.Mesh(new THREE.ConeGeometry(0.85, 1.5, 6), fireMat);
    fireCore.position.set(-6, 0.75, 11);
    this.plazaGroup.add(fireCore);
    this._addBoxPlatform({ x: -6, y: 0.35, z: 14.2, w: 4.2, h: 0.7, d: 1.1, color: '#92400e' });
    this._addBoxPlatform({ x: -9.2, y: 0.35, z: 11, w: 1.1, h: 0.7, d: 4.2, color: '#92400e' });

    const buildGridFloor = new THREE.GridHelper(16, 16, '#38bdf8', '#94a3b8');
    buildGridFloor.position.set(18, 0.06, -4);
    this.plazaGroup.add(buildGridFloor);

    // Crystal Cavern Dungeon Portal in Starlight Plaza
    this._createPortalArch(
      {
        x: this.plazaPortalPos.x,
        y: this.plazaPortalPos.y,
        z: this.plazaPortalPos.z,
        title: '🌀 CRYSTAL CAVERN',
        subtitle: 'Step In or Press E • Farm Coins & Loot!',
        primaryHex: '#c084fc',
        secondaryHex: '#7c3aed'
      },
      this.plazaGroup
    );

    const jumpPadCoords = [
      { x: 7.5, y: 0.2, z: -7.5, bounceVel: 20 },
      { x: -7.5, y: 0.2, z: -7.5, bounceVel: 20 },
      { x: 0, y: 8.6, z: -34, bounceVel: 23 }
    ];
    for (const jp of jumpPadCoords) {
      const pad = this._addBoxPlatform({
        x: jp.x,
        y: jp.y,
        z: jp.z,
        w: 2.6,
        h: 0.4,
        d: 2.6,
        color: '#ec4899',
        type: 'jumppad',
        bounceVel: jp.bounceVel
      });
      pad.mesh.material.emissive = new THREE.Color('#db2777');
      pad.mesh.material.emissiveIntensity = 0.45;
      this.jumpPads.push(pad);
    }

    const trunkGeo = new THREE.BoxGeometry(0.9, 2.8, 0.9);
    const leavesGeo = new THREE.BoxGeometry(2.8, 2.6, 2.8);
    const blossomMat = new THREE.MeshStandardMaterial({ color: '#f9a8d4', roughness: 0.6 });
    const greenLeafMat = new THREE.MeshStandardMaterial({ color: '#22c55e', roughness: 0.6 });
    const treePositions = [
      [-16, 26], [14, 24], [26, 20], [-34, 18],
      [-34, -18], [28, -22], [28, 10], [-16, -22]
    ];
    treePositions.forEach(([tx, tz], idx) => {
      const trunk = new THREE.Mesh(trunkGeo, logMat);
      trunk.position.set(tx, 1.4, tz);
      trunk.castShadow = true;
      const leaves = new THREE.Mesh(leavesGeo, idx % 2 === 0 ? blossomMat : greenLeafMat);
      leaves.position.set(tx, 3.6, tz);
      leaves.castShadow = true;
      this.plazaGroup.add(trunk, leaves);
    });

    this._addBoxPlatform({
      x: 0, y: 0.4, z: -12, w: 4.5, h: 0.8, d: 3.5, color: '#fbbf24', type: 'checkpoint', checkpointIndex: 1
    });

    const stage1Steps = [
      { x: 0, y: 1.4, z: -16.5, color: '#f472b6' },
      { x: -3.2, y: 2.5, z: -20.0, color: '#38bdf8' },
      { x: 0, y: 3.6, z: -23.5, color: '#34d399' },
      { x: 3.4, y: 4.8, z: -27.0, color: '#a78bfa' },
      { x: 0, y: 6.0, z: -30.5, color: '#fb923c' }
    ];
    for (const s of stage1Steps) {
      this._addBoxPlatform({ x: s.x, y: s.y, z: s.z, w: 3.1, h: 0.6, d: 3.1, color: s.color });
    }

    this._addBoxPlatform({
      x: 0, y: 8.2, z: -34, w: 5.5, h: 0.6, d: 4.2, color: '#fbbf24', type: 'checkpoint', checkpointIndex: 2
    });

    const moving1 = this._addBoxPlatform({
      x: -5, y: 11.5, z: -34, w: 3.6, h: 0.6, d: 3.6, color: '#38bdf8', type: 'moving'
    });
    moving1.baseX = -6.5;
    moving1.baseY = 11.5;
    moving1.baseZ = -34;
    moving1.ampX = 3.5;
    moving1.ampZ = 0;
    moving1.speed = 1.6;
    this.movingPlatforms.push(moving1);

    const moving2 = this._addBoxPlatform({
      x: -11, y: 13.5, z: -27, w: 3.6, h: 0.6, d: 3.6, color: '#c084fc', type: 'moving'
    });
    moving2.baseX = -11;
    moving2.baseY = 13.5;
    moving2.baseZ = -26;
    moving2.ampX = 0;
    moving2.ampZ = 4.0;
    moving2.speed = 1.8;
    this.movingPlatforms.push(moving2);

    const rainbowCols = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#38bdf8', '#8b5cf6'];
    rainbowCols.forEach((col, i) => {
      this._addBoxPlatform({
        x: -11 + (i + 1) * 2.2,
        y: 14.8 + i * 0.25,
        z: -20,
        w: 2.1,
        h: 0.45,
        d: 3.0,
        color: col
      });
    });

    this._addBoxPlatform({
      x: 6.5,
      y: 16.5,
      z: -20,
      w: 8.0,
      h: 1.0,
      d: 8.0,
      color: '#fef08a',
      type: 'trophy-pad',
      checkpointIndex: 3
    });

    const trophyGroup = new THREE.Group();
    const goldMat = new THREE.MeshStandardMaterial({
      color: '#fbbf24',
      emissive: '#f59e0b',
      emissiveIntensity: 0.35,
      metalness: 0.65,
      roughness: 0.2
    });
    const pedestal = new THREE.Mesh(new THREE.BoxGeometry(1.4, 0.6, 1.4), goldMat);
    const cup = new THREE.Mesh(new THREE.CylinderGeometry(0.9, 0.45, 1.4, 12), goldMat);
    cup.position.y = 1.0;
    trophyGroup.add(pedestal, cup);
    trophyGroup.position.set(6.5, 17.3, -20);
    this.plazaGroup.add(trophyGroup);
    this.trophyMesh = trophyGroup;
  }

  _buildDtiFashionRunway() {
    const rx = -24;
    this._addBoxPlatform({ x: -18.2, y: 0.14, z: 1.0, w: 2.0, h: 0.28, d: 6.0, color: '#fbcfe8', type: 'runway' });
    this._addBoxPlatform({ x: rx, y: 0.25, z: -9.5, w: 10.5, h: 0.5, d: 6.0, color: '#fdf2f8', type: 'runway' });
    this._addBoxPlatform({ x: rx, y: 0.25, z: 1.5, w: 5.4, h: 0.5, d: 16.0, color: '#fff1f2', type: 'runway' });
    this._addBoxPlatform({ x: rx, y: 0.28, z: 11.2, w: 7.4, h: 0.56, d: 5.6, color: '#fce7f3', type: 'runway' });

    const podiumDisc = new THREE.Mesh(
      new THREE.CylinderGeometry(2.7, 2.9, 0.12, 36),
      new THREE.MeshStandardMaterial({
        color: '#f472b6',
        emissive: '#ec4899',
        emissiveIntensity: 0.42,
        metalness: 0.35,
        roughness: 0.2
      })
    );
    podiumDisc.position.set(rx, 0.58, 11.2);
    this.plazaGroup.add(podiumDisc);

    const goldRailMat = new THREE.MeshStandardMaterial({
      color: '#fbbf24',
      emissive: '#f59e0b',
      emissiveIntensity: 0.55,
      metalness: 0.7,
      roughness: 0.2
    });
    for (const side of [-1, 1]) {
      const rail = new THREE.Mesh(new THREE.BoxGeometry(0.24, 0.12, 16.0), goldRailMat);
      rail.position.set(rx + side * 2.55, 0.54, 1.5);
      this.plazaGroup.add(rail);

      for (let i = 0; i < 8; i++) {
        const bulbMat = new THREE.MeshStandardMaterial({
          color: '#f472b6',
          emissive: '#ec4899',
          emissiveIntensity: 0.7
        });
        const bulb = new THREE.Mesh(new THREE.SphereGeometry(0.16, 10, 10), bulbMat);
        bulb.position.set(rx + side * 2.55, 0.64, -5.5 + i * 2.0);
        this.plazaGroup.add(bulb);
        this.runwayStripLights.push(bulb);
      }
    }

    const archMat = new THREE.MeshStandardMaterial({
      color: '#f9a8d4',
      emissive: '#db2777',
      emissiveIntensity: 0.28,
      metalness: 0.4,
      roughness: 0.22
    });
    for (const az of [-5.2, 1.5, 8.2]) {
      const arch = new THREE.Mesh(new THREE.TorusGeometry(3.4, 0.18, 12, 32, Math.PI), archMat);
      arch.position.set(rx, 0.5, az);
      this.plazaGroup.add(arch);
    }

    const wallMat = new THREE.MeshStandardMaterial({
      color: '#4c1d95',
      roughness: 0.38,
      metalness: 0.18
    });
    const wall = new THREE.Mesh(new THREE.BoxGeometry(10.2, 6.2, 0.55), wallMat);
    wall.position.set(rx, 3.3, -12.4);
    wall.castShadow = true;
    this.plazaGroup.add(wall);

    const frameTrim = new THREE.Mesh(new THREE.BoxGeometry(10.6, 6.6, 0.35), goldRailMat);
    frameTrim.position.set(rx, 3.3, -12.55);
    this.plazaGroup.add(frameTrim);

    const crestGroup = new THREE.Group();
    const starCore = new THREE.Mesh(new THREE.OctahedronGeometry(1.15, 0), goldRailMat);
    const starHalo = new THREE.Mesh(
      new THREE.TorusGeometry(1.55, 0.12, 12, 32),
      new THREE.MeshStandardMaterial({ color: '#f472b6', emissive: '#ec4899', emissiveIntensity: 0.65 })
    );
    crestGroup.add(starCore, starHalo);
    crestGroup.position.set(rx, 4.1, -11.95);
    this.plazaGroup.add(crestGroup);
    this.runwayStarCrest = crestGroup;
  }

  _buildDungeonEnvironment() {
    const cavernFloorMat = new THREE.MeshStandardMaterial({
      color: '#14102b',
      roughness: 0.65,
      metalness: 0.18
    });
    const cavernGround = new THREE.Mesh(new THREE.BoxGeometry(84, 2, 84), cavernFloorMat);
    cavernGround.position.set(0, -1, 0);
    cavernGround.receiveShadow = true;
    this.dungeonGroup.add(cavernGround);
    this.dungeonColliders.push({
      mesh: cavernGround,
      w: 84,
      h: 2,
      d: 84,
      minX: -42,
      maxX: 42,
      minZ: -42,
      maxZ: 42,
      topY: 0,
      bottomY: -2,
      type: 'ground'
    });

    // Atmospheric cavern point lights
    const centerGlow = new THREE.PointLight('#c084fc', 2.4, 52);
    centerGlow.position.set(0, 8, 2);
    const sanctumGlow = new THREE.PointLight('#38bdf8', 2.6, 48);
    sanctumGlow.position.set(0, 10, -24);
    this.dungeonGroup.add(centerGlow, sanctumGlow);

    // Glowing central runestone walkway & arrival circle
    this._addBoxPlatform(
      { x: 0, y: 0.04, z: 10, w: 6.4, h: 0.08, d: 34, color: '#271d54', emissive: '#6d28d9', emissiveIntensity: 0.22 },
      this.dungeonGroup,
      this.dungeonColliders
    );
    const arrivalDisc = new THREE.Mesh(
      new THREE.CylinderGeometry(3.2, 3.4, 0.12, 32),
      new THREE.MeshStandardMaterial({ color: '#38bdf8', emissive: '#0284c7', emissiveIntensity: 0.45 })
    );
    arrivalDisc.position.set(0, 0.07, 22);
    this.dungeonGroup.add(arrivalDisc);

    // Return Portal to Starlight Plaza at (0, 0, 28)
    this._createPortalArch(
      {
        x: this.dungeonReturnPortalPos.x,
        y: this.dungeonReturnPortalPos.y,
        z: this.dungeonReturnPortalPos.z,
        title: '🌟 STARLIGHT PLAZA',
        subtitle: 'Step In or Press E • Return to Plaza',
        primaryHex: '#38bdf8',
        secondaryHex: '#f472b6'
      },
      this.dungeonGroup
    );

    // Stepped platforms reaching every elevated Crystal Spire & Golden Vault Chest
    const addDungeonLedge = (x, topY, z, w, d, color = '#2e1065', emissive = '#7e22ce') =>
      this._addBoxPlatform(
        { x, y: topY / 2, z, w, h: topY, d, color, emissive, emissiveIntensity: 0.2 },
        this.dungeonGroup,
        this.dungeonColliders
      );

    // West & East Deep Prism Spire Terraces: (-16, 1.8, -16) and (16, 1.8, -16)
    addDungeonLedge(-16, 0.9, -9.5, 5.4, 4.4, '#312e81', '#4f46e5');
    addDungeonLedge(-16, 1.8, -16.0, 7.2, 7.2, '#3b0764', '#9333ea');
    addDungeonLedge(16, 0.9, -9.5, 5.4, 4.4, '#312e81', '#4f46e5');
    addDungeonLedge(16, 1.8, -16.0, 7.2, 7.2, '#3b0764', '#9333ea');

    // West Golden Vault Platform: (-18, 3.2, -26)
    addDungeonLedge(-17, 2.5, -21.2, 4.8, 4.0, '#4c1d95', '#a855f7');
    addDungeonLedge(-18, 3.2, -26.0, 7.6, 7.2, '#581c87', '#fbbf24');

    // Central Dais (0, 2.2, -18) & Royal Sanctum Platform (0, 4.5, -30)
    addDungeonLedge(0, 1.1, -12.5, 5.8, 4.2, '#312e81', '#6366f1');
    addDungeonLedge(0, 2.2, -18.0, 7.4, 6.4, '#3b0764', '#38bdf8');
    addDungeonLedge(0, 3.4, -23.5, 5.6, 4.4, '#4c1d95', '#c084fc');
    addDungeonLedge(0, 4.5, -30.0, 9.2, 8.2, '#581c87', '#fbbf24');

    // Perimeter giant bioluminescent crystal spires & cavern pillars
    const amethystMat = new THREE.MeshStandardMaterial({
      color: '#c084fc',
      emissive: '#9333ea',
      emissiveIntensity: 0.62,
      roughness: 0.2,
      metalness: 0.35
    });
    const cyanCrystalMat = new THREE.MeshStandardMaterial({
      color: '#38bdf8',
      emissive: '#0284c7',
      emissiveIntensity: 0.58,
      roughness: 0.2,
      metalness: 0.35
    });
    const spireCoords = [
      [-30, 16], [30, 16], [-32, -4], [32, -4],
      [-30, -24], [30, -24], [-10, -36], [10, -36],
      [-24, 28], [24, 28]
    ];
    spireCoords.forEach(([sx, sz], idx) => {
      const cluster = new THREE.Group();
      const mat = idx % 2 === 0 ? amethystMat : cyanCrystalMat;
      const mainSpire = new THREE.Mesh(new THREE.ConeGeometry(1.35, 6.4, 6), mat);
      mainSpire.position.y = 3.0;
      mainSpire.rotation.z = idx % 2 === 0 ? 0.12 : -0.12;
      const sideSpire = new THREE.Mesh(new THREE.ConeGeometry(0.85, 3.8, 6), mat);
      sideSpire.position.set(idx % 2 === 0 ? 1.1 : -1.1, 1.7, 0.6);
      sideSpire.rotation.z = idx % 2 === 0 ? -0.28 : 0.28;
      cluster.add(mainSpire, sideSpire);
      cluster.position.set(sx, 0, sz);
      this.dungeonGroup.add(cluster);
    });
  }

  loadBlocks(blocksArray) {
    for (const mesh of this.voxelMap.values()) {
      this.voxelGroup.remove(mesh);
    }
    this.voxelMap.clear();
    this.voxelColliders = [];

    for (const b of blocksArray || []) {
      this.addBlockMesh(b);
    }
    this._rebuildActiveColliders();
  }

  addBlockMesh({ x, y, z, color }) {
    const key = `${x},${y},${z}`;
    if (this.voxelMap.has(key)) {
      this.removeBlockMesh({ x, y, z });
    }
    const mat = new THREE.MeshStandardMaterial({ color: color || '#38bdf8', roughness: 0.45 });
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), mat);
    mesh.position.set(x, y - 0.5, z);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.userData.voxelCoords = { x, y, z };
    this.voxelGroup.add(mesh);
    this.voxelMap.set(key, mesh);

    this.voxelColliders.push({
      mesh,
      voxelKey: key,
      w: 1,
      h: 1,
      d: 1,
      minX: x - 0.5,
      maxX: x + 0.5,
      minZ: z - 0.5,
      maxZ: z + 0.5,
      topY: y,
      bottomY: y - 1,
      type: 'voxel'
    });
    this._rebuildActiveColliders();
  }

  removeBlockMesh({ x, y, z }) {
    const key = `${x},${y},${z}`;
    const mesh = this.voxelMap.get(key);
    if (mesh) {
      this.voxelGroup.remove(mesh);
      this.voxelMap.delete(key);
    }
    this.voxelColliders = this.voxelColliders.filter((c) => c.voxelKey !== key);
    this._rebuildActiveColliders();
  }

  initSelfPlayer(playerState) {
    if (this.selfAvatar) {
      this.scene.remove(this.selfAvatar.group);
    }
    this.selfAvatar = new WorldAvatar(playerState);
    this.scene.add(this.selfAvatar.group);
    this.setZone(Number(playerState?.zone) || 0, null);
  }

  upsertRemotePlayer(playerState) {
    if (!playerState || (this.selfAvatar && playerState.id === this.selfAvatar.id)) return;
    let av = this.remoteAvatars.get(playerState.id);
    if (!av) {
      av = new WorldAvatar(playerState);
      av.group.visible = av.zone === this.currentZone;
      this.remoteAvatars.set(playerState.id, av);
      this.scene.add(av.group);
    } else {
      if (playerState.zone !== undefined) {
        const nextZone = Number(playerState.zone) || 0;
        if (av.zone !== nextZone) {
          av.zone = nextZone;
          av.group.position.set(playerState.x, playerState.y, playerState.z);
        }
      }
      av.group.visible = av.zone === this.currentZone;
      av.targetPos.set(playerState.x, playerState.y, playerState.z);
      av.targetRy = playerState.ry || 0;
      av.anim = playerState.anim || 'idle';
      av.emote = playerState.emote || null;
      if (playerState.user) {
        const prevSig = JSON.stringify(av.user?.avatar || {});
        const nextSig = JSON.stringify(playerState.user?.avatar || {});
        av.user = playerState.user;
        if (prevSig !== nextSig) {
          av.updateAvatarAppearance(playerState.user.avatar || {});
        }
      }
    }
  }

  setRemotePlayerZone(id, zone, coords = null) {
    const av = this.remoteAvatars.get(id);
    if (!av) return;
    av.zone = Number(zone) || 0;
    if (coords && Number.isFinite(coords.x)) {
      av.targetPos.set(coords.x, coords.y || 0, coords.z || 0);
      av.group.position.copy(av.targetPos);
    }
    av.group.visible = av.zone === this.currentZone;
  }

  removeRemotePlayer(id) {
    const av = this.remoteAvatars.get(id);
    if (av) {
      this.scene.remove(av.group);
      this.remoteAvatars.delete(id);
    }
  }

  setBuffTimers({ speedBoostUntilMs, jumpBoostUntilMs }) {
    if (Number.isFinite(speedBoostUntilMs)) {
      this.speedBoostUntilMs = speedBoostUntilMs;
    }
    if (Number.isFinite(jumpBoostUntilMs)) {
      this.jumpBoostUntilMs = jumpBoostUntilMs;
    }
  }

  setZone(zone, spawnCoords = null) {
    const nextZone = Number(zone) === 1 ? 1 : 0;
    this.currentZone = nextZone;
    this.lastPortalTriggerAt = performance.now();

    if (nextZone === 1) {
      this.plazaGroup.visible = false;
      this.voxelGroup.visible = false;
      this.dungeonGroup.visible = true;
      this.ghostMesh.visible = false;
      this.scene.background.set('#0f0a28');
      this.scene.fog.color.set('#160d38');
      this.scene.fog.density = 0.014;
      this.hemiLight.color.set('#c4b5fd');
      this.hemiLight.groundColor.set('#1e1b4b');
      this.hemiLight.intensity = 0.75;
      this.sunLight.color.set('#e9d5ff');
      this.sunLight.intensity = 0.82;
      this.fillLight.color.set('#38bdf8');
      this.fillLight.intensity = 0.54;
      if (this.nearRunway) {
        this.nearRunway = false;
        this.onRunwayProximity(false);
      }
    } else {
      this.plazaGroup.visible = true;
      this.voxelGroup.visible = true;
      this.dungeonGroup.visible = false;
      this.scene.background.set('#7dd3fc');
      this.scene.fog.color.set('#bae6fd');
      this.scene.fog.density = 0.0078;
      this.hemiLight.color.set('#ffffff');
      this.hemiLight.groundColor.set('#fbcfe8');
      this.hemiLight.intensity = 0.92;
      this.sunLight.color.set('#fffbeb');
      this.sunLight.intensity = 1.28;
      this.fillLight.color.set('#fdf2f8');
      this.fillLight.intensity = 0.55;
    }

    this._rebuildActiveColliders();

    if (this.selfAvatar) {
      this.selfAvatar.zone = nextZone;
      if (spawnCoords && Number.isFinite(spawnCoords.x)) {
        this.selfAvatar.group.position.set(spawnCoords.x, (spawnCoords.y || 0) + 0.15, spawnCoords.z || 0);
        this.velocity.set(0, 0, 0);
        this.selfAvatar.emote = null;
        if (nextZone === 1) {
          this.selfAvatar.group.rotation.y = Math.PI;
          this.cameraYaw = 0;
        }
        this._emitMoveNow();
      }
    }

    for (const av of this.remoteAvatars.values()) {
      av.group.visible = av.zone === nextZone;
    }
  }

  _buildDungeonNodeEntry(node) {
    const group = new THREE.Group();
    group.position.set(node.x, node.y, node.z);
    let bodyGroup = new THREE.Group();
    let lidPivot = null;

    if (node.kind === 'wooden_chest' || node.kind === 'golden_chest') {
      const isGold = node.kind === 'golden_chest';
      const boxMat = new THREE.MeshStandardMaterial({
        color: isGold ? '#f59e0b' : '#92400e',
        emissive: isGold ? '#b45309' : '#000000',
        emissiveIntensity: isGold ? 0.32 : 0,
        metalness: isGold ? 0.68 : 0.18,
        roughness: isGold ? 0.22 : 0.65
      });
      const trimMat = new THREE.MeshStandardMaterial({
        color: isGold ? '#c084fc' : '#fbbf24',
        emissive: isGold ? '#9333ea' : '#d97706',
        emissiveIntensity: 0.45,
        metalness: 0.65,
        roughness: 0.2
      });
      const base = new THREE.Mesh(new THREE.BoxGeometry(1.35, 0.72, 0.95), boxMat);
      base.position.y = 0.36;
      base.castShadow = true;
      const strapL = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.76, 1.0), trimMat);
      strapL.position.set(-0.42, 0.36, 0);
      const strapR = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.76, 1.0), trimMat);
      strapR.position.set(0.42, 0.36, 0);
      const lockGem = new THREE.Mesh(new THREE.OctahedronGeometry(0.16, 0), trimMat);
      lockGem.position.set(0, 0.62, 0.52);

      lidPivot = new THREE.Group();
      lidPivot.position.set(0, 0.72, -0.46);
      const lid = new THREE.Mesh(new THREE.BoxGeometry(1.38, 0.34, 0.98), boxMat);
      lid.position.set(0, 0.17, 0.46);
      lidPivot.add(lid);

      bodyGroup.add(base, strapL, strapR, lockGem, lidPivot);
      const label = createWorldBannerSprite(
        isGold ? '🗝️ Golden Vault' : '📦 Supply Chest',
        'Press E to Open',
        isGold ? '#fbbf24' : '#38bdf8'
      );
      label.position.set(0, 1.95, 0);
      group.add(bodyGroup, label);
    } else {
      const crystalMat = new THREE.MeshStandardMaterial({
        color: '#e879f9',
        emissive: '#a855f7',
        emissiveIntensity: 0.68,
        roughness: 0.16,
        metalness: 0.32
      });
      const subMat = new THREE.MeshStandardMaterial({
        color: '#38bdf8',
        emissive: '#0284c7',
        emissiveIntensity: 0.55,
        roughness: 0.18,
        metalness: 0.28
      });
      const c1 = new THREE.Mesh(new THREE.ConeGeometry(0.46, 1.75, 6), crystalMat);
      c1.position.set(0, 0.88, 0);
      c1.rotation.z = 0.08;
      const c2 = new THREE.Mesh(new THREE.ConeGeometry(0.32, 1.2, 6), subMat);
      c2.position.set(-0.42, 0.58, 0.2);
      c2.rotation.z = 0.32;
      const c3 = new THREE.Mesh(new THREE.ConeGeometry(0.3, 1.1, 6), crystalMat);
      c3.position.set(0.4, 0.52, -0.18);
      c3.rotation.z = -0.3;
      bodyGroup.add(c1, c2, c3);

      const label = createWorldBannerSprite(`✨ ${node.label}`, 'Press E to Mine', '#c084fc');
      label.position.set(0, 2.35, 0);
      group.add(bodyGroup, label);
    }

    this.dungeonEntitiesGroup.add(group);
    return { id: node.id, data: node, group, bodyGroup, lidPivot };
  }

  _buildDungeonMobEntry(mob) {
    const group = new THREE.Group();
    group.position.set(mob.x, mob.y, mob.z);
    const visual = new THREE.Group();
    let leftWing = null;
    let rightWing = null;

    if (mob.kind === 'crystal_bat') {
      const bodyMat = new THREE.MeshStandardMaterial({
        color: '#312e81',
        emissive: '#4338ca',
        emissiveIntensity: 0.35,
        roughness: 0.3
      });
      const wingMat = new THREE.MeshStandardMaterial({
        color: '#38bdf8',
        emissive: '#a855f7',
        emissiveIntensity: 0.55,
        side: THREE.DoubleSide
      });
      const eyeMat = new THREE.MeshBasicMaterial({ color: '#fde047' });
      const body = new THREE.Mesh(new THREE.OctahedronGeometry(0.46, 1), bodyMat);
      body.position.y = 0.55;
      const earL = new THREE.Mesh(new THREE.ConeGeometry(0.14, 0.34, 4), wingMat);
      earL.position.set(-0.22, 0.96, 0);
      const earR = new THREE.Mesh(new THREE.ConeGeometry(0.14, 0.34, 4), wingMat);
      earR.position.set(0.22, 0.96, 0);
      const eyeL = new THREE.Mesh(new THREE.SphereGeometry(0.07, 8, 8), eyeMat);
      eyeL.position.set(-0.15, 0.62, 0.38);
      const eyeR = new THREE.Mesh(new THREE.SphereGeometry(0.07, 8, 8), eyeMat);
      eyeR.position.set(0.15, 0.62, 0.38);

      leftWing = new THREE.Mesh(new THREE.BoxGeometry(0.88, 0.42, 0.06), wingMat);
      leftWing.position.set(-0.62, 0.6, 0);
      rightWing = new THREE.Mesh(new THREE.BoxGeometry(0.88, 0.42, 0.06), wingMat);
      rightWing.position.set(0.62, 0.6, 0);

      visual.add(body, earL, earR, eyeL, eyeR, leftWing, rightWing);
    } else {
      const outerMat = new THREE.MeshStandardMaterial({
        color: '#7c3aed',
        emissive: '#6d28d9',
        emissiveIntensity: 0.38,
        transparent: true,
        opacity: 0.84,
        roughness: 0.2
      });
      const coreMat = new THREE.MeshStandardMaterial({
        color: '#f472b6',
        emissive: '#ec4899',
        emissiveIntensity: 0.72
      });
      const eyeMat = new THREE.MeshBasicMaterial({ color: '#ffffff' });
      const pupilMat = new THREE.MeshBasicMaterial({ color: '#0f172a' });

      const slimeCube = new THREE.Mesh(new THREE.BoxGeometry(1.12, 0.92, 1.12), outerMat);
      slimeCube.position.y = 0.46;
      const core = new THREE.Mesh(new THREE.OctahedronGeometry(0.28, 0), coreMat);
      core.position.y = 0.46;
      for (const sx of [-0.22, 0.22]) {
        const eye = new THREE.Mesh(new THREE.BoxGeometry(0.18, 0.18, 0.06), eyeMat);
        eye.position.set(sx, 0.56, 0.56);
        const pupil = new THREE.Mesh(new THREE.BoxGeometry(0.09, 0.09, 0.07), pupilMat);
        pupil.position.set(sx, 0.55, 0.58);
        visual.add(eye, pupil);
      }
      visual.add(slimeCube, core);
    }

    const hearts = '♥'.repeat(Math.max(1, mob.hp || 1));
    const label = createWorldBannerSprite(`${mob.name} ${hearts}`, 'Jump-Stomp or Press E!', '#f472b6');
    label.position.set(0, 1.92, 0);
    group.add(visual, label);
    this.dungeonEntitiesGroup.add(group);

    return {
      id: mob.id,
      data: mob,
      group,
      visual,
      leftWing,
      rightWing,
      label,
      lastHp: mob.hp,
      targetPos: new THREE.Vector3(mob.x, mob.y, mob.z),
      phaseOffset: Math.random() * 6.28
    };
  }

  _buildDungeonDropEntry(drop) {
    const group = new THREE.Group();
    group.position.set(drop.x, drop.y, drop.z);
    const rarityPalette = {
      common: '#38bdf8',
      uncommon: '#34d399',
      rare: '#f472b6',
      epic: '#fbbf24'
    };
    const hex = drop.itemId ? rarityPalette[drop.rarity] || '#38bdf8' : '#fbbf24';
    const orbMat = new THREE.MeshStandardMaterial({
      color: hex,
      emissive: hex,
      emissiveIntensity: 0.78,
      metalness: 0.45,
      roughness: 0.15
    });

    const mesh = drop.itemId
      ? new THREE.Mesh(new THREE.OctahedronGeometry(0.36, 0), orbMat)
      : new THREE.Mesh(new THREE.CylinderGeometry(0.34, 0.34, 0.12, 18), orbMat);
    if (!drop.itemId) {
      mesh.rotation.z = Math.PI / 2;
    }
    mesh.position.y = 0.55;

    const ring = new THREE.Mesh(
      new THREE.RingGeometry(0.38, 0.52, 20),
      new THREE.MeshBasicMaterial({ color: hex, side: THREE.DoubleSide, transparent: true, opacity: 0.62 })
    );
    ring.rotation.x = -Math.PI / 2;
    ring.position.y = 0.06;

    const label = createWorldBannerSprite(drop.label, '', hex);
    label.position.set(0, 1.38, 0);
    group.add(mesh, ring, label);
    this.dungeonEntitiesGroup.add(group);

    return { id: drop.id, data: drop, group, mesh, lastPickupAttemptAt: 0 };
  }

  syncDungeonSnapshot(dungeon) {
    if (!dungeon) return;
    if (Array.isArray(dungeon.portalPos) && dungeon.portalPos.length === 3) {
      this.plazaPortalPos = { x: dungeon.portalPos[0], y: dungeon.portalPos[1], z: dungeon.portalPos[2] };
    }
    if (Array.isArray(dungeon.returnPortalPos) && dungeon.returnPortalPos.length === 3) {
      this.dungeonReturnPortalPos = {
        x: dungeon.returnPortalPos[0],
        y: dungeon.returnPortalPos[1],
        z: dungeon.returnPortalPos[2]
      };
    }

    // Sync Crystal Veins & Treasure Chests
    for (const node of dungeon.nodes || []) {
      let entry = this.dungeonNodesMap.get(node.id);
      if (!entry) {
        entry = this._buildDungeonNodeEntry(node);
        this.dungeonNodesMap.set(node.id, entry);
      }
      entry.data = node;
      entry.group.position.set(node.x, node.y, node.z);
      if (entry.lidPivot) {
        entry.lidPivot.rotation.x = node.available ? 0 : -0.92;
        entry.bodyGroup.scale.setScalar(node.available ? 1.0 : 0.92);
      } else {
        entry.bodyGroup.scale.setScalar(node.available ? 1.0 : 0.38);
      }
    }

    // Sync Mobs
    this.syncDungeonMobs(dungeon.mobs || []);

    // Sync Ground Loot Drops
    const nextDropIds = new Set();
    for (const drop of dungeon.drops || []) {
      nextDropIds.add(drop.id);
      if (!this.dungeonDropsMap.has(drop.id)) {
        this.dungeonDropsMap.set(drop.id, this._buildDungeonDropEntry(drop));
      } else {
        const existing = this.dungeonDropsMap.get(drop.id);
        existing.data = drop;
      }
    }
    for (const [dropId, entry] of this.dungeonDropsMap.entries()) {
      if (!nextDropIds.has(dropId)) {
        this.dungeonEntitiesGroup.remove(entry.group);
        this.dungeonDropsMap.delete(dropId);
      }
    }
  }

  syncDungeonMobs(mobs) {
    if (!Array.isArray(mobs)) return;
    for (const mob of mobs) {
      let entry = this.dungeonMobsMap.get(mob.id);
      if (!entry) {
        entry = this._buildDungeonMobEntry(mob);
        this.dungeonMobsMap.set(mob.id, entry);
      }
      entry.data = mob;
      entry.targetPos.set(mob.x, mob.y, mob.z);
      entry.group.visible = Boolean(mob.alive);
      if (entry.lastHp !== mob.hp && mob.alive) {
        entry.lastHp = mob.hp;
        const hearts = '♥'.repeat(Math.max(1, mob.hp || 1));
        entry.group.remove(entry.label);
        entry.label = createWorldBannerSprite(`${mob.name} ${hearts}`, 'Jump-Stomp or Press E!', '#f472b6');
        entry.label.position.set(0, 1.92, 0);
        entry.group.add(entry.label);
      }
    }
  }

  spawnSparkleBurst(originPos, hexColor = '#f472b6') {
    const ring = new THREE.Mesh(
      new THREE.TorusGeometry(0.55, 0.08, 8, 28),
      new THREE.MeshBasicMaterial({ color: hexColor, transparent: true, opacity: 0.92 })
    );
    ring.rotation.x = Math.PI / 2;
    ring.position.set(originPos.x, originPos.y + 0.45, originPos.z);
    this.scene.add(ring);
    this.sparkleRings.push({ mesh: ring, bornAt: performance.now() });
  }

  triggerAction() {
    if (!this.selfAvatar || this.salonMirrorMode) return;
    const pos = this.selfAvatar.group.position;
    this.spawnSparkleBurst(pos, this.currentZone === 1 ? '#c084fc' : '#f472b6');

    if (this.currentZone === 0) {
      const dPortal = Math.hypot(pos.x - this.plazaPortalPos.x, pos.z - this.plazaPortalPos.z);
      if (dPortal < 4.2) {
        this.lastPortalTriggerAt = performance.now();
        this.onRequestZonePortal(1);
      }
      return;
    }

    const dReturn = Math.hypot(pos.x - this.dungeonReturnPortalPos.x, pos.z - this.dungeonReturnPortalPos.z);
    if (dReturn < 4.0) {
      this.lastPortalTriggerAt = performance.now();
      this.onRequestZonePortal(0);
      return;
    }

    let nearestMob = null;
    let nearestMobDist = 5.4;
    for (const entry of this.dungeonMobsMap.values()) {
      if (!entry.data.alive) continue;
      const d = pos.distanceTo(entry.group.position);
      if (d < nearestMobDist) {
        nearestMobDist = d;
        nearestMob = entry.data;
      }
    }
    if (nearestMob) {
      this.onAttackMob(nearestMob.id, 'blast');
      return;
    }

    let nearestNode = null;
    let nearestNodeDist = 5.8;
    for (const entry of this.dungeonNodesMap.values()) {
      if (!entry.data.available) continue;
      const d = pos.distanceTo(entry.group.position);
      if (d < nearestNodeDist) {
        nearestNodeDist = d;
        nearestNode = entry.data;
      }
    }
    if (nearestNode) {
      this.onInteractNode(nearestNode.id);
    }
  }

  triggerEmote(emoteName) {
    if (!this.selfAvatar) return;
    this.selfAvatar.emote = this.selfAvatar.emote === emoteName ? null : emoteName;
    this._emitMoveNow();
    return this.selfAvatar.emote;
  }

  setSalonMirrorMode(enabled) {
    if (!this.selfAvatar) return;
    if (enabled && !this.salonMirrorMode) {
      this.salonMirrorMode = true;
      this.savedCameraBeforeSalon = {
        yaw: this.cameraYaw,
        pitch: this.cameraPitch,
        dist: this.cameraDist
      };
      this.cameraYaw = this.selfAvatar.group.rotation.y;
      this.cameraPitch = 0.14;
      this.cameraDist = 4.65;
    } else if (!enabled && this.salonMirrorMode) {
      this.salonMirrorMode = false;
      if (this.savedCameraBeforeSalon) {
        this.cameraYaw = this.savedCameraBeforeSalon.yaw;
        this.cameraPitch = this.savedCameraBeforeSalon.pitch;
        this.cameraDist = this.savedCameraBeforeSalon.dist;
        this.savedCameraBeforeSalon = null;
      }
    }
  }

  warpToPlaza() {
    if (!this.selfAvatar) return;
    if (this.currentZone !== 0) {
      this.onRequestZonePortal(0);
      return;
    }
    this.selfAvatar.group.position.set(0, 0.5, 0);
    this.velocity.set(0, 0, 0);
    this.selfAvatar.emote = null;
    this._emitMoveNow();
  }

  warpToRunway() {
    if (!this.selfAvatar) return;
    if (this.currentZone !== 0) {
      this.onRequestZonePortal(0);
      return;
    }
    this.selfAvatar.group.position.set(-24, 0.85, -8.5);
    this.selfAvatar.group.rotation.y = 0;
    this.cameraYaw = Math.PI;
    this.cameraPitch = 0.28;
    this.velocity.set(0, 0, 0);
    this.selfAvatar.emote = null;
    this._emitMoveNow();
  }

  warpToObbyCheckpoint() {
    if (!this.selfAvatar) return;
    if (this.currentZone !== 0) {
      this.onRequestZonePortal(0);
      return;
    }
    this.selfAvatar.group.position.set(
      this.currentCheckpoint.x,
      this.currentCheckpoint.y + 0.6,
      this.currentCheckpoint.z
    );
    this.velocity.set(0, 0, 0);
    this.selfAvatar.emote = null;
    this._emitMoveNow();
  }

  spawnCelebrationConfetti(originPos) {
    const colors = ['#f472b6', '#38bdf8', '#fbbf24', '#34d399', '#a78bfa'];
    for (let i = 0; i < 45; i++) {
      const mat = new THREE.MeshBasicMaterial({ color: colors[i % colors.length] });
      const p = new THREE.Mesh(new THREE.BoxGeometry(0.22, 0.22, 0.22), mat);
      p.position.copy(originPos).add(new THREE.Vector3(0, 2.5, 0));
      this.scene.add(p);
      this.confettiParticles.push({
        mesh: p,
        vel: new THREE.Vector3(
          (Math.random() - 0.5) * 10,
          Math.random() * 10 + 4,
          (Math.random() - 0.5) * 10
        ),
        bornAt: performance.now()
      });
    }
  }

  _bindEvents() {
    window.addEventListener('resize', () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });

    window.addEventListener('keydown', (e) => {
      const activeTag = document.activeElement ? document.activeElement.tagName : '';
      if (activeTag === 'INPUT' || activeTag === 'TEXTAREA' || activeTag === 'SELECT') return;
      if (document.querySelector('dialog:modal')) return;

      const code = e.code;
      if (!this.salonMirrorMode) {
        this.keys.add(code);
        if (code === 'Space') {
          e.preventDefault();
          this.jump();
        } else if (code === 'KeyE') {
          e.preventDefault();
          this.triggerAction();
        }
      }
    });

    window.addEventListener('keyup', (e) => {
      this.keys.delete(e.code);
    });

    const canvas = this.renderer.domElement;
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    canvas.addEventListener('pointerdown', (e) => {
      this.isDraggingCamera = true;
      this.dragMoved = false;
      this.lastPointer.x = e.clientX;
      this.lastPointer.y = e.clientY;
    });

    window.addEventListener('pointermove', (e) => {
      const rect = canvas.getBoundingClientRect();
      this.pointerNdc.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      this.pointerNdc.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;

      if (this.isDraggingCamera) {
        const dx = e.clientX - this.lastPointer.x;
        const dy = e.clientY - this.lastPointer.y;
        if (Math.hypot(dx, dy) > 4) {
          this.dragMoved = true;
        }
        this.cameraYaw -= dx * 0.0065;
        this.cameraPitch = Math.max(0.05, Math.min(1.25, this.cameraPitch + dy * 0.0055));
        this.lastPointer.x = e.clientX;
        this.lastPointer.y = e.clientY;
      }

      if (this.buildMode && !this.salonMirrorMode) {
        this._updateBuildGhost();
      }
    });

    window.addEventListener('pointerup', (e) => {
      const wasClick = this.isDraggingCamera && !this.dragMoved && e.target === canvas;
      this.isDraggingCamera = false;

      if (wasClick && !this.salonMirrorMode) {
        if (this.currentZone === 1) {
          this.triggerAction();
        } else if (this.buildMode) {
          if (e.button === 2 || this.buildTool === 'remove') {
            this._handleVoxelRemoveClick();
          } else {
            this._handleVoxelPlaceClick();
          }
        }
      }
    });

    canvas.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const minDist = this.salonMirrorMode ? 2.6 : 3.5;
        const maxDist = this.salonMirrorMode ? 8.5 : 22;
        this.cameraDist = Math.max(minDist, Math.min(maxDist, this.cameraDist + e.deltaY * 0.01));
      },
      { passive: false }
    );
  }

  _updateBuildGhost() {
    if (!this.buildMode || this.currentZone !== 0) {
      this.ghostMesh.visible = false;
      return;
    }
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const staticSurfaces = this.colliders
      .filter((c) => c.mesh && c.type !== 'voxel' && c.type !== 'jumppad')
      .map((c) => c.mesh);
    const targets = [...staticSurfaces, ...this.voxelGroup.children];
    const hits = this.raycaster.intersectObjects(targets, false);
    if (hits.length === 0 || hits[0].distance > 36) {
      this.ghostMesh.visible = false;
      return;
    }
    const hit = hits[0];
    if (this.buildTool === 'remove' && hit.object.userData.voxelCoords) {
      const { x, y, z } = hit.object.userData.voxelCoords;
      this.ghostMesh.position.set(x, y - 0.5, z);
      this.ghostMesh.material.color.set('#ef4444');
      this.ghostMesh.visible = true;
      return;
    }

    const p = hit.point.clone().addScaledVector(hit.face.normal, 0.48);
    const gx = Math.round(p.x);
    const gy = Math.max(1, Math.round(p.y + 0.5));
    const gz = Math.round(p.z);
    this.ghostMesh.position.set(gx, gy - 0.5, gz);
    this.ghostMesh.material.color.set(this.selectedBlockColor);
    this.ghostMesh.visible = true;
  }

  _handleVoxelPlaceClick() {
    this._updateBuildGhost();
    if (!this.ghostMesh.visible) return;
    const x = Math.round(this.ghostMesh.position.x);
    const y = Math.round(this.ghostMesh.position.y + 0.5);
    const z = Math.round(this.ghostMesh.position.z);
    this.onPlaceBlock({ x, y, z, color: this.selectedBlockColor });
  }

  _handleVoxelRemoveClick() {
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const hits = this.raycaster.intersectObjects(this.voxelGroup.children, false);
    if (hits.length > 0 && hits[0].object.userData.voxelCoords) {
      this.onRemoveBlock(hits[0].object.userData.voxelCoords);
    }
  }

  jump() {
    if (!this.selfAvatar) return;
    if (this.isGrounded) {
      const serverNow = Date.now() + this.serverClockOffsetMs;
      const jumpMult = serverNow < this.jumpBoostUntilMs ? 1.3 : 1.0;
      this.velocity.y = 12.2 * jumpMult;
      this.isGrounded = false;
      this.selfAvatar.emote = null;
    }
  }

  _emitMoveNow() {
    if (!this.selfAvatar) return;
    const p = this.selfAvatar.group.position;
    this.onMove({
      x: Math.round(p.x * 100) / 100,
      y: Math.round(p.y * 100) / 100,
      z: Math.round(p.z * 100) / 100,
      ry: Math.round(this.selfAvatar.group.rotation.y * 100) / 100,
      anim: this.selfAvatar.anim,
      emote: this.selfAvatar.emote,
      zone: this.currentZone
    });
  }

  setServerClockOffset(offsetMs) {
    if (Number.isFinite(offsetMs)) {
      this.serverClockOffsetMs = offsetMs;
    }
  }

  _animate(now) {
    requestAnimationFrame(this._animate);
    const dt = Math.min(0.05, (now - this.lastFrameTime) / 1000);
    this.lastFrameTime = now;

    const serverNowMs = Date.now() + this.serverClockOffsetMs;
    const tSec = serverNowMs * 0.001;

    for (const pv of this.portalVortices) {
      pv.ring.rotation.z = tSec * 2.4;
      pv.disc.material.opacity = 0.56 + Math.sin(tSec * 4.2) * 0.16;
      pv.keystone.rotation.y = tSec * 1.9;
    }

    for (let i = this.sparkleRings.length - 1; i >= 0; i--) {
      const sr = this.sparkleRings[i];
      const age = (now - sr.bornAt) / 420;
      if (age >= 1) {
        this.scene.remove(sr.mesh);
        this.sparkleRings.splice(i, 1);
        continue;
      }
      const sc = 1 + age * 4.8;
      sr.mesh.scale.set(sc, sc, 1);
      sr.mesh.material.opacity = (1 - age) * 0.9;
    }

    if (this.currentZone === 0) {
      for (const mp of this.movingPlatforms) {
        const prevX = mp.mesh.position.x;
        const prevZ = mp.mesh.position.z;
        const nx = mp.baseX + Math.sin(tSec * mp.speed) * mp.ampX;
        const nz = mp.baseZ + Math.cos(tSec * mp.speed) * mp.ampZ;
        mp.dx = nx - prevX;
        mp.dz = nz - prevZ;
        mp.mesh.position.x = nx;
        mp.mesh.position.z = nz;
        mp.minX = nx - mp.w / 2;
        mp.maxX = nx + mp.w / 2;
        mp.minZ = nz - mp.d / 2;
        mp.maxZ = nz + mp.d / 2;
      }

      if (this.trophyMesh) {
        this.trophyMesh.rotation.y = tSec * 1.8;
        this.trophyMesh.position.y = 17.3 + Math.sin(tSec * 3) * 0.18;
      }
      if (this.runwayStarCrest) {
        this.runwayStarCrest.children[0].rotation.y = tSec * 1.4;
        this.runwayStarCrest.children[1].rotation.z = -tSec * 0.8;
      }
      if (Math.floor(tSec * 3) !== this._lastDiscoTick) {
        this._lastDiscoTick = Math.floor(tSec * 3);
        const cols = ['#f472b6', '#38bdf8', '#fbbf24', '#a78bfa', '#34d399'];
        this.discoTiles.forEach((tile, i) => {
          const c = cols[(i + this._lastDiscoTick) % cols.length];
          tile.material.color.set(c);
          tile.material.emissive.set(c);
        });
        this.runwayStripLights.forEach((bulb, idx) => {
          const active = idx % 4 === this._lastDiscoTick % 4;
          bulb.material.emissiveIntensity = active ? 1.15 : 0.35;
        });
      }
    } else {
      // Animate Crystal Cavern mobs, nodes, and spinning loot orbs
      for (const entry of this.dungeonMobsMap.values()) {
        if (!entry.data.alive) continue;
        entry.group.position.lerp(entry.targetPos, Math.min(1, dt * 8.5));
        entry.group.rotation.y = entry.data.ry || 0;
        if (entry.data.kind === 'crystal_bat') {
          const flap = Math.sin(tSec * 12 + entry.phaseOffset) * 0.48;
          if (entry.leftWing) entry.leftWing.rotation.z = flap;
          if (entry.rightWing) entry.rightWing.rotation.z = -flap;
          entry.visual.position.y = Math.sin(tSec * 4.2 + entry.phaseOffset) * 0.16;
        } else {
          const hop = Math.abs(Math.sin(tSec * 4.5 + entry.phaseOffset));
          entry.visual.position.y = hop * 0.34;
          entry.visual.scale.set(1 + (1 - hop) * 0.1, 0.88 + hop * 0.2, 1 + (1 - hop) * 0.1);
        }
      }

      for (const dropEntry of this.dungeonDropsMap.values()) {
        dropEntry.mesh.rotation.y = tSec * 2.8;
        dropEntry.mesh.position.y = 0.52 + Math.sin(tSec * 3.6) * 0.14;
      }
    }

    for (let i = this.confettiParticles.length - 1; i >= 0; i--) {
      const cp = this.confettiParticles[i];
      if (now - cp.bornAt > 3500) {
        this.scene.remove(cp.mesh);
        this.confettiParticles.splice(i, 1);
        continue;
      }
      cp.vel.y -= 9.8 * dt;
      cp.mesh.position.addScaledVector(cp.vel, dt);
      cp.mesh.rotation.x += 5 * dt;
      cp.mesh.rotation.y += 4 * dt;
    }

    if (this.selfAvatar) {
      const pos = this.selfAvatar.group.position;
      let moveForward = 0;
      let moveRight = 0;

      if (!this.salonMirrorMode) {
        if (this.keys.has('KeyW') || this.keys.has('ArrowUp') || this.touchMove.forward) moveForward += 1;
        if (this.keys.has('KeyS') || this.keys.has('ArrowDown') || this.touchMove.backward) moveForward -= 1;
        if (this.keys.has('KeyA') || this.keys.has('ArrowLeft') || this.touchMove.left) moveRight -= 1;
        if (this.keys.has('KeyD') || this.keys.has('ArrowRight') || this.touchMove.right) moveRight += 1;
      }

      const isSprinting = this.keys.has('ShiftLeft') || this.keys.has('ShiftRight');
      const speedMult = serverNowMs < this.speedBoostUntilMs ? 1.35 : 1.0;
      const speed = (isSprinting ? 11.4 : 8.4) * speedMult;
      const forwardVec = new THREE.Vector3(-Math.sin(this.cameraYaw), 0, -Math.cos(this.cameraYaw));
      const rightVec = new THREE.Vector3(Math.cos(this.cameraYaw), 0, -Math.sin(this.cameraYaw));

      const moveDir = new THREE.Vector3();
      moveDir.addScaledVector(forwardVec, moveForward);
      moveDir.addScaledVector(rightVec, moveRight);

      if (moveDir.lengthSq() > 0.001) {
        moveDir.normalize();
        pos.x += moveDir.x * speed * dt;
        pos.z += moveDir.z * speed * dt;
        this.selfAvatar.group.rotation.y = Math.atan2(moveDir.x, moveDir.z);
        this.selfAvatar.emote = null;
        this.selfAvatar.anim = this.isGrounded ? (isSprinting ? 'run' : 'walk') : 'jump';
      } else {
        this.selfAvatar.anim = this.isGrounded ? 'idle' : 'jump';
      }

      this.velocity.y -= 28.0 * dt;
      const prevFootY = pos.y;
      pos.y += this.velocity.y * dt;

      this.isGrounded = false;
      const playerRadius = 0.42;

      for (const col of this.colliders) {
        if (
          pos.x + playerRadius > col.minX &&
          pos.x - playerRadius < col.maxX &&
          pos.z + playerRadius > col.minZ &&
          pos.z - playerRadius < col.maxZ
        ) {
          if (this.velocity.y <= 0 && prevFootY >= col.topY - 0.35 && pos.y <= col.topY + 0.08) {
            pos.y = col.topY;
            this.velocity.y = 0;
            this.isGrounded = true;

            if (col.type === 'moving') {
              pos.x += col.dx || 0;
              pos.z += col.dz || 0;
            }

            if (col.type === 'jumppad') {
              this.velocity.y = col.bounceVel || 20;
              this.isGrounded = false;
              this.selfAvatar.anim = 'jump';
            }

            if (col.type === 'checkpoint' && col.checkpointIndex > this.currentCheckpoint.index) {
              this.currentCheckpoint = {
                x: col.mesh.position.x,
                y: col.topY,
                z: col.mesh.position.z,
                index: col.checkpointIndex
              };
              col.mesh.material.color.set('#22c55e');
              this.onObbyCheckpoint(col.checkpointIndex);
            }

            if (col.type === 'trophy-pad' && now - this.lastTrophyWinAt > 12000) {
              this.lastTrophyWinAt = now;
              this.currentCheckpoint = {
                x: col.mesh.position.x,
                y: col.topY,
                z: col.mesh.position.z,
                index: 3
              };
              this.spawnCelebrationConfetti(pos);
              this.onTrophyWin();
            }
          }
        }
      }

      if (pos.y < -14) {
        if (this.currentZone === 1) {
          pos.set(0, 0.6, 22);
          this.velocity.set(0, 0, 0);
          this._emitMoveNow();
        } else {
          this.warpToPlaza();
        }
      }

      // Zone-specific proximity, portal walk-in, jump-stomp, auto-pickup & contextual HUD prompt
      let nextHint = '';
      if (this.currentZone === 0) {
        const isNowNearRunway = pos.x < -14.5 && pos.x > -33.5 && pos.z > -15.5 && pos.z < 16.5;
        if (isNowNearRunway !== this.nearRunway) {
          this.nearRunway = isNowNearRunway;
          this.onRunwayProximity(isNowNearRunway);
        }
        const dPortal = Math.hypot(pos.x - this.plazaPortalPos.x, pos.z - this.plazaPortalPos.z);
        if (dPortal < 2.25 && now - this.lastPortalTriggerAt > 2600) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(1);
        } else if (dPortal < 5.2) {
          nextHint = '🌀 Step In or Press E — Enter Crystal Cavern Dungeon!';
        }
      } else {
        const dReturn = Math.hypot(pos.x - this.dungeonReturnPortalPos.x, pos.z - this.dungeonReturnPortalPos.z);
        if (dReturn < 2.25 && now - this.lastPortalTriggerAt > 2600) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(0);
        } else if (dReturn < 5.0) {
          nextHint = '🌟 Step In or Press E — Return to Starlight Plaza';
        }

        // Mario-style Jump-Stomp on live mobs + nearest mob hint
        for (const entry of this.dungeonMobsMap.values()) {
          if (!entry.data.alive) continue;
          const mPos = entry.group.position;
          const hDist = Math.hypot(pos.x - mPos.x, pos.z - mPos.z);
          if (
            this.velocity.y < -1.0 &&
            hDist < 1.55 &&
            pos.y >= mPos.y + 0.15 &&
            pos.y <= mPos.y + 2.1 &&
            now - (entry.lastStompAt || 0) > 280
          ) {
            entry.lastStompAt = now;
            this.velocity.y = 11.8;
            this.isGrounded = false;
            this.selfAvatar.anim = 'jump';
            this.spawnSparkleBurst(mPos, '#fbbf24');
            this.onAttackMob(entry.data.id, 'stomp');
          } else if (!nextHint && hDist < 5.0) {
            nextHint = `⚔️ Press E (Sparkle Blast) or Jump-Stomp ${entry.data.name}!`;
          }
        }

        // Nearest Crystal Vein / Treasure Chest hint
        if (!nextHint) {
          for (const entry of this.dungeonNodesMap.values()) {
            if (!entry.data.available) continue;
            if (pos.distanceTo(entry.group.position) < 5.5) {
              nextHint =
                entry.data.kind === 'golden_chest'
                  ? `🗝️ Press E to Unlock ${entry.data.label}`
                  : `✨ Press E to Harvest ${entry.data.label}`;
              break;
            }
          }
        }

        // Auto-pickup Ground Loot Orbs
        for (const dropEntry of this.dungeonDropsMap.values()) {
          if (
            pos.distanceTo(dropEntry.group.position) < 2.55 &&
            now - dropEntry.lastPickupAttemptAt > 650
          ) {
            dropEntry.lastPickupAttemptAt = now;
            this.spawnSparkleBurst(dropEntry.group.position, '#34d399');
            this.onPickupDrop(dropEntry.id);
          }
        }
      }

      if (nextHint !== this.lastHintText) {
        this.lastHintText = nextHint;
        this.onInteractionHint(nextHint || null);
      }

      this.selfAvatar.update(dt, true);

      const lookHeight = this.salonMirrorMode ? 1.58 : 2.15;
      const targetLookAt = new THREE.Vector3(pos.x, pos.y + lookHeight, pos.z);
      if (this.salonMirrorMode && window.innerWidth > 860) {
        const camRightX = Math.cos(this.cameraYaw);
        const camRightZ = -Math.sin(this.cameraYaw);
        targetLookAt.x += camRightX * 0.92;
        targetLookAt.z += camRightZ * 0.92;
      }

      const camX = targetLookAt.x + Math.sin(this.cameraYaw) * Math.cos(this.cameraPitch) * this.cameraDist;
      const camY = targetLookAt.y + Math.sin(this.cameraPitch) * this.cameraDist;
      const camZ = targetLookAt.z + Math.cos(this.cameraYaw) * Math.cos(this.cameraPitch) * this.cameraDist;
      this.camera.position.set(camX, Math.max(0.55, camY), camZ);
      this.camera.lookAt(targetLookAt);

      if (now - this.lastMoveSentAt > 55) {
        const sig = `${this.currentZone}:${pos.x.toFixed(2)},${pos.y.toFixed(2)},${pos.z.toFixed(2)},${this.selfAvatar.group.rotation.y.toFixed(2)},${this.selfAvatar.anim},${this.selfAvatar.emote}`;
        if (sig !== this._lastMoveSig) {
          this._lastMoveSig = sig;
          this.lastMoveSentAt = now;
          this._emitMoveNow();
        }
      }
    }

    for (const remoteAv of this.remoteAvatars.values()) {
      if (remoteAv.group.visible) {
        remoteAv.update(dt, false);
      }
    }

    this.renderer.render(this.scene, this.camera);
  }
}
