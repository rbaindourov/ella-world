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

export const BUILD_CATEGORIES = [
  { id: 'structure', icon: '🧱', label: 'Structure' },
  { id: 'kitchen', icon: '🍳', label: 'Kitchen' },
  { id: 'bathroom', icon: '🛁', label: 'Bathroom' },
  { id: 'bedroom', icon: '🛏️', label: 'Bedroom' },
  { id: 'loot', icon: '🏆', label: 'My Loot' }
];

export const BUILD_CATALOG = [
  // Structure
  { kind: 'block', category: 'structure', icon: '🧱', label: 'Wall / Floor', height: 1 },
  { kind: 'stairs', category: 'structure', icon: '🪜', label: 'Stairs', height: 1 },
  { kind: 'door', category: 'structure', icon: '🚪', label: 'Hinged Door', height: 2 },
  { kind: 'double_door', category: 'structure', icon: '🚪🚪', label: 'Double Door', height: 2, width: 2 },
  { kind: 'window', category: 'structure', icon: '🪟', label: 'Window', height: 1 },
  // Kitchen
  { kind: 'kitchen_counter', category: 'kitchen', icon: '🔪', label: 'Counter', height: 1 },
  { kind: 'kitchen_sink', category: 'kitchen', icon: '🚰', label: 'Sink', height: 1 },
  { kind: 'kitchen_stove', category: 'kitchen', icon: '🍳', label: 'Stove & Oven', height: 1 },
  { kind: 'kitchen_fridge', category: 'kitchen', icon: '🧊', label: 'Retro Fridge', height: 2 },
  { kind: 'dining_table', category: 'kitchen', icon: '🍽️', label: 'Dining Table', height: 1 },
  { kind: 'chair', category: 'kitchen', icon: '🪑', label: 'Cozy Chair', height: 1 },
  // Bathroom
  { kind: 'bath_tub', category: 'bathroom', icon: '🛁', label: 'Bubble Tub', height: 1 },
  { kind: 'bath_vanity', category: 'bathroom', icon: '🪞', label: 'Mirror Vanity', height: 1 },
  { kind: 'bath_toilet', category: 'bathroom', icon: '🚽', label: 'Toilet', height: 1 },
  // Bedroom & Living
  { kind: 'bed', category: 'bedroom', icon: '🛏️', label: 'Cozy Bed', height: 1 },
  { kind: 'sofa', category: 'bedroom', icon: '🛋️', label: 'Plush Sofa', height: 1 },
  { kind: 'nightstand_lamp', category: 'bedroom', icon: '🛋️', label: 'Nightstand Lamp', height: 1 },
  { kind: 'crystal_lamp', category: 'bedroom', icon: '🏮', label: 'Crystal Lamp', height: 1 },
  { kind: 'wardrobe', category: 'bedroom', icon: '🚪', label: 'Wardrobe', height: 2 },
  { kind: 'rug', category: 'bedroom', icon: '🧶', label: 'Starlight Rug', height: 1 }
];

const PROP_TEMPLATE_BY_KIND = {
  stairs: 'Prop_Stairs',
  kitchen_counter: 'Prop_Kitchen_Counter',
  kitchen_sink: 'Prop_Kitchen_Sink',
  kitchen_stove: 'Prop_Kitchen_Stove',
  kitchen_fridge: 'Prop_Kitchen_Fridge',
  dining_table: 'Prop_Dining_Table',
  chair: 'Prop_Chair',
  bath_tub: 'Prop_Bath_Tub',
  bath_vanity: 'Prop_Bath_Vanity',
  bath_toilet: 'Prop_Bath_Toilet',
  bed: 'Prop_Bed',
  sofa: 'Prop_Sofa',
  nightstand_lamp: 'Prop_Nightstand_Lamp',
  crystal_lamp: 'Prop_Crystal_Lamp',
  wardrobe: 'Prop_Wardrobe',
  rug: 'Prop_Rug'
};

const LOOT_TEMPLATE_BY_ITEM_ID = {
  legendary_ray_gun: 'Loot_Ray_Gun',
  crystal_blade: 'Loot_Crystal_Blade',
  starlight_wand: 'Loot_Starlight_Wand',
  aegis_heart_charm: 'Loot_Aegis_Charm',
  rift_beacon: 'Loot_Rift_Beacon',
  pet_egg: 'Loot_Pet_Egg',
  aurora_crystal_lamp: 'Loot_Aurora_Lamp',
  geode_lamp: 'Loot_Geode_Lamp',
  slime_lava_lamp: 'Loot_Slime_Lamp',
  bat_plushie: 'Loot_Bat_Plush',
  royal_chalice: 'Loot_Royal_Chalice',
  dragon_egg_relic: 'Loot_Dragon_Egg'
};

// Shared singleton loader for the Blender 5.2 Ella Forensics + Dress-to-Impress Wardrobe Pack
let dtiPackPromise = null;
let dtiPackAsset = null;

// Shared singleton loader for the Blender 5.2 Home Interiors, Doors/Windows & Legendary Loot Pack
let propsPackPromise = null;
let propsPackAsset = null;
const propsTemplates = new Map();

// Shared singleton loader for PolyHaven CC0 Obby PBR textures (512x512 seamless maps)
const obbyTexLoader = new THREE.TextureLoader();
const obbyTextureSets = {};

function getObbyTextureSet(style) {
  if (!style) return null;
  if (obbyTextureSets[style]) return obbyTextureSets[style];
  const base = `/ella-world/textures/obby/${style}`;
  const loadMap = (url, isColor) => {
    const tex = obbyTexLoader.load(url);
    tex.wrapS = THREE.RepeatWrapping;
    tex.wrapT = THREE.RepeatWrapping;
    if (isColor) tex.colorSpace = THREE.SRGBColorSpace;
    return tex;
  };
  const set = {
    map: loadMap(`${base}_diff.jpg`, true),
    normalMap: loadMap(`${base}_nor.jpg`, false),
    roughnessMap: loadMap(`${base}_rough.jpg`, false)
  };
  obbyTextureSets[style] = set;
  return set;
}

function createStargateEventHorizonMaterial(primaryHex = '#38bdf8', secondaryHex = '#7c3aed') {
  return new THREE.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
      uSurge: { value: 0 },
      uColorA: { value: new THREE.Color(primaryHex) },
      uColorB: { value: new THREE.Color(secondaryHex) }
    },
    vertexShader: `
      varying vec2 vUv;
      uniform float uTime;
      uniform float uSurge;
      void main() {
        vUv = uv;
        vec2 c = uv * 2.0 - 1.0;
        float r = length(c);
        float wave = sin(r * 18.0 - uTime * 4.2) * 0.055 * (1.0 - r * 0.65);
        float wave2 = cos(c.x * 11.0 + uTime * 3.1) * sin(c.y * 11.0 - uTime * 2.7) * 0.035;
        float surgeBulge = uSurge * (1.0 - r * r) * 0.52 * sin(uTime * 8.0 - r * 6.0);
        vec3 displaced = position + vec3(0.0, 0.0, wave + wave2 + surgeBulge);
        gl_Position = projectionMatrix * modelViewMatrix * vec4(displaced, 1.0);
      }
    `,
    fragmentShader: `
      varying vec2 vUv;
      uniform float uTime;
      uniform float uSurge;
      uniform vec3 uColorA;
      uniform vec3 uColorB;
      void main() {
        vec2 p = vUv * 2.0 - 1.0;
        float r = length(p);
        if (r > 1.0) discard;
        float ang = atan(p.y, p.x);
        float ripple1 = sin(r * 22.0 - uTime * 4.6 + sin(ang * 4.0 + uTime * 1.4) * 1.1);
        float ripple2 = cos(r * 14.0 - uTime * 3.1 - ang * 3.0);
        float caustic = pow(clamp(0.5 + 0.5 * (ripple1 * 0.6 + ripple2 * 0.4), 0.0, 1.0), 2.4);
        float core = exp(-r * 2.2) * (0.45 + 0.45 * uSurge);
        float rim = smoothstep(0.68, 0.99, r);
        vec3 col = mix(uColorB * 0.55, uColorA, caustic);
        col += vec3(0.85, 0.96, 1.0) * (caustic * 0.48 + core + rim * 0.65);
        col += vec3(0.45, 0.85, 1.0) * uSurge * 0.55;
        float alpha = clamp(0.76 + rim * 0.22 + core * 0.2, 0.0, 0.97);
        gl_FragColor = vec4(col, alpha);
      }
    `,
    transparent: true,
    side: THREE.DoubleSide,
    depthWrite: false
  });
}

// Shared singleton loader for the Blender 5.2 Realistic Trees & DTI Fashion Runway Stage Pack
let envPackPromise = null;
let envPackAsset = null;
const envTemplates = new Map();

export function loadEnvironmentPack() {
  if (envPackAsset) return Promise.resolve(envPackAsset);
  if (!envPackPromise) {
    const loader = new GLTFLoader();
    envPackPromise = new Promise((resolve, reject) => {
      loader.load(
        '/ella-world/models/ella_world_environment.glb',
        (gltf) => {
          envPackAsset = gltf;
          envTemplates.clear();
          for (const child of gltf.scene.children) {
            if (child.name) {
              envTemplates.set(child.name, child);
            }
          }
          gltf.scene.traverse((node) => {
            if (node.name && !envTemplates.has(node.name)) {
              envTemplates.set(node.name, node);
            }
          });
          resolve(gltf);
        },
        undefined,
        (err) => {
          console.warn('[EllaWorld] Could not load ella_world_environment.glb, using procedural env:', err);
          reject(err);
        }
      );
    });
  }
  return envPackPromise;
}

function cloneEnvTemplate(templateName) {
  const tpl = envTemplates.get(templateName);
  if (!tpl) return null;
  const clone = tpl.clone(true);
  clone.position.set(0, 0, 0);
  clone.rotation.set(0, 0, 0);
  clone.scale.set(1, 1, 1);
  clone.traverse((sub) => {
    if (sub.isMesh) {
      sub.castShadow = true;
      sub.receiveShadow = true;
      const fixMat = (m) => {
        if (!m) return;
        if (m.alphaTest > 0 || (m.name && m.name.includes('Leaves'))) {
          m.alphaTest = 0.42;
          m.side = THREE.DoubleSide;
          m.transparent = false;
          m.depthWrite = true;
        }
      };
      if (Array.isArray(sub.material)) {
        sub.material.forEach(fixMat);
      } else {
        fixMat(sub.material);
      }
    }
  });
  return clone;
}

export function loadPropsAndLootPack() {
  if (propsPackAsset) return Promise.resolve(propsPackAsset);
  if (!propsPackPromise) {
    const loader = new GLTFLoader();
    propsPackPromise = new Promise((resolve, reject) => {
      loader.load(
        '/ella-world/models/ella_world_props_loot.glb',
        (gltf) => {
          propsPackAsset = gltf;
          propsTemplates.clear();
          for (const child of gltf.scene.children) {
            if (child.name) {
              propsTemplates.set(child.name, child);
            }
          }
          gltf.scene.traverse((node) => {
            if (node.name && !propsTemplates.has(node.name)) {
              propsTemplates.set(node.name, node);
            }
          });
          resolve(gltf);
        },
        undefined,
        (err) => {
          console.warn('[EllaWorld] Could not load ella_world_props_loot.glb, using procedural props:', err);
          reject(err);
        }
      );
    });
  }
  return propsPackPromise;
}

function doubleDoorPartnerOffset(rot) {
  switch (((rot || 0) % 4 + 4) % 4) {
    case 0:
      return { dx: 1, dz: 0 };
    case 1:
      return { dx: 0, dz: -1 };
    case 2:
      return { dx: -1, dz: 0 };
    default:
      return { dx: 0, dz: 1 };
  }
}

function clonePropTemplate(templateName, tintHex = null, extraUserData = null, cloneGeo = false) {
  const tpl = propsTemplates.get(templateName);
  if (!tpl) return null;
  const clone = tpl.clone(true);
  clone.position.set(0, 0, 0);
  clone.rotation.set(0, 0, 0);
  clone.traverse((sub) => {
    if (extraUserData) {
      Object.assign(sub.userData, extraUserData);
    }
    if (sub.isMesh) {
      sub.castShadow = true;
      sub.receiveShadow = true;
      if (cloneGeo && sub.geometry) {
        sub.geometry = sub.geometry.clone();
      }
      const cloneMat = (m) => {
        if (!m) return m;
        const mc = m.clone();
        const mName = mc.name || '';
        if (tintHex && mName.startsWith('Mat_Prop_Tint')) {
          mc.color.set(tintHex);
        }
        if (mName.startsWith('Mat_Prop_Glass')) {
          mc.transparent = true;
          mc.opacity = 0.38;
          mc.depthWrite = false;
        } else if (mName.startsWith('Mat_Prop_Water')) {
          mc.transparent = true;
          mc.opacity = 0.78;
        }
        return mc;
      };
      if (Array.isArray(sub.material)) {
        sub.material = sub.material.map(cloneMat);
      } else if (sub.material) {
        sub.material = cloneMat(sub.material);
      }
    }
  });
  if (extraUserData) {
    Object.assign(clone.userData, extraUserData);
  }
  return clone;
}

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
  flip: 'Emote_Flip',
  sleep: 'Idle_Glam'
};

const ANIM_TO_CLIP = {
  idle: 'Idle_Glam',
  walk: 'Walk_Runway',
  run: 'Run_Obby',
  jump: 'Jump_Air',
  hover: 'Emote_Pose'
};

