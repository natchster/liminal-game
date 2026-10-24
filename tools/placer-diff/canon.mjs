// Canonical text of what a placer built. Must stay byte-identical with
// placers/difftest/src/lib.rs (q, geo_s, mat_s, obj_s, pp_s, inter_s, player_s, hash).

export const q = (v) => {
  v = typeof v === 'number' ? v : Number(v);
  if (Number.isNaN(v)) return 'NaN';
  if (!Number.isFinite(v)) return v > 0 ? 'Inf' : '-Inf';
  return '~' + String(Math.floor(v * 1e5 + 0.5));
};
const I = (v) => String(Math.floor(v));
const b = (v) => (v ? 1 : 0);
const hex = (c) => (c && c.getHex ? c.getHex().toString(16).padStart(6, '0') : '-');
export const esc = (s) => String(s).replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n');

const GEO = {
  BoxGeometry: ['Box', (p) => [q(p.width), q(p.height), q(p.depth), I(p.widthSegments), I(p.heightSegments), I(p.depthSegments)]],
  CylinderGeometry: ['Cylinder', (p) => [q(p.radiusTop), q(p.radiusBottom), q(p.height), I(p.radialSegments), I(p.heightSegments), b(p.openEnded), q(p.thetaStart), q(p.thetaLength)]],
  ConeGeometry: ['Cone', (p) => [q(p.radius), q(p.height), I(p.radialSegments), I(p.heightSegments), b(p.openEnded), q(p.thetaStart), q(p.thetaLength)]],
  SphereGeometry: ['Sphere', (p) => [q(p.radius), I(p.widthSegments), I(p.heightSegments), q(p.phiStart), q(p.phiLength), q(p.thetaStart), q(p.thetaLength)]],
  PlaneGeometry: ['Plane', (p) => [q(p.width), q(p.height), I(p.widthSegments), I(p.heightSegments)]],
  CircleGeometry: ['Circle', (p) => [q(p.radius), I(p.segments), q(p.thetaStart), q(p.thetaLength)]],
  TorusGeometry: ['Torus', (p) => [q(p.radius), q(p.tube), I(p.radialSegments), I(p.tubularSegments), q(p.arc)]],
  CapsuleGeometry: ['Capsule', (p) => [q(p.radius), q(p.length), I(p.capSegments), I(p.radialSegments)]],
  OctahedronGeometry: ['Octahedron', (p) => [q(p.radius), I(p.detail)]],
  IcosahedronGeometry: ['Icosahedron', (p) => [q(p.radius), I(p.detail)]],
  DodecahedronGeometry: ['Dodecahedron', (p) => [q(p.radius), I(p.detail)]],
  TetrahedronGeometry: ['Tetrahedron', (p) => [q(p.radius), I(p.detail)]],
  RingGeometry: ['Ring', (p) => [q(p.innerRadius), q(p.outerRadius), I(p.thetaSegments), I(p.phiSegments), q(p.thetaStart), q(p.thetaLength)]],
  LatheGeometry: ['Lathe', (p) => [I(p.segments), q(p.phiStart), q(p.phiLength)]],
  ShapeGeometry: ['Shape', (p) => [I(p.curveSegments)]],
};
function sumAttr(a) {
  let s = 0, w = 0;
  for (let i = 0; i < a.length; i++) { const v = a[i]; s += v; w += v * ((i % 7) + 1); }
  return [s, w];
}
function geoS(g) {
  const t = GEO[g.type];
  let ty, ps;
  if (t && g.parameters) { ty = t[0]; ps = t[1](g.parameters).join(','); }
  else if (g.type === 'BufferGeometry' || t) { ty = 'Buffer'; ps = ''; }
  else { ty = String(g.type).replace(/Geometry$/, ''); ps = '?'; }
  const pa = g.attributes.position;
  const arr = pa ? pa.array : [];
  const [s, w] = sumAttr(arr);
  let out = `${ty}(${ps}) v${pa ? pa.count : 0} i${g.index ? g.index.count : 0} P${q(s)} X${q(w)}`;
  for (const [tag, nm] of [['N', 'normal'], ['U', 'uv'], ['C', 'color']]) {
    const at = g.attributes[nm];
    out += at ? ` ${tag}${q(sumAttr(at.array)[0])}` : ` ${tag}-`;
  }
  return out + ` gr${g.groups.length}`;
}
const MK = { MeshBasicMaterial: 'Basic', MeshLambertMaterial: 'Lambert', MeshPhongMaterial: 'Phong', MeshStandardMaterial: 'Standard',
  PointsMaterial: 'Points', SpriteMaterial: 'Sprite', LineBasicMaterial: 'Line', MeshDepthMaterial: 'Depth' };