function createFaceTexture(faceType = 'happy', skinColor = '#ffdfc4', isMale = false) {
  const canvas = document.createElement('canvas');
  canvas.width = 256;
  canvas.height = 256;
  const ctx = canvas.getContext('2d');

  ctx.fillStyle = skinColor;
  ctx.fillRect(0, 0, 256, 256);

  if (!isMale && faceType !== 'hero') {
    ctx.fillStyle = 'rgba(244, 114, 182, 0.38)';
    ctx.beginPath();
    ctx.arc(58, 152, 20, 0, Math.PI * 2);
    ctx.arc(198, 152, 20, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.fillStyle = '#0f172a';
  ctx.strokeStyle = '#0f172a';
  ctx.lineWidth = 10;
  ctx.lineCap = 'round';

  if (faceType === 'hero') {
    // Strong straight masculine hero eyebrows
    ctx.fillStyle = '#1e1b18';
    ctx.fillRect(52, 74, 58, 12);
    ctx.fillRect(146, 74, 58, 12);
    // Sharp eyes + steel-blue Oblivion iris glint
    ctx.fillStyle = '#0f172a';
    ctx.beginPath();
    ctx.arc(82, 108, 14, 0, Math.PI * 2);
    ctx.arc(174, 108, 14, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#38bdf8';
    ctx.beginPath();
    ctx.arc(82, 110, 6, 0, Math.PI * 2);
    ctx.arc(174, 110, 6, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ffffff';
    ctx.beginPath();
    ctx.arc(77, 103, 4, 0, Math.PI * 2);
    ctx.arc(169, 103, 4, 0, Math.PI * 2);
    ctx.fill();
    // Confident asymmetric Jack Harper smirk
    ctx.strokeStyle = '#1e293b';
    ctx.lineWidth = 9;
    ctx.beginPath();
    ctx.moveTo(96, 168);
    ctx.quadraticCurveTo(132, 182, 166, 160);
    ctx.stroke();
  } else if (faceType === 'cool') {
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
  } else if (hatType === 'recon_cap') {
    const capMat = new THREE.MeshStandardMaterial({ color: hatColor || '#2c3e50', roughness: 0.52, metalness: 0.12 });
    const darkMat = new THREE.MeshStandardMaterial({ color: '#0f172a', roughness: 0.45, metalness: 0.25 });
    const patchMat = new THREE.MeshStandardMaterial({
      color: '#38bdf8',
      emissive: '#0284c7',
      emissiveIntensity: 0.45,
      roughness: 0.25
    });
    const crown = new THREE.Mesh(new THREE.BoxGeometry(0.98, 0.34, 0.98), capMat);
    crown.position.y = 0.48;
    group.add(crown);
    const visor = new THREE.Mesh(new THREE.BoxGeometry(0.86, 0.08, 0.52), darkMat);
    visor.position.set(0, 0.35, 0.62);
    visor.rotation.x = 0.12;
    group.add(visor);
    const patch = new THREE.Mesh(new THREE.BoxGeometry(0.34, 0.14, 0.05), patchMat);
    patch.position.set(0, 0.49, 0.5);
    group.add(patch);
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

function buildRayGunAttachment(accentHex = '#ec4899', cloneGeo = false) {
  const group = new THREE.Group();
  const glbGun = clonePropTemplate('Loot_Ray_Gun', accentHex, null, cloneGeo);
  if (glbGun) {
    glbGun.position.set(0, -0.14, 0.04);
    group.add(glbGun);
    group.userData.isBlenderMesh = true;
    return group;
  }

  // Instant procedural fallback while `ella_world_props_loot.glb` streams in
  const bodyMat = new THREE.MeshStandardMaterial({
    color: accentHex || '#ec4899',
    metalness: 0.72,
    roughness: 0.2,
    emissive: '#831843',
    emissiveIntensity: 0.25
  });
  const plasmaMat = new THREE.MeshStandardMaterial({
    color: '#22d3ee',
    emissive: '#06b6d4',
    emissiveIntensity: 1.4,
    roughness: 0.1
  });
  const goldMat = new THREE.MeshStandardMaterial({ color: '#fbbf24', metalness: 0.88, roughness: 0.18 });
  const grip = new THREE.Mesh(new THREE.BoxGeometry(0.08, 0.22, 0.1), new THREE.MeshStandardMaterial({ color: '#1e293b' }));
  grip.position.set(0, -0.04, -0.04);
  grip.rotation.x = 0.25;
  const receiver = new THREE.Mesh(new THREE.CylinderGeometry(0.075, 0.095, 0.34, 14), bodyMat);
  receiver.rotation.x = Math.PI / 2;
  receiver.position.set(0, 0.11, 0.06);
  const plasmaCore = new THREE.Mesh(new THREE.CylinderGeometry(0.098, 0.098, 0.1, 14), plasmaMat);
  plasmaCore.rotation.x = Math.PI / 2;
  plasmaCore.position.set(0, 0.11, 0.05);
  const barrel = new THREE.Mesh(new THREE.CylinderGeometry(0.025, 0.04, 0.28, 12), goldMat);
  barrel.rotation.x = Math.PI / 2;
  barrel.position.set(0, 0.11, 0.32);
  const muzzle = new THREE.Mesh(new THREE.SphereGeometry(0.045, 12, 10), plasmaMat);
  muzzle.position.set(0, 0.11, 0.47);
  group.add(grip, receiver, plasmaCore, barrel, muzzle);
  return group;
}

function buildCrystalBladeAttachment(accentHex = '#38bdf8', cloneGeo = false) {
  const group = new THREE.Group();
  const glbBlade = clonePropTemplate('Loot_Crystal_Blade', accentHex, null, cloneGeo);
  if (glbBlade) {
    glbBlade.position.set(0, -0.06, 0.04);
    glbBlade.rotation.x = Math.PI * 0.34;
    glbBlade.scale.setScalar(0.95);
    group.add(glbBlade);
    group.userData.isBlenderMesh = true;
    return group;
  }
  const hiltMat = new THREE.MeshStandardMaterial({ color: '#fbbf24', metalness: 0.85, roughness: 0.2 });
  const bladeMat = new THREE.MeshStandardMaterial({
    color: accentHex || '#38bdf8',
    emissive: '#0ea5e9',
    emissiveIntensity: 0.95,
    metalness: 0.4,
    roughness: 0.12
  });
  const grip = new THREE.Mesh(new THREE.CylinderGeometry(0.025, 0.025, 0.22, 8), hiltMat);
  const guard = new THREE.Mesh(new THREE.BoxGeometry(0.24, 0.04, 0.05), hiltMat);
  guard.position.y = 0.11;
  const blade = new THREE.Mesh(new THREE.ConeGeometry(0.055, 0.62, 4), bladeMat);
  blade.position.y = 0.43;
  const sub = new THREE.Group();
  sub.add(grip, guard, blade);
  sub.rotation.x = Math.PI * 0.42;
  group.add(sub);
  return group;
}

function buildStarlightWandAttachment(accentHex = '#f472b6', cloneGeo = false) {
  const group = new THREE.Group();
  const glbWand = clonePropTemplate('Loot_Starlight_Wand', accentHex, null, cloneGeo);
  if (glbWand) {
    glbWand.position.set(0, -0.08, 0.03);
    glbWand.rotation.x = Math.PI * 0.32;
    glbWand.scale.setScalar(0.95);
    group.add(glbWand);
    group.userData.isBlenderMesh = true;
    return group;
  }
  const shaftMat = new THREE.MeshStandardMaterial({ color: '#fde047', metalness: 0.82, roughness: 0.2 });
  const starMat = new THREE.MeshStandardMaterial({
    color: accentHex || '#f472b6',
    emissive: '#ec4899',
    emissiveIntensity: 1.1,
    roughness: 0.12
  });
  const shaft = new THREE.Mesh(new THREE.CylinderGeometry(0.02, 0.024, 0.56, 8), shaftMat);
  shaft.position.y = 0.2;
  const star = new THREE.Mesh(new THREE.OctahedronGeometry(0.095, 0), starMat);
  star.position.y = 0.5;
  const halo = new THREE.Mesh(new THREE.TorusGeometry(0.13, 0.015, 8, 20), shaftMat);
  halo.position.y = 0.5;
  const sub = new THREE.Group();
  sub.add(shaft, star, halo);
  sub.rotation.x = Math.PI * 0.36;
  group.add(sub);
  return group;
}

function buildCompanionMountVisual(mountId) {
  if (!mountId) return null;
  const group = new THREE.Group();
  const tplMap = {
    starlight_board: 'Env_Mount_Hoverboard',
    sunset_skates: 'Env_Mount_StarlightSkates',
    comet_surfer: 'Env_Mount_CometSurfer'
  };
  const glbMount = tplMap[mountId] ? cloneEnvTemplate(tplMap[mountId]) : null;
  if (glbMount) {
    group.add(glbMount);
    group.userData.isBlenderMesh = true;
  } else {
    const col = mountId === 'comet_surfer' ? '#a855f7' : mountId === 'sunset_skates' ? '#fb923c' : '#38bdf8';
    const deck = new THREE.Mesh(
      new THREE.BoxGeometry(0.56, 0.07, 1.38),
      new THREE.MeshStandardMaterial({ color: col, emissive: col, emissiveIntensity: 0.48, roughness: 0.2, metalness: 0.45 })
    );
    deck.position.y = 0.14;
    group.add(deck);
  }
  const glowRing = new THREE.Mesh(
    new THREE.RingGeometry(0.28, 0.52, 24),
    new THREE.MeshBasicMaterial({
      color: mountId === 'comet_surfer' ? '#f472b6' : '#38bdf8',
      transparent: true,
      opacity: 0.65,
      side: THREE.DoubleSide
    })
  );
  glowRing.rotation.x = -Math.PI / 2;
  glowRing.position.y = 0.04;
  group.add(glowRing);
  group.userData.glowRing = glowRing;
  return group;
}

function buildCompanionPetVisual(petId, petNickname = '') {
  if (!petId) return null;
  const root = new THREE.Group();
  const bodyWrap = new THREE.Group();
  root.add(bodyWrap);

  let leftWing = null;
  let rightWing = null;
  let hasBlender = false;

  if (petId === 'baby_crystal_dragon') {
    const body = cloneEnvTemplate('Env_Pet_BabyDragon_Body');
    const wL = cloneEnvTemplate('Env_Pet_BabyDragon_WingL');
    const wR = cloneEnvTemplate('Env_Pet_BabyDragon_WingR');
    if (body) {
      bodyWrap.add(body);
      if (wL) {
        wL.position.set(-0.16, 0.42, -0.02);
        bodyWrap.add(wL);
        leftWing = wL;
      }
      if (wR) {
        wR.position.set(0.16, 0.42, -0.02);
        bodyWrap.add(wR);
        rightWing = wR;
      }
      hasBlender = true;
    }
  } else {
    const tplMap = {
      cloud_puppy: 'Env_Pet_CloudPuppy',
      starlight_kitsune: 'Env_Pet_StarlightKitsune',
      shadow_kitty: 'Env_Pet_ShadowKitty'
    };
    const glbPet = tplMap[petId] ? cloneEnvTemplate(tplMap[petId]) : null;
    if (glbPet) {
      bodyWrap.add(glbPet);
      hasBlender = true;
    }
  }

  if (!hasBlender) {
    const palette = {
      cloud_puppy: '#f8fafc',
      starlight_kitsune: '#fb923c',
      shadow_kitty: '#312e81',
      baby_crystal_dragon: '#38bdf8'
    };
    const col = palette[petId] || '#f8fafc';
    const mat = new THREE.MeshStandardMaterial({ color: col, roughness: 0.35 });
    const torso = new THREE.Mesh(new THREE.SphereGeometry(0.24, 14, 12), mat);
    torso.position.y = 0.26;
    const head = new THREE.Mesh(new THREE.SphereGeometry(0.19, 14, 12), mat);
    head.position.set(0, 0.46, 0.16);
    bodyWrap.add(torso, head);
  }

  if (petNickname) {
    const tag = createWorldBannerSprite(`🐾 ${petNickname}`, '', '#f472b6');
    tag.scale.set(1.55, 0.34, 1);
    tag.position.set(0, 0.92, 0);
    root.add(tag);
  }

  root.userData = { petId, petNickname, bodyWrap, leftWing, rightWing, hasBlender };
  return root;
}

function createPlotMailboxTexture(plot) {
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = 176;
  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const claimed = Boolean(plot.ownerUsername);
  ctx.fillStyle = claimed ? 'rgba(15, 23, 42, 0.92)' : 'rgba(15, 23, 42, 0.84)';
  ctx.strokeStyle = claimed ? (plot.doorLocked ? '#fb7185' : '#34d399') : '#38bdf8';
  ctx.lineWidth = 5;
  ctx.beginPath();
  ctx.roundRect(12, 10, 488, 156, 26);
  ctx.fill();
  ctx.stroke();

  const title = plot.customTitle || plot.defaultName || plot.plotId;
  ctx.fillStyle = '#ffffff';
  ctx.font = 'bold 27px Inter, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(String(title).slice(0, 30), 256, 56);

  ctx.fillStyle = claimed ? '#fef08a' : '#bae6fd';
  ctx.font = '700 20px Inter, sans-serif';
  const sub = claimed
    ? `${plot.doorLocked ? '🔒 Locked • 🔔 Ring Doorbell (E)' : '🔓 Open Home'} • @${plot.ownerUsername}`
    : '🏡 Available Plot — Press E to Claim!';
  ctx.fillText(sub.slice(0, 42), 256, 116);

  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

function createObbyLeaderboardTexture(heading, records = [], accentHex = '#38bdf8') {
  const canvas = document.createElement('canvas');
  canvas.width = 640;
  canvas.height = 440;
  const ctx = canvas.getContext('2d');

  ctx.fillStyle = '#0f172a';
  ctx.strokeStyle = accentHex;
  ctx.lineWidth = 8;
  ctx.beginPath();
  ctx.roundRect(10, 10, 620, 420, 28);
  ctx.fill();
  ctx.stroke();

  ctx.fillStyle = accentHex;
  ctx.font = '900 32px Inter, sans-serif';
  ctx.textAlign = 'center';
  ctx.fillText(heading, 320, 58);

  ctx.fillStyle = '#cbd5e1';
  ctx.font = '700 18px Inter, sans-serif';
  ctx.fillText('Cross Green Start Gate on Foot → Summit Finish Gate!', 320, 92);

  if (!Array.isArray(records) || records.length === 0) {
    ctx.fillStyle = '#94a3b8';
    ctx.font = '700 24px Inter, sans-serif';
    ctx.fillText('⏱️ No times yet — set the #1 World Record!', 320, 245);
  } else {
    const medals = ['🥇 #1', '🥈 #2', '🥉 #3', '🏅 #4', '🏅 #5'];
    records.slice(0, 5).forEach((rec, idx) => {
      const ry = 138 + idx * 56;
      ctx.fillStyle = idx === 0 ? 'rgba(251, 191, 36, 0.22)' : 'rgba(30, 41, 59, 0.88)';
      ctx.beginPath();
      ctx.roundRect(36, ry - 22, 568, 46, 14);
      ctx.fill();

      ctx.fillStyle = idx === 0 ? '#fde047' : '#f8fafc';
      ctx.font = '800 23px Inter, sans-serif';
      ctx.textAlign = 'left';
      ctx.fillText(`${medals[idx]}  ${String(rec.displayName || rec.username).slice(0, 18)}`, 56, ry + 8);

      ctx.fillStyle = '#34d399';
      ctx.textAlign = 'right';
      ctx.fillText(rec.timeFormatted || `${(rec.elapsedMs / 1000).toFixed(2)}s`, 584, ry + 8);
    });
  }

  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

function createAuroraRibbonMaterial() {
  return new THREE.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
      uOpacity: { value: 0 }
    },
    vertexShader: `
      varying vec2 vUv;
      uniform float uTime;
      void main() {
        vUv = uv;
        vec3 p = position;
        p.x += sin(uv.x * 12.0 + uTime * 0.8) * 3.5;
        p.z += cos(uv.x * 9.0 - uTime * 0.65) * 3.5;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
      }
    `,
    fragmentShader: `
      varying vec2 vUv;
      uniform float uTime;
      uniform float uOpacity;
      void main() {
        if (uOpacity <= 0.01) discard;
        float wave = sin(vUv.x * 18.0 + uTime * 1.1) * 0.5 + 0.5;
        float curtain = pow(sin(vUv.y * 3.14159), 1.6);
        float streak = 0.55 + 0.45 * sin(vUv.x * 46.0 - uTime * 1.9);
        vec3 colA = vec3(0.18, 0.92, 0.68);
        vec3 colB = vec3(0.68, 0.36, 0.98);
        vec3 colC = vec3(0.96, 0.42, 0.76);
        vec3 col = mix(colA, colB, wave);
        col = mix(col, colC, 0.35 * sin(vUv.x * 7.0 + uTime * 0.5 + 1.2) + 0.35);
        float alpha = curtain * streak * uOpacity * 0.62;
        gl_FragColor = vec4(col, alpha);
      }
    `,
    transparent: true,
    side: THREE.DoubleSide,
    depthWrite: false,
    blending: THREE.AdditiveBlending
  });
}

function buildLootDecorVisual(lootItemId, tintHex, extraUserData) {
  const root = new THREE.Group();
  const stand = clonePropTemplate('Loot_Display_Pedestal', tintHex, extraUserData);
  let topY = 0.35;
  if (stand) {
    root.add(stand);
  } else {
    const ped = new THREE.Mesh(
      new THREE.CylinderGeometry(0.28, 0.34, 0.34, 8),
      new THREE.MeshStandardMaterial({ color: '#f1f5f9', roughness: 0.25, metalness: 0.1 })
    );
    ped.position.y = 0.17;
    ped.castShadow = true;
    ped.receiveShadow = true;
    if (extraUserData) Object.assign(ped.userData, extraUserData);
    root.add(ped);
  }

  const tplName = LOOT_TEMPLATE_BY_ITEM_ID[lootItemId];
  const relic = tplName ? clonePropTemplate(tplName, tintHex, extraUserData) : null;
  const trophyHolder = new THREE.Group();
  trophyHolder.position.y = topY;

  if (relic) {
    if (lootItemId === 'legendary_ray_gun') {
      relic.position.set(0, 0.08, -0.04);
      relic.rotation.x = -0.18;
      relic.scale.setScalar(1.12);
    } else if (lootItemId === 'crystal_blade' || lootItemId === 'starlight_wand') {
      relic.position.set(0, 0.06, 0);
      relic.scale.setScalar(0.88);
    }
    trophyHolder.add(relic);
  } else {
    // Stylized 3D display sculpture for classic dungeon materials/potions/cosmetics
    const palette = {
      star_shard: '#38bdf8',
      slime_gel: '#34d399',
      moon_opal: '#c084fc',
      dungeon_key: '#fbbf24',
      crown_relic: '#f59e0b',
      speed_potion: '#38bdf8',
      jump_potion: '#f472b6',
      starlight_potion: '#a855f7',
      prism_block: '#e879f9',
      pet_egg: '#fde047',
      crystal_blade: '#38bdf8',
      starlight_wand: '#f472b6',
      aegis_heart_charm: '#fb7185',
      rift_beacon: '#c084fc',
      aurora_crystal_lamp: '#34d399',
      cosmetic_hat_shadow_hood: '#334155',
      cosmetic_acc_star_lantern: '#fbbf24',
      cosmetic_hat_obsidian_crown: '#a855f7',
      cosmetic_acc_crystal_wings: '#38bdf8',
      legendary_ray_gun: '#22d3ee'
    };
    const col = palette[lootItemId] || tintHex || '#f472b6';
    if (lootItemId === 'legendary_ray_gun') {
      const rg = buildRayGunAttachment('#ec4899');
      rg.position.set(0, 0.14, 0);
      trophyHolder.add(rg);
    } else {
      const gem = new THREE.Mesh(
        new THREE.OctahedronGeometry(0.24, 0),
        new THREE.MeshStandardMaterial({
          color: col,
          emissive: col,
          emissiveIntensity: 0.65,
          metalness: 0.45,
          roughness: 0.15
        })
      );
      gem.position.y = 0.28;
      gem.castShadow = true;
      if (extraUserData) Object.assign(gem.userData, extraUserData);
      trophyHolder.add(gem);
    }
  }

  if (extraUserData) {
    Object.assign(trophyHolder.userData, extraUserData);
    Object.assign(root.userData, extraUserData);
  }
  root.add(trophyHolder);
  root.userData.trophyHolder = trophyHolder;
  return root;
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
  constructor(gltf, avatarConfig, options = {}) {
    this.cloneGeometries = Boolean(options.cloneGeometries);
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
      n === 'Body_Male' ||
      n === 'Face_Brows' ||
      n === 'Face_Brows_Male' ||
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
    this.rightHandBone = null;
    this.model.traverse((node) => {
      if (node.isBone) {
        if (node.name === 'head') this.headBone = node;
        else if (node.name === 'chest') this.chestBone = node;
        else if (node.name === 'hand.L') this.leftHandBone = node;
        else if (node.name === 'hand.R') this.rightHandBone = node;
      }
      if (node.isMesh) {
        node.castShadow = true;
        node.receiveShadow = true;
        node.frustumCulled = false;

        if (this.cloneGeometries && node.geometry) {
          node.geometry = node.geometry.clone();
        }

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

    this.dungeonRightHandMount = new THREE.Group();
    this.dungeonRightHandMount.scale.setScalar(0.52);
    if (this.rightHandBone) {
      this.dungeonRightHandMount.position.set(0, 0.08, 0.04);
      this.dungeonRightHandMount.rotation.set(Math.PI * 0.42, 0, 0);
      this.rightHandBone.add(this.dungeonRightHandMount);
    } else {
      this.dungeonRightHandMount.position.set(0.28, 0.96, 0.18);
      this.model.add(this.dungeonRightHandMount);
    }

    this.customCrystalWings = null;
    this.customStarLantern = null;
    this.customRayGun = null;

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

      this.slotsByName.set(node.name, { node, meshes: childMeshes, baseScale: node.scale.clone() });
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
    const isMale =
      av.bodyType === 'male' ||
      av.characterPreset === 'jack_tech49' ||
      av.characterPreset === 'maverick_cabin';

    const hairStyle = av.hairStyle || (isMale ? 'tech_crop' : 'twin_pigtails');
    const topStyle = av.topStyle || (isMale ? 'tech_jacket' : 'glam_gown');
    const bottomStyle = av.bottomStyle || (isMale ? 'tech_pants' : 'pleated_skirt');
    const shoeStyle = av.shoeStyle || (isMale ? 'flight_boots' : 'runway_heels');
    const hat = av.hat || (isMale ? 'none' : 'crown');
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
      if (slotName === 'Body_Male' || slotName === 'Face_Brows_Male') {
        isVisible = isMale;
      } else if (slotName === 'Body_Stylized' || slotName === 'Face_Brows') {
        isVisible = !isMale;
      } else if (
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

      if (slot.baseScale) {
        slot.node.scale.copy(slot.baseScale);
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
          } else if (mName.startsWith('Mat_Avatar_Lips')) {
            if (isMale) {
              const maleLip = new THREE.Color(skinColor).multiplyScalar(0.76);
              mat.color.copy(maleLip);
              mat.roughness = 0.55;
            } else {
              mat.color.set('#d85c7a');
              mat.roughness = 0.34;
            }
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

    while (this.dungeonRightHandMount.children.length > 0) {
      this.dungeonRightHandMount.remove(this.dungeonRightHandMount.children[0]);
    }
    this.customRayGun = null;
    this.customRightWeapon = null;
    if (accessory === 'ray_gun') {
      this.customRayGun = buildRayGunAttachment(accessoryColor, this.cloneGeometries);
      this.customRightWeapon = this.customRayGun;
      this.dungeonRightHandMount.add(this.customRayGun);
    } else if (accessory === 'crystal_blade') {
      this.customRightWeapon = buildCrystalBladeAttachment(accessoryColor, this.cloneGeometries);
      this.dungeonRightHandMount.add(this.customRightWeapon);
    } else if (accessory === 'starlight_wand') {
      this.customRightWeapon = buildStarlightWandAttachment(accessoryColor, this.cloneGeometries);
      this.dungeonRightHandMount.add(this.customRightWeapon);
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

    if (emoteState === 'sleep') {
      this.model.rotation.x = -Math.PI * 0.47;
      this.model.position.set(0, 0.56, 0.36);
    } else {
      this.model.rotation.x = 0;
      this.model.position.set(0, 0, 0);
    }

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
  constructor(avatarConfig, options = {}) {
    this.cloneGeometries = Boolean(options.cloneGeometries);
    this.root = new THREE.Group();
    this.bodyGroup = new THREE.Group();
    this.root.add(this.bodyGroup);
    this.applyAppearance(avatarConfig || {});
  }

  applyAppearance(av = {}) {
    while (this.bodyGroup.children.length > 0) {
      this.bodyGroup.remove(this.bodyGroup.children[0]);
    }

    const isMale =
      av.bodyType === 'male' ||
      av.characterPreset === 'jack_tech49' ||
      av.characterPreset === 'maverick_cabin';
    const skinColor = av.skinColor || '#ffdfc4';
    const torsoColor = av.topColor || av.torsoColor || '#f472b6';
    const legsColor = av.bottomColor || av.legsColor || '#6366f1';
    const hatType = av.hat || (isMale ? 'none' : 'crown');
    const hatColor = av.hatColor || '#fbbf24';
    const faceType = av.face || (isMale ? 'hero' : 'happy');

    const skinMat = new THREE.MeshStandardMaterial({ color: skinColor, roughness: 0.5 });
    const torsoMat = new THREE.MeshStandardMaterial({ color: torsoColor, roughness: 0.45 });
    const legsMat = new THREE.MeshStandardMaterial({ color: legsColor, roughness: 0.55 });
    const faceTex = createFaceTexture(faceType, skinColor, isMale);
    const faceMat = new THREE.MeshStandardMaterial({ map: faceTex, roughness: 0.45 });

    const headMaterials = [skinMat, skinMat, skinMat, skinMat, faceMat, skinMat];
    this.headMesh = new THREE.Mesh(new THREE.BoxGeometry(0.92, 0.92, 0.92), headMaterials);
    this.headMesh.position.set(0, 2.48, 0);
    this.headMesh.castShadow = true;

    if (av.accessory === 'aviators') {
      const frameMat = new THREE.MeshStandardMaterial({ color: '#cbd5e1', metalness: 0.85, roughness: 0.18 });
      const lensMat = new THREE.MeshStandardMaterial({ color: '#090d16', metalness: 0.72, roughness: 0.08 });
      const bar = new THREE.Mesh(new THREE.BoxGeometry(0.84, 0.05, 0.05), frameMat);
      bar.position.set(0, 0.14, 0.48);
      this.headMesh.add(bar);
      for (const side of [-1, 1]) {
        const lens = new THREE.Mesh(new THREE.BoxGeometry(0.34, 0.24, 0.05), lensMat);
        lens.position.set(side * 0.22, 0.04, 0.48);
        this.headMesh.add(lens);
      }
    }

    const hatGroup = buildHatMesh(hatType, hatColor);
    this.headMesh.add(hatGroup);
    this.bodyGroup.add(this.headMesh);

    const torsoWidth = isMale ? 1.14 : 1.05;
    this.torsoMesh = new THREE.Mesh(new THREE.BoxGeometry(torsoWidth, 1.08, 0.54), torsoMat);
    this.torsoMesh.position.set(0, 1.48, 0);
    this.torsoMesh.castShadow = true;
    if (av.topStyle === 'tech_jacket') {
      const darkTrimMat = new THREE.MeshStandardMaterial({ color: '#1e293b', roughness: 0.4, metalness: 0.3 });
      const badgeMat = new THREE.MeshStandardMaterial({
        color: av.accessoryColor || '#38bdf8',
        emissive: av.accessoryColor || '#38bdf8',
        emissiveIntensity: 0.45
      });
      const belt = new THREE.Mesh(new THREE.BoxGeometry(torsoWidth + 0.04, 0.14, 0.58), darkTrimMat);
      belt.position.set(0, -0.44, 0);
      const badge = new THREE.Mesh(new THREE.BoxGeometry(0.24, 0.1, 0.04), badgeMat);
      badge.position.set(-0.24, 0.24, 0.28);
      this.torsoMesh.add(belt, badge);
    }
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
    this.rayGunGroup = null;
    if (av.accessory === 'ray_gun') {
      this.rayGunGroup = buildRayGunAttachment(av.accessoryColor || '#ec4899', this.cloneGeometries);
      this.rayGunGroup.scale.setScalar(0.92);
      this.rayGunGroup.position.set(0.02, -0.88, 0.22);
      this.rayGunGroup.rotation.x = 0.55;
      this.rightArmPivot.add(this.rayGunGroup);
    } else if (av.accessory === 'crystal_blade') {
      this.rayGunGroup = buildCrystalBladeAttachment(av.accessoryColor || '#38bdf8', this.cloneGeometries);
      this.rayGunGroup.scale.setScalar(0.95);
      this.rayGunGroup.position.set(0.02, -0.88, 0.22);
      this.rightArmPivot.add(this.rayGunGroup);
    } else if (av.accessory === 'starlight_wand') {
      this.rayGunGroup = buildStarlightWandAttachment(av.accessoryColor || '#f472b6', this.cloneGeometries);
      this.rayGunGroup.scale.setScalar(0.95);
      this.rayGunGroup.position.set(0.02, -0.88, 0.22);
      this.rightArmPivot.add(this.rayGunGroup);
    }
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
    } else if (emote === 'sleep') {
      this.bodyGroup.rotation.x = -Math.PI * 0.47;
      this.bodyGroup.position.set(0, 0.56, 0.36);
      this.leftArmPivot.rotation.z = 0.16;
      this.rightArmPivot.rotation.z = -0.16;
    } else if (anim === 'hover') {
      this.bodyGroup.rotation.y = 0.24;
      this.leftArmPivot.rotation.z = 0.52;
      this.rightArmPivot.rotation.z = -0.52;
      this.leftLegPivot.rotation.x = -0.16;
      this.rightLegPivot.rotation.x = 0.16;
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
      if (this.rayGunGroup) {
        this.rightArmPivot.rotation.x = -0.52 + Math.sin(t * 2.6) * 0.04;
      }
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
  constructor(playerData, options = {}) {
    this.studioMode = Boolean(options.studioMode);
    this.id = playerData.id;
    this.user = playerData.user || { username: 'friend', displayName: 'Friend', avatar: {} };
    this.zone = Number(playerData.zone) || 0;
    this.group = new THREE.Group();
    this.mountHolder = new THREE.Group();
    this.petHolder = new THREE.Group();
    this.rigContainer = new THREE.Group();
    this.group.add(this.mountHolder, this.petHolder, this.rigContainer);

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
    this.animClock = this.studioMode ? 0 : Math.random() * 10;

    this.activeMode = null;
    this.stylizedRig = null;
    this.blockyRig = null;
    this.voiceRing = null;
    this.labelSprite = null;

    this.equippedPet = null;
    this.petNickname = '';
    this.equippedMount = null;
    this.petVisual = null;
    this.mountVisual = null;
    this.petWorldPos = new THREE.Vector3(
      (playerData.x || 0) - 1.1,
      playerData.y || 0,
      (playerData.z || 0) - 1.1
    );
    this.petYaw = this.targetRy;

    if (!this.studioMode) {
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
      this.syncCompanions(this.user);
    }
    this._syncRigMode();
  }

  syncCompanions(compSource = null, forceRebuild = false) {
    if (this.studioMode) return;
    const src = compSource || this.user || {};
    if (compSource && this.user) {
      if (src.equippedPet !== undefined) this.user.equippedPet = src.equippedPet;
      if (src.petNickname !== undefined) this.user.petNickname = src.petNickname;
      if (src.equippedMount !== undefined) this.user.equippedMount = src.equippedMount;
    }

    const nextPet = src.equippedPet !== undefined ? src.equippedPet : this.user?.equippedPet || null;
    const nextNick = String(
      src.petNickname !== undefined ? src.petNickname : this.user?.petNickname || ''
    ).trim();
    const nextMount =
      src.equippedMount !== undefined ? src.equippedMount : this.user?.equippedMount || null;

    if (forceRebuild || nextPet !== this.equippedPet || nextNick !== this.petNickname) {
      this.equippedPet = nextPet || null;
      this.petNickname = nextNick;
      this.petHolder.clear();
      this.petVisual = this.equippedPet
        ? buildCompanionPetVisual(this.equippedPet, this.petNickname)
        : null;
      if (this.petVisual) {
        this.petHolder.add(this.petVisual);
        const gx = this.group.position.x;
        const gy = this.group.position.y;
        const gz = this.group.position.z;
        const ry = this.group.rotation.y;
        this.petWorldPos.set(
          gx - Math.sin(ry) * 1.25 + Math.cos(ry) * 0.75,
          gy,
          gz - Math.cos(ry) * 1.25 - Math.sin(ry) * 0.75
        );
      }
    }

    if (forceRebuild || nextMount !== this.equippedMount) {
      this.equippedMount = nextMount || null;
      this.mountHolder.clear();
      this.mountVisual = this.equippedMount ? buildCompanionMountVisual(this.equippedMount) : null;
      if (this.mountVisual) {
        this.mountHolder.add(this.mountVisual);
      }
    }
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
      this.stylizedRig = new StylizedCharacterRig(gltf, av, { cloneGeometries: this.studioMode });
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
      this.blockyRig = new ClassicBlockyRig(av, { cloneGeometries: this.studioMode });
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
    if (this.studioMode) return;
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

    const mountActive = !this.studioMode && Boolean(this.equippedMount) && this.zone === 0 && !this.emote;
    this.mountHolder.visible = mountActive;
    if (mountActive && this.mountVisual) {
      const hoverBob = Math.sin(this.animClock * 5.4) * 0.05;
      this.mountHolder.position.y = 0.06 + hoverBob;
      this.rigContainer.position.y = 0.22 + hoverBob;
      if (this.mountVisual.userData?.glowRing) {
        const pulse = 0.92 + Math.sin(this.animClock * 8.0) * 0.12;
        this.mountVisual.userData.glowRing.scale.set(pulse, pulse, 1);
      }
    } else {
      this.rigContainer.position.y = 0;
    }

    if (!this.studioMode && this.petVisual) {
      this.petHolder.visible = true;
      const gx = this.group.position.x;
      const gy = this.group.position.y;
      const gz = this.group.position.z;
      const ry = this.group.rotation.y;
      const desiredX = gx - Math.sin(ry) * 1.28 + Math.cos(ry) * 0.76;
      const desiredZ = gz - Math.cos(ry) * 1.28 - Math.sin(ry) * 0.76;
      const desiredY = gy;
      const distToPet = Math.hypot(desiredX - this.petWorldPos.x, desiredZ - this.petWorldPos.z);
      if (distToPet > 11.0 || Math.abs(desiredY - this.petWorldPos.y) > 8.0) {
        this.petWorldPos.set(desiredX, desiredY, desiredZ);
      } else {
        const followRate = Math.min(1, dt * (distToPet > 1.8 ? 6.5 : 3.8));
        this.petWorldPos.x += (desiredX - this.petWorldPos.x) * followRate;
        this.petWorldPos.z += (desiredZ - this.petWorldPos.z) * followRate;
        this.petWorldPos.y += (desiredY - this.petWorldPos.y) * Math.min(1, dt * 8.5);
      }
      const lookPetYaw = Math.atan2(gx - this.petWorldPos.x, gz - this.petWorldPos.z);
      let yawDiff = lookPetYaw - this.petYaw;
      while (yawDiff > Math.PI) yawDiff -= Math.PI * 2;
      while (yawDiff < -Math.PI) yawDiff += Math.PI * 2;
      this.petYaw += yawDiff * Math.min(1, dt * 9.5);

      const dx = this.petWorldPos.x - gx;
      const dy = this.petWorldPos.y - gy;
      const dz = this.petWorldPos.z - gz;
      const cosR = Math.cos(ry);
      const sinR = Math.sin(ry);
      const hopBob =
        this.equippedPet === 'baby_crystal_dragon'
          ? 0.34 + Math.sin(this.animClock * 4.8) * 0.11
          : Math.abs(Math.sin(this.animClock * (distToPet > 0.3 ? 8.2 : 2.8))) * (distToPet > 0.3 ? 0.16 : 0.05);
      this.petHolder.position.set(dx * cosR - dz * sinR, dy + hopBob, dx * sinR + dz * cosR);
      this.petHolder.rotation.y = this.petYaw - ry;

      if (this.petVisual.userData?.leftWing && this.petVisual.userData?.rightWing) {
        const flap = Math.sin(this.animClock * 11.0) * 0.42;
        this.petVisual.userData.leftWing.rotation.z = flap;
        this.petVisual.userData.rightWing.rotation.z = -flap;
      }
    } else {
      this.petHolder.visible = false;
    }

    const effectiveAnim = mountActive && (this.anim === 'walk' || this.anim === 'run') ? 'hover' : this.anim;
    if (this.activeMode === 'stylized' && this.stylizedRig) {
      this.stylizedRig.update(dt, this.animClock, effectiveAnim, this.emote, this.speaking);
    } else if (this.blockyRig) {
      this.blockyRig.update(dt, this.animClock, effectiveAnim, this.emote);
    }
  }
}

export class BlockyWorld3D {
  constructor({
    container,
    onMove,
    onPlaceBlock,
    onRemoveBlock,
    onToggleBlock,
    onFireRayGun,
    onRotateBuild,
    onObbyCheckpoint,
    onTrophyWin,
    onRunwayProximity,
    onRequestZonePortal,
    onInteractNode,
    onPrisonWork,
    onAttackMob,
    onPickupDrop,
    onInteractionHint,
    onManageHome,
    onOpenCrafting,
    onObbyGateCrossed,
    onOpenObbyBoard,
    onCatchFallingStar,
    onSummonArenaBoss
  }) {
    this.container = container;
    this.onMove = onMove || (() => {});
    this.onPlaceBlock = onPlaceBlock || (() => {});
    this.onRemoveBlock = onRemoveBlock || (() => {});
    this.onToggleBlock = onToggleBlock || (() => {});
    this.onFireRayGun = onFireRayGun || (() => {});
    this.onRotateBuild = onRotateBuild || (() => {});
    this.onObbyCheckpoint = onObbyCheckpoint || (() => {});
    this.onTrophyWin = onTrophyWin || (() => {});
    this.onRunwayProximity = onRunwayProximity || (() => {});
    this.onRequestZonePortal = onRequestZonePortal || (() => {});
    this.onInteractNode = onInteractNode || (() => {});
    this.onPrisonWork = onPrisonWork || (() => {});
    this.onAttackMob = onAttackMob || (() => {});
    this.onPickupDrop = onPickupDrop || (() => {});
    this.onInteractionHint = onInteractionHint || (() => {});
    this.onManageHome = onManageHome || (() => {});
    this.onOpenCrafting = onOpenCrafting || (() => {});
    this.onObbyGateCrossed = onObbyGateCrossed || (() => {});
    this.onOpenObbyBoard = onOpenObbyBoard || (() => {});
    this.onCatchFallingStar = onCatchFallingStar || (() => {});
    this.onSummonArenaBoss = onSummonArenaBoss || (() => {});

    loadDtiWardrobePack().catch(() => {});
    loadPropsAndLootPack()
      .then(() => {
        this._upgradeLoadedPropsAndAvatars();
      })
      .catch(() => {});
    loadEnvironmentPack()
      .then(() => {
        this._upgradeEnvironmentModels();
      })
      .catch(() => {});

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color('#7dd3fc');
    this.scene.fog = new THREE.FogExp2('#bae6fd', 0.0036);

    this.camera = new THREE.PerspectiveCamera(58, window.innerWidth / window.innerHeight, 0.1, 380);
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.08;
    this.container.appendChild(this.renderer.domElement);

    this.currentZone = 0; // 0 = Starlight Plaza, 1 = Crystal Cavern Dungeon, 2 = Debtor's Prison
    this.plazaGroup = new THREE.Group();
    this.scene.add(this.plazaGroup);
    this.plazaTreeGroup = new THREE.Group();
    this.plazaGroup.add(this.plazaTreeGroup);
    this.plazaTreeSpecs = [];
    this.dtiStageDecorGroup = new THREE.Group();
    this.plazaGroup.add(this.dtiStageDecorGroup);
    this.campfireAnchor = new THREE.Group();
    this.plazaGroup.add(this.campfireAnchor);
    this.campBenchPlaceholders = [];
    this.campFlames = [];
    this.campEmbers = [];
    this.campLight = null;
    this.stargateInstances = [];
    this.scifiThrusters = [];

    this.craftingBenchPos = { x: 12.5, y: 0, z: 6.5 };
    this.craftingBenchGroup = new THREE.Group();
    this.plazaGroup.add(this.craftingBenchGroup);
    this.mailboxesGroup = new THREE.Group();
    this.plazaGroup.add(this.mailboxesGroup);
    this.mailboxesMap = new Map();
    this.obbyGatesGroup = new THREE.Group();
    this.plazaGroup.add(this.obbyGatesGroup);
    this.obbyGates = {};
    this.obbyBoardScreens = {};
    this.activeObbyCourse = null;
    this.lastObbyGateAt = { easy_start: 0, easy_finish: 0, hard_start: 0, hard_finish: 0 };

    this.skyState = {
      mode: 'auto',
      cycleMs: 720000,
      cycleOffsetMs: 0,
      meteorShowerUntilMs: 0,
      fallingStars: []
    };
    this.currentNightFactor = 0;
    this.fallingStarsGroup = new THREE.Group();
    this.plazaGroup.add(this.fallingStarsGroup);
    this.fallingStarsMap = new Map();
    this.auroraGroup = new THREE.Group();
    this.plazaGroup.add(this.auroraGroup);
    this.auroraMaterials = [];
    this.nightStarPoints = null;
    this.runwaySpotlightGroup = new THREE.Group();
    this.plazaGroup.add(this.runwaySpotlightGroup);
    this.runwayShowState = null;

    this.dungeonGroup = new THREE.Group();
    this.dungeonGroup.visible = false;
    this.scene.add(this.dungeonGroup);
    this.dungeonEntitiesGroup = new THREE.Group();
    this.dungeonGroup.add(this.dungeonEntitiesGroup);

    this.arenaGroup = new THREE.Group();
    this.arenaGroup.visible = false;
    this.scene.add(this.arenaGroup);
    this.arenaEntitiesGroup = new THREE.Group();
    this.arenaGroup.add(this.arenaEntitiesGroup);
    this.bossAltarPos = { x: 0, y: 0, z: -24.5 };
    this.bossAltarGroup = new THREE.Group();
    this.arenaGroup.add(this.bossAltarGroup);
    this.bossDragonEntry = null;
    this.bossShockwavesMap = new Map();

    this.prisonGroup = new THREE.Group();
    this.prisonGroup.visible = false;
    this.scene.add(this.prisonGroup);
    this.prisonEntitiesGroup = new THREE.Group();
    this.prisonGroup.add(this.prisonEntitiesGroup);

    this.plazaColliders = [];
    this.voxelColliders = [];
    this.dungeonColliders = [];
    this.arenaColliders = [];
    this.prisonColliders = [];
    this.arenaLavaMeshes = [];
    this.colliders = [];
    this.cameraOccluders = [];
    this.openDoorPortals = [];
    this.movingPlatforms = [];
    this.blinkingPlatforms = [];
    this.sweeperHazards = [];
    this.fountainShimmers = [];
    this.discoTiles = [];
    this.runwayStripLights = [];
    this.jumpPads = [];
    this.trophyMesh = null;
    this.hardTrophyMesh = null;
    this.runwayStarCrest = null;
    this.confettiParticles = [];
    this.sparkleRings = [];
    this.laserBeams = [];
    this.lastLocalBeamAt = 0;
    this.portalVortices = [];

    this.plazaPortalPos = { x: 18, y: 0, z: 16 };
    this.dungeonReturnPortalPos = { x: 0, y: 0, z: 28 };
    this.plazaArenaPortalPos = { x: -22, y: 0, z: 28 };
    this.arenaReturnPortalPos = { x: 0, y: 0, z: 30 };
    this.prisonReleasePortalPos = { x: 0, y: 0, z: 11.5 };
    this.prisonReleaseEligible = false;
    this.prisonGateRefs = null;
    this.arenaDead = false;
    this.lastPortalTriggerAt = 0;
    this.lastHintText = '';

    this.dungeonNodesMap = new Map();
    this.dungeonMobsMap = new Map();
    this.dungeonDropsMap = new Map();
    this.arenaNodesMap = new Map();
    this.arenaMobsMap = new Map();
    this.arenaDropsMap = new Map();
    this.prisonNodesMap = new Map();

    this.speedBoostUntilMs = 0;
    this.jumpBoostUntilMs = 0;

    this.voxelMap = new Map();
    this.savedBlocksByKey = new Map();
    this.animatedHinges = new Map();
    this.voxelGroup = new THREE.Group();
    this.scene.add(this.voxelGroup);

    this.buildMode = false;
    this.buildTool = 'place';
    this.selectedBlockColor = BLOCK_COLORS[0].hex;
    this.selectedBuildKind = 'block';
    this.selectedBuildRot = 0;
    this.selectedLootItemId = null;
    this.raycaster = new THREE.Raycaster();
    this.cameraRaycaster = new THREE.Raycaster();
    this.pointerNdc = new THREE.Vector2();
    this.ghostMesh = this._createBuildGhostGroup();
    this.ghostMesh.visible = false;
    this.scene.add(this.ghostMesh);

    this.selfAvatar = null;
    this.remoteAvatars = new Map();

    this.keys = new Set();
    this.touchMove = { forward: false, backward: false, left: false, right: false };
    this.velocity = new THREE.Vector3();
    this.isGrounded = true;
    this.obbyCheckpoints = {
      easy: { x: 0, y: 0.5, z: -12, index: 0 },
      hard: { x: 52, y: 0.9, z: 52, index: 0 }
    };
    this.currentCheckpoint = this.obbyCheckpoints.easy;
    this.lastActiveObby = 'easy';
    this.lastTrophyWinAt = 0;
    this.lastHardTrophyWinAt = 0;
    this.lastSweeperHitAt = 0;
    this.nearRunway = false;

    this.cameraYaw = 0;
    this.cameraPitch = 0.34;
    this.cameraDist = 9.2;
    this.effectiveCameraDist = 9.2;
    this.salonMirrorMode = false;
    this.savedCameraBeforeSalon = null;
    this.isDraggingCamera = false;
    this.dragMoved = false;
    this.lastPointer = { x: 0, y: 0 };
    this.serverClockOffsetMs = 0;

    this._setupLights();
    this._buildEnvironment();
    this._buildDungeonEnvironment();
    this._buildArenaEnvironment();
    this._buildPrisonEnvironment();
    this._rebuildActiveColliders();
    this._bindEvents();

    this.lastFrameTime = performance.now();
    this.lastMoveSentAt = 0;
    this._animate = this._animate.bind(this);
    requestAnimationFrame(this._animate);
  }

  _createBuildGhostGroup() {
    const grp = new THREE.Group();
    const boxMat = new THREE.MeshBasicMaterial({ color: '#38bdf8', wireframe: true });
    const box = new THREE.Mesh(new THREE.BoxGeometry(1.02, 1.02, 1.02), boxMat);
    box.position.y = 0.5;
    // Direction arrow on top/front (+Z at rot=0) so furniture & door facing is obvious before placing!
    const arrowMat = new THREE.MeshBasicMaterial({ color: '#fbbf24', transparent: true, opacity: 0.9 });
    const arrow = new THREE.Mesh(new THREE.ConeGeometry(0.18, 0.34, 4), arrowMat);
    arrow.rotation.x = Math.PI / 2;
    arrow.position.set(0, 0.16, 0.42);
    grp.add(box, arrow);
    grp.userData.wireBox = box;
    grp.userData.dirArrow = arrow;
    return grp;
  }

  rotateBuildSelection(step = 1) {
    this.selectedBuildRot = ((this.selectedBuildRot + step) % 4 + 4) % 4;
    if (this.buildMode && !this.salonMirrorMode) {
      this._updateBuildGhost();
    }
    this.onRotateBuild(this.selectedBuildRot);
    return this.selectedBuildRot;
  }

  _upgradeLoadedPropsAndAvatars() {
    for (const b of Array.from(this.savedBlocksByKey.values())) {
      if ((b.kind && b.kind !== 'block') || b.lootItemId) {
        this.addBlockMesh(b);
      }
    }
    const rightHandAcc = (acc) =>
      acc === 'ray_gun' || acc === 'crystal_blade' || acc === 'starlight_wand';
    if (this.selfAvatar && rightHandAcc(this.selfAvatar.user?.avatar?.accessory)) {
      this.selfAvatar.updateAvatarAppearance(this.selfAvatar.user.avatar);
    }
    for (const av of this.remoteAvatars.values()) {
      if (rightHandAcc(av.user?.avatar?.accessory)) {
        av.updateAvatarAppearance(av.user.avatar);
      }
    }
  }

  _rebuildActiveColliders() {
    if (this.currentZone === 1) {
      this.colliders = this.dungeonColliders;
    } else if (this.currentZone === 2) {
      this.colliders = this.prisonColliders;
    } else if (this.currentZone === 3) {
      this.colliders = this.arenaColliders;
    } else {
      this.colliders = [...this.plazaColliders, ...this.voxelColliders];
    }

    const seenMeshes = new Set();
    this.cameraOccluders = [];
    for (const c of this.colliders) {
      if (!c.mesh || seenMeshes.has(c.mesh)) continue;
      if (c.type === 'ground' || c.type === 'jumppad' || c.passable) continue;
      if (c.cameraOccluder === false) continue;
      if (c.cameraOccluder === true || (c.type !== 'voxel' && (c.h || 0) >= 0.75)) {
        seenMeshes.add(c.mesh);
        this.cameraOccluders.push(c.mesh);
      }
    }

    // Track open doors & double doors for smooth snag-free walk-through passage
    this.openDoorPortals = [];
    if (this.currentZone === 0) {
      for (const b of this.savedBlocksByKey.values()) {
        if (!b.open) continue;
        if (b.kind !== 'door' && b.kind !== 'double_door') continue;
        const rot = ((b.rot || 0) % 4 + 4) % 4;
        const isEW = rot % 2 === 1;
        let centerX = b.x;
        let centerZ = b.z;
        if (b.kind === 'double_door') {
          const { dx, dz } = doubleDoorPartnerOffset(rot);
          centerX = b.x + dx * 0.5;
          centerZ = b.z + dz * 0.5;
        }
        this.openDoorPortals.push({
          centerX,
          centerZ,
          bottomY: b.y - 1,
          topY: b.y + 1,
          isEW, // true => passage along X axis, width along Z; false => passage along Z axis, width along X
          halfOpening: b.kind === 'double_door' ? 0.98 : 0.54
        });
      }
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
    this.sunLight.shadow.camera.far = 165;
    const d = 56;
    this.sunLight.shadow.camera.left = -d;
    this.sunLight.shadow.camera.right = d;
    this.sunLight.shadow.camera.top = d;
    this.sunLight.shadow.camera.bottom = -d;
    this.scene.add(this.sunLight);
    this.scene.add(this.sunLight.target);

    this.fillLight = new THREE.DirectionalLight('#fdf2f8', 0.55);
    this.fillLight.position.set(-28, 18, 26);
    this.scene.add(this.fillLight);
  }

  _attachSciFiPlatformFx(mesh, w, h, d, accentHex = '#38bdf8') {
    const fxGroup = new THREE.Group();
    const minSpan = Math.min(w, d);
    const ringR = Math.max(0.42, minSpan * 0.34);

    // Glowing anti-grav thruster emitter ring beneath floating platform
    const outerRingMat = new THREE.MeshBasicMaterial({
      color: accentHex,
      transparent: true,
      opacity: 0.85,
      side: THREE.DoubleSide
    });
    const outerRing = new THREE.Mesh(new THREE.TorusGeometry(ringR, 0.055, 8, 24), outerRingMat);
    outerRing.rotation.x = Math.PI / 2;
    outerRing.position.y = -h * 0.5 - 0.08;
    fxGroup.add(outerRing);

    // Secondary smaller counter-rotating holo ring
    const innerRingMat = new THREE.MeshBasicMaterial({
      color: '#ffffff',
      transparent: true,
      opacity: 0.72,
      side: THREE.DoubleSide
    });
    const innerRing = new THREE.Mesh(new THREE.TorusGeometry(ringR * 0.62, 0.032, 6, 18), innerRingMat);
    innerRing.rotation.x = Math.PI / 2;
    innerRing.position.y = -h * 0.5 - 0.22;
    fxGroup.add(innerRing);

    // Translucent downward ion plasma thruster cone
    const coneMat = new THREE.MeshBasicMaterial({
      color: accentHex,
      transparent: true,
      opacity: 0.28,
      depthWrite: false,
      side: THREE.DoubleSide
    });
    const cone = new THREE.Mesh(new THREE.ConeGeometry(ringR * 0.68, 0.82, 14, 1, true), coneMat);
    cone.rotation.x = Math.PI;
    cone.position.y = -h * 0.5 - 0.45;
    fxGroup.add(cone);

    // Glowing corner edge holo-bars along the platform perimeter
    const rimMat = new THREE.MeshBasicMaterial({ color: accentHex, transparent: true, opacity: 0.78 });
    const barZ = new THREE.BoxGeometry(w + 0.06, 0.06, 0.06);
    const barX = new THREE.BoxGeometry(0.06, 0.06, d + 0.06);
    for (const s of [-1, 1]) {
      const bz = new THREE.Mesh(barZ, rimMat);
      bz.position.set(0, -h * 0.5 + 0.04, s * (d * 0.5 + 0.02));
      const bx = new THREE.Mesh(barX, rimMat);
      bx.position.set(s * (w * 0.5 + 0.02), -h * 0.5 + 0.04, 0);
      fxGroup.add(bz, bx);
    }

    mesh.add(fxGroup);
    this.scifiThrusters.push({
      outerRing,
      innerRing,
      cone,
      coneMat,
      outerRingMat,
      phase: Math.random() * Math.PI * 2
    });
  }

  _addBoxPlatform(
    {
      x,
      y,
      z,
      w,
      h,
      d,
      color,
      type = 'solid',
      bounceVel = 0,
      checkpointIndex = 0,
      obbyTier = 'easy',
      emissive = null,
      emissiveIntensity = 0,
      textureStyle = null,
      scifiFx = null
    },
    targetGroup = this.plazaGroup,
    targetColliders = this.plazaColliders
  ) {
    // Automatically choose rich PolyHaven CC0 PBR textures for floating obby platforms
    let resolvedTex = textureStyle;
    const colLow = String(color || '').toLowerCase();
    const isFloatingObby = targetGroup === this.plazaGroup && y > 0.75 && type !== 'runway';
    if (!resolvedTex && isFloatingObby) {
      if (colLow === '#ef4444' || colLow === '#f97316' || colLow === '#fb923c' || colLow === '#ec4899') {
        resolvedTex = 'red_brick';
      } else if (type === 'moving' || type === 'jumppad') {
        resolvedTex = 'metal_plate';
      } else if (type === 'blinking' || colLow === '#38bdf8' || colLow === '#a78bfa' || colLow === '#a855f7' || colLow === '#8b5cf6' || colLow === '#1e1b4b' || colLow === '#312e81') {
        resolvedTex = 'hex_scifi';
      } else {
        resolvedTex = 'marble_tiles';
      }
    } else if (!resolvedTex && targetGroup === this.plazaGroup && (type === 'checkpoint' || type === 'trophy-pad' || type === 'gauntlet-trophy-pad')) {
      resolvedTex = 'marble_tiles';
    }

    const matOpts = {
      color: resolvedTex === 'red_brick' && colLow === '#ef4444' ? '#ffffff' : color,
      roughness: type === 'ice' || type === 'runway' ? 0.18 : resolvedTex === 'metal_plate' ? 0.34 : 0.58,
      metalness:
        resolvedTex === 'metal_plate'
          ? 0.52
          : type === 'trophy-pad' || type === 'gauntlet-trophy-pad' || type === 'runway'
            ? 0.28
            : 0.08
    };

    const texSet = getObbyTextureSet(resolvedTex);
    if (texSet) {
      const repX = Math.max(1, Math.round(w / 1.8));
      const repY = Math.max(1, Math.round(d / 1.8));
      const cloneRep = (t) => {
        const c = t.clone();
        c.needsUpdate = true;
        c.repeat.set(repX, repY);
        return c;
      };
      matOpts.map = cloneRep(texSet.map);
      matOpts.normalMap = cloneRep(texSet.normalMap);
      matOpts.normalScale = new THREE.Vector2(0.7, 0.7);
      matOpts.roughnessMap = cloneRep(texSet.roughnessMap);
    }

    const mat = new THREE.MeshStandardMaterial(matOpts);
    if (emissive) {
      mat.emissive = new THREE.Color(emissive);
      mat.emissiveIntensity = emissiveIntensity || 0.3;
    }
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), mat);
    mesh.position.set(x, y, z);
    mesh.receiveShadow = true;
    mesh.castShadow = y > 0.5;
    targetGroup.add(mesh);

    // Attach sci-fi anti-grav thruster & holographic ring effects on floating movers, blinkers, and tech platforms
    const enableSciFi =
      scifiFx !== null
        ? Boolean(scifiFx)
        : isFloatingObby && (type === 'moving' || type === 'blinking' || resolvedTex === 'hex_scifi' || resolvedTex === 'metal_plate');
    if (enableSciFi) {
      const fxColor =
        type === 'blinking'
          ? '#f472b6'
          : colLow === '#c084fc' || colLow === '#a855f7' || colLow === '#a78bfa'
            ? '#c084fc'
            : '#38bdf8';
      this._attachSciFiPlatformFx(mesh, w, h, d, fxColor);
    }

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
      checkpointIndex,
      obbyTier
    };
    targetColliders.push(entry);
    return entry;
  }

  _createPortalArch({ x, y, z, title, subtitle, primaryHex, secondaryHex }, targetGroup) {
    const portal = new THREE.Group();
    portal.position.set(x, y, z);

    const frameHost = new THREE.Group();
    portal.add(frameHost);

    // Procedural fallback arch (upgraded cleanly to Blender Stargate Ring once GLB finishes loading)
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
    frameHost.add(pad, runeRing);

    for (const side of [-1, 1]) {
      const pillar = new THREE.Mesh(new THREE.BoxGeometry(0.68, 4.1, 0.68), stoneMat);
      pillar.position.set(side * 1.85, 2.15, 0);
      pillar.castShadow = true;
      const runeStrip = new THREE.Mesh(new THREE.BoxGeometry(0.2, 3.5, 0.74), runeMat);
      runeStrip.position.set(side * 1.85, 2.15, 0);
      frameHost.add(pillar, runeStrip);
    }

    const lintel = new THREE.Mesh(new THREE.BoxGeometry(4.65, 0.72, 0.88), stoneMat);
    lintel.position.set(0, 4.35, 0);
    const keystone = new THREE.Mesh(new THREE.OctahedronGeometry(0.46, 0), runeMat);
    keystone.position.set(0, 4.95, 0);
    frameHost.add(lintel, keystone);

    // Live Custom GLSL Shimmering Wormhole Event Horizon (active immediately & rescaled on Stargate upgrade)
    const horizonMat = createStargateEventHorizonMaterial(primaryHex, secondaryHex);
    const vortexDisc = new THREE.Mesh(new THREE.CircleGeometry(1.78, 40), horizonMat);
    vortexDisc.position.set(0, 2.28, 0);

    const swirlRing = new THREE.Mesh(
      new THREE.TorusGeometry(1.32, 0.06, 10, 32),
      new THREE.MeshBasicMaterial({ color: '#e0f2fe', transparent: true, opacity: 0.65 })
    );
    swirlRing.position.set(0, 2.28, 0.04);

    // Swirling quantum energy motes orbiting the Stargate aperture
    const moteCount = 28;
    const moteGeo = new THREE.BufferGeometry();
    const motePositions = new Float32Array(moteCount * 3);
    for (let i = 0; i < moteCount; i++) {
      const ang = (i / moteCount) * Math.PI * 2;
      const rad = 1.65 + (i % 4) * 0.16;
      motePositions[i * 3] = Math.cos(ang) * rad;
      motePositions[i * 3 + 1] = Math.sin(ang) * rad;
      motePositions[i * 3 + 2] = ((i % 3) - 1) * 0.22;
    }
    moteGeo.setAttribute('position', new THREE.BufferAttribute(motePositions, 3));
    const moteMat = new THREE.PointsMaterial({
      color: primaryHex,
      size: 0.22,
      transparent: true,
      opacity: 0.88,
      depthWrite: false
    });
    const motes = new THREE.Points(moteGeo, moteMat);
    motes.position.set(0, 2.28, 0);

    portal.add(vortexDisc, swirlRing, motes);
    const portalRec = {
      portal,
      frameHost,
      disc: vortexDisc,
      horizonMat,
      ring: swirlRing,
      motes,
      keystone,
      innerGlyphRing: null,
      worldPos: new THREE.Vector3(x, y, z),
      primaryHex,
      secondaryHex,
      surge: 0
    };
    this.portalVortices.push(portalRec);
    this.stargateInstances.push(portalRec);

    const banner = createWorldBannerSprite(title, subtitle, primaryHex);
    banner.position.set(0, 6.75, 0);
    portal.add(banner);

    targetGroup.add(portal);
    return portal;
  }

  _buildEnvironment() {
    const groundMat = new THREE.MeshStandardMaterial({ color: '#4ade80', roughness: 0.75 });
    this.groundMesh = new THREE.Mesh(new THREE.BoxGeometry(210, 2, 210), groundMat);
    this.groundMesh.position.set(0, -1, 0);
    this.groundMesh.receiveShadow = true;
    this.plazaGroup.add(this.groundMesh);
    this.plazaColliders.push({
      mesh: this.groundMesh,
      w: 210,
      h: 2,
      d: 210,
      minX: -105,
      maxX: 105,
      minZ: -105,
      maxZ: 105,
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
    this.campfireAnchor.position.set(-6, 0, 11);

    // Cozy gravel/dirt campfire clearing pad flush with grass
    const clearingMat = new THREE.MeshStandardMaterial({ color: '#a89f91', roughness: 0.92 });
    const clearingDisc = new THREE.Mesh(new THREE.CylinderGeometry(5.8, 6.2, 0.06, 28), clearingMat);
    clearingDisc.position.set(-6, 0.03, 11);
    clearingDisc.receiveShadow = true;
    this.plazaGroup.add(clearingDisc);

    // Crossed firewood logs & multi-layer dancing campfire flames + marshmallow sticks + rising embers
    for (let li = 0; li < 4; li++) {
      const fLog = new THREE.Mesh(new THREE.CylinderGeometry(0.11, 0.13, 1.35, 8), logMat);
      fLog.position.set(-6, 0.24, 11);
      fLog.rotation.z = Math.PI * 0.42;
      fLog.rotation.y = li * (Math.PI * 0.25);
      this.plazaGroup.add(fLog);
    }

    const flameSpecs = [
      { r: 0.62, h: 1.45, y: 0.72, color: '#ea580c', emissive: '#ef4444', speed: 7.5, phase: 0 },
      { r: 0.44, h: 1.15, y: 0.64, color: '#f59e0b', emissive: '#f97316', speed: 9.2, phase: 1.8 },
      { r: 0.26, h: 0.82, y: 0.54, color: '#fef08a', emissive: '#fde047', speed: 11.4, phase: 3.4 }
    ];
    for (const fs of flameSpecs) {
      const fm = new THREE.MeshStandardMaterial({
        color: fs.color,
        emissive: fs.emissive,
        emissiveIntensity: 0.95,
        transparent: true,
        opacity: 0.88
      });
      const cone = new THREE.Mesh(new THREE.ConeGeometry(fs.r, fs.h, 7), fm);
      cone.position.set(-6, fs.y, 11);
      this.plazaGroup.add(cone);
      this.campFlames.push({ mesh: cone, baseY: fs.y, speed: fs.speed, phase: fs.phase });
    }

    // Two cozy marshmallow roasting sticks leaning over the fire
    const stickMat = new THREE.MeshStandardMaterial({ color: '#92400e', roughness: 0.8 });
    const mallowMat = new THREE.MeshStandardMaterial({ color: '#fffbeb', roughness: 0.45 });
    for (const [sx, sz, rotY] of [
      [-5.1, 12.15, 0.55],
      [-7.1, 11.75, -0.78]
    ]) {
      const stickGrp = new THREE.Group();
      stickGrp.position.set(sx, 0.25, sz);
      stickGrp.rotation.y = rotY;
      const twig = new THREE.Mesh(new THREE.CylinderGeometry(0.02, 0.028, 1.35, 6), stickMat);
      twig.rotation.x = -0.68;
      twig.position.set(0, 0.45, -0.35);
      const mallow = new THREE.Mesh(new THREE.CylinderGeometry(0.065, 0.065, 0.11, 8), mallowMat);
      mallow.position.set(0, 0.92, -0.78);
      stickGrp.add(twig, mallow);
      this.plazaGroup.add(stickGrp);
    }

    // Rising glowing fire embers
    const emberMat = new THREE.MeshBasicMaterial({ color: '#fde047', transparent: true, opacity: 0.9 });
    const emberGeo = new THREE.BoxGeometry(0.075, 0.075, 0.075);
    for (let ei = 0; ei < 14; ei++) {
      const em = new THREE.Mesh(emberGeo, emberMat);
      em.position.set(-6, 0.6, 11);
      this.plazaGroup.add(em);
      this.campEmbers.push({
        mesh: em,
        angle: (ei / 14) * Math.PI * 2,
        radius: 0.15 + (ei % 4) * 0.14,
        phase: (ei / 14) * 2.8,
        speed: 0.85 + (ei % 3) * 0.28
      });
    }

    // Warm flickering campfire PointLight
    this.campLight = new THREE.PointLight('#fb923c', 2.4, 15, 1.8);
    this.campLight.position.set(-6, 1.85, 11);
    this.plazaGroup.add(this.campLight);

    const benchS = this._addBoxPlatform({ x: -6, y: 0.35, z: 14.2, w: 4.2, h: 0.7, d: 1.1, color: '#92400e' });
    const benchW = this._addBoxPlatform({ x: -9.2, y: 0.35, z: 11, w: 1.1, h: 0.7, d: 4.2, color: '#92400e' });
    this.campBenchPlaceholders.push(benchS.mesh, benchW.mesh);

    const buildGridFloor = new THREE.GridHelper(16, 16, '#38bdf8', '#94a3b8');
    buildGridFloor.position.set(18, 0.06, -4);
    this.plazaGroup.add(buildGridFloor);

    // Crystal Cavern Stargate Portal in Starlight Plaza
    this.plazaPortalGroup = this._createPortalArch(
      {
        x: this.plazaPortalPos.x,
        y: this.plazaPortalPos.y,
        z: this.plazaPortalPos.z,
        title: '🌀 STARGATE — CRYSTAL CAVERN',
        subtitle: 'Step Through Wormhole or Press E • Farm Coins & Loot!',
        primaryHex: '#38bdf8',
        secondaryHex: '#6366f1'
      },
      this.plazaGroup
    );

    // Shadow Rift Mortal Arena Stargate Portal in Southwest Starlight Plaza
    this.plazaArenaPortalGroup = this._createPortalArch(
      {
        x: this.plazaArenaPortalPos.x,
        y: this.plazaArenaPortalPos.y,
        z: this.plazaArenaPortalPos.z,
        title: '⚔️ SHADOW RIFT — MORTAL ARENA',
        subtitle: 'Step In or Press E • Watch HP & Lava! Respawn at Sanctuary!',
        primaryHex: '#ef4444',
        secondaryHex: '#f97316'
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
      const variant = idx % 2 === 0 ? 'Env_Tree_Blossom' : 'Env_Tree_Emerald';
      this.plazaTreeSpecs.push({
        x: tx,
        z: tz,
        variant,
        scale: 0.92 + (idx % 3) * 0.08,
        yaw: (idx * 1.17) % (Math.PI * 2)
      });
      const trunk = new THREE.Mesh(trunkGeo, logMat);
      trunk.position.set(tx, 1.4, tz);
      trunk.castShadow = true;
      const leaves = new THREE.Mesh(leavesGeo, idx % 2 === 0 ? blossomMat : greenLeafMat);
      leaves.position.set(tx, 3.6, tz);
      leaves.castShadow = true;
      this.plazaTreeGroup.add(trunk, leaves);
    });

    this._addBoxPlatform({
      x: 0, y: 0.4, z: -12, w: 4.5, h: 0.8, d: 3.5, color: '#fbbf24', type: 'checkpoint', checkpointIndex: 1
    });

    const stage1Steps = [
      { x: 0, y: 1.4, z: -16.5, color: '#ef4444', textureStyle: 'red_brick' },
      { x: -3.2, y: 2.5, z: -20.0, color: '#38bdf8', textureStyle: 'hex_scifi', scifiFx: true },
      { x: 0, y: 3.6, z: -23.5, color: '#34d399', textureStyle: 'marble_tiles' },
      { x: 3.4, y: 4.8, z: -27.0, color: '#a78bfa', textureStyle: 'hex_scifi', scifiFx: true },
      { x: 0, y: 6.0, z: -30.5, color: '#ef4444', textureStyle: 'red_brick' }
    ];
    for (const s of stage1Steps) {
      this._addBoxPlatform({
        x: s.x,
        y: s.y,
        z: s.z,
        w: 3.1,
        h: 0.6,
        d: 3.1,
        color: s.color,
        textureStyle: s.textureStyle,
        scifiFx: s.scifiFx || false
      });
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

    // Expanded 210×210 Plaza Districts (Boulevards, Neighborhood Build Plots, Blossom Park & Signposts)
    this._buildExpandedPlazaDistricts();

    // Second, More Difficult Obby: Starlight Sky Gauntlet (Pro Obby)
    this._buildHardObby();
  }

  _createSignpost(x, z, rotY, heading, subtext, accentHex = '#f472b6') {
    const grp = new THREE.Group();
    grp.position.set(x, 0, z);
    grp.rotation.y = rotY;

    const postMat = new THREE.MeshStandardMaterial({ color: '#78350f', roughness: 0.72 });
    const post = new THREE.Mesh(new THREE.CylinderGeometry(0.12, 0.14, 3.1, 8), postMat);
    post.position.y = 1.55;
    post.castShadow = true;
    grp.add(post);

    const canvas = document.createElement('canvas');
    canvas.width = 512;
    canvas.height = 192;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#0f172a';
    ctx.fillRect(0, 0, 512, 192);
    ctx.strokeStyle = accentHex;
    ctx.lineWidth = 10;
    ctx.strokeRect(8, 8, 496, 176);
    ctx.fillStyle = '#ffffff';
    ctx.font = 'bold 42px Inter, sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(heading, 256, 86);
    ctx.fillStyle = accentHex;
    ctx.font = 'bold 28px Inter, sans-serif';
    ctx.fillText(subtext, 256, 144);

    const tex = new THREE.CanvasTexture(canvas);
    const boardMat = new THREE.MeshBasicMaterial({ map: tex });
    const backMat = new THREE.MeshStandardMaterial({ color: '#1e293b', roughness: 0.5 });
    const board = new THREE.Mesh(new THREE.BoxGeometry(3.2, 1.2, 0.14), [
      backMat,
      backMat,
      backMat,
      backMat,
      boardMat,
      boardMat
    ]);
    board.position.y = 2.65;
    board.castShadow = true;
    grp.add(board);

    this.plazaGroup.add(grp);
    return grp;
  }

  _buildExpandedPlazaDistricts() {
    // 1. Paved Starlight Boulevards radiating from Central Plaza (flush decorative stone paths)
    const pathMat = new THREE.MeshStandardMaterial({ color: '#e2e8f0', roughness: 0.62 });
    const trimMat = new THREE.MeshStandardMaterial({
      color: '#cbd5e1',
      roughness: 0.45,
      metalness: 0.1
    });
    const addPath = (x, z, w, d) => {
      const p = new THREE.Mesh(new THREE.BoxGeometry(w, 0.06, d), pathMat);
      p.position.set(x, 0.03, z);
      p.receiveShadow = true;
      this.plazaGroup.add(p);
    };

    // East Boulevard -> Sunset Build Meadow
    addPath(52, -4, 68, 5.2);
    // East Cross Street connecting north & south build plots
    addPath(68, 0, 4.4, 72);
    // South Boulevard -> Blossom Fountain Park
    addPath(0, 32, 5.4, 46);
    // Southeast Diagonal/Connector Path -> Starlight Sky Gauntlet (Pro Obby)
    addPath(26, 48, 52, 4.4);
    // West Boulevard -> Westside Villa Estates
    addPath(-56, 12, 52, 4.8);
    // Southwest Obsidian Connector -> Shadow Rift Mortal Arena Stargate
    addPath(-11, 28, 22, 4.4);

    // 2. Central Plaza 3D Directional Signposts
    this._createSignpost(9.5, 9.5, -Math.PI * 0.25, '🔥 PRO OBBY GAUNTLET', 'Southeast Path • Press T', '#f472b6');
    this._createSignpost(-9.5, 9.5, Math.PI * 0.25, '⚔️ SHADOW RIFT & PARK', 'SW Mortal Portal (K) • Park', '#ef4444');
    this._createSignpost(10.5, -6.5, -Math.PI * 0.5, '🏡 VILLAGE & PLOTS', 'Prebuilt Homes & Build Lots', '#34d399');
    this._createSignpost(-10.5, -9.5, Math.PI * 0.75, '🏃‍♀️ SKY OBBY & RUNWAY', 'North (R) • West (F)', '#fbbf24');
    this._createSignpost(-16.0, 24.5, 0.35, '⚔️ MORTAL SHADOW RIFT', 'Chasing Bosses & Lava (Press K)', '#ef4444');

    // 3. Labeled Neighborhood Build Plots & Prebuilt Model Homes
    const lanternPostMat = new THREE.MeshStandardMaterial({ color: '#334155', roughness: 0.45, metalness: 0.35 });
    const lanternGlowMat = new THREE.MeshStandardMaterial({
      color: '#fef08a',
      emissive: '#fbbf24',
      emissiveIntensity: 0.85
    });
    const buildPlots = [
      { x: 52, z: -26, size: 18, name: '🏡 Sunset Plot #1', sub: 'Open Build Lot (Press B)', accent: '#38bdf8' },
      { x: 82, z: -26, size: 18, name: '🌸 Rosewater Villa', sub: '2-Story Prebuilt Home (Press E!)', accent: '#f472b6' },
      { x: 52, z: 20, size: 18, name: '🌊 Starlight Cottage', sub: 'Prebuilt Beach Home (Walk In!)', accent: '#38bdf8' },
      { x: 82, z: 20, size: 18, name: '🏡 Starlight Plot #4', sub: 'Open Build Lot (Press B)', accent: '#34d399' },
      { x: -64, z: -16, size: 20, name: '🌲 Woodland Chalet', sub: 'Prebuilt Pine Home (Press E!)', accent: '#34d399' },
      { x: -64, z: 32, size: 20, name: '🏡 West Villa Plot #6', sub: 'Open Build Lot (Press B)', accent: '#f472b6' }
    ];
    for (const plot of buildPlots) {
      const grid = new THREE.GridHelper(plot.size, plot.size, plot.accent, '#94a3b8');
      grid.position.set(plot.x, 0.05, plot.z);
      this.plazaGroup.add(grid);

      const half = plot.size / 2 + 0.6;
      for (const [cx, cz] of [
        [-half, -half],
        [half, -half],
        [-half, half],
        [half, half]
      ]) {
        const p = new THREE.Mesh(new THREE.CylinderGeometry(0.12, 0.16, 2.3, 8), lanternPostMat);
        p.position.set(plot.x + cx, 1.15, plot.z + cz);
        const orb = new THREE.Mesh(new THREE.SphereGeometry(0.24, 10, 10), lanternGlowMat);
        orb.position.set(plot.x + cx, 2.42, plot.z + cz);
        this.plazaGroup.add(p, orb);
      }
      this._createSignpost(plot.x, plot.z + half + 1.4, 0, plot.name, plot.sub, plot.accent);
    }

    // 4. Blossom Fountain Park & Koi Pond (South: x ∈ [-28, 28], z ∈ [54, 92])
    const parkPlaza = new THREE.Mesh(new THREE.CylinderGeometry(18, 18.6, 0.1, 40), trimMat);
    parkPlaza.position.set(0, 0.05, 72);
    parkPlaza.receiveShadow = true;
    this.plazaGroup.add(parkPlaza);

    const pondRim = new THREE.Mesh(
      new THREE.TorusGeometry(8.2, 0.55, 12, 42),
      new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.35 })
    );
    pondRim.rotation.x = Math.PI / 2;
    pondRim.position.set(0, 0.36, 72);
    this.plazaGroup.add(pondRim);

    const waterMat = new THREE.MeshStandardMaterial({
      color: '#38bdf8',
      emissive: '#0284c7',
      emissiveIntensity: 0.32,
      roughness: 0.12,
      metalness: 0.25,
      transparent: true,
      opacity: 0.86
    });
    const pondWater = new THREE.Mesh(new THREE.CylinderGeometry(7.9, 7.9, 0.25, 36), waterMat);
    pondWater.position.set(0, 0.18, 72);
    this.plazaGroup.add(pondWater);

    // Tiered Crystal Fountain Centerpiece
    const stoneMat = new THREE.MeshStandardMaterial({ color: '#f1f5f9', roughness: 0.3, metalness: 0.15 });
    const fBase = new THREE.Mesh(new THREE.CylinderGeometry(2.3, 2.8, 0.9, 18), stoneMat);
    fBase.position.set(0, 0.65, 72);
    const fPillar = new THREE.Mesh(new THREE.CylinderGeometry(0.65, 0.95, 2.8, 14), stoneMat);
    fPillar.position.set(0, 2.2, 72);
    const fBowl = new THREE.Mesh(new THREE.CylinderGeometry(2.2, 1.2, 0.55, 20), stoneMat);
    fBowl.position.set(0, 3.3, 72);
    const fCrystal = new THREE.Mesh(
      new THREE.OctahedronGeometry(0.95, 0),
      new THREE.MeshStandardMaterial({
        color: '#f472b6',
        emissive: '#ec4899',
        emissiveIntensity: 0.65,
        roughness: 0.15,
        metalness: 0.35
      })
    );
    fCrystal.position.set(0, 4.9, 72);
    this.plazaGroup.add(fBase, fPillar, fBowl, fCrystal);
    this.fountainShimmers.push(fCrystal);

    // Floating fountain water rings
    for (let i = 0; i < 3; i++) {
      const ring = new THREE.Mesh(
        new THREE.TorusGeometry(1.4 + i * 1.3, 0.07, 8, 28),
        new THREE.MeshStandardMaterial({
          color: '#bae6fd',
          emissive: '#38bdf8',
          emissiveIntensity: 0.6,
          transparent: true,
          opacity: 0.72
        })
      );
      ring.rotation.x = Math.PI / 2;
      ring.position.set(0, 0.35 + i * 0.04, 72);
      ring.userData.shimmerPhase = i * 1.4;
      this.plazaGroup.add(ring);
      this.fountainShimmers.push(ring);
    }

    // Park benches around the fountain
    const benchAngles = [0, Math.PI * 0.5, Math.PI, Math.PI * 1.5];
    for (const ang of benchAngles) {
      const bx = Math.cos(ang) * 13.8;
      const bz = 72 + Math.sin(ang) * 13.8;
      const isZBench = Math.abs(Math.sin(ang)) > 0.5;
      this._addBoxPlatform({
        x: bx,
        y: 0.38,
        z: bz,
        w: isZBench ? 4.0 : 1.1,
        h: 0.76,
        d: isZBench ? 1.1 : 4.0,
        color: '#b45309'
      });
    }

    // 5. Lush Outer Park & Perimeter Trees + Flower Clusters + Glowing Perimeter Border Posts
    const logMat = new THREE.MeshStandardMaterial({ color: '#78350f', roughness: 0.8 });
    const trunkGeo = new THREE.BoxGeometry(1.0, 3.2, 1.0);
    const leavesGeo = new THREE.BoxGeometry(3.2, 3.0, 3.2);
    const blossomMat = new THREE.MeshStandardMaterial({ color: '#f9a8d4', roughness: 0.6 });
    const greenLeafMat = new THREE.MeshStandardMaterial({ color: '#22c55e', roughness: 0.6 });
    const lilacLeafMat = new THREE.MeshStandardMaterial({ color: '#c4b5fd', roughness: 0.6 });
    const leafMats = [blossomMat, greenLeafMat, lilacLeafMat];

    const outerTrees = [
      [-24, 58], [-28, 72], [-22, 86], [24, 58], [28, 72], [22, 86],
      [-10, 93], [10, 93], [-42, 66], [-48, 84], [-86, 60], [-88, 14],
      [-88, -36], [-72, -62], [-38, -62], [-18, -74], [18, -72], [46, -64],
      [78, -64], [94, -44], [95, 2], [95, 38], [34, -44], [-40, -40]
    ];
    const outerVariants = ['Env_Tree_Blossom', 'Env_Tree_Emerald', 'Env_Tree_Lilac'];
    outerTrees.forEach(([tx, tz], idx) => {
      this.plazaTreeSpecs.push({
        x: tx,
        z: tz,
        variant: outerVariants[idx % outerVariants.length],
        scale: 0.96 + (idx % 4) * 0.08,
        yaw: (idx * 1.41) % (Math.PI * 2)
      });
      const trunk = new THREE.Mesh(trunkGeo, logMat);
      trunk.position.set(tx, 1.6, tz);
      trunk.castShadow = true;
      const leaves = new THREE.Mesh(leavesGeo, leafMats[idx % leafMats.length]);
      leaves.position.set(tx, 4.0, tz);
      leaves.castShadow = true;
      this.plazaTreeGroup.add(trunk, leaves);
    });

    // Subtle perimeter lantern fence markers at ±103 so players see the 210×210 world edge
    const railMat = new THREE.MeshStandardMaterial({ color: '#e2e8f0', roughness: 0.5 });
    for (let p = -100; p <= 100; p += 20) {
      for (const [fx, fz] of [
        [p, -103],
        [p, 103],
        [-103, p],
        [103, p]
      ]) {
        const post = new THREE.Mesh(new THREE.BoxGeometry(0.45, 1.6, 0.45), railMat);
        post.position.set(fx, 0.8, fz);
        const cap = new THREE.Mesh(new THREE.SphereGeometry(0.24, 8, 8), lanternGlowMat);
        cap.position.set(fx, 1.78, fz);
        this.plazaGroup.add(post, cap);
      }
    }
  }

  _buildHardObby() {
    // Starlight Sky Gauntlet (Pro Obby #2) — Southeast Sector (x ∈ [48, 92], z ∈ [48, 92], y ∈ [0, 44])
    // Entrance Archway & Checkpoint 1 at (52, 0.4, 52)
    this._createSignpost(47.2, 52, Math.PI * 0.5, '🔥 STARLIGHT GAUNTLET', 'Pro Obby • +25 🪙 Reward', '#ec4899');
    const archMat = new THREE.MeshStandardMaterial({
      color: '#ec4899',
      emissive: '#db2777',
      emissiveIntensity: 0.45,
      metalness: 0.45,
      roughness: 0.2
    });
    const startArch = new THREE.Mesh(new THREE.TorusGeometry(3.2, 0.2, 12, 32, Math.PI), archMat);
    startArch.position.set(52, 0.8, 52);
    startArch.rotation.y = Math.PI * 0.5;
    this.plazaGroup.add(startArch);

    // CP 1: Start Pad
    this._addBoxPlatform({
      x: 52,
      y: 0.4,
      z: 52,
      w: 5.0,
      h: 0.8,
      d: 5.0,
      color: '#f59e0b',
      emissive: '#d97706',
      emissiveIntensity: 0.25,
      type: 'checkpoint',
      checkpointIndex: 1,
      obbyTier: 'hard'
    });

    // Stage 1 -> CP 2: Narrow Zig-Zag Steps (Red Brick & Sci-Fi Hex) + Fast Split-Axis Movers
    const s1Steps = [
      { x: 56.2, y: 2.0, z: 52.0, w: 2.0, d: 2.0, color: '#ef4444', textureStyle: 'red_brick', scifiFx: false },
      { x: 59.8, y: 3.5, z: 49.2, w: 1.85, d: 1.85, color: '#a855f7', textureStyle: 'hex_scifi', scifiFx: true },
      { x: 63.6, y: 5.0, z: 52.4, w: 1.85, d: 1.85, color: '#38bdf8', textureStyle: 'metal_plate', scifiFx: true }
    ];
    for (const s of s1Steps) {
      this._addBoxPlatform({
        x: s.x,
        y: s.y,
        z: s.z,
        w: s.w,
        h: 0.5,
        d: s.d,
        color: s.color,
        emissive: s.textureStyle === 'red_brick' ? null : s.color,
        emissiveIntensity: s.textureStyle === 'red_brick' ? 0 : 0.18,
        textureStyle: s.textureStyle,
        scifiFx: s.scifiFx
      });
    }

    const hm1 = this._addBoxPlatform({
      x: 67.8,
      y: 6.6,
      z: 52.4,
      w: 2.4,
      h: 0.5,
      d: 2.4,
      color: '#22d3ee',
      emissive: '#0891b2',
      emissiveIntensity: 0.25,
      type: 'moving',
      obbyTier: 'hard'
    });
    hm1.baseX = 67.8;
    hm1.baseY = 6.6;
    hm1.baseZ = 52.4;
    hm1.ampX = 0;
    hm1.ampZ = 3.4;
    hm1.speed = 2.2;
    this.movingPlatforms.push(hm1);

    const hm2 = this._addBoxPlatform({
      x: 71.2,
      y: 8.2,
      z: 52.4,
      w: 2.4,
      h: 0.5,
      d: 2.4,
      color: '#c084fc',
      emissive: '#7c3aed',
      emissiveIntensity: 0.25,
      type: 'moving',
      obbyTier: 'hard'
    });
    hm2.baseX = 71.2;
    hm2.baseY = 8.2;
    hm2.baseZ = 52.4;
    hm2.ampX = 2.2;
    hm2.ampZ = 0;
    hm2.speed = 2.4;
    this.movingPlatforms.push(hm2);

    // CP 2
    this._addBoxPlatform({
      x: 76.0,
      y: 9.6,
      z: 52.4,
      w: 4.8,
      h: 0.6,
      d: 4.8,
      color: '#f59e0b',
      emissive: '#d97706',
      emissiveIntensity: 0.25,
      type: 'checkpoint',
      checkpointIndex: 2,
      obbyTier: 'hard'
    });

    // Stage 2 -> CP 3: 5 Server-Synced Blinking / Disappearing Star Tiles
    const blinkDefs = [
      { x: 79.0, y: 11.1, z: 56.2, phaseOffsetMs: 0, color: '#f472b6' },
      { x: 82.0, y: 12.6, z: 60.0, phaseOffsetMs: 1200, color: '#fb7185' },
      { x: 85.0, y: 14.1, z: 63.8, phaseOffsetMs: 2400, color: '#f472b6' },
      { x: 85.0, y: 15.6, z: 68.4, phaseOffsetMs: 600, color: '#fb7185' },
      { x: 84.0, y: 17.0, z: 72.8, phaseOffsetMs: 1800, color: '#f472b6' }
    ];
    for (const bd of blinkDefs) {
      const bp = this._addBoxPlatform({
        x: bd.x,
        y: bd.y,
        z: bd.z,
        w: 2.2,
        h: 0.5,
        d: 2.2,
        color: bd.color,
        emissive: bd.color,
        emissiveIntensity: 0.35,
        type: 'blinking',
        obbyTier: 'hard'
      });
      bp.mesh.material.transparent = true;
      bp.baseColor = new THREE.Color(bd.color);
      bp.cycleMs = 5100;
      bp.solidMs = 2600;
      bp.warnMs = 900;
      bp.phaseOffsetMs = bd.phaseOffsetMs;
      this.blinkingPlatforms.push(bp);
    }

    // CP 3
    this._addBoxPlatform({
      x: 83.0,
      y: 18.0,
      z: 77.6,
      w: 4.8,
      h: 0.6,
      d: 4.8,
      color: '#f59e0b',
      emissive: '#d97706',
      emissiveIntensity: 0.25,
      type: 'checkpoint',
      checkpointIndex: 3,
      obbyTier: 'hard'
    });

    // Stage 3 -> CP 4: Chained High-Arc Bounce Pads & Sky Islands
    const addBouncePad = (x, y, z, bounceVel) => {
      const pad = this._addBoxPlatform({
        x,
        y,
        z,
        w: 2.2,
        h: 0.4,
        d: 2.2,
        color: '#ec4899',
        emissive: '#db2777',
        emissiveIntensity: 0.55,
        type: 'jumppad',
        bounceVel,
        obbyTier: 'hard'
      });
      this.jumpPads.push(pad);
    };
    addBouncePad(80.6, 18.5, 78.6, 21.5);
    this._addBoxPlatform({
      x: 74.8,
      y: 22.0,
      z: 81.2,
      w: 2.8,
      h: 0.5,
      d: 2.8,
      color: '#38bdf8',
      emissive: '#0284c7',
      emissiveIntensity: 0.22
    });
    addBouncePad(74.8, 22.45, 81.2, 22.0);
    this._addBoxPlatform({
      x: 68.6,
      y: 25.8,
      z: 84.4,
      w: 2.6,
      h: 0.5,
      d: 2.6,
      color: '#a78bfa',
      emissive: '#7c3aed',
      emissiveIntensity: 0.22
    });
    addBouncePad(68.6, 26.25, 84.4, 22.0);

    // CP 4
    this._addBoxPlatform({
      x: 61.5,
      y: 29.0,
      z: 85.5,
      w: 5.0,
      h: 0.6,
      d: 5.0,
      color: '#f59e0b',
      emissive: '#d97706',
      emissiveIntensity: 0.25,
      type: 'checkpoint',
      checkpointIndex: 4,
      obbyTier: 'hard'
    });

    // Stage 4 -> CP 5: 3 Sky Discs with Spinning Sweeper Beams (Jump Over!)
    const sweeperDefs = [
      { x: 59.2, y: 30.2, z: 79.4, size: 5.4, radius: 2.75, speed: 1.75, color: '#1e1b4b' },
      { x: 56.6, y: 31.6, z: 73.0, size: 5.2, radius: 2.65, speed: -2.0, color: '#312e81' },
      { x: 54.6, y: 33.0, z: 66.8, size: 5.0, radius: 2.55, speed: 2.2, color: '#1e1b4b' }
    ];
    const beamMat = new THREE.MeshStandardMaterial({
      color: '#fb7185',
      emissive: '#f43f5e',
      emissiveIntensity: 0.85,
      roughness: 0.2
    });
    const hubMat = new THREE.MeshStandardMaterial({
      color: '#fbbf24',
      emissive: '#d97706',
      emissiveIntensity: 0.4,
      metalness: 0.6,
      roughness: 0.25
    });
    for (const sd of sweeperDefs) {
      this._addBoxPlatform({
        x: sd.x,
        y: sd.y,
        z: sd.z,
        w: sd.size,
        h: 0.6,
        d: sd.size,
        color: sd.color,
        emissive: '#4f46e5',
        emissiveIntensity: 0.2
      });
      const beamY = sd.y + 0.82;
      const sweeperGroup = new THREE.Group();
      sweeperGroup.position.set(sd.x, beamY, sd.z);
      const hub = new THREE.Mesh(new THREE.CylinderGeometry(0.32, 0.38, 0.72, 12), hubMat);
      hub.position.y = -0.18;
      const bar = new THREE.Mesh(new THREE.CylinderGeometry(0.14, 0.14, sd.radius * 2, 10), beamMat);
      bar.rotation.z = Math.PI / 2;
      sweeperGroup.add(hub, bar);
      this.plazaGroup.add(sweeperGroup);
      this.sweeperHazards.push({
        group: sweeperGroup,
        cx: sd.x,
        cy: beamY,
        cz: sd.z,
        radius: sd.radius,
        speed: sd.speed,
        angle: 0
      });
    }

    // CP 5
    this._addBoxPlatform({
      x: 55.5,
      y: 34.4,
      z: 60.4,
      w: 4.8,
      h: 0.6,
      d: 4.8,
      color: '#f59e0b',
      emissive: '#d97706',
      emissiveIntensity: 0.25,
      type: 'checkpoint',
      checkpointIndex: 5,
      obbyTier: 'hard'
    });

    // Stage 5 -> Diamond Crown Summit: 2 Alternating Blinkers + Fast Diagonal Mover
    for (const bd of [
      { x: 59.5, y: 35.8, z: 62.6, phaseOffsetMs: 0, color: '#38bdf8' },
      { x: 63.6, y: 37.3, z: 64.6, phaseOffsetMs: 1700, color: '#a855f7' }
    ]) {
      const bp = this._addBoxPlatform({
        x: bd.x,
        y: bd.y,
        z: bd.z,
        w: 2.0,
        h: 0.5,
        d: 2.0,
        color: bd.color,
        emissive: bd.color,
        emissiveIntensity: 0.35,
        type: 'blinking',
        obbyTier: 'hard'
      });
      bp.mesh.material.transparent = true;
      bp.baseColor = new THREE.Color(bd.color);
      bp.cycleMs = 4800;
      bp.solidMs = 2400;
      bp.warnMs = 850;
      bp.phaseOffsetMs = bd.phaseOffsetMs;
      this.blinkingPlatforms.push(bp);
    }

    const finalMover = this._addBoxPlatform({
      x: 67.5,
      y: 38.8,
      z: 66.6,
      w: 2.5,
      h: 0.5,
      d: 2.5,
      color: '#f472b6',
      emissive: '#ec4899',
      emissiveIntensity: 0.3,
      type: 'moving',
      obbyTier: 'hard'
    });
    finalMover.baseX = 67.5;
    finalMover.baseY = 38.8;
    finalMover.baseZ = 66.6;
    finalMover.ampX = 1.8;
    finalMover.ampZ = 1.8;
    finalMover.speed = 2.2;
    this.movingPlatforms.push(finalMover);

    // Diamond Crown Summit Pad
    this._addBoxPlatform({
      x: 72.6,
      y: 40.2,
      z: 69.2,
      w: 7.6,
      h: 1.0,
      d: 7.6,
      color: '#e0f2fe',
      emissive: '#38bdf8',
      emissiveIntensity: 0.32,
      type: 'gauntlet-trophy-pad',
      checkpointIndex: 6,
      obbyTier: 'hard'
    });

    // Spinning Diamond Crown Monument
    const crownGroup = new THREE.Group();
    const diamondMat = new THREE.MeshStandardMaterial({
      color: '#67e8f9',
      emissive: '#38bdf8',
      emissiveIntensity: 0.65,
      metalness: 0.4,
      roughness: 0.12
    });
    const goldMat = new THREE.MeshStandardMaterial({
      color: '#fbbf24',
      emissive: '#f59e0b',
      emissiveIntensity: 0.4,
      metalness: 0.72,
      roughness: 0.18
    });
    const basePedestal = new THREE.Mesh(new THREE.CylinderGeometry(1.15, 1.45, 0.7, 8), goldMat);
    const crownRing = new THREE.Mesh(new THREE.TorusGeometry(0.95, 0.16, 10, 24), goldMat);
    crownRing.rotation.x = Math.PI / 2;
    crownRing.position.y = 1.15;
    const gemCore = new THREE.Mesh(new THREE.OctahedronGeometry(1.1, 0), diamondMat);
    gemCore.position.y = 1.75;
    crownGroup.add(basePedestal, crownRing, gemCore);
    crownGroup.position.set(72.6, 41.3, 69.2);
    this.plazaGroup.add(crownGroup);
    this.hardTrophyMesh = crownGroup;
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
      this.dtiStageDecorGroup.add(arch);
    }

    const wallMat = new THREE.MeshStandardMaterial({
      color: '#4c1d95',
      roughness: 0.38,
      metalness: 0.18
    });
    const wall = new THREE.Mesh(new THREE.BoxGeometry(10.2, 6.2, 0.55), wallMat);
    wall.position.set(rx, 3.3, -12.4);
    wall.castShadow = true;
    this.dtiStageDecorGroup.add(wall);

    const frameTrim = new THREE.Mesh(new THREE.BoxGeometry(10.6, 6.6, 0.35), goldRailMat);
    frameTrim.position.set(rx, 3.3, -12.55);
    this.dtiStageDecorGroup.add(frameTrim);

    const crestGroup = new THREE.Group();
    const starCore = new THREE.Mesh(new THREE.OctahedronGeometry(1.15, 0), goldRailMat);
    const starHalo = new THREE.Mesh(
      new THREE.TorusGeometry(1.55, 0.12, 12, 32),
      new THREE.MeshStandardMaterial({ color: '#f472b6', emissive: '#ec4899', emissiveIntensity: 0.65 })
    );
    crestGroup.add(starCore, starHalo);
    crestGroup.position.set(rx, 4.1, -11.95);
    this.dtiStageDecorGroup.add(crestGroup);
    this.runwayStarCrest = crestGroup;

    this._buildExpansionLandmarks();
  }

  _buildExpansionLandmarks() {
    // 1. Starlight Crafting Workshop at (12.5, 0, 6.5)
    this.craftingBenchGroup.position.set(
      this.craftingBenchPos.x,
      this.craftingBenchPos.y,
      this.craftingBenchPos.z
    );
    const benchModelHost = new THREE.Group();
    const benchTable = new THREE.Mesh(
      new THREE.BoxGeometry(2.1, 0.92, 1.1),
      new THREE.MeshStandardMaterial({ color: '#78350f', roughness: 0.65 })
    );
    benchTable.position.y = 0.46;
    benchTable.castShadow = true;
    const benchCrystal = new THREE.Mesh(
      new THREE.OctahedronGeometry(0.28, 0),
      new THREE.MeshStandardMaterial({
        color: '#38bdf8',
        emissive: '#0ea5e9',
        emissiveIntensity: 0.95,
        roughness: 0.15
      })
    );
    benchCrystal.position.set(-0.55, 1.18, 0);
    benchModelHost.add(benchTable, benchCrystal);
    const benchRing = new THREE.Mesh(
      new THREE.RingGeometry(1.45, 1.72, 32),
      new THREE.MeshBasicMaterial({
        color: '#38bdf8',
        transparent: true,
        opacity: 0.65,
        side: THREE.DoubleSide
      })
    );
    benchRing.rotation.x = -Math.PI / 2;
    benchRing.position.y = 0.04;
    const benchBanner = createWorldBannerSprite(
      '⚒️ STARLIGHT WORKSHOP',
      'Press E or Click • Craft Blades, Wands, Charms & Pet Eggs!',
      '#38bdf8'
    );
    benchBanner.position.set(0, 2.55, 0);
    this.craftingBenchGroup.add(benchModelHost, benchRing, benchBanner);
    this.craftingBenchGroup.userData = {
      landmarkKind: 'crafting_bench',
      modelHost: benchModelHost,
      floatingGem: benchCrystal
    };
    this.plazaColliders.push({
      mesh: this.craftingBenchGroup,
      w: 2.2,
      h: 1.0,
      d: 1.2,
      minX: this.craftingBenchPos.x - 1.1,
      maxX: this.craftingBenchPos.x + 1.1,
      minZ: this.craftingBenchPos.z - 0.6,
      maxZ: this.craftingBenchPos.z + 0.6,
      topY: 1.0,
      bottomY: 0,
      type: 'solid',
      cameraOccluder: false
    });

    // 2. Six Claimable Neighborhood Plot Mailboxes
    const defaultPlots = [
      { plotId: 'plot_1', defaultName: 'Sunrise Lawn Plot #1', mailboxPos: [52.0, 0.0, -16.2], mailboxRot: 0.0 },
      { plotId: 'plot_2', defaultName: 'Rosewater Villa', mailboxPos: [79.0, 0.0, -16.2], mailboxRot: 0.0 },
      { plotId: 'plot_3', defaultName: 'Starlight Cottage', mailboxPos: [49.5, 0.0, 10.2], mailboxRot: Math.PI },
      { plotId: 'plot_4', defaultName: 'Meadow View Plot #4', mailboxPos: [82.0, 0.0, 10.2], mailboxRot: Math.PI },
      { plotId: 'plot_5', defaultName: 'Woodland Chalet', mailboxPos: [-61.5, 0.0, -5.2], mailboxRot: 0.0 },
      { plotId: 'plot_6', defaultName: 'Pinecrest Plot #6', mailboxPos: [-64.0, 0.0, 21.2], mailboxRot: Math.PI }
    ];
    for (const p of defaultPlots) {
      this._upsertMailboxEntry(p);
    }

    // 3. Obby Speedrun Start/Finish Laser Gates + 3D Top-5 Leaderboard Boards
    const buildGateVisual = (x, y, z, ry, isFinish, titleText, subText) => {
      const grp = new THREE.Group();
      grp.position.set(x, y, z);
      grp.rotation.y = ry;
      const modelHost = new THREE.Group();
      const postMat = new THREE.MeshStandardMaterial({ color: '#1e293b', metalness: 0.7, roughness: 0.25 });
      const beamColor = isFinish ? '#fbbf24' : '#34d399';
      const beamMat = new THREE.MeshBasicMaterial({
        color: beamColor,
        transparent: true,
        opacity: 0.42,
        side: THREE.DoubleSide,
        depthWrite: false
      });
      for (const side of [-1, 1]) {
        const post = new THREE.Mesh(new THREE.CylinderGeometry(0.14, 0.18, 3.4, 12), postMat);
        post.position.set(side * 1.85, 1.7, 0);
        modelHost.add(post);
      }
      const curtain = new THREE.Mesh(new THREE.PlaneGeometry(3.5, 3.0), beamMat);
      curtain.position.set(0, 1.65, 0);
      modelHost.add(curtain);
      const banner = createWorldBannerSprite(titleText, subText, beamColor);
      banner.position.set(0, 4.05, 0);
      grp.add(modelHost, banner);
      this.obbyGatesGroup.add(grp);
      return { grp, modelHost, isFinish, beamColor, pos: { x, y, z } };
    };

    this.obbyGates = {
      easy: {
        start: buildGateVisual(
          0.0,
          0.0,
          -13.5,
          0,
          false,
          '⏱️ SKY OBBY START',
          'Run Through on Foot to Start Timer!',
          '#34d399'
        ),
        finish: buildGateVisual(
          6.5,
          16.5,
          -20.0,
          0,
          true,
          '🏁 SKY OBBY FINISH',
          'Cross Gate to Record Best Time!',
          '#fbbf24'
        ),
        radius: 4.2
      },
      hard: {
        start: buildGateVisual(
          54.2,
          0.0,
          52.0,
          Math.PI * 0.25,
          false,
          '⏱️ PRO GAUNTLET START',
          'Run Through on Foot to Start Timer!',
          '#34d399'
        ),
        finish: buildGateVisual(
          72.6,
          40.2,
          69.2,
          Math.PI * 0.25,
          true,
          '🏁 PRO GAUNTLET FINISH',
          'Cross Gate to Claim Record!',
          '#fbbf24'
        ),
        radius: 4.8
      }
    };

    const buildLeaderboardBoard = (courseKey, x, y, z, ry, heading, accentHex) => {
      const grp = new THREE.Group();
      grp.position.set(x, y, z);
      grp.rotation.y = ry;
      grp.userData = { landmarkKind: 'obby_board', course: courseKey };
      const frameHost = new THREE.Group();
      const postL = new THREE.Mesh(
        new THREE.BoxGeometry(0.18, 3.2, 0.18),
        new THREE.MeshStandardMaterial({ color: '#1e293b', metalness: 0.72, roughness: 0.28 })
      );
      postL.position.set(-1.55, 1.6, 0);
      const postR = postL.clone();
      postR.position.set(1.55, 1.6, 0);
      const backing = new THREE.Mesh(
        new THREE.BoxGeometry(3.4, 2.35, 0.16),
        new THREE.MeshStandardMaterial({ color: '#0f172a', metalness: 0.5, roughness: 0.35 })
      );
      backing.position.set(0, 2.25, 0);
      frameHost.add(postL, postR, backing);

      const tex = createObbyLeaderboardTexture(heading, [], accentHex);
      const screenMat = new THREE.MeshBasicMaterial({ map: tex });
      const screenMesh = new THREE.Mesh(new THREE.PlaneGeometry(3.08, 2.12), screenMat);
      screenMesh.position.set(0, 2.25, 0.12);
      screenMesh.userData = { landmarkKind: 'obby_board', course: courseKey };
      grp.add(frameHost, screenMesh);
      this.obbyGatesGroup.add(grp);
      this.obbyBoardScreens[courseKey] = {
        grp,
        frameHost,
        screenMat,
        screenMesh,
        heading,
        accentHex,
        pos: { x, y, z }
      };
    };

    buildLeaderboardBoard('easy', -5.8, 0, -12.6, 0.32, '🏆 SKY OBBY TOP 5', '#38bdf8');
    buildLeaderboardBoard('hard', 48.2, 0, 49.5, 0.56, '🔥 PRO GAUNTLET TOP 5', '#f472b6');

    // 4. DTI Fashion Show Dynamic Catwalk Spotlight
    const spotConeMat = new THREE.MeshBasicMaterial({
      color: '#fde047',
      transparent: true,
      opacity: 0.18,
      side: THREE.DoubleSide,
      depthWrite: false,
      blending: THREE.AdditiveBlending
    });
    const spotCone = new THREE.Mesh(new THREE.ConeGeometry(1.85, 6.4, 24, 1, true), spotConeMat);
    spotCone.position.set(0, 3.25, 0);
    const spotRing = new THREE.Mesh(
      new THREE.RingGeometry(1.15, 1.55, 32),
      new THREE.MeshBasicMaterial({
        color: '#fde047',
        transparent: true,
        opacity: 0.78,
        side: THREE.DoubleSide
      })
    );
    spotRing.rotation.x = -Math.PI / 2;
    spotRing.position.y = 0.52;
    const spotPoint = new THREE.PointLight('#fef08a', 2.4, 14);
    spotPoint.position.set(0, 4.8, 0);
    this.runwaySpotlightGroup.add(spotCone, spotRing, spotPoint);
    this.runwaySpotlightGroup.position.set(-24, 0, 2.0);
    this.runwaySpotlightGroup.visible = false;

    // 5. Dynamic Aurora Night Sky Ribbons + Twinkling Starfield
    this.auroraMaterials = [];
    const ribbonSpecs = [
      { y: 68, z: -48, rotY: 0.14, scaleX: 1.15 },
      { y: 74, z: -8, rotY: -0.18, scaleX: 1.25 },
      { y: 71, z: 34, rotY: 0.1, scaleX: 1.1 }
    ];
    for (const rs of ribbonSpecs) {
      const mat = createAuroraRibbonMaterial();
      const mesh = new THREE.Mesh(new THREE.PlaneGeometry(165, 26, 48, 8), mat);
      mesh.position.set(0, rs.y, rs.z);
      mesh.rotation.y = rs.rotY;
      mesh.rotation.x = Math.PI * 0.18;
      mesh.scale.x = rs.scaleX;
      this.auroraGroup.add(mesh);
      this.auroraMaterials.push(mat);
    }

    const starCount = 260;
    const starPositions = new Float32Array(starCount * 3);
    for (let i = 0; i < starCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.random() * Math.PI * 0.44 + 0.06;
      const r = 148 + Math.random() * 24;
      starPositions[i * 3] = Math.cos(theta) * Math.sin(phi) * r;
      starPositions[i * 3 + 1] = Math.cos(phi) * r;
      starPositions[i * 3 + 2] = Math.sin(theta) * Math.sin(phi) * r;
    }
    const starGeo = new THREE.BufferGeometry();
    starGeo.setAttribute('position', new THREE.BufferAttribute(starPositions, 3));
    const starMat = new THREE.PointsMaterial({
      color: '#f8fafc',
      size: 1.25,
      transparent: true,
      opacity: 0,
      depthWrite: false
    });
    this.nightStarPoints = new THREE.Points(starGeo, starMat);
    this.auroraGroup.add(this.nightStarPoints);
  }

  _upsertMailboxEntry(plot) {
    if (!plot || !plot.plotId) return;
    let entry = this.mailboxesMap.get(plot.plotId);
    const mbPos = Array.isArray(plot.mailboxPos) ? plot.mailboxPos : [52, 0, -16.2];
    const mbRot = Number(plot.mailboxRotY ?? plot.mailboxRot) || 0;

    if (!entry) {
      const group = new THREE.Group();
      group.position.set(mbPos[0], mbPos[1] || 0, mbPos[2]);
      group.rotation.y = mbRot;
      const modelHost = new THREE.Group();
      const post = new THREE.Mesh(
        new THREE.CylinderGeometry(0.07, 0.09, 1.05, 8),
        new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.4 })
      );
      post.position.y = 0.52;
      const box = new THREE.Mesh(
        new THREE.BoxGeometry(0.38, 0.28, 0.52),
        new THREE.MeshStandardMaterial({ color: '#ec4899', roughness: 0.35, metalness: 0.25 })
      );
      box.position.y = 1.12;
      modelHost.add(post, box);
      if (envTemplates.has('Env_Plot_Mailbox')) {
        const glbMb = cloneEnvTemplate('Env_Plot_Mailbox');
        if (glbMb) {
          modelHost.clear();
          modelHost.add(glbMb);
        }
      }

      const tex = createPlotMailboxTexture(plot);
      const spriteMat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false });
      const sprite = new THREE.Sprite(spriteMat);
      sprite.scale.set(3.5, 1.2, 1);
      sprite.position.set(0, 2.08, 0);

      group.add(modelHost, sprite);
      group.traverse((n) => {
        n.userData = { ...(n.userData || {}), landmarkKind: 'mailbox', plotId: plot.plotId };
      });
      this.mailboxesGroup.add(group);
      entry = { plotId: plot.plotId, data: plot, group, modelHost, sprite };
      this.mailboxesMap.set(plot.plotId, entry);
      return;
    }

    entry.data = { ...entry.data, ...plot };
    entry.group.position.set(mbPos[0], mbPos[1] || 0, mbPos[2]);
    entry.group.rotation.y = mbRot;
    if (entry.sprite.material.map) {
      entry.sprite.material.map.dispose();
    }
    entry.sprite.material.map = createPlotMailboxTexture(entry.data);
    entry.sprite.material.needsUpdate = true;
  }

  _upgradeEnvironmentModels() {
    if (envTemplates.size === 0) return;

    // 1. Replace all 32 procedural box trees with realistic Blender/PolyHaven CC0 botanical trees
    if (this.plazaTreeSpecs.length > 0 && envTemplates.has('Env_Tree_Blossom')) {
      this.plazaTreeGroup.clear();
      for (const spec of this.plazaTreeSpecs) {
        const treeObj = cloneEnvTemplate(spec.variant) || cloneEnvTemplate('Env_Tree_Blossom');
        if (!treeObj) continue;
        treeObj.position.set(spec.x, 0, spec.z);
        treeObj.rotation.y = spec.yaw;
        treeObj.scale.setScalar(spec.scale);
        this.plazaTreeGroup.add(treeObj);
      }
    }

    // 2. Upgrade DTI Fashion Runway Stage to Couture Blender Models + CC0 Studio Spotlights, Chandeliers, Benches & Lamps
    const rx = -24;
    if (envTemplates.has('Env_DTI_Arch') && envTemplates.has('Env_DTI_Backdrop')) {
      this.dtiStageDecorGroup.clear();

      // 3 Couture Fluted Marble & Rose-Gold Arches + Hanging Crystal Chandeliers
      for (const az of [-5.2, 1.5, 8.2]) {
        const arch = cloneEnvTemplate('Env_DTI_Arch');
        if (arch) {
          arch.position.set(rx, 0.0, az);
          this.dtiStageDecorGroup.add(arch);
        }
        const chandelier = cloneEnvTemplate('Env_DTI_Chandelier');
        if (chandelier) {
          chandelier.position.set(rx, 6.75, az);
          this.dtiStageDecorGroup.add(chandelier);
        }
      }

      // Pleated Royal Velvet Couture Backstage Photo Wall + Spinning Faceted Starlight Crest
      const backdrop = cloneEnvTemplate('Env_DTI_Backdrop');
      if (backdrop) {
        backdrop.position.set(rx, 0.45, -12.4);
        this.dtiStageDecorGroup.add(backdrop);
      }
      const crest = cloneEnvTemplate('Env_DTI_StarCrest');
      if (crest) {
        crest.position.set(rx, 4.15, -11.78);
        crest.userData.isGlbCrest = true;
        this.dtiStageDecorGroup.add(crest);
        this.runwayStarCrest = crest;
      }

      // Draped Velvet Rope Gold Stanchions lining both catwalk edges
      for (const side of [-1, 1]) {
        for (let i = 0; i < 7; i++) {
          const st = cloneEnvTemplate('Env_DTI_Stanchion');
          if (st) {
            st.position.set(rx + side * 2.95, 0.0, -5.2 + i * 2.0);
            this.dtiStageDecorGroup.add(st);
          }
        }
      }

      // 4 PolyHaven CC0 Studio Tripod Spotlights flanking the runway aimed inward
      const spotSpecs = [
        { x: rx - 4.8, z: -2.0, ry: Math.PI * 0.35 },
        { x: rx + 4.8, z: -2.0, ry: -Math.PI * 0.35 },
        { x: rx - 4.8, z: 7.5, ry: Math.PI * 0.65 },
        { x: rx + 4.8, z: 7.5, ry: -Math.PI * 0.65 }
      ];
      for (const sp of spotSpecs) {
        const spot = cloneEnvTemplate('Env_DTI_Spotlight');
        if (spot) {
          spot.position.set(sp.x, 0.0, sp.z);
          spot.rotation.y = sp.ry;
          this.dtiStageDecorGroup.add(spot);
        }
      }

      // 4 VIP Front-Row Audience Seating Benches flanking the catwalk
      const benchSpecs = [
        { x: rx - 5.2, z: 0.6, ry: Math.PI * 0.5 },
        { x: rx - 5.2, z: 4.4, ry: Math.PI * 0.5 },
        { x: rx + 5.2, z: 0.6, ry: -Math.PI * 0.5 },
        { x: rx + 5.2, z: 4.4, ry: -Math.PI * 0.5 }
      ];
      for (const bs of benchSpecs) {
        const bench = cloneEnvTemplate('Env_DTI_Bench');
        if (bench) {
          bench.position.set(bs.x, 0.0, bs.z);
          bench.rotation.y = bs.ry;
          this.dtiStageDecorGroup.add(bench);
        }
      }

      // 2 Ornate Streetlamps framing the Runway Podium Entrance
      for (const side of [-1, 1]) {
        const lamp = cloneEnvTemplate('Env_DTI_Lamp');
        if (lamp) {
          lamp.position.set(rx + side * 4.5, 0.0, 13.6);
          this.dtiStageDecorGroup.add(lamp);
        }
      }
    }

    // 3. Upgrade Plaza Campfire & Glamping Clearing with Blender & PolyHaven CC0 Models
    if (envTemplates.has('Env_Campfire_Pit')) {
      this.campfireAnchor.clear();
      const pit = cloneEnvTemplate('Env_Campfire_Pit');
      if (pit) this.campfireAnchor.add(pit);

      // Replace the 2 plain box bench meshes with sculpted cedar log benches (keeping exact walkable colliders!)
      if (envTemplates.has('Env_Camp_LogBench')) {
        for (const ph of this.campBenchPlaceholders) {
          if (ph) ph.visible = false;
        }
        const logBenchS = cloneEnvTemplate('Env_Camp_LogBench');
        if (logBenchS) {
          logBenchS.position.set(0, 0, 3.2);
          this.campfireAnchor.add(logBenchS);
        }
        const logBenchW = cloneEnvTemplate('Env_Camp_LogBench');
        if (logBenchW) {
          logBenchW.position.set(-3.2, 0, 0);
          logBenchW.rotation.y = Math.PI * 0.5;
          this.campfireAnchor.add(logBenchW);
        }
      }

      // Cozy Striped A-Frame Glamping Tent beside the clearing
      const tent = cloneEnvTemplate('Env_Camp_Tent');
      if (tent) {
        tent.position.set(-4.6, 0, 5.2);
        tent.rotation.y = Math.PI * 0.24;
        this.campfireAnchor.add(tent);
      }

      // Tree Stump Side Table + Glowing Brass Lantern + Enamel Camp Pot + Folding Stool + Boulders
      const stump = cloneEnvTemplate('Env_Camp_Stump');
      if (stump) {
        stump.position.set(2.6, 0, 0.6);
        this.campfireAnchor.add(stump);
      }
      const lantern = cloneEnvTemplate('Env_Camp_Lantern');
      if (lantern) {
        lantern.position.set(2.6, 0.72, 0.6);
        this.campfireAnchor.add(lantern);
      }
      const pot = cloneEnvTemplate('Env_Camp_Pot');
      if (pot) {
        pot.position.set(1.55, 0, -0.75);
        pot.rotation.y = 0.6;
        this.campfireAnchor.add(pot);
      }
      const stool = cloneEnvTemplate('Env_Camp_Stool');
      if (stool) {
        stool.position.set(2.3, 0, 2.75);
        stool.rotation.y = -0.5;
        this.campfireAnchor.add(stool);
      }
      const b1 = cloneEnvTemplate('Env_Camp_Boulder');
      if (b1) {
        b1.position.set(-4.8, 0, -2.6);
        b1.rotation.y = 0.85;
        b1.scale.setScalar(0.85);
        this.campfireAnchor.add(b1);
      }
      const b2 = cloneEnvTemplate('Env_Camp_Boulder');
      if (b2) {
        b2.position.set(3.8, 0, -2.2);
        b2.rotation.y = -1.2;
        b2.scale.setScalar(0.72);
        this.campfireAnchor.add(b2);
      }
    }

    // 4. Upgrade All Portals (Plaza -> Cavern & Cavern -> Plaza) to Blender Stargate Rings!
    if (envTemplates.has('Env_Stargate_Frame') && envTemplates.has('Env_Stargate_InnerRing')) {
      for (const sg of this.stargateInstances) {
        sg.frameHost.clear();
        const frame = cloneEnvTemplate('Env_Stargate_Frame');
        if (frame) sg.frameHost.add(frame);

        const innerRing = cloneEnvTemplate('Env_Stargate_InnerRing');
        if (innerRing) {
          innerRing.position.set(0, 2.95, 0);
          sg.frameHost.add(innerRing);
          sg.innerGlyphRing = innerRing;
        }
        sg.keystone = null;

        // Align Event Horizon shader disc, swirl halo & quantum motes cleanly to the 2.28m Stargate aperture
        if (sg.disc) {
          sg.disc.position.set(0, 2.95, 0);
          sg.disc.scale.setScalar(1.3);
        }
        if (sg.ring) {
          sg.ring.position.set(0, 2.95, 0.08);
          sg.ring.scale.setScalar(1.42);
        }
        if (sg.motes) {
          sg.motes.position.set(0, 2.95, 0);
          sg.motes.scale.setScalar(1.22);
        }
      }
    }

    // 5. Upgrade all active Crystal Cavern & Shadow Rift Arena Mobs to Sculpted Blender 5.2 Models!
    for (const entry of this.dungeonMobsMap.values()) {
      this._populateMobVisual(entry);
    }
    for (const entry of this.arenaMobsMap.values()) {
      this._populateMobVisual(entry);
    }

    // 6. Upgrade Expansion Landmarks (Crafting Workshop, Mailboxes, Obby Gates/Boards, Boss Altar & Dragon, Companion Pets/Mounts)
    if (envTemplates.has('Env_Crafting_Bench') && this.craftingBenchGroup.userData?.modelHost) {
      const benchGlb = cloneEnvTemplate('Env_Crafting_Bench');
      if (benchGlb) {
        this.craftingBenchGroup.userData.modelHost.clear();
        benchGlb.traverse((n) => {
          n.userData = { ...(n.userData || {}), landmarkKind: 'crafting_bench' };
        });
        this.craftingBenchGroup.userData.modelHost.add(benchGlb);
        this.craftingBenchGroup.userData.floatingGem = null;
      }
    }

    if (envTemplates.has('Env_Plot_Mailbox')) {
      for (const mb of this.mailboxesMap.values()) {
        const glbMb = cloneEnvTemplate('Env_Plot_Mailbox');
        if (glbMb) {
          mb.modelHost.clear();
          glbMb.traverse((n) => {
            n.userData = { ...(n.userData || {}), landmarkKind: 'mailbox', plotId: mb.plotId };
          });
          mb.modelHost.add(glbMb);
        }
      }
    }

    if (envTemplates.has('Env_Obby_Gate')) {
      for (const courseObj of Object.values(this.obbyGates)) {
        for (const gateRef of [courseObj.start, courseObj.finish]) {
          if (!gateRef || !gateRef.modelHost) continue;
          const glbGate = cloneEnvTemplate('Env_Obby_Gate');
          if (glbGate) {
            gateRef.modelHost.clear();
            const curtain = new THREE.Mesh(
              new THREE.PlaneGeometry(3.4, 3.1),
              new THREE.MeshBasicMaterial({
                color: gateRef.beamColor,
                transparent: true,
                opacity: 0.34,
                side: THREE.DoubleSide,
                depthWrite: false
              })
            );
            curtain.position.set(0, 1.75, 0);
            gateRef.modelHost.add(glbGate, curtain);
          }
        }
      }
    }

    if (envTemplates.has('Env_Leaderboard_Board')) {
      for (const [courseKey, bd] of Object.entries(this.obbyBoardScreens)) {
        const glbBoard = cloneEnvTemplate('Env_Leaderboard_Board');
        if (glbBoard && bd.frameHost) {
          bd.frameHost.clear();
          glbBoard.traverse((n) => {
            n.userData = { ...(n.userData || {}), landmarkKind: 'obby_board', course: courseKey };
          });
          bd.frameHost.add(glbBoard);
          bd.screenMesh.position.set(0, 2.15, 0.14);
        }
      }
    }

    if (envTemplates.has('Env_Boss_Altar') && this.bossAltarGroup.userData?.modelHost) {
      const glbAltar = cloneEnvTemplate('Env_Boss_Altar');
      if (glbAltar) {
        this.bossAltarGroup.userData.modelHost.clear();
        glbAltar.traverse((n) => {
          n.userData = { ...(n.userData || {}), landmarkKind: 'boss_altar' };
        });
        this.bossAltarGroup.userData.modelHost.add(glbAltar);
      }
    }

    if (this.bossDragonEntry) {
      this._populateBossDragonVisual(this.bossDragonEntry);
    }

    if (this.selfAvatar) {
      this.selfAvatar.syncCompanions(null, true);
    }
    for (const av of this.remoteAvatars.values()) {
      av.syncCompanions(null, true);
    }
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

  _buildArenaEnvironment() {
    // Sunken catch floor beneath molten lava river & outer lava moat
    const magmaBedMat = new THREE.MeshStandardMaterial({
      color: '#450a0a',
      roughness: 0.85,
      metalness: 0.1
    });
    const magmaBed = new THREE.Mesh(new THREE.BoxGeometry(120, 2, 120), magmaBedMat);
    magmaBed.position.set(0, -1.65, 0);
    this.arenaGroup.add(magmaBed);
    this.arenaColliders.push({
      mesh: magmaBed,
      w: 120,
      h: 2,
      d: 120,
      minX: -60,
      maxX: 60,
      minZ: -60,
      maxZ: 60,
      topY: -0.65,
      bottomY: -2.65,
      type: 'ground'
    });

    // Glowing Molten Lava Sea (Outer Moat + Central Lava River at z ∈ [-7, -3])
    const lavaSurfaceMat = new THREE.MeshStandardMaterial({
      color: '#ff4500',
      emissive: '#ea580c',
      emissiveIntensity: 1.35,
      roughness: 0.25,
      metalness: 0.1
    });
    const lavaSea = new THREE.Mesh(new THREE.PlaneGeometry(116, 116), lavaSurfaceMat);
    lavaSea.rotation.x = -Math.PI / 2;
    lavaSea.position.set(0, -0.45, 0);
    this.arenaGroup.add(lavaSea);
    this.arenaLavaMeshes.push(lavaSea);

    // South Basalt Mainland (x ∈ [-30, 30], z ∈ [-3, 33], topY = 0)
    this._addBoxPlatform(
      {
        x: 0,
        y: -0.5,
        z: 15,
        w: 60,
        h: 1.0,
        d: 36,
        color: '#1c1917',
        emissive: '#450a0a',
        emissiveIntensity: 0.14
      },
      this.arenaGroup,
      this.arenaColliders
    );

    // North Basalt Citadel Mainland (x ∈ [-30, 30], z ∈ [-33, -7], topY = 0)
    this._addBoxPlatform(
      {
        x: 0,
        y: -0.5,
        z: -20,
        w: 60,
        h: 1.0,
        d: 26,
        color: '#18181b',
        emissive: '#7f1d1d',
        emissiveIntensity: 0.18
      },
      this.arenaGroup,
      this.arenaColliders
    );

    // 3 Basalt Stone Bridges spanning the Molten Lava River (x = -16, 0, +16; z ∈ [-7.4, -2.6])
    const bridgeRuneMat = new THREE.MeshStandardMaterial({
      color: '#fb923c',
      emissive: '#ea580c',
      emissiveIntensity: 0.85,
      roughness: 0.25
    });
    for (const bx of [-16, 0, 16]) {
      this._addBoxPlatform(
        {
          x: bx,
          y: 0.08,
          z: -5.0,
          w: 5.2,
          h: 0.36,
          d: 5.2,
          color: '#292524',
          emissive: '#9a3412',
          emissiveIntensity: 0.22
        },
        this.arenaGroup,
        this.arenaColliders
      );
      for (const side of [-1, 1]) {
        const curb = new THREE.Mesh(new THREE.BoxGeometry(0.24, 0.28, 5.2), bridgeRuneMat);
        curb.position.set(bx + side * 2.5, 0.32, -5.0);
        this.arenaGroup.add(curb);
      }
    }

    // Safe Sanctuary Shield Pad around Spawn (0, 0, 25) & Return Stargate (0, 0, 30)
    const sanctDisc = new THREE.Mesh(
      new THREE.CylinderGeometry(6.8, 7.1, 0.1, 40),
      new THREE.MeshStandardMaterial({
        color: '#064e3b',
        emissive: '#10b981',
        emissiveIntensity: 0.42,
        roughness: 0.3
      })
    );
    sanctDisc.position.set(0, 0.05, 25);
    const sanctHalo = new THREE.Mesh(
      new THREE.TorusGeometry(6.85, 0.1, 10, 48),
      new THREE.MeshBasicMaterial({ color: '#34d399', transparent: true, opacity: 0.82 })
    );
    sanctHalo.rotation.x = Math.PI / 2;
    sanctHalo.position.set(0, 0.12, 25);
    const sanctBanner = createWorldBannerSprite(
      '🛡️ SAFE SANCTUARY CIRCLE',
      'Monsters Cannot Enter Here • Drink Potions or Return to Plaza!',
      '#34d399'
    );
    sanctBanner.position.set(0, 3.1, 20.2);
    this.arenaGroup.add(sanctDisc, sanctHalo, sanctBanner);

    // Return Stargate to Starlight Plaza at (0, 0, 30)
    this._createPortalArch(
      {
        x: this.arenaReturnPortalPos.x,
        y: this.arenaReturnPortalPos.y,
        z: this.arenaReturnPortalPos.z,
        title: '🌟 STARLIGHT PLAZA',
        subtitle: 'Step In or Press E • Return Safely to Plaza',
        primaryHex: '#34d399',
        secondaryHex: '#38bdf8'
      },
      this.arenaGroup
    );

    // Atmospheric Volcanic Point Lights
    const southGlow = new THREE.PointLight('#fb923c', 2.6, 54);
    southGlow.position.set(0, 8.5, 6);
    const northBossGlow = new THREE.PointLight('#ef4444', 3.0, 58);
    northBossGlow.position.set(0, 10.5, -22);
    this.arenaGroup.add(southGlow, northBossGlow);

    // Jagged Obsidian Perimeter Spires & Magma Pillars
    const obsidianMat = new THREE.MeshStandardMaterial({
      color: '#0f172a',
      roughness: 0.35,
      metalness: 0.55
    });
    const crimsonCrystalMat = new THREE.MeshStandardMaterial({
      color: '#f87171',
      emissive: '#dc2626',
      emissiveIntensity: 0.82,
      roughness: 0.18,
      metalness: 0.35
    });
    const spireSpots = [
      [-26, 22], [26, 22], [-27, 4], [27, 4],
      [-27, -12], [27, -12], [-25, -29], [25, -29],
      [0, -31]
    ];
    spireSpots.forEach(([sx, sz], idx) => {
      const grp = new THREE.Group();
      const tall = new THREE.Mesh(new THREE.ConeGeometry(1.5, 7.2, 5), obsidianMat);
      tall.position.y = 3.5;
      tall.rotation.z = idx % 2 === 0 ? 0.1 : -0.1;
      const shard = new THREE.Mesh(new THREE.ConeGeometry(0.82, 4.1, 5), crimsonCrystalMat);
      shard.position.set(idx % 2 === 0 ? 1.15 : -1.15, 1.9, 0.5);
      shard.rotation.z = idx % 2 === 0 ? -0.26 : 0.26;
      grp.add(tall, shard);
      grp.position.set(sx, 0, sz);
      this.arenaGroup.add(grp);
    });

    // North Island Obsidian Rift Dragon Summoning Altar at (0, 0, -24.5)
    this.bossAltarGroup.position.set(
      this.bossAltarPos.x,
      this.bossAltarPos.y,
      this.bossAltarPos.z
    );
    const altarModelHost = new THREE.Group();
    const altarPlinth = new THREE.Mesh(
      new THREE.CylinderGeometry(1.85, 2.25, 0.62, 8),
      obsidianMat
    );
    altarPlinth.position.y = 0.31;
    const altarGem = new THREE.Mesh(
      new THREE.OctahedronGeometry(0.52, 0),
      new THREE.MeshStandardMaterial({
        color: '#c084fc',
        emissive: '#9333ea',
        emissiveIntensity: 1.1,
        roughness: 0.14
      })
    );
    altarGem.position.y = 1.45;
    altarModelHost.add(altarPlinth, altarGem);
    const altarHalo = new THREE.Mesh(
      new THREE.RingGeometry(2.35, 2.65, 36),
      new THREE.MeshBasicMaterial({
        color: '#c084fc',
        transparent: true,
        opacity: 0.72,
        side: THREE.DoubleSide
      })
    );
    altarHalo.rotation.x = -Math.PI / 2;
    altarHalo.position.y = 0.05;
    const altarBanner = createWorldBannerSprite(
      '🐉 OBSIDIAN DRAGON ALTAR',
      'Press E or Click • Summon Co-Op World Boss!',
      '#c084fc'
    );
    altarBanner.position.set(0, 3.45, 0);
    this.bossAltarGroup.add(altarModelHost, altarHalo, altarBanner);
    this.bossAltarGroup.userData = {
      landmarkKind: 'boss_altar',
      modelHost: altarModelHost,
      banner: altarBanner
    };
  }

  _populateBossDragonVisual(entry) {
    if (!entry || !entry.visual) return;
    entry.visual.clear();
    entry.leftWing = null;
    entry.rightWing = null;
    const glbBody = cloneEnvTemplate('Env_Boss_ObsidianDragon_Body');
    const glbWingL = cloneEnvTemplate('Env_Boss_ObsidianDragon_WingL');
    const glbWingR = cloneEnvTemplate('Env_Boss_ObsidianDragon_WingR');
    if (glbBody && glbWingL && glbWingR) {
      glbWingL.position.set(-0.95, 2.35, -0.15);
      glbWingR.position.set(0.95, 2.35, -0.15);
      entry.leftWing = glbWingL;
      entry.rightWing = glbWingR;
      entry.visual.add(glbBody, glbWingL, glbWingR);
      return;
    }
    const bodyMat = new THREE.MeshStandardMaterial({
      color: '#0f172a',
      emissive: '#7e22ce',
      emissiveIntensity: 0.35,
      metalness: 0.55,
      roughness: 0.3
    });
    const wingMat = new THREE.MeshStandardMaterial({
      color: '#c084fc',
      emissive: '#dc2626',
      emissiveIntensity: 0.72,
      side: THREE.DoubleSide
    });
    const body = new THREE.Mesh(new THREE.ConeGeometry(1.25, 3.4, 8), bodyMat);
    body.position.y = 1.9;
    entry.leftWing = new THREE.Mesh(new THREE.BoxGeometry(2.6, 1.35, 0.1), wingMat);
    entry.leftWing.position.set(-1.6, 2.2, 0);
    entry.rightWing = new THREE.Mesh(new THREE.BoxGeometry(2.6, 1.35, 0.1), wingMat);
    entry.rightWing.position.set(1.6, 2.2, 0);
    entry.visual.add(body, entry.leftWing, entry.rightWing);
  }

  _ensureBossDragonEntry() {
    if (this.bossDragonEntry) return this.bossDragonEntry;
    const group = new THREE.Group();
    group.visible = false;
    const visual = new THREE.Group();
    const label = createWorldBannerSprite(
      '🐉 Obsidian Rift Dragon',
      'JUMP over Purple Shockwaves! Attack with Blade, Wand, Ray Gun or Stomp!',
      '#c084fc'
    );
    label.position.set(0, 4.95, 0);
    group.add(visual, label);
    this.arenaEntitiesGroup.add(group);
    const entry = {
      group,
      visual,
      leftWing: null,
      rightWing: null,
      label,
      data: null,
      lastHpSig: '',
      targetPos: new THREE.Vector3(0, 0, -22),
      lastStompAt: 0
    };
    this._populateBossDragonVisual(entry);
    this.bossDragonEntry = entry;
    return entry;
  }

  _buildPrisonEnvironment() {
    const stoneFloorMat = new THREE.MeshStandardMaterial({
      color: '#334155',
      roughness: 0.78,
      metalness: 0.14
    });
    const yardGround = new THREE.Mesh(new THREE.BoxGeometry(34, 2, 34), stoneFloorMat);
    yardGround.position.set(0, -1, 0);
    yardGround.receiveShadow = true;
    this.prisonGroup.add(yardGround);
    this.prisonColliders.push({
      mesh: yardGround,
      w: 34,
      h: 2,
      d: 34,
      minX: -17,
      maxX: 17,
      minZ: -17,
      maxZ: 17,
      topY: 0,
      bottomY: -2,
      type: 'ground'
    });

    // Cobblestone grid + central community-service courtyard circle
    const yardGrid = new THREE.GridHelper(28, 14, '#475569', '#1e293b');
    yardGrid.position.set(0, 0.03, 0);
    this.prisonGroup.add(yardGrid);

    const spawnRing = new THREE.Mesh(
      new THREE.CylinderGeometry(2.6, 2.8, 0.1, 28),
      new THREE.MeshStandardMaterial({ color: '#475569', emissive: '#f59e0b', emissiveIntensity: 0.18 })
    );
    spawnRing.position.set(0, 0.05, 3.5);
    this.prisonGroup.add(spawnRing);

    // Warm courtyard lantern lights
    const northLight = new THREE.PointLight('#fbbf24', 2.2, 32);
    northLight.position.set(0, 7.5, -8);
    const southLight = new THREE.PointLight('#38bdf8', 1.8, 28);
    southLight.position.set(0, 7.0, 8);
    this.prisonGroup.add(northLight, southLight);

    // Perimeter stone curb & wrought-iron bar fence at ±13.8m
    const curbMat = new THREE.MeshStandardMaterial({ color: '#1e293b', roughness: 0.7, metalness: 0.25 });
    const barMat = new THREE.MeshStandardMaterial({ color: '#0f172a', roughness: 0.32, metalness: 0.82 });
    const goldMat = new THREE.MeshStandardMaterial({
      color: '#fbbf24',
      emissive: '#d97706',
      emissiveIntensity: 0.55,
      metalness: 0.65,
      roughness: 0.22
    });

    const addFenceSpan = (x1, z1, x2, z2) => {
      const len = Math.hypot(x2 - x1, z2 - z1);
      const cx = (x1 + x2) / 2;
      const cz = (z1 + z2) / 2;
      const rotY = Math.atan2(x2 - x1, z2 - z1);
      const span = new THREE.Group();
      span.position.set(cx, 0, cz);
      span.rotation.y = rotY;

      const curb = new THREE.Mesh(new THREE.BoxGeometry(0.65, 0.7, len), curbMat);
      curb.position.y = 0.35;
      const topRail = new THREE.Mesh(new THREE.BoxGeometry(0.28, 0.18, len), barMat);
      topRail.position.y = 3.85;
      span.add(curb, topRail);

      const barCount = Math.max(2, Math.floor(len / 0.85));
      for (let i = 0; i <= barCount; i++) {
        const bz = -len / 2 + (i / barCount) * len;
        const bar = new THREE.Mesh(new THREE.CylinderGeometry(0.065, 0.065, 3.3, 8), barMat);
        bar.position.set(0, 2.2, bz);
        span.add(bar);
      }
      this.prisonGroup.add(span);
    };

    // North, West, East full spans + South split around Release Gate
    addFenceSpan(-13.8, -13.8, 13.8, -13.8);
    addFenceSpan(-13.8, -13.8, -13.8, 13.8);
    addFenceSpan(13.8, -13.8, 13.8, 13.8);
    addFenceSpan(-13.8, 13.8, -2.8, 13.8);
    addFenceSpan(2.8, 13.8, 13.8, 13.8);

    // 4 Corner Watchtower Stone Pillars with Amber Lanterns
    for (const [px, pz] of [[-13.8, -13.8], [13.8, -13.8], [-13.8, 13.8], [13.8, 13.8]]) {
      const pillar = new THREE.Mesh(new THREE.BoxGeometry(1.4, 4.6, 1.4), curbMat);
      pillar.position.set(px, 2.3, pz);
      pillar.castShadow = true;
      const lantern = new THREE.Mesh(new THREE.OctahedronGeometry(0.42, 0), goldMat);
      lantern.position.set(px, 5.05, pz);
      this.prisonGroup.add(pillar, lantern);
    }

    // Judge Starlight Judicial Podium at North Courtyard (0, 0, -12.1)
    this._addBoxPlatform(
      { x: 0, y: 0.45, z: -12.1, w: 6.4, h: 0.9, d: 2.6, color: '#1e1b4b', emissive: '#4338ca', emissiveIntensity: 0.22 },
      this.prisonGroup,
      this.prisonColliders
    );
    const scalesEmblem = new THREE.Mesh(new THREE.OctahedronGeometry(0.62, 0), goldMat);
    scalesEmblem.position.set(0, 1.75, -12.1);
    const judgeBanner = createWorldBannerSprite(
      "⚖️ DEBTOR'S PRISON COURTYARD",
      'Press E on Glowing Chores (+5 🪙) to Clear Negative Debt!',
      '#fbbf24'
    );
    judgeBanner.position.set(0, 3.35, -12.1);
    this.prisonGroup.add(scalesEmblem, judgeBanner);

    // Dynamic Release Gate Arch at (0, 0, 11.5)
    const gate = new THREE.Group();
    gate.position.set(this.prisonReleasePortalPos.x, this.prisonReleasePortalPos.y, this.prisonReleasePortalPos.z);

    const gateRuneMat = new THREE.MeshStandardMaterial({
      color: '#ef4444',
      emissive: '#dc2626',
      emissiveIntensity: 0.72,
      roughness: 0.2,
      metalness: 0.35
    });
    const gatePad = new THREE.Mesh(new THREE.CylinderGeometry(2.6, 2.9, 0.2, 28), curbMat);
    gatePad.position.y = 0.1;
    const gateRing = new THREE.Mesh(new THREE.TorusGeometry(2.3, 0.09, 10, 32), gateRuneMat);
    gateRing.rotation.x = Math.PI / 2;
    gateRing.position.y = 0.22;
    gate.add(gatePad, gateRing);

    for (const side of [-1, 1]) {
      const pillar = new THREE.Mesh(new THREE.BoxGeometry(0.68, 4.1, 0.68), curbMat);
      pillar.position.set(side * 1.85, 2.15, 0);
      const strip = new THREE.Mesh(new THREE.BoxGeometry(0.2, 3.5, 0.74), gateRuneMat);
      strip.position.set(side * 1.85, 2.15, 0);
      gate.add(pillar, strip);
    }
    const lintel = new THREE.Mesh(new THREE.BoxGeometry(4.65, 0.72, 0.88), curbMat);
    lintel.position.set(0, 4.35, 0);
    const keystone = new THREE.Mesh(new THREE.OctahedronGeometry(0.46, 0), gateRuneMat);
    keystone.position.set(0, 4.95, 0);
    gate.add(lintel, keystone);

    const gateVortexMat = new THREE.MeshBasicMaterial({
      color: '#f59e0b',
      transparent: true,
      opacity: 0.65,
      side: THREE.DoubleSide
    });
    const vortexDisc = new THREE.Mesh(new THREE.CircleGeometry(1.75, 28), gateVortexMat);
    vortexDisc.position.set(0, 2.28, 0);
    const swirlRing = new THREE.Mesh(
      new THREE.TorusGeometry(1.25, 0.08, 10, 28),
      new THREE.MeshBasicMaterial({ color: '#ffffff', transparent: true, opacity: 0.72 })
    );
    swirlRing.position.set(0, 2.28, 0.04);
    gate.add(vortexDisc, swirlRing);
    this.portalVortices.push({ disc: vortexDisc, ring: swirlRing, keystone });

    const gateBanner = createWorldBannerSprite(
      '🔒 RELEASE GATE (LOCKED)',
      'Clear Negative Debt to 0 🪙 & Finish Chores — or Parent Pardon',
      '#f87171'
    );
    gateBanner.position.set(0, 6.05, 0);
    gate.add(gateBanner);
    this.prisonGroup.add(gate);

    this.prisonGateRefs = {
      group: gate,
      runeMat: gateRuneMat,
      vortexMat: gateVortexMat,
      banner: gateBanner,
      unlocked: false
    };
  }

  _buildPrisonNodeEntry(node) {
    const group = new THREE.Group();
    group.position.set(node.x, node.y, node.z);
    const bodyGroup = new THREE.Group();

    const ringMat = new THREE.MeshBasicMaterial({
      color: '#fbbf24',
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.65
    });
    const ring = new THREE.Mesh(new THREE.RingGeometry(0.85, 1.08, 24), ringMat);
    ring.rotation.x = -Math.PI / 2;
    ring.position.y = 0.05;
    group.add(ring);

    if (node.kind === 'boulder') {
      const rockMat = new THREE.MeshStandardMaterial({ color: '#64748b', roughness: 0.75, metalness: 0.15 });
      const oreMat = new THREE.MeshStandardMaterial({
        color: '#fbbf24',
        emissive: '#d97706',
        emissiveIntensity: 0.55,
        roughness: 0.25
      });
      const r1 = new THREE.Mesh(new THREE.DodecahedronGeometry(0.72, 0), rockMat);
      r1.position.set(0, 0.55, 0);
      const r2 = new THREE.Mesh(new THREE.DodecahedronGeometry(0.45, 0), rockMat);
      r2.position.set(0.52, 0.35, 0.22);
      const vein = new THREE.Mesh(new THREE.OctahedronGeometry(0.26, 0), oreMat);
      vein.position.set(-0.15, 0.92, 0.25);
      bodyGroup.add(r1, r2, vein);
    } else if (node.kind === 'sweep') {
      const dustMat = new THREE.MeshStandardMaterial({ color: '#a8a29e', roughness: 0.9 });
      const woodMat = new THREE.MeshStandardMaterial({ color: '#92400e', roughness: 0.7 });
      const strawMat = new THREE.MeshStandardMaterial({ color: '#fde047', roughness: 0.65 });
      const mound1 = new THREE.Mesh(new THREE.ConeGeometry(0.68, 0.42, 10), dustMat);
      mound1.position.set(0, 0.21, 0);
      const handle = new THREE.Mesh(new THREE.CylinderGeometry(0.05, 0.05, 1.45, 8), woodMat);
      handle.position.set(0.28, 0.75, 0);
      handle.rotation.z = -0.24;
      const bristles = new THREE.Mesh(new THREE.BoxGeometry(0.42, 0.28, 0.16), strawMat);
      bristles.position.set(0.44, 0.18, 0);
      bodyGroup.add(mound1, handle, bristles);
    } else {
      const ironMat = new THREE.MeshStandardMaterial({ color: '#1e293b', metalness: 0.75, roughness: 0.32 });
      const brassMat = new THREE.MeshStandardMaterial({
        color: '#fbbf24',
        emissive: '#f59e0b',
        emissiveIntensity: 0.62,
        metalness: 0.55,
        roughness: 0.22
      });
      const post = new THREE.Mesh(new THREE.CylinderGeometry(0.1, 0.14, 1.45, 10), ironMat);
      post.position.y = 0.72;
      const globe = new THREE.Mesh(new THREE.OctahedronGeometry(0.36, 0), brassMat);
      globe.position.y = 1.55;
      bodyGroup.add(post, globe);
    }

    const label = createWorldBannerSprite(
      `${node.icon || '🪨'} ${node.name}`,
      'Press E • +5 🪙 Toward Clearing Debt',
      '#fbbf24'
    );
    label.position.set(0, 2.25, 0);
    group.add(bodyGroup, label);
    this.prisonEntitiesGroup.add(group);
    return { id: node.id, data: node, group, bodyGroup, ring, label };
  }

  setPrisonReleaseEligible(eligible) {
    const next = Boolean(eligible);
    this.prisonReleaseEligible = next;
    if (!this.prisonGateRefs || this.prisonGateRefs.unlocked === next) return;
    this.prisonGateRefs.unlocked = next;
    const { group, runeMat, vortexMat } = this.prisonGateRefs;
    if (next) {
      runeMat.color.set('#10b981');
      runeMat.emissive.set('#059669');
      vortexMat.color.set('#34d399');
    } else {
      runeMat.color.set('#ef4444');
      runeMat.emissive.set('#dc2626');
      vortexMat.color.set('#f59e0b');
    }
    group.remove(this.prisonGateRefs.banner);
    const nextBanner = next
      ? createWorldBannerSprite(
          '🔓 SENTENCE SERVED!',
          'Step In or Press E • Return to Starlight Plaza!',
          '#34d399'
        )
      : createWorldBannerSprite(
          '🔒 RELEASE GATE (LOCKED)',
          'Clear Negative Debt to 0 🪙 & Finish Chores — or Parent Pardon',
          '#f87171'
        );
    nextBanner.position.set(0, 6.05, 0);
    group.add(nextBanner);
    this.prisonGateRefs.banner = nextBanner;
  }

  syncPrisonSnapshot(prison, isEligibleForRelease = null) {
    if (isEligibleForRelease !== null && isEligibleForRelease !== undefined) {
      this.setPrisonReleaseEligible(isEligibleForRelease);
    }
    if (!prison) return;
    if (Array.isArray(prison.releasePortalPos) && prison.releasePortalPos.length === 3) {
      this.prisonReleasePortalPos = {
        x: prison.releasePortalPos[0],
        y: prison.releasePortalPos[1],
        z: prison.releasePortalPos[2]
      };
      if (this.prisonGateRefs) {
        this.prisonGateRefs.group.position.set(
          this.prisonReleasePortalPos.x,
          this.prisonReleasePortalPos.y,
          this.prisonReleasePortalPos.z
        );
      }
    }
    for (const node of prison.nodes || []) {
      let entry = this.prisonNodesMap.get(node.id);
      if (!entry) {
        entry = this._buildPrisonNodeEntry(node);
        this.prisonNodesMap.set(node.id, entry);
      }
      entry.data = node;
      entry.group.position.set(node.x, node.y, node.z);
      entry.ring.visible = Boolean(node.available);
      entry.bodyGroup.scale.setScalar(node.available ? 1.0 : 0.45);
      entry.label.visible = Boolean(node.available);
    }
  }

  loadBlocks(blocksArray) {
    // Preserve current visual hinge angles across state updates so doors/windows swing smoothly!
    const prevAngles = new Map();
    for (const [k, hObj] of this.animatedHinges.entries()) {
      prevAngles.set(k, hObj.currentAngle);
    }

    for (const mesh of this.voxelMap.values()) {
      this.voxelGroup.remove(mesh);
    }
    this.voxelMap.clear();
    this.savedBlocksByKey.clear();
    this.animatedHinges.clear();
    this.voxelColliders = [];

    for (const b of blocksArray || []) {
      this.addBlockMesh(b, false, prevAngles.get(`${b.x},${b.y},${b.z}`));
    }
    this._rebuildActiveColliders();
  }

  addBlockMesh(block, rebuildImmediately = true, preservedHingeAngle = undefined) {
    const x = Number(block.x) || 0;
    const y = Number(block.y) || 1;
    const z = Number(block.z) || 0;
    const color = block.color || '#38bdf8';
    const kind = block.kind || 'block';
    const rot = ((Number(block.rot) || 0) % 4 + 4) % 4;
    const isOpen = Boolean(block.open);
    const lootItemId = block.lootItemId || null;

    const key = `${x},${y},${z}`;
    if (this.voxelMap.has(key)) {
      this.removeBlockMesh({ x, y, z }, false);
    }
    this.savedBlocksByKey.set(key, {
      x,
      y,
      z,
      color,
      kind,
      rot,
      open: isOpen,
      lootItemId,
      author: block.author || ''
    });

    const baseBottomY = y - 1;
    const rotRad = rot * (Math.PI / 2);
    const meta = {
      voxelCoords: { x, y, z },
      voxelKind: kind,
      voxelAuthor: block.author || '',
      lootItemId
    };

    // Classic 1x1x1 wall / floor block
    if (kind === 'block') {
      const mat = new THREE.MeshStandardMaterial({ color, roughness: 0.45 });
      const mesh = new THREE.Mesh(new THREE.BoxGeometry(1, 1, 1), mat);
      mesh.position.set(x, y - 0.5, z);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      Object.assign(mesh.userData, meta);
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
        bottomY: baseBottomY,
        type: 'voxel',
        cameraOccluder: true
      });
      if (rebuildImmediately) this._rebuildActiveColliders();
      return;
    }

    const root = new THREE.Group();
    root.position.set(x, baseBottomY, z);
    root.rotation.y = rotRad;
    Object.assign(root.userData, meta);

    if (kind === 'door') {
      const frame = clonePropTemplate('Prop_Door_Frame', color, meta);
      const leaf = clonePropTemplate('Prop_Door_Leaf', color, meta);
      const leafPivot = new THREE.Group();
      leafPivot.position.set(-0.41, 0, 0);
      Object.assign(leafPivot.userData, meta);

      if (frame && leaf) {
        root.add(frame);
        leafPivot.add(leaf);
        root.add(leafPivot);
      } else {
        const jambMat = new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.4 });
        const jL = new THREE.Mesh(new THREE.BoxGeometry(0.1, 2.0, 0.18), jambMat);
        jL.position.set(-0.45, 1.0, 0);
        const jR = new THREE.Mesh(new THREE.BoxGeometry(0.1, 2.0, 0.18), jambMat);
        jR.position.set(0.45, 1.0, 0);
        const hdr = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.12, 0.2), jambMat);
        hdr.position.set(0, 1.94, 0);
        const slab = new THREE.Mesh(
          new THREE.BoxGeometry(0.82, 1.88, 0.08),
          new THREE.MeshStandardMaterial({ color, roughness: 0.38 })
        );
        slab.position.set(0.41, 0.96, 0);
        for (const p of [jL, jR, hdr, slab]) {
          p.castShadow = true;
          Object.assign(p.userData, meta);
        }
        leafPivot.add(slab);
        root.add(jL, jR, hdr, leafPivot);
      }

      const targetAngle = isOpen ? Math.PI * 0.48 : 0;
      const startAngle = preservedHingeAngle !== undefined ? preservedHingeAngle : targetAngle;
      leafPivot.rotation.y = startAngle;
      this.animatedHinges.set(key, { pivot: leafPivot, currentAngle: startAngle, targetAngle });

      // Thin oriented door threshold/frame collider (only blocks when closed!)
      const isEW = rot % 2 === 1;
      const halfW = isEW ? 0.18 : 0.48;
      const halfD = isEW ? 0.48 : 0.18;
      this.voxelColliders.push({
        mesh: root,
        voxelKey: key,
        w: halfW * 2,
        h: 2,
        d: halfD * 2,
        minX: x - halfW,
        maxX: x + halfW,
        minZ: z - halfD,
        maxZ: z + halfD,
        topY: baseBottomY + 2.0,
        bottomY: baseBottomY,
        type: 'voxel',
        passable: isOpen,
        noStand: isOpen,
        cameraOccluder: !isOpen
      });
    } else if (kind === 'double_door') {
      // 2-block-wide hinged French Double Door: spans local X in [-0.5, +1.5], Y in [0, 2.0]
      // Note: root is placed at anchor (x, baseBottomY, z) and rotated by rot * PI/2.
      // At rot=0 (+Z), local +X maps to world (+1, 0); rot=1 (+X), local +X maps to world (0, -1);
      // rot=2 (-Z), local +X maps to world (-1, 0); rot=3 (-X), local +X maps to world (0, +1).
      // That matches doubleDoorPartnerOffset(rot) identically!
      const jambMat = new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.36 });
      const goldMat = new THREE.MeshStandardMaterial({ color: '#fbbf24', metalness: 0.65, roughness: 0.22 });
      const doorMat = new THREE.MeshStandardMaterial({ color, roughness: 0.38 });
      const glassMat = new THREE.MeshStandardMaterial({
        color: '#bae6fd',
        transparent: true,
        opacity: 0.38,
        roughness: 0.12,
        depthWrite: false
      });

      const jL = new THREE.Mesh(new THREE.BoxGeometry(0.1, 2.0, 0.2), jambMat);
      jL.position.set(-0.45, 1.0, 0);
      const jR = new THREE.Mesh(new THREE.BoxGeometry(0.1, 2.0, 0.2), jambMat);
      jR.position.set(1.45, 1.0, 0);
      const hdr = new THREE.Mesh(new THREE.BoxGeometry(2.0, 0.12, 0.22), jambMat);
      hdr.position.set(0.5, 1.94, 0);
      const thresh = new THREE.Mesh(new THREE.BoxGeometry(1.98, 0.03, 0.22), goldMat);
      thresh.position.set(0.5, 0.015, 0);
      for (const p of [jL, jR, hdr, thresh]) {
        p.castShadow = true;
        p.receiveShadow = true;
        Object.assign(p.userData, meta);
        root.add(p);
      }

      const buildLeaf = (dirSign) => {
        const g = new THREE.Group();
        const cx = dirSign * 0.45;
        const lower = new THREE.Mesh(new THREE.BoxGeometry(0.9, 1.06, 0.08), doorMat);
        lower.position.set(cx, 0.56, 0);
        const stileOuter = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.78, 0.08), doorMat);
        stileOuter.position.set(dirSign * 0.07, 1.48, 0);
        const stileInner = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.78, 0.08), doorMat);
        stileInner.position.set(dirSign * 0.83, 1.48, 0);
        const topRail = new THREE.Mesh(new THREE.BoxGeometry(0.9, 0.14, 0.08), doorMat);
        topRail.position.set(cx, 1.81, 0);
        const trimPanel = new THREE.Mesh(new THREE.BoxGeometry(0.62, 0.64, 0.105), jambMat);
        trimPanel.position.set(cx, 0.56, 0);
        const glass = new THREE.Mesh(new THREE.BoxGeometry(0.62, 0.64, 0.03), glassMat);
        glass.position.set(cx, 1.45, 0);
        const knobF = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), goldMat);
        knobF.position.set(dirSign * 0.78, 0.96, 0.08);
        const knobB = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), goldMat);
        knobB.position.set(dirSign * 0.78, 0.96, -0.08);
        for (const p of [lower, stileOuter, stileInner, topRail, trimPanel, glass, knobF, knobB]) {
          p.castShadow = true;
          Object.assign(p.userData, meta);
          g.add(p);
        }
        return g;
      };

      const leftPivot = new THREE.Group();
      leftPivot.position.set(-0.4, 0, 0);
      Object.assign(leftPivot.userData, meta);
      leftPivot.add(buildLeaf(1));

      const rightPivot = new THREE.Group();
      rightPivot.position.set(1.4, 0, 0);
      Object.assign(rightPivot.userData, meta);
      rightPivot.add(buildLeaf(-1));

      root.add(leftPivot, rightPivot);

      const targetAngle = isOpen ? Math.PI * 0.48 : 0;
      const startAngle = preservedHingeAngle !== undefined ? preservedHingeAngle : targetAngle;
      leftPivot.rotation.y = startAngle;
      rightPivot.rotation.y = -startAngle;
      this.animatedHinges.set(key, {
        pivot: leftPivot,
        secondaryPivot: rightPivot,
        currentAngle: startAngle,
        targetAngle
      });

      const { dx, dz } = doubleDoorPartnerOffset(rot);
      const isEW = rot % 2 === 1;
      const centerX = x + dx * 0.5;
      const centerZ = z + dz * 0.5;
      const halfW = isEW ? 0.18 : 0.98;
      const halfD = isEW ? 0.98 : 0.18;
      this.voxelColliders.push({
        mesh: root,
        voxelKey: key,
        w: halfW * 2,
        h: 2,
        d: halfD * 2,
        minX: centerX - halfW,
        maxX: centerX + halfW,
        minZ: centerZ - halfD,
        maxZ: centerZ + halfD,
        topY: baseBottomY + 2.0,
        bottomY: baseBottomY,
        type: 'voxel',
        passable: isOpen,
        noStand: isOpen,
        cameraOccluder: !isOpen
      });
    } else if (kind === 'window') {
      const frame = clonePropTemplate('Prop_Window_Frame', color, meta);
      const sash = clonePropTemplate('Prop_Window_Sash', color, meta);
      const sashPivot = new THREE.Group();
      sashPivot.position.set(-0.42, 0, 0);
      Object.assign(sashPivot.userData, meta);

      if (frame && sash) {
        root.add(frame);
        sashPivot.add(sash);
        root.add(sashPivot);
      } else {
        const sill = new THREE.Mesh(
          new THREE.BoxGeometry(1.0, 0.1, 0.2),
          new THREE.MeshStandardMaterial({ color: '#f8fafc', roughness: 0.4 })
        );
        sill.position.set(0, 0.05, 0);
        const pane = new THREE.Mesh(
          new THREE.BoxGeometry(0.84, 0.82, 0.05),
          new THREE.MeshStandardMaterial({ color: '#7dd3fc', transparent: true, opacity: 0.42 })
        );
        pane.position.set(0.42, 0.5, 0);
        Object.assign(sill.userData, meta);
        Object.assign(pane.userData, meta);
        sashPivot.add(pane);
        root.add(sill, sashPivot);
      }

      const targetAngle = isOpen ? 1.12 : 0;
      const startAngle = preservedHingeAngle !== undefined ? preservedHingeAngle : targetAngle;
      sashPivot.rotation.y = startAngle;
      this.animatedHinges.set(key, { pivot: sashPivot, currentAngle: startAngle, targetAngle });

      const isEW = rot % 2 === 1;
      const halfW = isEW ? 0.18 : 0.48;
      const halfD = isEW ? 0.48 : 0.18;
      this.voxelColliders.push({
        mesh: root,
        voxelKey: key,
        w: halfW * 2,
        h: 1,
        d: halfD * 2,
        minX: x - halfW,
        maxX: x + halfW,
        minZ: z - halfD,
        maxZ: z + halfD,
        topY: y,
        bottomY: baseBottomY,
        type: 'voxel',
        passable: isOpen,
        noStand: isOpen,
        cameraOccluder: false
      });
    } else if (kind === 'stairs') {
      const stairsObj = clonePropTemplate('Prop_Stairs', color, meta);
      if (stairsObj) {
        root.add(stairsObj);
      } else {
        const sMat = new THREE.MeshStandardMaterial({ color, roughness: 0.45 });
        const s1 = new THREE.Mesh(new THREE.BoxGeometry(0.98, 0.5, 0.5), sMat);
        s1.position.set(0, 0.25, 0.25);
        const s2 = new THREE.Mesh(new THREE.BoxGeometry(0.98, 1.0, 0.5), sMat);
        s2.position.set(0, 0.5, -0.25);
        Object.assign(s1.userData, meta);
        Object.assign(s2.userData, meta);
        root.add(s1, s2);
      }

      // Two stepped AABB colliders rotated cleanly by `rot` so players walk up 2-story stairs smoothly!
      let lowBounds;
      let highBounds;
      if (rot === 0) {
        lowBounds = { minX: x - 0.49, maxX: x + 0.49, minZ: z, maxZ: z + 0.5 };
        highBounds = { minX: x - 0.49, maxX: x + 0.49, minZ: z - 0.5, maxZ: z };
      } else if (rot === 1) {
        lowBounds = { minX: x, maxX: x + 0.5, minZ: z - 0.49, maxZ: z + 0.49 };
        highBounds = { minX: x - 0.5, maxX: x, minZ: z - 0.49, maxZ: z + 0.49 };
      } else if (rot === 2) {
        lowBounds = { minX: x - 0.49, maxX: x + 0.49, minZ: z - 0.5, maxZ: z };
        highBounds = { minX: x - 0.49, maxX: x + 0.49, minZ: z, maxZ: z + 0.5 };
      } else {
        lowBounds = { minX: x - 0.5, maxX: x, minZ: z - 0.49, maxZ: z + 0.49 };
        highBounds = { minX: x, maxX: x + 0.5, minZ: z - 0.49, maxZ: z + 0.49 };
      }
      this.voxelColliders.push(
        {
          mesh: root,
          voxelKey: key,
          w: lowBounds.maxX - lowBounds.minX,
          h: 0.5,
          d: lowBounds.maxZ - lowBounds.minZ,
          ...lowBounds,
          topY: baseBottomY + 0.5,
          bottomY: baseBottomY,
          type: 'voxel',
          cameraOccluder: false
        },
        {
          mesh: root,
          voxelKey: key,
          w: highBounds.maxX - highBounds.minX,
          h: 1.0,
          d: highBounds.maxZ - highBounds.minZ,
          ...highBounds,
          topY: y,
          bottomY: baseBottomY,
          type: 'voxel',
          cameraOccluder: false
        }
      );
    } else if (kind === 'loot_decor') {
      const decorVis = buildLootDecorVisual(lootItemId || 'star_shard', color, meta);
      root.add(decorVis);
      root.userData.trophyHolder = decorVis.userData.trophyHolder;

      this.voxelColliders.push({
        mesh: root,
        voxelKey: key,
        w: 0.72,
        h: 0.85,
        d: 0.72,
        minX: x - 0.36,
        maxX: x + 0.36,
        minZ: z - 0.36,
        maxZ: z + 0.36,
        topY: baseBottomY + 0.85,
        bottomY: baseBottomY,
        type: 'voxel',
        cameraOccluder: false
      });
    } else {
      // Kitchen, Bathroom, Living Room & Bedroom 3D Interior Props
      const tplName = PROP_TEMPLATE_BY_KIND[kind];
      const propObj = tplName ? clonePropTemplate(tplName, color, meta) : null;
      const isTwoTall = kind === 'kitchen_fridge' || kind === 'wardrobe';
      const isRug = kind === 'rug';
      const isLowSeat = kind === 'chair' || kind === 'sofa' || kind === 'bed';
      const isLamp = kind === 'nightstand_lamp' || kind === 'crystal_lamp';

      if (propObj) {
        root.add(propObj);
      } else {
        const fbW = kind === 'sofa' ? 1.62 : 0.92;
        const fbH = isRug ? 0.05 : isTwoTall ? 1.95 : isLowSeat ? 0.5 : 0.92;
        const fb = new THREE.Mesh(
          new THREE.BoxGeometry(fbW, fbH, 0.92),
          new THREE.MeshStandardMaterial({ color, roughness: 0.42 })
        );
        fb.position.y = fbH / 2;
        fb.castShadow = !isRug;
        fb.receiveShadow = true;
        Object.assign(fb.userData, meta);
        root.add(fb);
      }

      if (isLamp) {
        const isLit = !isOpen; // open === false means LIT by default; open === true means switched OFF
        const glowHex = kind === 'crystal_lamp' ? '#34d399' : '#fde047';
        root.traverse((n) => {
          if (n.isMesh && n.material) {
            const mats = Array.isArray(n.material) ? n.material : [n.material];
            for (const m of mats) {
              if (!m || !m.emissive) continue;
              const mName = m.name || '';
              if (
                mName.includes('Lamp') ||
                mName.includes('Crystal') ||
                mName.includes('Tint') ||
                m.emissiveIntensity > 0.05
              ) {
                m.emissive.set(isLit ? glowHex : '#0f172a');
                m.emissiveIntensity = isLit ? (kind === 'crystal_lamp' ? 1.15 : 0.88) : 0.02;
              }
            }
          }
        });
        if (isLit) {
          const halo = new THREE.Mesh(
            new THREE.SphereGeometry(kind === 'crystal_lamp' ? 0.28 : 0.24, 12, 10),
            new THREE.MeshBasicMaterial({
              color: glowHex,
              transparent: true,
              opacity: 0.26,
              depthWrite: false,
              blending: THREE.AdditiveBlending
            })
          );
          halo.position.y = kind === 'crystal_lamp' ? 0.62 : 0.76;
          Object.assign(halo.userData, meta);
          root.add(halo);
        }
      }

      const colH = isRug
        ? 0.04
        : isTwoTall
          ? 1.96
          : isLowSeat
            ? 0.5
            : kind === 'bath_tub'
              ? 0.62
              : kind === 'dining_table'
                ? 0.85
                : 0.94;
      const isEW = rot % 2 === 1;
      const padX = kind === 'chair' ? 0.32 : kind === 'sofa' ? (isEW ? 0.44 : 0.78) : 0.45;
      const padZ = kind === 'chair' ? 0.32 : kind === 'sofa' ? (isEW ? 0.78 : 0.44) : 0.45;
      this.voxelColliders.push({
        mesh: root,
        voxelKey: key,
        w: padX * 2,
        h: colH,
        d: padZ * 2,
        minX: x - padX,
        maxX: x + padX,
        minZ: z - padZ,
        maxZ: z + padZ,
        topY: baseBottomY + colH,
        bottomY: baseBottomY,
        type: 'voxel',
        passable: isRug,
        noStand: isRug,
        cameraOccluder: false
      });
    }

    this.voxelGroup.add(root);
    this.voxelMap.set(key, root);
    if (rebuildImmediately) this._rebuildActiveColliders();
  }

  removeBlockMesh({ x, y, z }, rebuildImmediately = true) {
    const key = `${x},${y},${z}`;
    const mesh = this.voxelMap.get(key);
    if (mesh) {
      this.voxelGroup.remove(mesh);
      this.voxelMap.delete(key);
    }
    this.savedBlocksByKey.delete(key);
    this.animatedHinges.delete(key);
    this.voxelColliders = this.voxelColliders.filter((c) => c.voxelKey !== key);
    if (rebuildImmediately) this._rebuildActiveColliders();
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
        av.syncCompanions(playerState.user);
      }
    }
  }

  updateUserCompanions(username, compPayload = {}) {
    if (!username) return;
    if (this.selfAvatar && this.selfAvatar.user?.username === username) {
      this.selfAvatar.syncCompanions(compPayload);
    }
    for (const av of this.remoteAvatars.values()) {
      if (av.user?.username === username) {
        av.syncCompanions(compPayload);
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

  setArenaDead(isDead) {
    this.arenaDead = Boolean(isDead);
    if (this.arenaDead && this.selfAvatar) {
      this.velocity.set(0, 0, 0);
      this.selfAvatar.anim = 'idle';
      this.selfAvatar.emote = 'sit';
    } else if (!this.arenaDead && this.selfAvatar && this.selfAvatar.emote === 'sit') {
      this.selfAvatar.emote = null;
    }
  }

  setZone(zone, spawnCoords = null) {
    const zNum = Number(zone);
    const nextZone = zNum === 1 ? 1 : zNum === 2 ? 2 : zNum === 3 ? 3 : 0;
    this.currentZone = nextZone;
    if (nextZone !== 3) {
      this.arenaDead = false;
    }
    this.lastPortalTriggerAt = performance.now();

    if (nextZone === 1) {
      this.plazaGroup.visible = false;
      this.voxelGroup.visible = false;
      this.dungeonGroup.visible = true;
      this.arenaGroup.visible = false;
      this.prisonGroup.visible = false;
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
    } else if (nextZone === 3) {
      this.plazaGroup.visible = false;
      this.voxelGroup.visible = false;
      this.dungeonGroup.visible = false;
      this.arenaGroup.visible = true;
      this.prisonGroup.visible = false;
      this.ghostMesh.visible = false;
      this.scene.background.set('#1a0812');
      this.scene.fog.color.set('#2b0c18');
      this.scene.fog.density = 0.0135;
      this.hemiLight.color.set('#fca5a5');
      this.hemiLight.groundColor.set('#450a0a');
      this.hemiLight.intensity = 0.8;
      this.sunLight.color.set('#fdba74');
      this.sunLight.intensity = 0.95;
      this.fillLight.color.set('#ef4444');
      this.fillLight.intensity = 0.62;
      if (this.nearRunway) {
        this.nearRunway = false;
        this.onRunwayProximity(false);
      }
    } else if (nextZone === 2) {
      this.plazaGroup.visible = false;
      this.voxelGroup.visible = false;
      this.dungeonGroup.visible = false;
      this.arenaGroup.visible = false;
      this.prisonGroup.visible = true;
      this.ghostMesh.visible = false;
      this.scene.background.set('#1e293b');
      this.scene.fog.color.set('#0f172a');
      this.scene.fog.density = 0.016;
      this.hemiLight.color.set('#cbd5e1');
      this.hemiLight.groundColor.set('#1e293b');
      this.hemiLight.intensity = 0.82;
      this.sunLight.color.set('#fde68a');
      this.sunLight.intensity = 0.88;
      this.fillLight.color.set('#94a3b8');
      this.fillLight.intensity = 0.48;
      if (this.nearRunway) {
        this.nearRunway = false;
        this.onRunwayProximity(false);
      }
    } else {
      this.plazaGroup.visible = true;
      this.voxelGroup.visible = true;
      this.dungeonGroup.visible = false;
      this.arenaGroup.visible = false;
      this.prisonGroup.visible = false;
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
        if (nextZone === 1 || nextZone === 2 || nextZone === 3) {
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

  _buildDungeonNodeEntry(node, targetEntitiesGroup = this.dungeonEntitiesGroup, isArena = false) {
    const group = new THREE.Group();
    group.position.set(node.x, node.y, node.z);
    let bodyGroup = new THREE.Group();
    let lidPivot = null;

    if (node.kind === 'wooden_chest' || node.kind === 'golden_chest') {
      const isGold = node.kind === 'golden_chest';
      const boxMat = new THREE.MeshStandardMaterial({
        color: isGold ? '#f59e0b' : isArena ? '#292524' : '#92400e',
        emissive: isGold ? '#b45309' : isArena ? '#7f1d1d' : '#000000',
        emissiveIntensity: isGold ? 0.32 : isArena ? 0.25 : 0,
        metalness: isGold ? 0.68 : 0.28,
        roughness: isGold ? 0.22 : 0.55
      });
      const trimMat = new THREE.MeshStandardMaterial({
        color: isGold ? '#c084fc' : isArena ? '#f87171' : '#fbbf24',
        emissive: isGold ? '#9333ea' : isArena ? '#dc2626' : '#d97706',
        emissiveIntensity: 0.48,
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
        isGold ? '🗝️ Golden Vault' : isArena ? '🔥 Obsidian Chest' : '📦 Supply Chest',
        'Press E to Open',
        isGold ? '#fbbf24' : isArena ? '#f87171' : '#38bdf8'
      );
      label.position.set(0, 1.95, 0);
      group.add(bodyGroup, label);
    } else {
      const crystalMat = new THREE.MeshStandardMaterial({
        color: isArena ? '#f87171' : '#e879f9',
        emissive: isArena ? '#dc2626' : '#a855f7',
        emissiveIntensity: 0.72,
        roughness: 0.16,
        metalness: 0.32
      });
      const subMat = new THREE.MeshStandardMaterial({
        color: isArena ? '#fb923c' : '#38bdf8',
        emissive: isArena ? '#ea580c' : '#0284c7',
        emissiveIntensity: 0.58,
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

      const label = createWorldBannerSprite(
        `✨ ${node.label}`,
        'Press E to Mine',
        isArena ? '#fca5a5' : '#c084fc'
      );
      label.position.set(0, 2.35, 0);
      group.add(bodyGroup, label);
    }

    targetEntitiesGroup.add(group);
    return { id: node.id, data: node, group, bodyGroup, lidPivot };
  }

  _mobBannerConfig(mob) {
    const hearts = '♥'.repeat(Math.max(1, mob.hp || 1));
    if (mob.kind === 'lava_golem') {
      return {
        title: `🔥 ${mob.name} ${hearts}`,
        sub: 'BOSS • Deals 32 DMG! Ray Gun or Stomp!',
        color: '#fb923c',
        y: 3.15
      };
    }
    if (mob.kind === 'void_wraith') {
      return {
        title: `👻 ${mob.name} ${hearts}`,
        sub: 'Chases Players (22 DMG) • Press E / Stomp!',
        color: '#38bdf8',
        y: 2.65
      };
    }
    if (mob.kind === 'rift_spider') {
      return {
        title: `🕷️ ${mob.name} ${hearts}`,
        sub: 'Chases Players (15 DMG) • Press E / Stomp!',
        color: '#f87171',
        y: 1.65
      };
    }
    return {
      title: `${mob.name} ${hearts}`,
      sub: 'Jump-Stomp or Press E!',
      color: '#f472b6',
      y: 1.92
    };
  }

  _populateMobVisual(entry) {
    if (!entry || !entry.visual) return;
    entry.visual.clear();
    entry.leftWing = null;
    entry.rightWing = null;
    entry.leftArm = null;
    entry.rightArm = null;
    const kind = entry.data.kind;

    if (kind === 'crystal_bat') {
      const glbBody = cloneEnvTemplate('Env_Mob_CrystalBat_Body');
      const glbWingL = cloneEnvTemplate('Env_Mob_CrystalBat_WingL');
      const glbWingR = cloneEnvTemplate('Env_Mob_CrystalBat_WingR');
      if (glbBody && glbWingL && glbWingR) {
        glbWingL.position.set(-0.34, 0.68, 0);
        glbWingR.position.set(0.34, 0.68, 0);
        entry.leftWing = glbWingL;
        entry.rightWing = glbWingR;
        entry.visual.add(glbBody, glbWingL, glbWingR);
        return;
      }
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
      const body = new THREE.Mesh(new THREE.OctahedronGeometry(0.46, 1), bodyMat);
      body.position.y = 0.55;
      entry.leftWing = new THREE.Mesh(new THREE.BoxGeometry(0.88, 0.42, 0.06), wingMat);
      entry.leftWing.position.set(-0.62, 0.6, 0);
      entry.rightWing = new THREE.Mesh(new THREE.BoxGeometry(0.88, 0.42, 0.06), wingMat);
      entry.rightWing.position.set(0.62, 0.6, 0);
      entry.visual.add(body, entry.leftWing, entry.rightWing);
      return;
    }

    if (kind === 'lava_golem') {
      const glbBody = cloneEnvTemplate('Env_Mob_LavaGolem_Body');
      const glbArmL = cloneEnvTemplate('Env_Mob_LavaGolem_ArmL');
      const glbArmR = cloneEnvTemplate('Env_Mob_LavaGolem_ArmR');
      if (glbBody && glbArmL && glbArmR) {
        glbArmL.position.set(-0.84, 1.72, 0);
        glbArmR.position.set(0.84, 1.72, 0);
        entry.leftArm = glbArmL;
        entry.rightArm = glbArmR;
        entry.visual.add(glbBody, glbArmL, glbArmR);
        return;
      }
      const rockMat = new THREE.MeshStandardMaterial({ color: '#1c1917', roughness: 0.6 });
      const magmaMat = new THREE.MeshStandardMaterial({
        color: '#fb923c',
        emissive: '#ea580c',
        emissiveIntensity: 0.9
      });
      const torso = new THREE.Mesh(new THREE.BoxGeometry(1.4, 1.8, 0.9), rockMat);
      torso.position.y = 1.2;
      const core = new THREE.Mesh(new THREE.OctahedronGeometry(0.42, 0), magmaMat);
      core.position.set(0, 1.35, 0.4);
      entry.leftArm = new THREE.Mesh(new THREE.BoxGeometry(0.42, 1.25, 0.42), rockMat);
      entry.leftArm.position.set(-0.95, 1.3, 0);
      entry.rightArm = new THREE.Mesh(new THREE.BoxGeometry(0.42, 1.25, 0.42), rockMat);
      entry.rightArm.position.set(0.95, 1.3, 0);
      entry.visual.add(torso, core, entry.leftArm, entry.rightArm);
      return;
    }

    if (kind === 'void_wraith') {
      const glbWraith = cloneEnvTemplate('Env_Mob_VoidWraith');
      if (glbWraith) {
        entry.visual.add(glbWraith);
        return;
      }
      const cloakMat = new THREE.MeshStandardMaterial({
        color: '#1e1b4b',
        emissive: '#38bdf8',
        emissiveIntensity: 0.35
      });
      const cone = new THREE.Mesh(new THREE.ConeGeometry(0.62, 1.9, 10), cloakMat);
      cone.position.y = 1.05;
      entry.visual.add(cone);
      return;
    }

    if (kind === 'rift_spider') {
      const glbSpider = cloneEnvTemplate('Env_Mob_RiftSpider');
      if (glbSpider) {
        entry.visual.add(glbSpider);
        return;
      }
      const shellMat = new THREE.MeshStandardMaterial({
        color: '#18181b',
        emissive: '#dc2626',
        emissiveIntensity: 0.45
      });
      const body = new THREE.Mesh(new THREE.SphereGeometry(0.48, 12, 10), shellMat);
      body.position.y = 0.48;
      entry.visual.add(body);
      return;
    }

    const glbSlime = cloneEnvTemplate('Env_Mob_ShadowSlime');
    if (glbSlime) {
      entry.visual.add(glbSlime);
      return;
    }
    const outerMat = new THREE.MeshStandardMaterial({
      color: '#7c3aed',
      emissive: '#6d28d9',
      emissiveIntensity: 0.38,
      transparent: true,
      opacity: 0.84,
      roughness: 0.2
    });
    const slimeCube = new THREE.Mesh(new THREE.BoxGeometry(1.12, 0.92, 1.12), outerMat);
    slimeCube.position.y = 0.46;
    entry.visual.add(slimeCube);
  }

  _buildDungeonMobEntry(mob, targetEntitiesGroup = this.dungeonEntitiesGroup) {
    const group = new THREE.Group();
    group.position.set(mob.x, mob.y, mob.z);
    const visual = new THREE.Group();

    const bannerCfg = this._mobBannerConfig(mob);
    const label = createWorldBannerSprite(bannerCfg.title, bannerCfg.sub, bannerCfg.color);
    label.position.set(0, bannerCfg.y, 0);
    group.add(visual, label);
    targetEntitiesGroup.add(group);

    const entry = {
      id: mob.id,
      data: mob,
      group,
      visual,
      leftWing: null,
      rightWing: null,
      leftArm: null,
      rightArm: null,
      label,
      lastHp: mob.hp,
      targetPos: new THREE.Vector3(mob.x, mob.y, mob.z),
      phaseOffset: Math.random() * 6.28
    };
    this._populateMobVisual(entry);
    return entry;
  }

  _buildDungeonDropEntry(drop, targetEntitiesGroup = this.dungeonEntitiesGroup) {
    const group = new THREE.Group();
    group.position.set(drop.x, drop.y, drop.z);
    const rarityPalette = {
      common: '#38bdf8',
      uncommon: '#34d399',
      rare: '#f472b6',
      epic: '#fbbf24',
      legendary: '#22d3ee'
    };
    const isLegendary = drop.rarity === 'legendary';
    const hex = drop.itemId ? rarityPalette[drop.rarity] || '#38bdf8' : '#fbbf24';
    const orbMat = new THREE.MeshStandardMaterial({
      color: hex,
      emissive: isLegendary ? '#f472b6' : hex,
      emissiveIntensity: isLegendary ? 1.15 : 0.78,
      metalness: 0.45,
      roughness: 0.15
    });

    const mesh = drop.itemId
      ? new THREE.Mesh(new THREE.OctahedronGeometry(isLegendary ? 0.46 : 0.36, isLegendary ? 1 : 0), orbMat)
      : new THREE.Mesh(new THREE.CylinderGeometry(0.34, 0.34, 0.12, 18), orbMat);
    if (!drop.itemId) {
      mesh.rotation.z = Math.PI / 2;
    }
    mesh.position.y = 0.55;

    const ring = new THREE.Mesh(
      new THREE.RingGeometry(0.38, 0.52, 20),
      new THREE.MeshBasicMaterial({ color: hex, side: THREE.DoubleSide, transparent: true, opacity: 0.68 })
    );
    ring.rotation.x = -Math.PI / 2;
    ring.position.y = 0.06;

    if (isLegendary) {
      const goldHalo = new THREE.Mesh(
        new THREE.RingGeometry(0.58, 0.74, 28),
        new THREE.MeshBasicMaterial({
          color: '#fbbf24',
          side: THREE.DoubleSide,
          transparent: true,
          opacity: 0.85
        })
      );
      goldHalo.rotation.x = -Math.PI / 2;
      goldHalo.position.y = 0.07;
      const beacon = new THREE.Mesh(
        new THREE.CylinderGeometry(0.12, 0.24, 2.35, 14, 1, true),
        new THREE.MeshBasicMaterial({
          color: '#22d3ee',
          transparent: true,
          opacity: 0.24,
          side: THREE.DoubleSide,
          depthWrite: false
        })
      );
      beacon.position.y = 1.18;
      group.add(goldHalo, beacon);
    }

    const label = createWorldBannerSprite(
      drop.label,
      isLegendary ? '✦ LEGENDARY DROP!' : '',
      isLegendary ? '#fbbf24' : hex
    );
    label.position.set(0, isLegendary ? 1.62 : 1.38, 0);
    group.add(mesh, ring, label);
    targetEntitiesGroup.add(group);

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
        entry = this._buildDungeonNodeEntry(node, this.dungeonEntitiesGroup, false);
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
        this.dungeonDropsMap.set(drop.id, this._buildDungeonDropEntry(drop, this.dungeonEntitiesGroup));
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
        entry = this._buildDungeonMobEntry(mob, this.dungeonEntitiesGroup);
        this.dungeonMobsMap.set(mob.id, entry);
      }
      entry.data = mob;
      entry.targetPos.set(mob.x, mob.y, mob.z);
      entry.group.visible = Boolean(mob.alive);
      if (entry.lastHp !== mob.hp && mob.alive) {
        entry.lastHp = mob.hp;
        const bannerCfg = this._mobBannerConfig(mob);
        entry.group.remove(entry.label);
        entry.label = createWorldBannerSprite(bannerCfg.title, bannerCfg.sub, bannerCfg.color);
        entry.label.position.set(0, bannerCfg.y, 0);
        entry.group.add(entry.label);
      }
    }
  }

  syncArenaSnapshot(arena) {
    if (!arena) return;
    if (Array.isArray(arena.portalPos) && arena.portalPos.length === 3) {
      this.plazaArenaPortalPos = { x: arena.portalPos[0], y: arena.portalPos[1], z: arena.portalPos[2] };
    }
    if (Array.isArray(arena.returnPortalPos) && arena.returnPortalPos.length === 3) {
      this.arenaReturnPortalPos = {
        x: arena.returnPortalPos[0],
        y: arena.returnPortalPos[1],
        z: arena.returnPortalPos[2]
      };
    }

    for (const node of arena.nodes || []) {
      let entry = this.arenaNodesMap.get(node.id);
      if (!entry) {
        entry = this._buildDungeonNodeEntry(node, this.arenaEntitiesGroup, true);
        this.arenaNodesMap.set(node.id, entry);
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

    this.syncArenaMobs(arena.mobs || [], arena.boss || null);

    const nextDropIds = new Set();
    for (const drop of arena.drops || []) {
      nextDropIds.add(drop.id);
      if (!this.arenaDropsMap.has(drop.id)) {
        this.arenaDropsMap.set(drop.id, this._buildDungeonDropEntry(drop, this.arenaEntitiesGroup));
      } else {
        const existing = this.arenaDropsMap.get(drop.id);
        existing.data = drop;
      }
    }
    for (const [dropId, entry] of this.arenaDropsMap.entries()) {
      if (!nextDropIds.has(dropId)) {
        this.arenaEntitiesGroup.remove(entry.group);
        this.arenaDropsMap.delete(dropId);
      }
    }
  }

  syncArenaMobs(mobs, boss = undefined) {
    if (Array.isArray(mobs)) {
      for (const mob of mobs) {
        let entry = this.arenaMobsMap.get(mob.id);
        if (!entry) {
          entry = this._buildDungeonMobEntry(mob, this.arenaEntitiesGroup);
          this.arenaMobsMap.set(mob.id, entry);
        }
        entry.data = mob;
        entry.targetPos.set(mob.x, mob.y, mob.z);
        entry.group.visible = Boolean(mob.alive);
        if (entry.lastHp !== mob.hp && mob.alive) {
          entry.lastHp = mob.hp;
          const bannerCfg = this._mobBannerConfig(mob);
          entry.group.remove(entry.label);
          entry.label = createWorldBannerSprite(bannerCfg.title, bannerCfg.sub, bannerCfg.color);
          entry.label.position.set(0, bannerCfg.y, 0);
          entry.group.add(entry.label);
        }
      }
    }
    if (boss !== undefined) {
      this.syncArenaBoss(boss);
    }
  }

  syncArenaBoss(boss) {
    const entry = this._ensureBossDragonEntry();
    entry.data = boss || null;
    const isAlive = Boolean(boss && boss.active && boss.hp > 0);
    entry.group.visible = isAlive;

    if (isAlive) {
      if (!entry.wasAlive) {
        entry.group.position.set(boss.x || 0, boss.y || 0, boss.z || -22);
      }
      entry.targetPos.set(boss.x || 0, boss.y || 0, boss.z || -22);
      entry.group.rotation.y = boss.ry || 0;
      const hpSig = `${boss.hp}/${boss.maxHp}`;
      if (hpSig !== entry.lastHpSig) {
        entry.lastHpSig = hpSig;
        entry.group.remove(entry.label);
        entry.label = createWorldBannerSprite(
          `🐉 ${boss.name || 'Obsidian Rift Dragon'} (${hpSig} HP)`,
          'JUMP Over Purple Shockwaves! Blade / Wand / Ray Gun / Stomp!',
          '#c084fc'
        );
        entry.label.position.set(0, 4.95, 0);
        entry.group.add(entry.label);
      }
    }
    entry.wasAlive = isAlive;

    const nextSwIds = new Set();
    for (const sw of boss?.shockwaves || []) {
      nextSwIds.add(sw.id);
      let swMesh = this.bossShockwavesMap.get(sw.id);
      if (!swMesh) {
        const ringGeo = new THREE.RingGeometry(0.86, 1.0, 48);
        ringGeo.rotateX(-Math.PI / 2);
        const ringMat = new THREE.MeshBasicMaterial({
          color: '#e879f9',
          transparent: true,
          opacity: 0.85,
          side: THREE.DoubleSide,
          depthWrite: false
        });
        swMesh = new THREE.Mesh(ringGeo, ringMat);
        this.arenaEntitiesGroup.add(swMesh);
        this.bossShockwavesMap.set(sw.id, swMesh);
      }
      const r = Math.max(0.6, Number(sw.radius) || 1.0);
      const maxR = Math.max(2.0, Number(sw.maxRadius) || 18.0);
      swMesh.position.set(sw.x || 0, 0.16, sw.z || -22);
      swMesh.scale.set(r, 1, r);
      swMesh.material.opacity = Math.max(0.18, 0.92 * (1 - r / (maxR * 1.08)));
    }
    for (const [swId, swMesh] of this.bossShockwavesMap.entries()) {
      if (!nextSwIds.has(swId)) {
        this.arenaEntitiesGroup.remove(swMesh);
        this.bossShockwavesMap.delete(swId);
      }
    }
  }

  syncHomesSnapshot(homesPayload) {
    const plots = Array.isArray(homesPayload)
      ? homesPayload
      : Array.isArray(homesPayload?.plots)
        ? homesPayload.plots
        : null;
    if (!plots) return;
    for (const plot of plots) {
      this._upsertMailboxEntry(plot);
    }
  }

  triggerDoorbellFx(plotId) {
    const mb = this.mailboxesMap.get(plotId);
    if (!mb) return;
    this.spawnSparkleBurst(mb.group.position, '#fbbf24');
  }

  syncObbyLeaderboards(obbyPayload) {
    if (!obbyPayload) return;
    for (const courseKey of ['easy', 'hard']) {
      const bd = this.obbyBoardScreens[courseKey];
      const records = obbyPayload[courseKey];
      if (bd && Array.isArray(records)) {
        if (bd.screenMat.map) {
          bd.screenMat.map.dispose();
        }
        bd.screenMat.map = createObbyLeaderboardTexture(bd.heading, records, bd.accentHex);
        bd.screenMat.needsUpdate = true;
      }
    }
  }

  setActiveObbyRun(course) {
    this.activeObbyCourse = course || null;
  }

  syncRunwayShowState(showState) {
    this.runwayShowState = showState || null;
  }

  syncSkySnapshot(skyPayload) {
    if (!skyPayload) return;
    this.skyState = {
      ...this.skyState,
      ...skyPayload
    };

    const nextIds = new Set();
    for (const star of skyPayload.fallingStars || []) {
      nextIds.add(star.id);
      let entry = this.fallingStarsMap.get(star.id);
      if (!entry) {
        const grp = new THREE.Group();
        grp.position.set(star.x || 0, star.y || 0.6, star.z || 0);
        const core = new THREE.Mesh(
          new THREE.OctahedronGeometry(0.45, 0),
          new THREE.MeshStandardMaterial({
            color: '#fde047',
            emissive: '#f59e0b',
            emissiveIntensity: 1.25,
            roughness: 0.12,
            metalness: 0.4
          })
        );
        core.position.y = 0.65;
        const halo = new THREE.Mesh(
          new THREE.RingGeometry(0.55, 0.82, 24),
          new THREE.MeshBasicMaterial({
            color: '#38bdf8',
            transparent: true,
            opacity: 0.78,
            side: THREE.DoubleSide
          })
        );
        halo.rotation.x = -Math.PI / 2;
        halo.position.y = 0.05;
        const beam = new THREE.Mesh(
          new THREE.CylinderGeometry(0.12, 0.32, 4.5, 14, 1, true),
          new THREE.MeshBasicMaterial({
            color: '#fde047',
            transparent: true,
            opacity: 0.22,
            side: THREE.DoubleSide,
            depthWrite: false
          })
        );
        beam.position.y = 2.25;
        const label = createWorldBannerSprite(
          '🌠 FALLING STAR!',
          'Walk Over or Press E • Catch Coins + Star Shards!',
          '#fde047'
        );
        label.position.set(0, 1.85, 0);
        grp.add(core, halo, beam, label);
        grp.traverse((n) => {
          n.userData = { ...(n.userData || {}), landmarkKind: 'falling_star', starId: star.id };
        });
        this.fallingStarsGroup.add(grp);
        entry = { id: star.id, data: star, group: grp, core, lastCatchAttemptAt: 0 };
        this.fallingStarsMap.set(star.id, entry);
      } else {
        entry.data = star;
        entry.group.position.set(star.x || 0, star.y || 0.6, star.z || 0);
      }
    }
    for (const [starId, entry] of this.fallingStarsMap.entries()) {
      if (!nextIds.has(starId)) {
        this.fallingStarsGroup.remove(entry.group);
        this.fallingStarsMap.delete(starId);
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

  spawnRayGunBeam(fromArr, toArr, isDungeonHit = false, style = null) {
    if (!Array.isArray(fromArr) || !Array.isArray(toArr)) return;
    const fromVec = new THREE.Vector3(Number(fromArr[0]) || 0, Number(fromArr[1]) || 0, Number(fromArr[2]) || 0);
    const toVec = new THREE.Vector3(Number(toArr[0]) || 0, Number(toArr[1]) || 0, Number(toArr[2]) || 0);
    const delta = toVec.clone().sub(fromVec);
    const dist = delta.length();
    if (dist < 0.1) return;

    const dir = delta.clone().divideScalar(dist);
    const mid = fromVec.clone().addScaledVector(dir, dist * 0.5);
    const beamGroup = new THREE.Group();
    beamGroup.position.copy(mid);
    beamGroup.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), dir);

    const coreHex = style === 'star_bolt' ? '#fef08a' : '#ffffff';
    const outerColor =
      style === 'star_bolt'
        ? '#f472b6'
        : style === 'crystal_slash'
          ? '#38bdf8'
          : isDungeonHit
            ? '#22d3ee'
            : '#f472b6';
    const coreMat = new THREE.MeshBasicMaterial({
      color: coreHex,
      transparent: true,
      opacity: 0.96
    });
    const outerMat = new THREE.MeshBasicMaterial({
      color: outerColor,
      transparent: true,
      opacity: 0.68,
      depthWrite: false
    });

    const core = new THREE.Mesh(new THREE.CylinderGeometry(0.045, 0.045, dist, 8), coreMat);
    const sheath = new THREE.Mesh(new THREE.CylinderGeometry(0.125, 0.125, dist, 10), outerMat);
    const muzzleFlash = new THREE.Mesh(new THREE.SphereGeometry(0.16, 10, 10), outerMat);
    muzzleFlash.position.y = -dist * 0.5;
    const impactFlash = new THREE.Mesh(new THREE.OctahedronGeometry(0.26, 0), coreMat);
    impactFlash.position.y = dist * 0.5;

    beamGroup.add(core, sheath, muzzleFlash, impactFlash);
    this.scene.add(beamGroup);
    this.laserBeams.push({
      group: beamGroup,
      coreMat,
      outerMat,
      bornAt: performance.now()
    });
    this.spawnSparkleBurst(new THREE.Vector3(toVec.x, toVec.y - 0.35, toVec.z), outerColor);
  }

  _findNearestToggleableBlock(maxDist = 3.8) {
    if (!this.selfAvatar || this.currentZone !== 0) return null;
    const pos = this.selfAvatar.group.position;
    const eyeY = pos.y + 0.95;
    let best = null;
    let bestDist = maxDist;
    for (const b of this.savedBlocksByKey.values()) {
      if (
        b.kind !== 'door' &&
        b.kind !== 'double_door' &&
        b.kind !== 'window' &&
        b.kind !== 'nightstand_lamp' &&
        b.kind !== 'crystal_lamp'
      ) {
        continue;
      }
      let cx = b.x;
      let cz = b.z;
      if (b.kind === 'double_door') {
        const { dx, dz } = doubleDoorPartnerOffset(b.rot || 0);
        cx += dx * 0.5;
        cz += dz * 0.5;
      }
      const centerY = b.kind === 'door' || b.kind === 'double_door' ? b.y : b.y - 0.5;
      const d = Math.hypot(pos.x - cx, eyeY - centerY, pos.z - cz);
      if (d < bestDist) {
        bestDist = d;
        best = b;
      }
    }
    return best;
  }

  _findNearestInteractiveFurniture(maxDist = 2.65) {
    if (!this.selfAvatar || this.currentZone !== 0) return null;
    const pos = this.selfAvatar.group.position;
    let best = null;
    let bestDist = maxDist;
    for (const b of this.savedBlocksByKey.values()) {
      if (b.kind !== 'chair' && b.kind !== 'sofa' && b.kind !== 'bed') continue;
      const d = Math.hypot(pos.x - b.x, pos.y + 0.5 - (b.y - 0.5), pos.z - b.z);
      if (d < bestDist) {
        bestDist = d;
        best = b;
      }
    }
    return best;
  }

  _interactWithFurniture(b) {
    if (!this.selfAvatar || !b) return;
    const wantEmote = b.kind === 'bed' ? 'sleep' : 'sit';
    if (this.selfAvatar.emote === wantEmote) {
      this.selfAvatar.emote = null;
      this._emitMoveNow();
      return;
    }
    this.selfAvatar.group.position.set(b.x, b.y - 0.5, b.z);
    this.selfAvatar.group.rotation.y = ((b.rot || 0) % 4) * (Math.PI / 2);
    this.velocity.set(0, 0, 0);
    this.isGrounded = true;
    this.selfAvatar.anim = 'idle';
    this.selfAvatar.emote = wantEmote;
    this._emitMoveNow();
  }

  _findNearestMailbox(maxDist = 4.2) {
    if (!this.selfAvatar || this.currentZone !== 0) return null;
    const pos = this.selfAvatar.group.position;
    let best = null;
    let bestDist = maxDist;
    for (const mb of this.mailboxesMap.values()) {
      const d = Math.hypot(pos.x - mb.group.position.x, pos.z - mb.group.position.z);
      if (d < bestDist) {
        bestDist = d;
        best = mb;
      }
    }
    return best;
  }

  triggerAction() {
    if (!this.selfAvatar || this.salonMirrorMode) return;
    if (this.currentZone === 3 && this.arenaDead) return;
    const pos = this.selfAvatar.group.position;
    this.spawnSparkleBurst(
      pos,
      this.currentZone === 3
        ? '#ef4444'
        : this.currentZone === 2
          ? '#fbbf24'
          : this.currentZone === 1
            ? '#c084fc'
            : '#f472b6'
    );

    if (this.currentZone === 0) {
      // 1. Catch nearest Falling Star
      for (const star of this.fallingStarsMap.values()) {
        if (pos.distanceTo(star.group.position) < 4.2) {
          star.lastCatchAttemptAt = performance.now();
          this.spawnSparkleBurst(star.group.position, '#fde047');
          this.onCatchFallingStar(star.id);
          return;
        }
      }

      // 2. Interact with Neighborhood Home Mailbox
      const nearMb = this._findNearestMailbox(4.1);
      if (nearMb) {
        this.onManageHome(nearMb.data);
        return;
      }

      // 3. Interact with Starlight Crafting Bench
      const dCraft = Math.hypot(pos.x - this.craftingBenchPos.x, pos.z - this.craftingBenchPos.z);
      if (dCraft < 4.2) {
        this.onOpenCrafting();
        return;
      }

      // 4. Interact with Obby Speedrun Leaderboard Boards
      for (const [courseKey, bd] of Object.entries(this.obbyBoardScreens)) {
        if (Math.hypot(pos.x - bd.pos.x, pos.z - bd.pos.z) < 4.2) {
          this.onOpenObbyBoard(courseKey);
          return;
        }
      }

      // 5. Doors, Windows & Toggleable Lamps
      const toggleTarget = this._findNearestToggleableBlock(3.8);
      if (toggleTarget) {
        this.onToggleBlock({ x: toggleTarget.x, y: toggleTarget.y, z: toggleTarget.z });
        return;
      }

      // 6. Interactive Chairs, Sofas & Beds
      const furnTarget = this._findNearestInteractiveFurniture(2.65);
      if (furnTarget) {
        this._interactWithFurniture(furnTarget);
        return;
      }

      // 7. Zone Stargates
      const dPortal = Math.hypot(pos.x - this.plazaPortalPos.x, pos.z - this.plazaPortalPos.z);
      if (dPortal < 4.2) {
        this.lastPortalTriggerAt = performance.now();
        this.onRequestZonePortal(1);
        return;
      }
      const dArenaPortal = Math.hypot(
        pos.x - this.plazaArenaPortalPos.x,
        pos.z - this.plazaArenaPortalPos.z
      );
      if (dArenaPortal < 4.2) {
        this.lastPortalTriggerAt = performance.now();
        this.onRequestZonePortal(3);
        return;
      }

      // 8. Equipped Right-Hand Ray Gun or Starlight Wand Sparkle Beam in Plaza
      const acc = this.selfAvatar.user?.avatar?.accessory;
      if (
        !this.buildMode &&
        (acc === 'ray_gun' || acc === 'starlight_wand') &&
        performance.now() - this.lastLocalBeamAt > 210
      ) {
        const ry = this.selfAvatar.group.rotation.y;
        const fromArr = [pos.x + Math.sin(ry) * 0.55, pos.y + 1.05, pos.z + Math.cos(ry) * 0.55];
        const toArr = [pos.x + Math.sin(ry) * 11.0, pos.y + 1.12, pos.z + Math.cos(ry) * 11.0];
        this.lastLocalBeamAt = performance.now();
        const beamStyle = acc === 'starlight_wand' ? 'star_bolt' : null;
        this.spawnRayGunBeam(fromArr, toArr, false, beamStyle);
        this.onFireRayGun(toArr, beamStyle);
      }
      return;
    }

    if (this.currentZone === 2) {
      const dGate = Math.hypot(
        pos.x - this.prisonReleasePortalPos.x,
        pos.z - this.prisonReleasePortalPos.z
      );
      if (dGate < 4.0) {
        this.lastPortalTriggerAt = performance.now();
        this.onRequestZonePortal(0);
        return;
      }
      let nearestChore = null;
      let nearestDist = 5.8;
      for (const entry of this.prisonNodesMap.values()) {
        if (!entry.data.available) continue;
        const d = pos.distanceTo(entry.group.position);
        if (d < nearestDist) {
          nearestDist = d;
          nearestChore = entry.data;
        }
      }
      if (nearestChore) {
        this.onPrisonWork(nearestChore.id);
      }
      return;
    }

    const activeReturnPos =
      this.currentZone === 3 ? this.arenaReturnPortalPos : this.dungeonReturnPortalPos;
    const activeMobsMap = this.currentZone === 3 ? this.arenaMobsMap : this.dungeonMobsMap;
    const activeNodesMap = this.currentZone === 3 ? this.arenaNodesMap : this.dungeonNodesMap;

    const dReturn = Math.hypot(pos.x - activeReturnPos.x, pos.z - activeReturnPos.z);
    if (dReturn < 4.0) {
      this.lastPortalTriggerAt = performance.now();
      this.onRequestZonePortal(0);
      return;
    }

    const acc = this.selfAvatar.user?.avatar?.accessory;
    const hasRayGun = acc === 'ray_gun';
    const hasWand = acc === 'starlight_wand';
    const hasBlade = acc === 'crystal_blade';
    const maxWeaponRange = hasWand ? 16.2 : hasRayGun ? 14.2 : hasBlade ? 6.2 : 5.6;
    const beamStyle = hasWand ? 'star_bolt' : hasBlade ? 'crystal_slash' : null;

    // In Shadow Rift Arena, check live Co-Op Obsidian Rift Dragon Boss first!
    if (
      this.currentZone === 3 &&
      this.bossDragonEntry?.data?.active &&
      this.bossDragonEntry.data.hp > 0
    ) {
      const bPos = this.bossDragonEntry.group.position;
      const dBoss = pos.distanceTo(bPos);
      if (dBoss < maxWeaponRange + 1.8) {
        const ry = Math.atan2(bPos.x - pos.x, bPos.z - pos.z);
        this.selfAvatar.group.rotation.y = ry;
        const fromArr = [pos.x + Math.sin(ry) * 0.55, pos.y + 1.05, pos.z + Math.cos(ry) * 0.55];
        const toArr = [bPos.x, bPos.y + 1.8, bPos.z];
        this.lastLocalBeamAt = performance.now();
        this.spawnRayGunBeam(fromArr, toArr, true, beamStyle);
        this.onAttackMob('obsidian_dragon', hasRayGun ? 'raygun' : 'blast');
        return;
      }
    }

    let nearestMob = null;
    let nearestMobDist = maxWeaponRange;
    for (const entry of activeMobsMap.values()) {
      if (!entry.data.alive) continue;
      const d = pos.distanceTo(entry.group.position);
      if (d < nearestMobDist) {
        nearestMobDist = d;
        nearestMob = entry.data;
      }
    }
    if (nearestMob) {
      const ry = Math.atan2(nearestMob.x - pos.x, nearestMob.z - pos.z);
      this.selfAvatar.group.rotation.y = ry;
      if (hasRayGun || hasWand || hasBlade) {
        const fromArr = [pos.x + Math.sin(ry) * 0.55, pos.y + 1.05, pos.z + Math.cos(ry) * 0.55];
        const toArr = [nearestMob.x, nearestMob.y + 0.75, nearestMob.z];
        this.lastLocalBeamAt = performance.now();
        this.spawnRayGunBeam(fromArr, toArr, true, beamStyle);
      }
      this.onAttackMob(nearestMob.id, hasRayGun ? 'raygun' : 'blast');
      return;
    }

    // Summon Obsidian Dragon at North Island Altar if not active
    if (
      this.currentZone === 3 &&
      Math.hypot(pos.x - this.bossAltarPos.x, pos.z - this.bossAltarPos.z) < 5.4 &&
      !(this.bossDragonEntry?.data?.active && this.bossDragonEntry.data.hp > 0)
    ) {
      this.onSummonArenaBoss();
      return;
    }

    let nearestNode = null;
    let nearestNodeDist = 5.8;
    for (const entry of activeNodesMap.values()) {
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
      this.keys.clear();
      this.touchMove = { forward: false, backward: false, left: false, right: false };
      this.isDraggingCamera = false;
      if (this.ghostMesh) {
        this.ghostMesh.visible = false;
      }
      this.savedCameraBeforeSalon = {
        yaw: this.cameraYaw,
        pitch: this.cameraPitch,
        dist: this.cameraDist
      };
    } else if (!enabled && this.salonMirrorMode) {
      this.salonMirrorMode = false;
      this.keys.clear();
      this.touchMove = { forward: false, backward: false, left: false, right: false };
      this.isDraggingCamera = false;
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
    const cp = this.obbyCheckpoints.easy;
    this.lastActiveObby = 'easy';
    this.selfAvatar.group.position.set(cp.x, cp.y + 0.6, cp.z);
    this.velocity.set(0, 0, 0);
    this.selfAvatar.emote = null;
    this._emitMoveNow();
  }

  warpToHardObbyCheckpoint() {
    if (!this.selfAvatar) return;
    if (this.currentZone !== 0) {
      this.onRequestZonePortal(0);
      return;
    }
    const cp = this.obbyCheckpoints.hard;
    this.lastActiveObby = 'hard';
    this.selfAvatar.group.position.set(cp.x, cp.y + 0.6, cp.z);
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
        } else if (this.buildMode && code === 'KeyQ') {
          e.preventDefault();
          this.rotateBuildSelection(1);
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
        if (this.currentZone === 1 || this.currentZone === 2 || this.currentZone === 3) {
          this.triggerAction();
        } else if (this.buildMode) {
          if (e.button === 2 || this.buildTool === 'remove') {
            this._handleVoxelRemoveClick();
          } else {
            this._handleVoxelPlaceClick();
          }
        } else if (this.currentZone === 0 && this.selfAvatar) {
          this.raycaster.setFromCamera(this.pointerNdc, this.camera);
          const p = this.selfAvatar.group.position;

          // Check Plaza Expansion Landmarks (Falling Stars, Mailboxes, Crafting Bench, Obby Boards)
          const landmarkTargets = [
            ...this.fallingStarsGroup.children,
            ...this.mailboxesGroup.children,
            this.craftingBenchGroup,
            ...this.obbyGatesGroup.children
          ];
          const lmHits = this.raycaster.intersectObjects(landmarkTargets, true);
          if (lmHits.length > 0 && lmHits[0].distance <= 14.0) {
            const lmUd = lmHits[0].object.userData || {};
            if (lmUd.landmarkKind === 'falling_star' && lmUd.starId) {
              const star = this.fallingStarsMap.get(lmUd.starId);
              if (star && p.distanceTo(star.group.position) <= 6.2) {
                this.spawnSparkleBurst(star.group.position, '#fde047');
                this.onCatchFallingStar(star.id);
                return;
              }
            } else if (lmUd.landmarkKind === 'mailbox' && lmUd.plotId) {
              const mb = this.mailboxesMap.get(lmUd.plotId);
              if (mb && Math.hypot(p.x - mb.group.position.x, p.z - mb.group.position.z) <= 7.5) {
                this.onManageHome(mb.data);
                return;
              }
            } else if (lmUd.landmarkKind === 'crafting_bench') {
              if (Math.hypot(p.x - this.craftingBenchPos.x, p.z - this.craftingBenchPos.z) <= 7.5) {
                this.onOpenCrafting();
                return;
              }
            } else if (lmUd.landmarkKind === 'obby_board' && lmUd.course) {
              this.onOpenObbyBoard(lmUd.course);
              return;
            }
          }

          const voxelHits = this.raycaster.intersectObjects(this.voxelGroup.children, true);
          if (voxelHits.length > 0 && voxelHits[0].distance <= 9.5) {
            const ud = voxelHits[0].object.userData || {};
            if (
              (ud.voxelKind === 'door' ||
                ud.voxelKind === 'double_door' ||
                ud.voxelKind === 'window' ||
                ud.voxelKind === 'nightstand_lamp' ||
                ud.voxelKind === 'crystal_lamp') &&
              ud.voxelCoords
            ) {
              const d = Math.hypot(p.x - ud.voxelCoords.x, p.y + 0.9 - ud.voxelCoords.y, p.z - ud.voxelCoords.z);
              if (d <= 6.2) {
                this.onToggleBlock(ud.voxelCoords);
                return;
              }
            } else if (
              (ud.voxelKind === 'chair' || ud.voxelKind === 'sofa' || ud.voxelKind === 'bed') &&
              ud.voxelCoords
            ) {
              const savedB = this.savedBlocksByKey.get(
                `${ud.voxelCoords.x},${ud.voxelCoords.y},${ud.voxelCoords.z}`
              );
              const d = Math.hypot(p.x - ud.voxelCoords.x, p.y + 0.5 - (ud.voxelCoords.y - 0.5), p.z - ud.voxelCoords.z);
              if (savedB && d <= 4.2) {
                this._interactWithFurniture(savedB);
                return;
              }
            }
          }

          if (this.plazaPortalGroup) {
            const portalHits = this.raycaster.intersectObject(this.plazaPortalGroup, true);
            if (portalHits.length > 0 && portalHits[0].distance <= 14.0) {
              const dPortal = Math.hypot(p.x - this.plazaPortalPos.x, p.z - this.plazaPortalPos.z);
              if (dPortal <= 6.5 && performance.now() - this.lastPortalTriggerAt > 1200) {
                this.lastPortalTriggerAt = performance.now();
                this.onRequestZonePortal(1);
                return;
              }
            }
          }

          if (this.plazaArenaPortalGroup) {
            const arenaPortalHits = this.raycaster.intersectObject(this.plazaArenaPortalGroup, true);
            if (arenaPortalHits.length > 0 && arenaPortalHits[0].distance <= 14.0) {
              const dArena = Math.hypot(
                p.x - this.plazaArenaPortalPos.x,
                p.z - this.plazaArenaPortalPos.z
              );
              if (dArena <= 6.5 && performance.now() - this.lastPortalTriggerAt > 1200) {
                this.lastPortalTriggerAt = performance.now();
                this.onRequestZonePortal(3);
                return;
              }
            }
          }

          const acc = this.selfAvatar.user?.avatar?.accessory;
          if (
            e.button === 0 &&
            (acc === 'ray_gun' || acc === 'starlight_wand') &&
            performance.now() - this.lastLocalBeamAt > 210
          ) {
            const aimTargets = [...this.plazaGroup.children, ...this.voxelGroup.children];
            const hits = this.raycaster.intersectObjects(aimTargets, true);
            const pos = this.selfAvatar.group.position;
            let aimVec;
            if (hits.length > 0 && hits[0].distance <= 16.0) {
              aimVec = hits[0].point.clone();
            } else {
              aimVec = new THREE.Vector3();
              this.raycaster.ray.at(12.0, aimVec);
              aimVec.y = Math.max(0.25, aimVec.y);
            }
            const ry = Math.atan2(aimVec.x - pos.x, aimVec.z - pos.z);
            this.selfAvatar.group.rotation.y = ry;
            const fromArr = [pos.x + Math.sin(ry) * 0.55, pos.y + 1.05, pos.z + Math.cos(ry) * 0.55];
            const toArr = [aimVec.x, aimVec.y, aimVec.z];
            this.lastLocalBeamAt = performance.now();
            const beamStyle = acc === 'starlight_wand' ? 'star_bolt' : null;
            this.spawnRayGunBeam(fromArr, toArr, false, beamStyle);
            this.onFireRayGun(toArr, beamStyle);
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
    if (this.buildTool === 'place' && this.selectedBuildKind === 'loot_decor' && !this.selectedLootItemId) {
      this.ghostMesh.visible = false;
      return;
    }
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const staticSurfaces = this.colliders
      .filter((c) => c.mesh && c.type !== 'voxel' && c.type !== 'jumppad')
      .map((c) => c.mesh);
    const targets = [...staticSurfaces, ...this.voxelGroup.children];
    const hits = this.raycaster.intersectObjects(targets, true);
    if (hits.length === 0 || hits[0].distance > 36) {
      this.ghostMesh.visible = false;
      return;
    }
    const hit = hits[0];
    const wireBox = this.ghostMesh.userData.wireBox;
    const dirArrow = this.ghostMesh.userData.dirArrow;

    if (this.buildTool === 'remove') {
      if (!hit.object.userData?.voxelCoords) {
        this.ghostMesh.visible = false;
        return;
      }
      const { x, y, z } = hit.object.userData.voxelCoords;
      const hitKind = hit.object.userData.voxelKind || 'block';
      const hitTwoTall =
        hitKind === 'door' ||
        hitKind === 'double_door' ||
        hitKind === 'kitchen_fridge' ||
        hitKind === 'wardrobe';
      const hitDoubleWide = hitKind === 'double_door';
      const hitSaved = this.savedBlocksByKey.get(`${x},${y},${z}`);
      const hitRot = hitSaved ? hitSaved.rot || 0 : 0;
      if (wireBox) {
        wireBox.scale.set(hitDoubleWide ? 2 : 1, hitTwoTall ? 2 : 1, 1);
        wireBox.position.set(hitDoubleWide ? 0.5 : 0, hitTwoTall ? 1.0 : 0.5, 0);
        wireBox.material.color.set('#ef4444');
      }
      if (dirArrow) dirArrow.visible = false;
      this.ghostMesh.position.set(x, y - 1, z);
      this.ghostMesh.rotation.y = hitDoubleWide ? hitRot * (Math.PI / 2) : 0;
      this.ghostMesh.visible = true;
      return;
    }

    const worldNormal = hit.face
      ? hit.face.normal.clone().transformDirection(hit.object.matrixWorld)
      : new THREE.Vector3(0, 1, 0);
    const p = hit.point.clone().addScaledVector(worldNormal, 0.48);
    const gx = Math.round(p.x);
    const gy = Math.max(1, Math.round(p.y + 0.5));
    const gz = Math.round(p.z);
    const isTwoTall =
      this.selectedBuildKind === 'door' ||
      this.selectedBuildKind === 'double_door' ||
      this.selectedBuildKind === 'kitchen_fridge' ||
      this.selectedBuildKind === 'wardrobe';
    const isDoubleWide = this.selectedBuildKind === 'double_door';

    if (wireBox) {
      wireBox.scale.set(isDoubleWide ? 2 : 1, isTwoTall ? 2 : 1, 1);
      wireBox.position.set(isDoubleWide ? 0.5 : 0, isTwoTall ? 1.0 : 0.5, 0);
      wireBox.material.color.set(
        this.selectedBuildKind === 'loot_decor' ? '#22d3ee' : this.selectedBlockColor
      );
    }
    if (dirArrow) {
      dirArrow.visible = this.selectedBuildKind !== 'block';
    }
    this.ghostMesh.position.set(gx, gy - 1, gz);
    this.ghostMesh.rotation.y = this.selectedBuildRot * (Math.PI / 2);
    this.ghostMesh.visible = true;
  }

  _handleVoxelPlaceClick() {
    this._updateBuildGhost();
    if (!this.ghostMesh.visible) return;
    const x = Math.round(this.ghostMesh.position.x);
    const y = Math.round(this.ghostMesh.position.y + 1);
    const z = Math.round(this.ghostMesh.position.z);
    this.onPlaceBlock({
      x,
      y,
      z,
      color: this.selectedBlockColor,
      kind: this.selectedBuildKind,
      rot: this.selectedBuildRot,
      lootItemId: this.selectedBuildKind === 'loot_decor' ? this.selectedLootItemId : null
    });
  }

  _handleVoxelRemoveClick() {
    this.raycaster.setFromCamera(this.pointerNdc, this.camera);
    const hits = this.raycaster.intersectObjects(this.voxelGroup.children, true);
    if (hits.length > 0 && hits[0].object.userData?.voxelCoords) {
      this.onRemoveBlock(hits[0].object.userData.voxelCoords);
    }
  }

  _hasHeadroomAt(x, z, footY, playerRadius = 0.36, playerHeight = 1.75) {
    const rSq = (playerRadius * 0.92) * (playerRadius * 0.92);
    const minBodyY = footY + 0.08;
    const maxBodyY = footY + playerHeight;
    for (const col of this.colliders) {
      if (col.type === 'ground' || col.type === 'jumppad' || col.passable) continue;
      if (col.topY <= minBodyY || col.bottomY >= maxBodyY) continue;
      const cx = Math.max(col.minX, Math.min(col.maxX, x));
      const cz = Math.max(col.minZ, Math.min(col.maxZ, z));
      const dx = x - cx;
      const dz = z - cz;
      if (dx * dx + dz * dz < rSq) {
        return false;
      }
    }
    return true;
  }

  jump() {
    if (!this.selfAvatar || (this.currentZone === 3 && this.arenaDead)) return;
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

    const selfPos = this.selfAvatar ? this.selfAvatar.group.position : null;
    for (const pv of this.portalVortices) {
      if (pv.ring) pv.ring.rotation.z = tSec * 2.1;
      if (pv.motes) pv.motes.rotation.z = -tSec * 1.15;
      if (pv.innerGlyphRing) pv.innerGlyphRing.rotation.z = -tSec * 0.42;
      if (pv.horizonMat && pv.horizonMat.uniforms) {
        pv.horizonMat.uniforms.uTime.value = tSec;
        let targetSurge = 0.0;
        if (selfPos && pv.worldPos) {
          const dGate = Math.hypot(selfPos.x - pv.worldPos.x, selfPos.z - pv.worldPos.z);
          if (dGate < 7.0) {
            targetSurge = Math.max(0, 1.0 - dGate / 7.0);
          }
        }
        pv.surge = (pv.surge || 0) + (targetSurge - (pv.surge || 0)) * Math.min(1, dt * 4.5);
        pv.horizonMat.uniforms.uSurge.value = pv.surge;
      } else if (pv.disc?.material && typeof pv.disc.material.opacity === 'number') {
        pv.disc.material.opacity = 0.56 + Math.sin(tSec * 4.2) * 0.16;
      }
      if (pv.keystone) pv.keystone.rotation.y = tSec * 1.9;
    }

    // Sci-Fi Floating Obby Platform Anti-Grav Thrusters & Holographic Rings
    for (const tf of this.scifiThrusters) {
      tf.outerRing.rotation.z = tSec * 1.6 + tf.phase;
      tf.innerRing.rotation.z = -tSec * 2.8 + tf.phase;
      const pulse = 0.78 + Math.sin(tSec * 5.2 + tf.phase) * 0.22;
      tf.cone.scale.set(pulse, 0.85 + pulse * 0.25, pulse);
      tf.coneMat.opacity = 0.18 + pulse * 0.16;
    }

    // Cozy Campfire Dancing Flames, Warm Flickering Glow & Rising Sparks
    if (this.currentZone === 0) {
      for (const fl of this.campFlames) {
        const flicker = Math.sin(tSec * fl.speed + fl.phase) * 0.14 + Math.cos(tSec * (fl.speed * 1.37) + fl.phase) * 0.08;
        fl.mesh.scale.set(1.0 - flicker * 0.35, 1.0 + flicker, 1.0 - flicker * 0.35);
        fl.mesh.position.y = fl.baseY + flicker * 0.16;
        fl.mesh.rotation.y = tSec * 1.8 + fl.phase;
      }
      if (this.campLight) {
        this.campLight.intensity = 2.2 + Math.sin(tSec * 13.0) * 0.35 + Math.cos(tSec * 7.5) * 0.22;
      }
      for (const em of this.campEmbers) {
        const life = (tSec * em.speed + em.phase) % 2.8;
        const frac = life / 2.8;
        const swirlAng = em.angle + tSec * 1.6;
        const r = em.radius * (1 + frac * 1.4);
        em.mesh.position.set(
          -6 + Math.cos(swirlAng) * r,
          0.55 + frac * 2.4,
          11 + Math.sin(swirlAng) * r
        );
        const s = Math.max(0.05, 1.0 - frac * 0.85);
        em.mesh.scale.setScalar(s);
      }
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

    for (let i = this.laserBeams.length - 1; i >= 0; i--) {
      const bm = this.laserBeams[i];
      const age = (now - bm.bornAt) / 320;
      if (age >= 1) {
        this.scene.remove(bm.group);
        this.laserBeams.splice(i, 1);
        continue;
      }
      const fade = 1 - age;
      bm.coreMat.opacity = fade * 0.96;
      bm.outerMat.opacity = fade * 0.62;
      const pulse = 1 + age * 0.65;
      bm.group.scale.set(pulse, 1, pulse);
    }

    if (this.currentZone === 0) {
      for (const hObj of this.animatedHinges.values()) {
        const diff = hObj.targetAngle - hObj.currentAngle;
        if (Math.abs(diff) > 0.002) {
          hObj.currentAngle += diff * Math.min(1, dt * 11.5);
          hObj.pivot.rotation.y = hObj.currentAngle;
          if (hObj.secondaryPivot) hObj.secondaryPivot.rotation.y = -hObj.currentAngle;
        } else if (hObj.currentAngle !== hObj.targetAngle) {
          hObj.currentAngle = hObj.targetAngle;
          hObj.pivot.rotation.y = hObj.targetAngle;
          if (hObj.secondaryPivot) hObj.secondaryPivot.rotation.y = -hObj.targetAngle;
        }
      }

      for (const vObj of this.voxelMap.values()) {
        if (vObj.userData?.trophyHolder) {
          vObj.userData.trophyHolder.rotation.y = tSec * 0.95;
          vObj.userData.trophyHolder.position.y = 0.35 + Math.sin(tSec * 2.6) * 0.025;
        }
      }

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

      // Server-synced Blinking / Disappearing Star Tiles (Pro Obby Stage 2 & Stage 5)
      for (const bp of this.blinkingPlatforms) {
        const phase = ((serverNowMs + bp.phaseOffsetMs) % bp.cycleMs + bp.cycleMs) % bp.cycleMs;
        if (phase < bp.solidMs) {
          bp.passable = false;
          bp.noStand = false;
          bp.mesh.visible = true;
          bp.mesh.material.opacity = 1.0;
          bp.mesh.material.emissiveIntensity = 0.35;
        } else if (phase < bp.solidMs + bp.warnMs) {
          bp.passable = false;
          bp.noStand = false;
          bp.mesh.visible = true;
          const pulse = 0.35 + 0.65 * Math.abs(Math.sin(phase * 0.025));
          bp.mesh.material.opacity = pulse;
          bp.mesh.material.emissiveIntensity = 0.85;
        } else {
          bp.passable = true;
          bp.noStand = true;
          bp.mesh.visible = false;
        }
      }

      // Server-synced Spinning Sweeper Beams (Pro Obby Stage 4)
      for (const sw of this.sweeperHazards) {
        sw.angle = (tSec * sw.speed) % (Math.PI * 2);
        sw.group.rotation.y = sw.angle;
      }

      // Blossom Park Crystal Fountain Shimmer
      for (const sh of this.fountainShimmers) {
        if (sh.userData.shimmerPhase !== undefined) {
          const s = 0.92 + Math.sin(tSec * 2.4 + sh.userData.shimmerPhase) * 0.1;
          sh.scale.set(s, s, 1);
        } else {
          sh.rotation.y = tSec * 1.4;
          sh.position.y = 4.9 + Math.sin(tSec * 2.8) * 0.16;
        }
      }

      if (this.trophyMesh) {
        this.trophyMesh.rotation.y = tSec * 1.8;
        this.trophyMesh.position.y = 17.3 + Math.sin(tSec * 3) * 0.18;
      }
      if (this.hardTrophyMesh) {
        this.hardTrophyMesh.rotation.y = -tSec * 1.9;
        this.hardTrophyMesh.position.y = 41.3 + Math.sin(tSec * 3.2) * 0.22;
      }
      if (this.runwayStarCrest) {
        if (this.runwayStarCrest.userData?.isGlbCrest) {
          this.runwayStarCrest.rotation.z = tSec * 0.55;
        } else {
          if (this.runwayStarCrest.children[0]) this.runwayStarCrest.children[0].rotation.y = tSec * 1.4;
          if (this.runwayStarCrest.children[1]) this.runwayStarCrest.children[1].rotation.z = -tSec * 0.8;
        }
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

      // Dynamic 12-Minute Sky / Cotton-Candy Sunset / Aurora Night Cycle
      this._applyPlazaSkyCycle(serverNowMs, tSec);

      // Spinning & bobbing Falling Stars
      for (const star of this.fallingStarsMap.values()) {
        if (star.core) {
          star.core.rotation.y = tSec * 2.8;
          star.core.rotation.x = Math.sin(tSec * 2.1) * 0.35;
          star.core.position.y = 0.65 + Math.sin(tSec * 3.8) * 0.16;
        }
      }

      // Crafting Bench floating crystal idle bob
      if (this.craftingBenchGroup.userData?.floatingGem) {
        this.craftingBenchGroup.userData.floatingGem.rotation.y = tSec * 1.8;
        this.craftingBenchGroup.userData.floatingGem.position.y = 1.18 + Math.sin(tSec * 3.0) * 0.08;
      }

      // Timed Fashion Show Catwalk Spotlight
      const rwShow = this.runwayShowState;
      if (rwShow && rwShow.phase && rwShow.phase !== 'idle') {
        this.runwaySpotlightGroup.visible = true;
        let spotX = -24;
        let spotZ = 2.0;
        const spotUser = rwShow.spotlightUsername || rwShow.activeContestant || null;
        if ((rwShow.phase === 'catwalk' || rwShow.phase === 'walking') && spotUser) {
          let foundAv = null;
          if (this.selfAvatar?.user?.username === spotUser) {
            foundAv = this.selfAvatar;
          } else {
            for (const rAv of this.remoteAvatars.values()) {
              if (rAv.zone === 0 && rAv.user?.username === spotUser) {
                foundAv = rAv;
                break;
              }
            }
          }
          if (foundAv && foundAv.group.position.x < -14 && foundAv.group.position.x > -34) {
            spotX = foundAv.group.position.x;
            spotZ = foundAv.group.position.z;
          } else {
            spotZ = 2.0 + Math.sin(tSec * 1.4) * 6.5;
          }
        } else if (rwShow.phase === 'podium') {
          spotZ = 11.2;
        } else {
          spotZ = 1.5 + Math.sin(tSec * 0.9) * 5.0;
        }
        this.runwaySpotlightGroup.position.x += (spotX - this.runwaySpotlightGroup.position.x) * Math.min(1, dt * 8);
        this.runwaySpotlightGroup.position.z += (spotZ - this.runwaySpotlightGroup.position.z) * Math.min(1, dt * 8);
      } else {
        this.runwaySpotlightGroup.visible = false;
      }
    } else {
      // Animate Crystal Cavern & Shadow Rift Arena mobs, Boss Dragon, molten lava, and spinning loot orbs
      if (this.currentZone === 3) {
        const lavaPulse = 1.2 + Math.sin(tSec * 3.4) * 0.28;
        for (const lm of this.arenaLavaMeshes) {
          if (lm.material) lm.material.emissiveIntensity = lavaPulse;
        }
        if (this.bossDragonEntry && this.bossDragonEntry.group.visible && this.bossDragonEntry.data) {
          const bd = this.bossDragonEntry;
          bd.group.position.lerp(bd.targetPos, Math.min(1, dt * 7.5));
          bd.group.rotation.y = bd.data.ry || 0;
          const flap = Math.sin(tSec * 6.4) * 0.42;
          if (bd.leftWing) bd.leftWing.rotation.z = flap;
          if (bd.rightWing) bd.rightWing.rotation.z = -flap;
          bd.visual.position.y = 0.28 + Math.sin(tSec * 3.2) * 0.22;
        }
      }
      const activeMobs = this.currentZone === 3 ? this.arenaMobsMap : this.dungeonMobsMap;
      const activeDrops = this.currentZone === 3 ? this.arenaDropsMap : this.dungeonDropsMap;
      for (const entry of activeMobs.values()) {
        if (!entry.data.alive) continue;
        entry.group.position.lerp(entry.targetPos, Math.min(1, dt * 8.5));
        entry.group.rotation.y = entry.data.ry || 0;
        const kind = entry.data.kind;
        if (kind === 'crystal_bat') {
          const flap = Math.sin(tSec * 12 + entry.phaseOffset) * 0.48;
          if (entry.leftWing) entry.leftWing.rotation.z = flap;
          if (entry.rightWing) entry.rightWing.rotation.z = -flap;
          entry.visual.position.y = Math.sin(tSec * 4.2 + entry.phaseOffset) * 0.16;
          entry.visual.scale.setScalar(1);
        } else if (kind === 'lava_golem') {
          const stride = Math.sin(tSec * 4.8 + entry.phaseOffset);
          entry.visual.position.y = Math.abs(stride) * 0.1;
          if (entry.leftArm) entry.leftArm.rotation.x = stride * 0.45;
          if (entry.rightArm) entry.rightArm.rotation.x = -stride * 0.45;
          entry.visual.scale.setScalar(1);
        } else if (kind === 'void_wraith') {
          entry.visual.position.y = 0.18 + Math.sin(tSec * 3.2 + entry.phaseOffset) * 0.18;
          entry.visual.rotation.z = Math.sin(tSec * 2.2 + entry.phaseOffset) * 0.06;
          entry.visual.scale.setScalar(1);
        } else if (kind === 'rift_spider') {
          const scuttle = Math.sin(tSec * 14 + entry.phaseOffset);
          entry.visual.position.y = Math.abs(scuttle) * 0.06;
          entry.visual.rotation.z = scuttle * 0.05;
          entry.visual.scale.setScalar(1);
        } else {
          const hop = Math.abs(Math.sin(tSec * 4.5 + entry.phaseOffset));
          entry.visual.position.y = hop * 0.34;
          entry.visual.scale.set(1 + (1 - hop) * 0.1, 0.88 + hop * 0.2, 1 + (1 - hop) * 0.1);
        }
      }

      for (const dropEntry of activeDrops.values()) {
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
      const playerRadius = 0.36;
      const playerHeight = 1.75;
      const stepHeight = 0.55;
      let moveForward = 0;
      let moveRight = 0;

      if (!this.salonMirrorMode && !(this.currentZone === 3 && this.arenaDead)) {
        if (this.keys.has('KeyW') || this.keys.has('ArrowUp') || this.touchMove.forward) moveForward += 1;
        if (this.keys.has('KeyS') || this.keys.has('ArrowDown') || this.touchMove.backward) moveForward -= 1;
        if (this.keys.has('KeyA') || this.keys.has('ArrowLeft') || this.touchMove.left) moveRight -= 1;
        if (this.keys.has('KeyD') || this.keys.has('ArrowRight') || this.touchMove.right) moveRight += 1;
      }

      const isSprinting = this.keys.has('ShiftLeft') || this.keys.has('ShiftRight');
      const potionMult = serverNowMs < this.speedBoostUntilMs ? 1.35 : 1.0;
      const eqMount = this.currentZone === 0 && !this.selfAvatar.emote ? this.selfAvatar.equippedMount : null;
      const mountMult =
        eqMount === 'comet_surfer'
          ? 1.78
          : eqMount === 'sunset_skates'
            ? 1.68
            : eqMount === 'starlight_board'
              ? 1.62
              : 1.0;
      const speed = (isSprinting ? 11.4 : 8.4) * Math.max(potionMult, mountMult);
      const forwardVec = new THREE.Vector3(-Math.sin(this.cameraYaw), 0, -Math.cos(this.cameraYaw));
      const rightVec = new THREE.Vector3(Math.cos(this.cameraYaw), 0, -Math.sin(this.cameraYaw));

      const moveDir = new THREE.Vector3();
      moveDir.addScaledVector(forwardVec, moveForward);
      moveDir.addScaledVector(rightVec, moveRight);

      let totalDx = 0;
      let totalDz = 0;
      if (moveDir.lengthSq() > 0.001) {
        moveDir.normalize();
        totalDx = moveDir.x * speed * dt;
        totalDz = moveDir.z * speed * dt;
        this.selfAvatar.group.rotation.y = Math.atan2(moveDir.x, moveDir.z);
        this.selfAvatar.emote = null;
        this.selfAvatar.anim = this.isGrounded ? (isSprinting ? 'run' : 'walk') : 'jump';
      } else {
        this.selfAvatar.anim = this.isGrounded ? 'idle' : 'jump';
      }

      const horizDist = Math.hypot(totalDx, totalDz);

      // Check if player is walking through an open door or double door for snag-free passage
      let inOpenDoorway = false;
      if (this.currentZone === 0 && this.openDoorPortals.length > 0) {
        for (const dp of this.openDoorPortals) {
          if (pos.y + playerHeight <= dp.bottomY + 0.05 || pos.y >= dp.topY - 0.15) continue;
          const throughDist = dp.isEW ? Math.abs(pos.x - dp.centerX) : Math.abs(pos.z - dp.centerZ);
          const lateralOffset = dp.isEW ? pos.z - dp.centerZ : pos.x - dp.centerX;
          if (throughDist <= 1.1 && Math.abs(lateralOffset) <= dp.halfOpening + 0.22) {
            inOpenDoorway = true;
            // Gentle lateral centering assist while actively walking toward/through an open single doorway
            if (horizDist > 0.001) {
              const movingThrough = dp.isEW ? Math.abs(totalDx) > Math.abs(totalDz) * 0.5 : Math.abs(totalDz) > Math.abs(totalDx) * 0.5;
              const maxLateralFree = Math.max(0.02, dp.halfOpening - 0.28);
              if (movingThrough && Math.abs(lateralOffset) > maxLateralFree) {
                const targetLat = Math.sign(lateralOffset) * maxLateralFree;
                const nudge = (targetLat - lateralOffset) * Math.min(1, dt * 14);
                if (dp.isEW) pos.z += nudge;
                else pos.x += nudge;
              }
            }
            break;
          }
        }
      }

      const activeRadius = inOpenDoorway ? 0.26 : playerRadius;
      // Substepped horizontal movement + solid wall push-out + 2-story stair auto-stepping + unstuck guard
      const substeps = Math.max(1, Math.ceil(horizDist / 0.22));
      const stepDx = totalDx / substeps;
      const stepDz = totalDz / substeps;
      const rSq = activeRadius * activeRadius;

      for (let s = 0; s < substeps; s++) {
        pos.x += stepDx;
        pos.z += stepDz;
        if (this.currentZone === 2) {
          pos.x = Math.max(-13.1, Math.min(13.1, pos.x));
          pos.z = Math.max(-13.1, Math.min(13.1, pos.z));
        }

        for (let pass = 0; pass < 2; pass++) {
          for (const col of this.colliders) {
            if (col.type === 'ground' || col.type === 'jumppad' || col.passable) continue;
            if (col.topY <= pos.y + 0.02 || col.bottomY >= pos.y + playerHeight) continue;

            const cx = Math.max(col.minX, Math.min(col.maxX, pos.x));
            const cz = Math.max(col.minZ, Math.min(col.maxZ, pos.z));
            const dx = pos.x - cx;
            const dz = pos.z - cz;
            const dSq = dx * dx + dz * dz;
            if (dSq >= rSq) continue;

            // Smooth stair / low-curb auto-step when headroom is clear
            if (
              horizDist > 0.0005 &&
              this.isGrounded &&
              !col.noStand &&
              col.topY > pos.y + 0.01 &&
              col.topY <= pos.y + stepHeight &&
              this._hasHeadroomAt(pos.x, pos.z, col.topY, activeRadius, playerHeight)
            ) {
              pos.y = col.topY;
              if (this.velocity.y < 0) this.velocity.y = 0;
              continue;
            }

            if (dSq > 1e-6) {
              const d = Math.sqrt(dSq);
              const pen = activeRadius - d;
              pos.x += (dx / d) * pen;
              pos.z += (dz / d) * pen;
            } else {
              // Center is inside collider (e.g. friend placed a wall on top of avatar): push out cleanly
              const pushLeft = pos.x - col.minX + activeRadius;
              const pushRight = col.maxX - pos.x + activeRadius;
              const pushBack = pos.z - col.minZ + activeRadius;
              const pushFront = col.maxZ - pos.z + activeRadius;
              const minPush = Math.min(pushLeft, pushRight, pushBack, pushFront);
              if (minPush === pushLeft) pos.x = col.minX - activeRadius;
              else if (minPush === pushRight) pos.x = col.maxX + activeRadius;
              else if (minPush === pushBack) pos.z = col.minZ - activeRadius;
              else pos.z = col.maxZ + activeRadius;
            }
          }
        }
      }

      this.velocity.y -= 28.0 * dt;
      const prevFootY = pos.y;
      const prevHeadY = prevFootY + playerHeight;
      pos.y += this.velocity.y * dt;

      // Upward ceiling head-bonk prevention (blocks jumping up through 2nd-story floors/roofs)
      if (this.velocity.y > 0) {
        const ceilR2 = (playerRadius * 0.92) * (playerRadius * 0.92);
        for (const col of this.colliders) {
          if (col.type === 'ground' || col.type === 'jumppad' || col.passable) continue;
          const cx = Math.max(col.minX, Math.min(col.maxX, pos.x));
          const cz = Math.max(col.minZ, Math.min(col.maxZ, pos.z));
          const dx = pos.x - cx;
          const dz = pos.z - cz;
          if (dx * dx + dz * dz >= ceilR2) continue;

          if (prevHeadY <= col.bottomY + 0.24 && pos.y + playerHeight >= col.bottomY) {
            pos.y = col.bottomY - playerHeight - 0.01;
            this.velocity.y = 0;
          }
        }
      }

      // Downward landing: select the highest valid overlapping surface so 2nd-story floors & stairs take priority
      this.isGrounded = false;
      if (this.velocity.y <= 0) {
        let bestFloorCol = null;
        let bestTopY = -Infinity;
        for (const col of this.colliders) {
          if (col.passable || col.noStand) continue;
          const cx = Math.max(col.minX, Math.min(col.maxX, pos.x));
          const cz = Math.max(col.minZ, Math.min(col.maxZ, pos.z));
          const dx = pos.x - cx;
          const dz = pos.z - cz;
          if (dx * dx + dz * dz >= rSq) continue;

          if (prevFootY >= col.topY - 0.38 && pos.y <= col.topY + 0.08) {
            if (col.topY > bestTopY || (col.topY === bestTopY && col.type === 'jumppad')) {
              bestTopY = col.topY;
              bestFloorCol = col;
            }
          }
        }

        if (bestFloorCol) {
          const col = bestFloorCol;
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

          if (col.type === 'checkpoint') {
            const tier = col.obbyTier === 'hard' ? 'hard' : 'easy';
            this.lastActiveObby = tier;
            const track = this.obbyCheckpoints[tier];
            if (col.checkpointIndex > track.index) {
              const nextCp = {
                x: col.mesh.position.x,
                y: col.topY,
                z: col.mesh.position.z,
                index: col.checkpointIndex
              };
              this.obbyCheckpoints[tier] = nextCp;
              if (tier === 'easy') this.currentCheckpoint = nextCp;
              col.mesh.material.color.set('#22c55e');
              this.onObbyCheckpoint(col.checkpointIndex, tier);
            }
          }

          if (col.type === 'trophy-pad' && now - this.lastTrophyWinAt > 12000) {
            this.lastTrophyWinAt = now;
            this.lastActiveObby = 'easy';
            const nextCp = {
              x: col.mesh.position.x,
              y: col.topY,
              z: col.mesh.position.z,
              index: 3
            };
            this.obbyCheckpoints.easy = nextCp;
            this.currentCheckpoint = nextCp;
            this.spawnCelebrationConfetti(pos);
            this.onTrophyWin('easy');
          }

          if (col.type === 'gauntlet-trophy-pad' && now - this.lastHardTrophyWinAt > 12000) {
            this.lastHardTrophyWinAt = now;
            this.lastActiveObby = 'hard';
            this.obbyCheckpoints.hard = {
              x: col.mesh.position.x,
              y: col.topY,
              z: col.mesh.position.z,
              index: 6
            };
            this.spawnCelebrationConfetti(pos);
            this.onTrophyWin('hard');
          }
        }
      }

      // Obby Speedrun Start & Finish Laser Gate Crossing (Zone 0)
      if (this.currentZone === 0 && !this.salonMirrorMode) {
        if (!this._inStartGate) this._inStartGate = { easy: false, hard: false };
        for (const [courseKey, cfg] of Object.entries(this.obbyGates)) {
          if (!cfg || !cfg.start || !cfg.finish) continue;
          const dStart = Math.hypot(
            pos.x - cfg.start.pos.x,
            pos.y - (cfg.start.pos.y + 0.8),
            pos.z - cfg.start.pos.z
          );
          const insideStart = dStart <= 2.9;
          if (
            insideStart &&
            !this._inStartGate[courseKey] &&
            horizDist > 0.0005 &&
            now - (this.lastObbyGateAt[`${courseKey}_start`] || 0) > 2200
          ) {
            this.lastObbyGateAt[`${courseKey}_start`] = now;
            this.spawnSparkleBurst(pos, '#34d399');
            this.onObbyGateCrossed(courseKey, 'start');
          }
          this._inStartGate[courseKey] = insideStart;

          if (this.activeObbyCourse === courseKey) {
            const dFin = Math.hypot(
              pos.x - cfg.finish.pos.x,
              pos.y - (cfg.finish.pos.y + 0.6),
              pos.z - cfg.finish.pos.z
            );
            if (dFin <= 3.6 && now - (this.lastObbyGateAt[`${courseKey}_finish`] || 0) > 2500) {
              this.lastObbyGateAt[`${courseKey}_finish`] = now;
              this.activeObbyCourse = null;
              this.spawnCelebrationConfetti(pos);
              this.onObbyGateCrossed(courseKey, 'finish');
            }
          }
        }

        // Auto-catch Falling Stars when walking right through them!
        for (const star of this.fallingStarsMap.values()) {
          if (
            pos.distanceTo(star.group.position) < 2.35 &&
            now - (star.lastCatchAttemptAt || 0) > 750
          ) {
            star.lastCatchAttemptAt = now;
            this.spawnSparkleBurst(star.group.position, '#fde047');
            this.onCatchFallingStar(star.id);
          }
        }
      }

      // Spinning Sweeper Beam Collision (Pro Obby Stage 4 — Jump Over!)
      if (this.currentZone === 0 && this.sweeperHazards.length > 0 && now - this.lastSweeperHitAt > 480) {
        for (const sw of this.sweeperHazards) {
          if (pos.y > sw.cy + 0.14 || pos.y + playerHeight < sw.cy - 0.22) continue;
          const relX = pos.x - sw.cx;
          const relZ = pos.z - sw.cz;
          const cosA = Math.cos(sw.angle);
          const sinA = Math.sin(sw.angle);
          // Bar lies along local X axis rotated around Y by sw.angle => world dir = (cosA, -sinA)
          const along = relX * cosA - relZ * sinA;
          const perp = relX * sinA + relZ * cosA;
          if (Math.abs(along) <= sw.radius + 0.25 && Math.abs(perp) <= 0.42) {
            this.lastSweeperHitAt = now;
            // Push along tangent sweep direction + small upward pop
            const sweepSign = sw.speed >= 0 ? 1 : -1;
            const tangX = -sinA * sweepSign * (along >= 0 ? 1 : -1);
            const tangZ = -cosA * sweepSign * (along >= 0 ? 1 : -1);
            pos.x += tangX * 1.45;
            pos.z += tangZ * 1.45;
            this.velocity.y = Math.max(this.velocity.y, 7.2);
            this.isGrounded = false;
            this.spawnSparkleBurst(pos, '#fb7185');
            break;
          }
        }
      }

      // Keep directional sun shadow frustum centered around player across the 210×210 Plaza
      if (this.currentZone === 0 && this.sunLight) {
        const snapX = Math.round(pos.x / 2) * 2;
        const snapZ = Math.round(pos.z / 2) * 2;
        this.sunLight.target.position.set(snapX, 0, snapZ);
        this.sunLight.position.set(snapX + 28, 52, snapZ + 24);
      }

      if (pos.y < -14) {
        if (this.currentZone === 1) {
          pos.set(0, 0.6, 22);
          this.velocity.set(0, 0, 0);
          this._emitMoveNow();
        } else if (this.currentZone === 3) {
          // In Shadow Rift Arena, falling into the abyss deals lethal server damage; clamp Y so camera stays framed
          pos.y = -3.0;
          this.velocity.set(0, 0, 0);
          this._emitMoveNow();
        } else if (this.currentZone === 2) {
          pos.set(0, 0.6, 3.5);
          this.velocity.set(0, 0, 0);
          this._emitMoveNow();
        } else if (this.lastActiveObby === 'hard' && pos.x > 36 && pos.z > 36) {
          this.warpToHardObbyCheckpoint();
        } else {
          this.warpToPlaza();
        }
      }

      // Zone-specific proximity, portal walk-in, doors/windows/lamps, furniture, jump-stomp, auto-pickup & contextual HUD prompt
      let nextHint = '';
      const acc = this.selfAvatar.user?.avatar?.accessory;
      const hasRayGun = acc === 'ray_gun';
      const hasWand = acc === 'starlight_wand';
      const hasBlade = acc === 'crystal_blade';
      if (this.currentZone === 0) {
        const isNowNearRunway = pos.x < -14.5 && pos.x > -33.5 && pos.z > -15.5 && pos.z < 16.5;
        if (isNowNearRunway !== this.nearRunway) {
          this.nearRunway = isNowNearRunway;
          this.onRunwayProximity(isNowNearRunway);
        }
        const dPortal = Math.hypot(pos.x - this.plazaPortalPos.x, pos.z - this.plazaPortalPos.z);
        const dArenaPortal = Math.hypot(
          pos.x - this.plazaArenaPortalPos.x,
          pos.z - this.plazaArenaPortalPos.z
        );
        const dCraft = Math.hypot(pos.x - this.craftingBenchPos.x, pos.z - this.craftingBenchPos.z);
        const nearMb = this._findNearestMailbox(4.1);

        if (dPortal < 2.65 && Math.abs(pos.y - this.plazaPortalPos.y) < 3.5 && now - this.lastPortalTriggerAt > 2600) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(1);
        } else if (
          dArenaPortal < 2.65 &&
          Math.abs(pos.y - this.plazaArenaPortalPos.y) < 3.5 &&
          now - this.lastPortalTriggerAt > 2600
        ) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(3);
        } else if (dPortal < 5.5) {
          nextHint = '🌀 Step In or Press E — Enter Crystal Cavern Dungeon!';
        } else if (dArenaPortal < 5.5) {
          nextHint = '⚔️ Step In or Press E — Enter Shadow Rift Mortal Arena (Watch HP & Lava!)';
        } else if (nearMb) {
          const pl = nearMb.data || {};
          if (!pl.ownerUsername) {
            nextHint = `🏡 Press E or Click — Claim ${pl.defaultName || pl.plotId}!`;
          } else if (pl.doorLocked) {
            nextHint = `🔔 Press E or Click — Ring Doorbell at ${pl.customTitle || pl.defaultName} (@${pl.ownerUsername})`;
          } else {
            nextHint = `🏡 Press E or Click — View ${pl.customTitle || pl.defaultName} (@${pl.ownerUsername})`;
          }
        } else if (dCraft < 4.5) {
          nextHint = '⚒️ Press E or Click — Open Starlight Crafting Workshop (Craft Gear & Pet Eggs!)';
        } else {
          for (const [, bd] of Object.entries(this.obbyBoardScreens)) {
            if (Math.hypot(pos.x - bd.pos.x, pos.z - bd.pos.z) < 4.4) {
              nextHint = `🏆 Press E or Click — View ${bd.heading} Speedrun Records!`;
              break;
            }
          }
          if (!nextHint) {
            const nearToggle = this._findNearestToggleableBlock(3.8);
            const nearFurn = this._findNearestInteractiveFurniture(2.65);
            if (nearToggle) {
              const isLamp = nearToggle.kind === 'nightstand_lamp' || nearToggle.kind === 'crystal_lamp';
              if (isLamp) {
                const verb = nearToggle.open ? 'Turn On' : 'Dim';
                nextHint = `💡 Press E or Click — ${verb} ${nearToggle.kind === 'crystal_lamp' ? 'Aurora Lamp' : 'Warm Lamp'}`;
              } else {
                const noun =
                  nearToggle.kind === 'double_door'
                    ? 'Double Door'
                    : nearToggle.kind === 'door'
                      ? 'Door'
                      : 'Window';
                const verb = nearToggle.open ? 'Close' : 'Open';
                const icon = nearToggle.kind === 'window' ? '🪟' : '🚪';
                nextHint = this.buildMode
                  ? `${icon} Press E — ${verb} ${noun}`
                  : `${icon} Press E or Click — ${verb} ${noun}`;
              }
            } else if (!this.buildMode && nearFurn) {
              nextHint =
                nearFurn.kind === 'bed'
                  ? '😴 Press E or Click — Lie Down & Sleep on Bed!'
                  : `🪑 Press E or Click — Sit on ${nearFurn.kind === 'sofa' ? 'Plush Sofa' : 'Chair'}!`;
            } else if (!this.buildMode && hasRayGun) {
              nextHint = '🔫 Ray Gun Equipped — Press E or Click to Zap Starlight Beam!';
            } else if (!this.buildMode && hasWand) {
              nextHint = '🪄 Starlight Wand Equipped — Press E or Click to Cast Star-Bolt!';
            }
          }
        }
      } else if (this.currentZone === 2) {
        const dGate = Math.hypot(
          pos.x - this.prisonReleasePortalPos.x,
          pos.z - this.prisonReleasePortalPos.z
        );
        if (this.prisonReleaseEligible && dGate < 2.2 && now - this.lastPortalTriggerAt > 2600) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(0);
        } else if (dGate < 4.8) {
          nextHint = this.prisonReleaseEligible
            ? '🔓 Sentence Served! Step In or Press E — Return to Starlight Plaza!'
            : '🔒 Release Gate Locked — Work Glowing Chores (+5 🪙) to Clear Debt!';
        }
        if (!nextHint) {
          for (const entry of this.prisonNodesMap.values()) {
            if (!entry.data.available) continue;
            if (pos.distanceTo(entry.group.position) < 5.4) {
              nextHint = `${entry.data.icon || '🪨'} Press E to Work ${entry.data.name} (+5 🪙 Toward Debt)`;
              break;
            }
          }
        }
      } else if (this.currentZone === 3 && this.arenaDead) {
        nextHint = '💀 You fell in the Shadow Rift! Click ⚡ Respawn at Sanctuary (or press Enter)!';
      } else {
        const returnPos =
          this.currentZone === 3 ? this.arenaReturnPortalPos : this.dungeonReturnPortalPos;
        const activeMobsMap = this.currentZone === 3 ? this.arenaMobsMap : this.dungeonMobsMap;
        const activeNodesMap = this.currentZone === 3 ? this.arenaNodesMap : this.dungeonNodesMap;
        const activeDropsMap = this.currentZone === 3 ? this.arenaDropsMap : this.dungeonDropsMap;

        const dReturn = Math.hypot(pos.x - returnPos.x, pos.z - returnPos.z);
        if (dReturn < 2.25 && now - this.lastPortalTriggerAt > 2600) {
          this.lastPortalTriggerAt = now;
          this.onRequestZonePortal(0);
        } else if (dReturn < 5.0) {
          nextHint = '🌟 Step In or Press E — Return to Starlight Plaza';
        }

        // Boss Stomp & Boss Altar Hint in Shadow Rift (Zone 3)
        if (this.currentZone === 3) {
          if (this.bossDragonEntry?.data?.active && this.bossDragonEntry.data.hp > 0) {
            const bPos = this.bossDragonEntry.group.position;
            const hBoss = Math.hypot(pos.x - bPos.x, pos.z - bPos.z);
            if (
              this.velocity.y < -1.0 &&
              hBoss < 2.8 &&
              pos.y >= bPos.y + 0.4 &&
              pos.y <= bPos.y + 4.8 &&
              now - (this.bossDragonEntry.lastStompAt || 0) > 320
            ) {
              this.bossDragonEntry.lastStompAt = now;
              this.velocity.y = 13.5;
              this.isGrounded = false;
              this.selfAvatar.anim = 'jump';
              this.spawnSparkleBurst(bPos, '#c084fc');
              this.onAttackMob('obsidian_dragon', 'stomp');
            } else if (!nextHint && hBoss < 16.5) {
              nextHint =
                '🐉 Press E or Click to Strike Obsidian Dragon — JUMP Over Purple Shockwave Rings!';
            }
          } else if (
            !nextHint &&
            Math.hypot(pos.x - this.bossAltarPos.x, pos.z - this.bossAltarPos.z) < 5.8
          ) {
            nextHint = '🔮 Press E or Click — Summon Co-Op Obsidian Rift Dragon Boss!';
          }
        }

        // Mario-style Jump-Stomp on live mobs + nearest mob hint
        const mobHintRange = hasWand ? 16.0 : hasRayGun ? 14.0 : hasBlade ? 6.2 : 5.2;
        for (const entry of activeMobsMap.values()) {
          if (!entry.data.alive) continue;
          const mPos = entry.group.position;
          const hDist = Math.hypot(pos.x - mPos.x, pos.z - mPos.z);
          const stompHeight = entry.data.kind === 'lava_golem' ? 3.2 : 2.3;
          if (
            this.velocity.y < -1.0 &&
            hDist < 1.65 &&
            pos.y >= mPos.y + 0.15 &&
            pos.y <= mPos.y + stompHeight &&
            now - (entry.lastStompAt || 0) > 280
          ) {
            entry.lastStompAt = now;
            this.velocity.y = 12.2;
            this.isGrounded = false;
            this.selfAvatar.anim = 'jump';
            this.spawnSparkleBurst(mPos, '#fbbf24');
            this.onAttackMob(entry.data.id, 'stomp');
          } else if (!nextHint && hDist < mobHintRange) {
            nextHint = hasRayGun
              ? `🔫 Press E or Click — Zap ${entry.data.name} with Ray Gun (3 DMG)!`
              : hasWand
                ? `🪄 Press E or Click — Cast Starlight Wand at ${entry.data.name} (2 DMG + Mend)!`
                : hasBlade
                  ? `⚔️ Press E or Click — Slash ${entry.data.name} with Crystal Blade (2 DMG)!`
                  : `⚔️ Press E (Sparkle Blast) or Jump-Stomp ${entry.data.name}!`;
          }
        }

        // Nearest Crystal Vein / Treasure Chest hint
        if (!nextHint) {
          for (const entry of activeNodesMap.values()) {
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

        // Auto-pickup Ground Loot Orbs (extended magnet radius when a Pet is equipped!)
        const petMagnet =
          this.selfAvatar.equippedPet === 'baby_crystal_dragon'
            ? 4.6
            : this.selfAvatar.equippedPet
              ? 3.85
              : 2.55;
        for (const dropEntry of activeDropsMap.values()) {
          if (
            pos.distanceTo(dropEntry.group.position) < petMagnet &&
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

      // Keep lookAt at 1.62m (below 2.0m 2-story ceilings!) and raycast-pull the camera inside rooms
      const lookHeight = this.salonMirrorMode ? 1.58 : 1.62;
      const targetLookAt = new THREE.Vector3(pos.x, pos.y + lookHeight, pos.z);
      if (this.salonMirrorMode && window.innerWidth > 860) {
        const camRightX = Math.cos(this.cameraYaw);
        const camRightZ = -Math.sin(this.cameraYaw);
        targetLookAt.x += camRightX * 0.92;
        targetLookAt.z += camRightZ * 0.92;
      }

      const camDir = new THREE.Vector3(
        Math.sin(this.cameraYaw) * Math.cos(this.cameraPitch),
        Math.sin(this.cameraPitch),
        Math.cos(this.cameraYaw) * Math.cos(this.cameraPitch)
      ).normalize();

      let targetCamDist = this.cameraDist;
      if (!this.salonMirrorMode && this.cameraOccluders.length > 0) {
        this.cameraRaycaster.set(targetLookAt, camDir);
        this.cameraRaycaster.near = 0.18;
        this.cameraRaycaster.far = this.cameraDist + 0.35;
        const camHits = this.cameraRaycaster.intersectObjects(this.cameraOccluders, true);
        if (camHits.length > 0) {
          targetCamDist = Math.max(1.05, Math.min(this.cameraDist, camHits[0].distance - 0.28));
        }
      }

      const distLerp = targetCamDist < this.effectiveCameraDist ? Math.min(1, dt * 18) : Math.min(1, dt * 7.5);
      this.effectiveCameraDist += (targetCamDist - this.effectiveCameraDist) * distLerp;

      const camX = targetLookAt.x + camDir.x * this.effectiveCameraDist;
      const camY = targetLookAt.y + camDir.y * this.effectiveCameraDist;
      const camZ = targetLookAt.z + camDir.z * this.effectiveCameraDist;
      this.camera.position.set(camX, Math.max(0.45, camY), camZ);
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

  getSkyPhaseSummary(serverNowMs = Date.now() + this.serverClockOffsetMs) {
    const meteorActive =
      Boolean(this.skyState?.meteorShowerActive) ||
      serverNowMs < (Number(this.skyState?.meteorShowerUntilMs) || 0);
    const mode = this.skyState?.mode || 'auto';
    if (meteorActive) {
      return { icon: '🌠', label: 'Meteor Shower!', phase: 'meteor' };
    }
    if (mode === 'day') return { icon: '☀️', label: 'Sunny Day', phase: 'day' };
    if (mode === 'sunset') return { icon: '🌅', label: 'Cotton-Candy Sunset', phase: 'sunset' };
    if (mode === 'aurora_night' || mode === 'night') {
      return { icon: '🌌', label: 'Aurora Night', phase: 'aurora_night' };
    }

    const cycleMs = Math.max(
      60000,
      Number(this.skyState?.cycleDurationMs || this.skyState?.cycleMs) || 720000
    );
    const anchorMs = Number(this.skyState?.cycleAnchorMs) || 0;
    const elapsed = Math.max(0, serverNowMs - anchorMs);
    const frac = (elapsed % cycleMs) / cycleMs;
    if (frac < 0.42 || frac >= 0.96) return { icon: '☀️', label: 'Sunny Day', phase: 'day' };
    if (frac < 0.64) return { icon: '🌅', label: 'Cotton-Candy Sunset', phase: 'sunset' };
    if (frac < 0.91) return { icon: '🌌', label: 'Aurora Night', phase: 'aurora_night' };
    return { icon: '🌸', label: 'Rosy Dawn', phase: 'dawn' };
  }

  _applyPlazaSkyCycle(serverNowMs, tSec) {
    const cycleMs = Math.max(
      60000,
      Number(this.skyState?.cycleDurationMs || this.skyState?.cycleMs) || 720000
    );
    const mode = this.skyState?.mode || 'auto';
    let frac = 0.18;
    if (mode === 'day') frac = 0.18;
    else if (mode === 'sunset') frac = 0.52;
    else if (mode === 'aurora_night' || mode === 'night') frac = 0.78;
    else {
      const anchorMs = Number(this.skyState?.cycleAnchorMs) || 0;
      const elapsed = Math.max(0, serverNowMs - anchorMs);
      frac = (elapsed % cycleMs) / cycleMs;
    }

    const keyframes = [
      {
        t: 0.0,
        bg: '#7dd3fc',
        fog: '#bae6fd',
        hemiSky: '#ffffff',
        hemiGround: '#fbcfe8',
        hemiInt: 0.92,
        sunCol: '#fffbeb',
        sunInt: 1.28,
        fillCol: '#fdf2f8',
        fillInt: 0.55,
        aurora: 0.0,
        stars: 0.0
      },
      {
        t: 0.34,
        bg: '#7dd3fc',
        fog: '#bae6fd',
        hemiSky: '#ffffff',
        hemiGround: '#fbcfe8',
        hemiInt: 0.92,
        sunCol: '#fffbeb',
        sunInt: 1.28,
        fillCol: '#fdf2f8',
        fillInt: 0.55,
        aurora: 0.0,
        stars: 0.0
      },
      {
        t: 0.5,
        bg: '#f472b6',
        fog: '#fdba74',
        hemiSky: '#fbcfe8',
        hemiGround: '#7c2d12',
        hemiInt: 0.86,
        sunCol: '#fb923c',
        sunInt: 1.08,
        fillCol: '#c084fc',
        fillInt: 0.62,
        aurora: 0.22,
        stars: 0.28
      },
      {
        t: 0.66,
        bg: '#0b1026',
        fog: '#15193b',
        hemiSky: '#818cf8',
        hemiGround: '#1e1b4b',
        hemiInt: 0.68,
        sunCol: '#c4b5fd',
        sunInt: 0.58,
        fillCol: '#34d399',
        fillInt: 0.5,
        aurora: 0.95,
        stars: 0.95
      },
      {
        t: 0.86,
        bg: '#0b1026',
        fog: '#15193b',
        hemiSky: '#818cf8',
        hemiGround: '#1e1b4b',
        hemiInt: 0.68,
        sunCol: '#c4b5fd',
        sunInt: 0.58,
        fillCol: '#34d399',
        fillInt: 0.5,
        aurora: 0.95,
        stars: 0.95
      },
      {
        t: 0.95,
        bg: '#f9a8d4',
        fog: '#bae6fd',
        hemiSky: '#fde68a',
        hemiGround: '#fbcfe8',
        hemiInt: 0.85,
        sunCol: '#fde047',
        sunInt: 1.05,
        fillCol: '#38bdf8',
        fillInt: 0.52,
        aurora: 0.16,
        stars: 0.2
      },
      {
        t: 1.0,
        bg: '#7dd3fc',
        fog: '#bae6fd',
        hemiSky: '#ffffff',
        hemiGround: '#fbcfe8',
        hemiInt: 0.92,
        sunCol: '#fffbeb',
        sunInt: 1.28,
        fillCol: '#fdf2f8',
        fillInt: 0.55,
        aurora: 0.0,
        stars: 0.0
      }
    ];

    let k0 = keyframes[0];
    let k1 = keyframes[1];
    for (let i = 0; i < keyframes.length - 1; i++) {
      if (frac >= keyframes[i].t && frac <= keyframes[i + 1].t) {
        k0 = keyframes[i];
        k1 = keyframes[i + 1];
        break;
      }
    }
    const span = Math.max(0.0001, k1.t - k0.t);
    const alpha = Math.min(1, Math.max(0, (frac - k0.t) / span));

    if (!this._tmpColA) {
      this._tmpColA = new THREE.Color();
      this._tmpColB = new THREE.Color();
    }
    this.scene.background.copy(this._tmpColA.set(k0.bg).lerp(this._tmpColB.set(k1.bg), alpha));
    this.scene.fog.color.copy(this._tmpColA.set(k0.fog).lerp(this._tmpColB.set(k1.fog), alpha));
    this.hemiLight.color.copy(this._tmpColA.set(k0.hemiSky).lerp(this._tmpColB.set(k1.hemiSky), alpha));
    this.hemiLight.groundColor.copy(
      this._tmpColA.set(k0.hemiGround).lerp(this._tmpColB.set(k1.hemiGround), alpha)
    );
    this.hemiLight.intensity = k0.hemiInt + (k1.hemiInt - k0.hemiInt) * alpha;
    this.sunLight.color.copy(this._tmpColA.set(k0.sunCol).lerp(this._tmpColB.set(k1.sunCol), alpha));
    this.sunLight.intensity = k0.sunInt + (k1.sunInt - k0.sunInt) * alpha;
    this.fillLight.color.copy(this._tmpColA.set(k0.fillCol).lerp(this._tmpColB.set(k1.fillCol), alpha));
    this.fillLight.intensity = k0.fillInt + (k1.fillInt - k0.fillInt) * alpha;

    const meteorActive = serverNowMs < (Number(this.skyState?.meteorShowerUntilMs) || 0);
    const auroraOp = Math.max(
      meteorActive ? 0.85 : 0,
      k0.aurora + (k1.aurora - k0.aurora) * alpha
    );
    const starOp = Math.max(meteorActive ? 0.85 : 0, k0.stars + (k1.stars - k0.stars) * alpha);
    this.currentNightFactor = starOp;

    for (let i = 0; i < this.auroraMaterials.length; i++) {
      const m = this.auroraMaterials[i];
      if (m && m.uniforms) {
        m.uniforms.uTime.value = tSec + i * 2.1;
        m.uniforms.uOpacity.value = auroraOp;
      }
    }
    if (this.nightStarPoints) {
      this.nightStarPoints.visible = starOp > 0.02;
      this.nightStarPoints.material.opacity = starOp * (0.82 + Math.sin(tSec * 2.5) * 0.16);
      this.nightStarPoints.rotation.y = tSec * 0.008;
    }
  }
}

/**
 * Top-Layer 3D Glam Fitting Stage rendered inside `<dialog id="avatar-dialog">`
 * directly IN FRONT OF `dialog::backdrop`, isolating character geometry/materials
 * in its own transparent WebGL context with zero world or HUD obstructions.
 */
export class GlamSalonStage3D {
  constructor({ container }) {
    this.container = container;
    this.active = false;
    this.rafId = 0;
    this.lastFrameTime = 0;
    this.elapsed = 0;
    this.studioAvatar = null;
    this.currentPose = 'idle';
    this.autoSpin = false;

    this.FRAMING_PRESETS = {
      full: { lookY: 1.44, dist: 4.55, pitch: 0.08 },
      head: { lookY: 2.22, dist: 2.18, pitch: 0.06 },
      torso: { lookY: 1.54, dist: 2.95, pitch: 0.07 },
      shoes: { lookY: 0.72, dist: 2.65, pitch: 0.14 }
    };

    this.currentFraming = 'full';
    this.lookY = 1.44;
    this.targetLookY = 1.44;
    this.dist = 4.55;
    this.targetDist = 4.55;
    this.pitch = 0.08;
    this.targetPitch = 0.08;
    this.aspectDistBoost = 0;

    this.turntableYaw = 0;
    this.targetTurntableYaw = 0;

    this.activePointers = new Map();
    this.lastPinchDist = null;
    this.sparkles = [];
    this.burstRings = [];

    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(44, 1, 0.1, 40);

    this.renderer = null;
    if (this.container) {
      try {
        this.renderer = new THREE.WebGLRenderer({
          alpha: true,
          antialias: true,
          powerPreference: 'high-performance'
        });
        this.renderer.setClearColor(0x000000, 0);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
        this.renderer.outputColorSpace = THREE.SRGBColorSpace;
        this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
        this.renderer.toneMappingExposure = 1.14;
        this.container.prepend(this.renderer.domElement);
      } catch (err) {
        console.warn('[EllaWorld] Could not initialize GlamSalonStage3D WebGLRenderer:', err);
      }
    }

    this.turntableGroup = new THREE.Group();
    this.scene.add(this.turntableGroup);

    this._setupStudioLighting();
    this._buildStarlightPedestal();
    this._bindStageEvents();

    loadDtiWardrobePack().catch(() => {});
    loadPropsAndLootPack()
      .then(() => {
        const acc = this.studioAvatar?.user?.avatar?.accessory;
        if (
          this.studioAvatar &&
          (acc === 'ray_gun' || acc === 'crystal_blade' || acc === 'starlight_wand')
        ) {
          this.studioAvatar.updateAvatarAppearance(this.studioAvatar.user.avatar);
        }
      })
      .catch(() => {});

    this._tick = this._tick.bind(this);
  }

  _setupStudioLighting() {
    const hemi = new THREE.HemisphereLight('#fff1f2', '#312e81', 1.18);
    hemi.position.set(0, 6, 0);
    this.scene.add(hemi);

    const keyLight = new THREE.DirectionalLight('#fffbeb', 1.85);
    keyLight.position.set(2.8, 4.6, 4.4);
    this.scene.add(keyLight);

    const fillLight = new THREE.DirectionalLight('#bae6fd', 0.92);
    fillLight.position.set(-3.2, 3.2, 3.6);
    this.scene.add(fillLight);

    const pinkRim = new THREE.PointLight('#f472b6', 2.4, 9.5, 1.4);
    pinkRim.position.set(-2.1, 2.7, -2.4);
    this.scene.add(pinkRim);

    const cyanRim = new THREE.PointLight('#38bdf8', 2.1, 9.5, 1.4);
    cyanRim.position.set(2.2, 2.5, -2.3);
    this.scene.add(cyanRim);

    const pedestalUpLight = new THREE.PointLight('#fde047', 0.85, 4.5, 1.6);
    pedestalUpLight.position.set(0, 0.28, 1.15);
    this.scene.add(pedestalUpLight);
  }

  _buildStarlightPedestal() {
    this.pedestalGroup = new THREE.Group();

    const basePlinth = new THREE.Mesh(
      new THREE.CylinderGeometry(1.26, 1.42, 0.22, 48),
      new THREE.MeshStandardMaterial({
        color: '#1e1b4b',
        roughness: 0.28,
        metalness: 0.52
      })
    );
    basePlinth.position.y = -0.11;
    this.pedestalGroup.add(basePlinth);

    const mirrorDeck = new THREE.Mesh(
      new THREE.CylinderGeometry(1.18, 1.18, 0.024, 48),
      new THREE.MeshStandardMaterial({
        color: '#312e81',
        emissive: '#4c1d95',
        emissiveIntensity: 0.28,
        roughness: 0.14,
        metalness: 0.65
      })
    );
    mirrorDeck.position.y = -0.01;
    this.pedestalGroup.add(mirrorDeck);

    const goldRim = new THREE.Mesh(
      new THREE.TorusGeometry(1.25, 0.042, 12, 64),
      new THREE.MeshStandardMaterial({
        color: '#fbbf24',
        emissive: '#f59e0b',
        emissiveIntensity: 0.28,
        roughness: 0.2,
        metalness: 0.82
      })
    );
    goldRim.rotation.x = Math.PI / 2;
    goldRim.position.y = -0.01;
    this.pedestalGroup.add(goldRim);

    const innerGlowGeo = new THREE.RingGeometry(0.86, 1.0, 48);
    innerGlowGeo.rotateX(-Math.PI / 2);
    this.innerGlowMat = new THREE.MeshBasicMaterial({
      color: '#f472b6',
      transparent: true,
      opacity: 0.78,
      side: THREE.DoubleSide
    });
    const innerGlowRing = new THREE.Mesh(innerGlowGeo, this.innerGlowMat);
    innerGlowRing.position.y = 0.005;
    this.pedestalGroup.add(innerGlowRing);

    this.pedestalGemsGroup = new THREE.Group();
    const gemGeo = new THREE.OctahedronGeometry(0.055, 0);
    const gemPinkMat = new THREE.MeshStandardMaterial({
      color: '#f472b6',
      emissive: '#ec4899',
      emissiveIntensity: 0.75,
      roughness: 0.15
    });
    const gemCyanMat = new THREE.MeshStandardMaterial({
      color: '#38bdf8',
      emissive: '#0ea5e9',
      emissiveIntensity: 0.75,
      roughness: 0.15
    });
    for (let i = 0; i < 8; i++) {
      const angle = (i / 8) * Math.PI * 2;
      const gem = new THREE.Mesh(gemGeo, i % 2 === 0 ? gemPinkMat : gemCyanMat);
      gem.position.set(Math.cos(angle) * 1.24, 0.055, Math.sin(angle) * 1.24);
      this.pedestalGemsGroup.add(gem);
    }
    this.pedestalGroup.add(this.pedestalGemsGroup);
    this.turntableGroup.add(this.pedestalGroup);

    this.sharedSparkleGeo = new THREE.OctahedronGeometry(0.052, 0);
    this.sharedRingGeo = new THREE.TorusGeometry(0.55, 0.032, 8, 36);
  }

  _bindStageEvents() {
    if (!this.container) return;

    if (typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver(() => this.handleResize());
      this.resizeObserver.observe(this.container);
    }
    window.addEventListener('resize', () => {
      if (this.active) this.handleResize();
    });

    this.container.addEventListener('pointerdown', (e) => {
      if (e.button !== undefined && e.button !== 0) return;
      this.activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
      try {
        this.container.setPointerCapture(e.pointerId);
      } catch {
        // ignore synthetic events
      }
      if (this.activePointers.size === 2) {
        const pts = [...this.activePointers.values()];
        this.lastPinchDist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
      }
    });

    this.container.addEventListener('pointermove', (e) => {
      const prev = this.activePointers.get(e.pointerId);
      if (!prev) return;
      const cur = { x: e.clientX, y: e.clientY };
      this.activePointers.set(e.pointerId, cur);

      if (this.activePointers.size === 1) {
        const dx = cur.x - prev.x;
        const dy = cur.y - prev.y;
        this.targetTurntableYaw += dx * 0.0125;
        this.targetPitch = Math.max(-0.12, Math.min(0.42, this.targetPitch + dy * 0.0055));
      } else if (this.activePointers.size === 2) {
        const pts = [...this.activePointers.values()];
        const dist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
        if (this.lastPinchDist && dist > 8) {
          const delta = (this.lastPinchDist - dist) * 0.014;
          this.targetDist = Math.max(1.65, Math.min(6.2, this.targetDist + delta));
        }
        this.lastPinchDist = dist;
      }
    });

    const releasePointer = (e) => {
      this.activePointers.delete(e.pointerId);
      if (this.activePointers.size < 2) {
        this.lastPinchDist = null;
      }
    };
    this.container.addEventListener('pointerup', releasePointer);
    this.container.addEventListener('pointercancel', releasePointer);

    this.container.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const step = Math.sign(e.deltaY) * 0.28;
        this.targetDist = Math.max(1.65, Math.min(6.2, this.targetDist + step));
      },
      { passive: false }
    );
  }

  handleResize() {
    if (!this.container || !this.renderer) return;
    const width = Math.max(200, this.container.clientWidth || 480);
    const height = Math.max(180, this.container.clientHeight || 520);
    const aspect = width / height;
    this.camera.aspect = aspect;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height, false);
    this.aspectDistBoost = aspect < 0.85 ? (0.85 - aspect) * 1.45 : 0;
  }

  setActive(enabled) {
    const next = Boolean(enabled);
    if (next === this.active) return;
    this.active = next;
    this.activePointers.clear();
    this.lastPinchDist = null;

    if (this.active) {
      this.handleResize();
      this.lastFrameTime = performance.now();
      this.rafId = requestAnimationFrame(this._tick);
    } else if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = 0;
    }
  }

  setCharacter(userProfile, draftAvatar) {
    const safeUser = {
      username: userProfile?.username || 'friend',
      displayName: userProfile?.displayName || 'Friend',
      role: userProfile?.role || 'friend',
      avatar: { ...(draftAvatar || userProfile?.avatar || {}) }
    };

    if (!this.studioAvatar) {
      this.studioAvatar = new WorldAvatar(
        {
          id: 'glam-studio-self',
          user: safeUser,
          x: 0,
          y: 0,
          z: 0,
          ry: 0,
          anim: 'idle',
          emote: this.currentPose === 'idle' ? null : this.currentPose
        },
        { studioMode: true }
      );
      this.turntableGroup.add(this.studioAvatar.group);
    } else {
      this.studioAvatar.user = safeUser;
      this.studioAvatar.updateAvatarAppearance(safeUser.avatar);
      this.studioAvatar.emote = this.currentPose === 'idle' ? null : this.currentPose;
      this.studioAvatar.anim = 'idle';
    }
  }

  setFraming(presetName, snap = false) {
    const preset = this.FRAMING_PRESETS[presetName] || this.FRAMING_PRESETS.full;
    this.currentFraming = this.FRAMING_PRESETS[presetName] ? presetName : 'full';
    this.targetLookY = preset.lookY;
    this.targetDist = preset.dist;
    this.targetPitch = preset.pitch;
    if (snap) {
      this.lookY = preset.lookY;
      this.dist = preset.dist;
      this.pitch = preset.pitch;
    }
  }

  resetTurntable() {
    this.turntableYaw = 0;
    this.targetTurntableYaw = 0;
    this.turntableGroup.rotation.y = 0;
  }

  setAutoSpin(enabled) {
    this.autoSpin = Boolean(enabled);
    return this.autoSpin;
  }

  setPose(poseName) {
    const nextPose = poseName && poseName !== 'idle' ? poseName : 'idle';
    this.currentPose = nextPose;
    if (this.studioAvatar) {
      this.studioAvatar.emote = nextPose === 'idle' ? null : nextPose;
      this.studioAvatar.anim = 'idle';
    }
    return this.currentPose;
  }

  triggerSparkleBurst(hexColor = '#f472b6') {
    const ringMat = new THREE.MeshBasicMaterial({
      color: hexColor,
      transparent: true,
      opacity: 0.9
    });
    const ringMesh = new THREE.Mesh(this.sharedRingGeo, ringMat);
    ringMesh.rotation.x = Math.PI / 2;
    ringMesh.position.set(0, 0.06, 0);
    this.scene.add(ringMesh);
    this.burstRings.push({ mesh: ringMesh, mat: ringMat, age: 0 });

    const palette = [hexColor, '#fde047', '#38bdf8', '#f472b6'];
    for (let i = 0; i < 16; i++) {
      const col = palette[i % palette.length];
      const spMat = new THREE.MeshBasicMaterial({
        color: col,
        transparent: true,
        opacity: 0.95
      });
      const spMesh = new THREE.Mesh(this.sharedSparkleGeo, spMat);
      const angle = (i / 16) * Math.PI * 2 + Math.random() * 0.35;
      const radius = 0.52 + Math.random() * 0.62;
      spMesh.position.set(
        Math.cos(angle) * radius,
        0.18 + Math.random() * 1.45,
        Math.sin(angle) * radius
      );
      this.scene.add(spMesh);
      this.sparkles.push({
        mesh: spMesh,
        mat: spMat,
        vy: 0.85 + Math.random() * 0.95,
        vr: 3.5 + Math.random() * 3.5,
        age: 0,
        life: 0.65 + Math.random() * 0.35
      });
    }
  }

  _tick(now) {
    if (!this.active) return;
    this.rafId = requestAnimationFrame(this._tick);

    const dt = Math.min(0.05, Math.max(0.001, (now - this.lastFrameTime) / 1000));
    this.lastFrameTime = now;
    this.elapsed += dt;

    if (this.autoSpin && this.activePointers.size === 0) {
      this.targetTurntableYaw += dt * 0.68;
    }

    const smooth = Math.min(1, dt * 11.5);
    this.turntableYaw += (this.targetTurntableYaw - this.turntableYaw) * smooth;
    this.lookY += (this.targetLookY - this.lookY) * smooth;
    this.dist += (this.targetDist + this.aspectDistBoost - this.dist) * smooth;
    this.pitch += (this.targetPitch - this.pitch) * smooth;

    this.turntableGroup.rotation.y = this.turntableYaw;

    if (this.pedestalGemsGroup) {
      this.pedestalGemsGroup.rotation.y = -this.elapsed * 0.45;
    }
    if (this.innerGlowMat) {
      this.innerGlowMat.opacity = 0.64 + Math.sin(this.elapsed * 3.4) * 0.16;
    }

    for (let i = this.burstRings.length - 1; i >= 0; i--) {
      const br = this.burstRings[i];
      br.age += dt;
      const t = br.age / 0.48;
      if (t >= 1) {
        this.scene.remove(br.mesh);
        br.mat.dispose();
        this.burstRings.splice(i, 1);
        continue;
      }
      const sc = 1 + t * 2.35;
      br.mesh.scale.set(sc, sc, 1);
      br.mat.opacity = (1 - t) * 0.88;
    }

    for (let i = this.sparkles.length - 1; i >= 0; i--) {
      const sp = this.sparkles[i];
      sp.age += dt;
      const t = sp.age / sp.life;
      if (t >= 1) {
        this.scene.remove(sp.mesh);
        sp.mat.dispose();
        this.sparkles.splice(i, 1);
        continue;
      }
      sp.mesh.position.y += sp.vy * dt;
      sp.mesh.rotation.y += sp.vr * dt;
      sp.mesh.rotation.x += sp.vr * 0.6 * dt;
      const scale = 1 - t * 0.55;
      sp.mesh.scale.setScalar(scale);
      sp.mat.opacity = (1 - t) * 0.95;
    }

    if (this.studioAvatar) {
      this.studioAvatar.update(dt, true);
    }

    const cosP = Math.cos(this.pitch);
    const camY = Math.max(0.28, this.lookY + Math.sin(this.pitch) * this.dist);
    const camZ = cosP * this.dist;
    this.camera.position.set(0, camY, camZ);
    this.camera.lookAt(0, this.lookY, 0);

    if (this.renderer) {
      this.renderer.render(this.scene, this.camera);
    }
  }
}