const WR = { 1000: 'R', 1001: 'C', 1002: 'M' };
function matS(m) {
  const kind = MK[m.type] || m.type;
  const hasEm = kind === 'Lambert' || kind === 'Phong' || kind === 'Standard';
  let s = `${kind} c${hex(m.color)}`;
  if (hasEm) s += ` e${hex(m.emissive)} ei${q(m.emissiveIntensity)}`;
  s += ` o${q(m.opacity)} t${b(m.transparent)} s${m.side} dw${b(m.depthWrite)} dt${b(m.depthTest)} b${m.blending} at${q(m.alphaTest)} vc${b(m.vertexColors)} fog${b(m.fog)} vis${b(m.visible)}`;
  if (m.polygonOffset) s += ` po${q(m.polygonOffsetFactor)},${q(m.polygonOffsetUnits)}`;
  const t = m.map;
  s += t ? ` map${q(t.repeat.x)},${q(t.repeat.y)},${q(t.offset.x)},${q(t.offset.y)},${q(t.rotation)},${WR[t.wrapS]}${WR[t.wrapT]}` : ' map-';
  if (kind === 'Basic' || hasEm) s += ` am${b(m.alphaMap)}`;
  if (hasEm) s += ` bm${b(m.bumpMap)} em${b(m.emissiveMap)}`;
  if (kind === 'Phong') s += ` sp${hex(m.specular)} sh${q(m.shininess)}`;
  if (kind === 'Points') s += ` sz${q(m.size)} sa${b(m.sizeAttenuation)}`;
  if (kind === 'Sprite') s += ` rot${q(m.rotation)}`;
  if (m.wireframe) s += ' wf';
  return s;
}
function kindOf(o) {
  if (o.isInstancedMesh) return 'Inst';
  if (o.isSkinnedMesh) return 'Skin';
  if (o.isMesh) return 'Mesh';
  if (o.isPoints) return 'Points';
  if (o.isLineSegments) return 'LineSeg';
  if (o.isLineLoop) return 'LineLoop';
  if (o.isLine) return 'Line';
  if (o.isSprite) return 'Sprite';
  if (o.isPointLight) return 'PointL';
  if (o.isSpotLight) return 'SpotL';
  if (o.isAmbientLight) return 'AmbL';
  if (o.isHemisphereLight) return 'HemiL';
  if (o.isDirectionalLight) return 'DirL';
  if (o.isBone) return 'Bone';
  if (o.isCamera) return 'Cam';
  if (o.isScene) return 'Scene';
  return 'Group';
}
function objS(o, depth, out) {
  const p = o.position, r = o.rotation, qt = o.quaternion, s = o.scale;
  const kind = kindOf(o);
  let line = ' '.repeat(depth) + kind;
  if (o.name) line += ' n=' + String(o.name).replace(/ /g, '_');
  line += ` p${q(p.x)},${q(p.y)},${q(p.z)} r${q(r.x)},${q(r.y)},${q(r.z)},${r.order} q${q(qt.x)},${q(qt.y)},${q(qt.z)},${q(qt.w)} s${q(s.x)},${q(s.y)},${q(s.z)}`;
  if (!o.visible) line += ' hidden';
  if (o.renderOrder) line += ` ro${o.renderOrder}`;
  if (!o.frustumCulled) line += ' nofc';
  if (!o.matrixAutoUpdate) line += ' mau0';
  if (kind === 'Mesh' || kind === 'Skin' || kind === 'Inst') {
    line += ' | ' + geoS(o.geometry);
    for (const m of (Array.isArray(o.material) ? o.material : [o.material])) line += ' | ' + matS(m);
    if (kind === 'Inst') {
      const a = o.instanceMatrix.array;
      let sm = 0, wm = 0;
      for (let i = 0; i < o.count * 16 && i < a.length; i++) { sm += a[i]; wm += a[i] * ((i % 7) + 1); }
      line += ` | inst n${o.count} M${q(sm)} W${q(wm)} ic${b(o.instanceColor)}`;
    }
  } else if (kind === 'Points' || kind.startsWith('Line')) {
    line += ' | ' + geoS(o.geometry) + ' | ' + matS(o.material);
  } else if (kind === 'Sprite') {
    line += ' | ' + matS(o.material);
  } else if (o.isLight) {
    line += ` | L c${hex(o.color)} i${q(o.intensity)}`;
    if (kind === 'PointL' || kind === 'SpotL') line += ` d${q(o.distance)} k${q(o.decay)}`;
    if (kind === 'SpotL') line += ` a${q(o.angle)} pe${q(o.penumbra)}`;
    if (kind === 'HemiL') line += ` gc${hex(o.groundColor)}`;
    if ((kind === 'SpotL' || kind === 'DirL') && o.target) { const t = o.target.position; line += ` tgt${q(t.x)},${q(t.y)},${q(t.z)}`; }
  }
  out.push(line);
  for (const c of o.children) objS(c, depth + 1, out);
}
export function sceneS(world, out) { for (const c of world.children) objS(c, 0, out); }
// ppx: [[list, field]] the Rust side keeps in parallel lists; any other
// per-entry key (besides x, y, z) is reported through `unported`.
export function ppS(lists, ppx, out, unported) {
  for (const [name, v] of lists) {
    if (!v || !v.length) continue;
    out.push('PP ' + name + v.map((p) => ` ${q(p.x)},${q(p.y ?? 0)},${q(p.z)}`).join(''));
    if (unported) for (const p of v) {
      if (!p || p.isVector3) continue;
      for (const k of Object.keys(p)) if (k !== 'x' && k !== 'y' && k !== 'z' && !ppx.some(([l, f]) => l === name && f === k)) unported.add(`${name}[i].${k}`);
    }
  }
  const byName = new Map(lists);
  for (const [l, f] of ppx) {
    const vals = (byName.get(l) || []).filter((p) => p && f in p).map((p) => p[f]);
    if (vals.length) out.push(`PPX ${l}.${f}` + vals.map((v) => ' ' + q(v)).join(''));
  }
}
export function interS(inter, out) {
  for (const it of inter) {
    const en = it.enabled ? it.enabled() : true;
    out.push(`I ${q(it.pos.x)},${q(it.pos.y ?? 0)},${q(it.pos.z)} r${q(it.r)} "${esc(it.prompt)}" en${b(en)}`);
  }
}
export function playerS(p, msg, out) {
  out.push(`PL san${q(p.sanity)} bat${q(p.battery)} stam${q(p.stamina)} bottles${p.bottles} batteries${p.batteries} papers${p.papers} boost${q(p.speedBoostUntil)} fl${b(p.flashlightOn)} msg"${esc(msg)}"`);
}
export function hash(s) {
  let a = 0x811c9dc5 | 0, c = (0x01000193 ^ 0x5bd1e995) | 0;
  for (let i = 0; i < s.length; i++) {
    const u = s.charCodeAt(i);
    a ^= u; a = Math.imul(a, 0x01000193);
    c = Math.imul(c ^ u, 0x5bd1e995); c ^= c >>> 13;
  }
  return (a >>> 0).toString(16).padStart(8, '0') + (c >>> 0).toString(16).padStart(8, '0');
}

export function stateS(r1, tiles, lvTiles, out) {
  if (r1) {
    const sp = r1.switchPos, tp = r1.targetPos;
    out.push(`R1 ${q(sp.x)},${q(sp.z)},${sp.tileX},${sp.tileZ} ${q(tp.x)},${q(tp.z)},${tp.tileX},${tp.tileZ} on${b(r1.on)}`);
  }
  const changed = [];
  for (let i = 0; i < tiles.length; i++) if (tiles[i] !== lvTiles[i]) changed.push(`${i}=${tiles[i]}`);
  if (changed.length || tiles.length !== lvTiles.length) out.push(`TILES ${tiles.length} ${changed.join(',')}`);
}
