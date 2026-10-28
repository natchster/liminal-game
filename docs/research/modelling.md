# Modelling: procedural creature meshes that look real, from CC0 inputs

Scope: free inputs (CC0 or public domain only) for our Blender-script creature pipeline, and techniques that fit our renderer. The renderer does Lambert/Phong, a rest-pose triplanar bump ("skin detail"), emissive, alpha, fog, a place-based light map and a flashlight. It has no normal maps, and creatures have no UVs. Licences were checked at the source on 28 Oct 2026. Where I could, I verified claims by reading the licence files, querying the Poly Haven, ambientCG and Wikimedia Commons APIs, and opening assets in the installed Blender 5.2.2 LTS headless with `--factory-startup --disable-autoexec`. The downloads and test scripts were working files and are not kept.

## 0. Short answer

- **Two CC0 human bases cover almost everything.** The first is **Blender's Human Base Meshes v1.4.1**: a realistic body sculpt plus a complete, bone-by-bone realistic skeleton and skull. The second is **MakeHuman/MPFB2**: parametric targets (minimum weight, minimum muscle, old age), UVs, CC0 photographic skins, and a 53-bone game rig with CC0 weights. Both are verified CC0 below.
- **Poly Haven and ambientCG are clean CC0 texture sources.** ShareTextures is not: its "custom CC0" forbids redistribution in collections. Quaternius changed to a non-CC0 licence in August 2026.
- **No CC0 library has human skin photos except MakeHuman's skin packs.** Pores are better left to our procedural rest-pose bump. Vertex colour can only hold features of about 3 cm or larger at our triangle budget.
- **Under a flashlight held at the eye, relief casts almost no visible shadow.** Ribs, sockets and tendons have to be sold by three things: silhouette, baked cavity in vertex colour, and specular glints on convex bumps. Geometry alone won't do it.
- **Headless cloth simulation works in Blender 5.2** (tested). Draped sheet poses can be stored as morph targets.
- **Eyeshine is retroreflection.** Drive it from the flashlight's illuminance at the eye, gated by eye facing. Never make it a constant glow.

---

## 1. Base meshes

### 1.1 MakeHuman / MPFB2: CC0 since September 2020

**Licence.**
- The project licence page says: "All core assets are shared under Creative Commons, CC0." It adds that no payment, copyright notice or attribution is required ([makehumancommunity.org/about/license](https://static.makehumancommunity.org/about/license.html)).
- The 2020 FAQ explains that assets were AGPL before September 2020. Since then, "Targets, proxies and the base mesh are considered graphical assets, covered by CC0" ([2020 licence FAQ](https://static.makehumancommunity.org/oldsite/faq/what_changed_regarding_the_license_in_2020.html)).
- MPFB2's `LICENSE.md` lists "The base mesh and proxies / Targets and modifiers / Textures / Clothes / Rigs, poses and expressions / JSON data with mesh information". It says "These assets have been released under CC0 1.0 Universal". The add-on's code is GPLv3, but we would use only the data ([LICENSE.md](https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.md)).
- The files repeat it. `base.obj` starts with "This asset was explicitly released as CC0 in september 2020". `weights.game_engine.json` has `"license": "CC0"`. I read both from the repo.

**Where the raw files are (no GUI needed).** The [mpfb2 GitHub repo](https://github.com/makehumancommunity/mpfb2) keeps the data under `src/mpfb/data/`:
- `3dobjs/base.obj`: a text OBJ, 1.7 MB ([raw link](https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/src/mpfb/data/3dobjs/base.obj)).
- `targets/<region>/*.target.gz`: gzipped text, one line per moved vertex in the form `index dx dy dz`.
- `rigs/standard/rig.*.json` and `weights.*.json`: the default, game_engine, mixamo, cmu_mb and openpose rigs.
- `uv_layers/` and a few small textures.

Skins, proxies, eyes, eyebrows, teeth and hair are in the "MakeHuman system assets" pack (267 MB, every item marked CC0) ([pack page](https://static.makehumancommunity.org/assets/assetpacks/makehuman_system_assets.html)). The zip is at `https://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip`, and it answered HTTP 200 today.

**Measured geometry.**
- `base.obj` has 19,158 vertices and 18,486 quads.
- The `body` group has 13,378 quads, about 26.8k triangles.
- The rest is helper geometry that must be deleted: `helper-tights` (2,650 quads), skirt, hair, eyelashes, teeth, tongue, and the `joint-*` marker cubes.
- It has UVs (21,334 `vt`). It is Y-up, in decimetre units, in an A-pose; the bounding box is 16.9 × 9.9 units.

**Targets that matter for starved bodies** (listed through the GitHub API):
- `macrodetails/universal-{male,female}-{young,old}-minmuscle-minweight` (there are 99 macro files).
- Per-limb `*-fat-decr` and `*-muscle-decr`.
- `stomach-pregnant-decr`, for a concave belly.
- In `torso/`: `torso-scale-depth-decr`, `measure-waist-circ-decr`, `torso-muscle-pectoral-decr`.
- In `neck/`: `neck-scale-horiz-decr` and `measure-neck-circ-decr`.

A target is just per-vertex offsets, so applying one in our own script takes about 10 lines of Python.

**Rigging.** Yes. `rig.game_engine.json` has 53 bones (Root, clavicle_l, calf_l, …) and comes with CC0 weights. It is MakeHuman's skeleton, not our `humanRig()`, so the rig has to be fitted (see Implications).

**Watch-outs.**
- **Mixed licences in the pack list.** The asset-pack page mixes CC0 and CC-BY packs: Animal 02–04 targets, Hair 02/03, Shirts 02/03 and others are CC-BY ([asset packs](https://static.makehumancommunity.org/assets/assetpacks.html)). Only "_cc0" packs are usable.
- **Partial shutdown.** On 21 Oct 2026 the forum and the old asset repository were taken down as a precaution. The news item says the static site with the asset packs and "the github repos are still online and will remain so" ([news](https://static.makehumancommunity.org/)). Mirror what we need now.

### 1.2 Blender "Human Base Meshes" v1.4.1: CC0 (verified inside the file)

**Licence.** Blender's demo-files page lists "Human Base Meshes v1.4.1 … 49 MB – CC0", updated 20 Jan 2025, requiring Blender 4.2+ ([blender.org/download/demo-files](https://www.blender.org/download/demo-files/)). The zip comes from `https://mirror.blender.org/demo/asset-bundles/human-base-meshes/human-base-meshes-bundle-v1.4.1.zip`. Inside the `.blend`, every asset's metadata says `license='CC0'`. The authors are Dan Ulrich, Julien Kaspar, Paul Kotelevets and Tonatiuh de San Julián. The zip contains no separate licence file.

**What's inside** (measured in Blender 5.2):

| Asset | Base mesh | Notes |
|---|---|---|
| Body Male/Female – Realistic | 10,590 quads (21,160 tris) | Multires level 3 sculpt, face sets, 1 UV map, separate eyes (1,088 tris each) |
| Body Male/Female – Stylized | 12,500 quads | not useful for us |
| Head (animation topology) – Realistic | 3,234 quads | iris and sclera as separate objects |
| Hand / Foot / Jaw – Realistic | 1,658 / 2,990 / 10,472 tris | the jaw includes teeth |
| Skull – Realistic | 11,634 quads | Multires level 3 |
| **Skeleton – Realistic** (Paul Kotelevets) | one object per bone, in a parent hierarchy | ribcage 2,084 quads; clavicles, scapulae, every vertebra, pelvis, sacrum, long bones, hand and foot bones; Subdivision modifiers |

- It is not rigged (there is no armature).
- The skeleton is 1.76 m tall and the male body 1.68 m, and the two are placed apart. They must be fitted by script.
- The release notes describe the bundle as quad topology with watertight volumes for voxel remeshing, face sets and UV maps ([3.6 release notes](https://developer.blender.org/docs/release_notes/3.6/asset_bundles/)).

### 1.3 Other sources, and what we can't use

- **Poly Haven models.** The licence page says "Our assets are all licensed as CC0" ([polyhaven.com/license](https://polyhaven.com/license)). Polycounts below are from the [Poly Haven API](https://api.polyhaven.com/assets?t=models).
  - [Old Gas Mask](https://polyhaven.com/a/old_gas_mask), polycount 18,900: a vintage mask with a cracked rubber hood, round lenses, a corrugated hose and a ribbed green filter. Close to a GP-5 with a hose.
  - [Rubber Boots](https://polyhaven.com/a/rubber_boots), 35,004.
  - [Street Rat](https://polyhaven.com/a/street_rat), 14,286, realistic.
  - There are no human figures.
- **Quaternius: licence changed.** The packs were CC0 for years, and OpenGameArt copies still say CC0. For example, Universal Animation Library 2 (130+ humanoid animations, posted Jan 2026) is listed as "CC0" ([OGA](https://opengameart.org/content/universal-animation-library-2)). But quaternius.com now publishes the "Quaternius Asset License (QAL) v1.0", last updated 8/28/2026. Section 3(a) forbids redistributing "the Assets" as a standalone asset or pack ([quaternius.com/license](https://quaternius.com/license.html)). Use only copies obtained under CC0, keep a dated snapshot, and remember the style is low-poly anyway.
- **Kenney.** "all game assets on the asset pages are public domain licensed (CC0)" ([kenney.nl/support](https://kenney.nl/support)). Stylised low-poly; no use for realism.
- **Smithsonian 3D.** "If you see 'CC0' by a 3D model, it is yours to take and remix" ([3d.si.edu](https://3d.si.edu/collections/openaccesshighlights)). The explore listing shows about 3,500 CC0 items ([explore](https://3d.si.edu/explore)), including chimpanzee and gorilla bone scans and a *Homo sapiens* cranium ([listing](https://3d.si.edu/cc0?page=16)). Check each item: a search snippet of one sloth skeleton record shows a CC0 tag next to a "non-commercial" download note ([record](https://3d.si.edu/object/3d/3fc3b13c-af28-4186-8d73-5765469d8190)). The site blocked automated fetches, so I could not open records myself.
- **NIH 3D.** Contributors choose each file's licence, and users "must review and comply with the licensing terms associated with each file" ([terms](https://3d.nih.gov/terms)). Mostly CC BY; low value to us.
- **MorphoSource.** Every download carries a use agreement, and many media are CC BY-NC ([docs](https://duke.atlassian.net/wiki/spaces/MD/pages/35422314), [Florida Museum policy](https://www.floridamuseum.ufl.edu/digital-lab/digital-data-policies/morphosource-digital-data-policy/)). Avoid.
- **Not allowed in the repo** (viewing them as reference is fine):
  - BodyParts3D is CC BY-SA 2.1 Japan ([README](https://dbarchive.biosciencedbc.jp/data/bodyparts3d/20110915/README_e.html)).
  - Z-Anatomy is CC BY-SA 4.0 and built on BodyParts3D ([GitHub](https://github.com/Z-Anatomy/Models-of-human-anatomy)).
  - Blender Studio characters are CC-BY, e.g. the Wing It! dog and cat rigs ([dog](https://studio.blender.org/characters/dog/v1/), [Wing It licensing](https://studio.blender.org/projects/wing-it/pages/licensing/)).
  - Mixamo raw files may not be redistributed ([Adobe FAQ copy](https://community.adobe.com/t5/mixamo-discussions/mixamo-faq-licensing-royalties-ownership-eula-and-tos/m-p/13234775)).

---

## 2. CC0 textures for creatures

### 2.1 Licences

| Source | Licence line | Use? |
|---|---|---|
| ambientCG | "All ambientCG assets are provided under the Creative Commons CC0 1.0 Universal License." ([docs](https://docs.ambientcg.com/license/)) | yes |
| Poly Haven | "Our assets are all licensed as CC0." ([license](https://polyhaven.com/license)) | yes |
| ShareTextures | "Custom CC0 … with specific restrictions": "No asset redistribution on other websites, in plugins, or as part of collections without our written permission"; "CC0 only applies to assets downloaded directly from our site." ([license](https://sharetextures.com/p/license)) | **no**: an open repo would redistribute the files |
| cgbookcase | Reported as CC0 by third parties ([CG Channel 2019](https://www.cgchannel.com/?p=102394)); the home page says only "100% free, no restrictions", and `/license` and `/faq` returned 404 | only after checking the download's own licence |
| MakeHuman skins | each skin listed "CC0" ([skins02](https://static.makehumancommunity.org/assets/assetpacks/skins02.html)) | yes |
| Blender Essentials (hair node groups etc.) | `datafiles/assets/LICENSE` in Blender 5.2 is "CC0 1.0 Universal"; the repo README says all assets are CC0 ([blender-assets](https://projects.blender.org/blender/blender-assets)) | yes |

### 2.2 What exists (API queries, 28 Oct 2026)

- **Human skin.**
  - Neither [ambientCG](https://ambientcg.com/api/v2/full_json?q=skin) nor Poly Haven has human skin. ambientCG's only "skin" hit is crocodile leather.
  - The CC0 photographic skins are MakeHuman's, all UV-mapped to its base mesh. The system assets include young, middle-aged and old variants. The skins02 pack (about 72 MB) has, for example, `jartur69_old_slavic_male…` and `onlytheghosts_old_eurasian_male`.
- **Dust sheet: linen, muslin, cotton.**
  - Poly Haven: [Rough Linen](https://polyhaven.com/a/rough_linen), [Cotton Jersey](https://polyhaven.com/a/cotton_jersey), [Hessian 230](https://polyhaven.com/a/hessian_230) and [Hessian 380](https://polyhaven.com/a/hessian_380) (burlap), [Terry Cloth](https://polyhaven.com/a/terry_cloth).
  - ambientCG: [Fabric001](https://ambientcg.com/view?id=Fabric001) (tagged "old, white"), [Fabric032](https://ambientcg.com/view?id=Fabric032) (white wool cloth), [Fabric019](https://ambientcg.com/view?id=Fabric019) (white woven), [Fabric061](https://ambientcg.com/view?id=Fabric061)–[Fabric066](https://ambientcg.com/view?id=Fabric066) ("rough"), [Fabric034](https://ambientcg.com/view?id=Fabric034) (felt).
- **Rubberised fabric (OZK) and mask rubber.** There is no direct CC0 match. The closest:
  - weave scale: [Scuba Suede](https://polyhaven.com/a/scuba_suede) (neoprene);
  - cracked coated cloth: [Fabric081B](https://ambientcg.com/view?id=Fabric081B) (damaged plastisol);
  - creased, aged surfaces: leathers such as [Brown Leather](https://polyhaven.com/a/brown_leather), [Fabric Leather 01](https://polyhaven.com/a/fabric_leather_01) and ambientCG [Leather014](https://ambientcg.com/view?id=Leather014);
  - ambientCG's "Rubber001–004" are gym floors and not useful.

  OZK is plain grey-green rubberised cloth, so build it procedurally: folds in the geometry (from cloth sim), wear and dust in vertex colour, a faint weave bump, and a satin highlight. For the mask, bake the Poly Haven Old Gas Mask textures.
- **Hair.**
  - MakeHuman system-asset hair (long01, ponytail01, short01–04…).
  - The Hair 01 pack: 25 CC0 styles, 217 MB ([hair01](https://static.makehumancommunity.org/assets/assetpacks/hair01.html)).
  - Blender 5.2's bundled hair node groups (CC0; listed in §3.3).

### 2.3 Turning a texture into per-vertex colour

**Resolution first.**
- About 15k triangles is about 7.5k vertices.
- Adult body surface area is about 1.8–2.1 m² ([Wikipedia: BSA](https://en.wikipedia.org/wiki/Body_surface_area)).
- That gives about 4k vertices per m², or one vertex every 1.5–1.6 cm.
- So vertex colour can carry features of about 3 cm and up: mottling, bruises, large veins, dirt and wet gradients, stains on the sheet. Pores and weave go in the bump.

**Two ways to do it, both scriptable:**
1. **Cycles bake to colour attributes.** In Blender 5.2, `scene.render.bake.target` accepts `'VERTEX_COLORS'` (checked). Bake the Emit pass from an Emission shader that reads the image through UVs. Use "selected to active" to carry colour from the UV'd source (MakeHuman or HBM) onto our voxel-remeshed, UV-less body.
2. **numpy sampling.** For each face corner, look up its UV in a *pre-blurred* copy of the image (Gaussian sigma about half the vertex spacing, in texels), then average per vertex. Without the blur you get speckled aliasing.

**Cleaning photo skins.** They carry some baked lighting. Before baking, high-pass the luminance at a radius of about 10 cm on the body to remove broad shading. Keep the colour in sRGB, as our pipeline expects.

### 2.4 Turning a texture into the triplanar bump

Our engine already samples a 256² tiling height map triplanar at the **rest-pose** position: `skin_detail_tex()` and `Patch::SkinDetail { rest_space: true }` in `native/crates/game/src/entities/mod.rs`. Rest pose is the right choice. Projections anchored in world space or object space swim on deforming meshes; Redshift's "Reference" projection uses rest positions and rest normals for exactly this reason ([Maxon docs](https://help.maxon.net/r3d/maya/en-us/Content/html/Tri+Planar.html.md)).

To use a CC0 material:
- Take its Displacement map, or for a photo without one, a high-passed luminance.
- Store it linear (no colour space), downsample to 256–512², and keep it seamless.
- Set `u_detail_scale` so one tile matches the material's real size (Poly Haven's API gives each texture's physical dimensions).
- A sheet and an OZK suit each need their own tile: weave or rubber crinkle instead of pores.

The tangent-free maths behind height bump is Mikkelsen's surface-gradient method, which needs "only … the surface position, the height value and the original normal" and treats triplanar as a volume bump ([paper PDF](https://docs.unrealengine.com/4.27/Attachments/RenderingAndGraphics/Materials/BumpMappingWithoutTangentSpace/mm_sfgrad_bump.pdf)).

---

## 3. Techniques

### 3.1 Emaciation and anatomy reads (scripted, no sculpting)

**What reads as starved.** Loss of muscle and of the fat under the skin makes "the ribs and joints protrude" ([Wikipedia: Marasmus](https://en.wikipedia.org/wiki/Marasmus)). Other signs: sunken eyes, loose wrinkled skin, an "old man" face from loss of facial fat ([summary](https://www.geeksforgeeks.org/?p=1167492)). Landmarks to target:
- clavicles and the notch above the sternum;
- the sternocleidomastoid cords;
- the lower and side ribs;
- the scapular spine and the inner edge of the shoulder blade;
- the vertebral bumps along the spine;
- the hip crests;
- the zygomatic arches and hollow temples;
- the orbit rims;
- knees, elbows and wrists that look too big for the limbs.

**Steps:**
1. **Base.** Either the MakeHuman base with the min-weight/min-muscle/old macros and the per-part `*-decr` targets, or the HBM realistic body with its Multires applied at level 1–2.
2. **Skeleton inside.** Each HBM skeleton bone is its own object. Script it onto our rig's rest frames: humerus to upper arm, radius and ulna to forearm, femur to thigh, and so on. Scale each along the bone length. Fit the ribcage, pelvis and skull by landmarks such as sternum top, iliac crests and orbit centres.
3. **Pull skin onto bone where bone sits just under the skin.**
   - Use a Shrinkwrap modifier: `NEAREST_SURFACEPOINT`, mode `OUTSIDE_SURFACE`, offset equal to skin thickness (2–5 mm).
   - Drive it with a vertex group: a smoothstep of distance to bone, times a region mask over the landmarks above. Both enums exist in 5.2.
   - Where vertices fall between ribs they sink and the grooves appear. That needs about 1 cm spacing over the ribcage.
4. **Cords and tendons.** Add our existing `tube`/`ell` primitives (neck cords, Achilles, hamstrings, forearm flexors) and fuse everything with the existing voxel remesh in `fuse()`.
5. **Loose skin and lumps.** A Displace modifier with a seeded Blender noise texture, applied in rest pose, keeps the result deterministic.
6. **Spend the triangles where the anatomy is.**
   - Voxel remesh at 5–7 mm.
   - Then Decimate `COLLAPSE` using its vertex-group factor, so the ribcage, face and hands stay dense and the thighs and back go sparse.
   - Target 12–15k triangles.
7. **Bake the anatomy into colour.** Our flashlight sits at the camera, so the light comes from the view direction and the bumps cast almost no visible shadows. Relief shows only as N·L darkening at grazing angles. So bake cavity (we already compute `concavity()`) into the albedo: darker grooves between ribs, temples, eye sockets, the suprasternal notch. Under the overhead fluorescent light map the geometry relief reads on its own.

**Metaballs** are fine for blobby masses, but our ellipsoid-plus-tube voxel union is more controllable and already exists.

### 3.2 Cloth draping headless (the dust sheet, the OZK suit)

**Tested in Blender 5.2.2, background mode:**
- Setup: a 65×65 grid (4,225 vertices), cloth modifier with quality 8, mass 0.15, bending stiffness 0.05, self-collision on, collision distance 6 mm. It fell from 1.9 m onto a head sphere plus torso cylinder, each with a Collision modifier.
- 90 frames took **5.2 s**.
- The settled sheet's top sat at 1.76 m, just above the head.

The recipe is to step frames, then take the evaluated mesh; no bake operator is needed:

```python
for f in range(1, 91): scene.frame_set(f)
me = bpy.data.meshes.new_from_object(sheet.evaluated_get(bpy.context.evaluated_depsgraph_get()))
```

**Settings and notes:**
- A cache must be cleared if the cloth is edited after a sim. A bake is stored per subdivision level ([manual: cache](https://docs.blender.org/manual/en/dev/physics/cloth/settings/cache.html)).
- Use high air damping for a still drape, and give the sheet a 1–2 mm Solidify so the flashlight never shows its back face.

**The Watcher.**
- It only moves when unseen, so skip skinning the sheet.
- Re-simulate the *same* flat sheet over 3–5 body poses: standing, head turned, leaning, one arm raised.
- Cloth keeps the topology, so each result is a **morph target** of one mesh, and the renderer already has morph targets.
- Switch poses while unseen. A head turn that happened while you looked away is then a visible fact in the drape.

**OZK suit.**
- Make the suit an offset shell of the body (+1.5–3 cm, voxel remesh).
- Get folds at elbows, knees, crotch and waist either from a short cloth run with negative shrink against a posed body, or from our `ridge()` fold helper placed by bone.
- Hood and mask: decimate the Poly Haven gas mask to about 3–4k triangles and bake its albedo to vertex colour.
- The lenses get a high specular and a dark interior, with no emissive.

**New option.** Blender 5.2 bundles CC0 geometry-node groups "Cloth Dynamics (Experimental)" and "Hair Dynamics" (`geometry_nodes_dynamics_assets.blend`, checked). They are experimental, so prefer the classic modifier.

### 3.3 Hair cards (the Hound's wet hair)

A hair card is "a flat rectangle carrying a hair-strand texture" ([Epic](https://dev.epicgames.com/documentation/metahuman/hair-card-generator-in-dataflow-in-unreal-engine)).

**Building them:**
- Blender 5.2's bundled CC0 node groups include Generate, Clump, Curl, Frizz, Shrinkwrap and Trim Hair Curves (listed from `procedural_hair_node_assets.blend`).
- Make about 40–80 clump guides. Wet hair clumps into ropes: few, heavy, straight, hanging over the face.
- Convert each guide to a ribbon (Curve to Mesh with a two-point line profile, 1.5–3 cm wide, 8–12 segments). That is about 1.5–3k triangles in all.
- Skin the cards to the head and neck bones.

**Without UVs.**
- Store an "across" coordinate (0–1) and an "along" coordinate in a second vertex attribute.
- Compute strand alpha in the fragment shader: hashed thin stripes that thin out toward the tip.
- Use an alpha test with dithering to avoid sorting.
- The along-card direction is also the tangent a Kajiya-Kay-style highlight needs.

**Wet look:** darker albedo, a tight specular. MakeHuman's CC0 hair meshes can serve as starting shapes.

### 3.4 Eyeshine

**Physics.**
- The tapetum lucidum is a reflective layer behind the retina. It sends light back through the retina along the path it came in ([Wikipedia](https://en.wikipedia.org/wiki/Tapetum_lucidum)).
- It is seen when the light source is roughly coaxial with the viewer ([EPFL graph summary](https://graphsearch.epfl.ch/en/concept/276083)). Spotters hold the light at eye level for this reason ([Gizmodo](https://gizmodo.com/this-is-how-to-find-the-spiders-that-are-staring-at-you-1721584332)).
- Our flashlight is at the camera, so the condition holds whenever the beam lands on an eye.

**Colours** ([Wikipedia](https://en.wikipedia.org/wiki/Tapetum_lucidum), [Walking Mountains](https://www.walkingmountains.org/curious-nature/eye-shine-color-animals/)):

| Animal | Eyeshine |
|---|---|
| Dogs | yellow-green adults, blue puppies; "generally green" |
| Cats | blue when young, then yellow to green, sometimes orange to red |
| Deer, elk | white |
| Horses | blue |
| Moose, rabbits | red |
| Bears, mountain lions | yellow to red |

Colour shifts with angle, age and light colour.

**Humans have no tapetum.** Their eyes show only the weak red reflex of red-eye ([Wikipedia](https://en.wikipedia.org/wiki/Red-eye_effect)). So a Hound, a starved *person*, would honestly show a dim red-orange pupil, while green or white shine says "no longer human". Choose deliberately per creature.

**Rendering:**

`shine = gain × E_flash(eye) × smoothstep(0.6, 0.9, dot(eyeForward, toCamera)) × pupil`

- `E_flash` is the flashlight illuminance the shader already computes (cone × 1/d² × battery).
- The eye's shrinking screen area supplies the second distance falloff.
- Put the shine on the pupil disc only, never the whole eyeball.
- It is zero when the lamp is off or points elsewhere, which keeps the canon of "never glow like lamps".
- Clamp the eye to at least 1–2 px at engagement range (or use a billboard) so it doesn't flicker sub-pixel.
- Fog attenuates it like anything else; the high intensity is what keeps it visible past the body.

### 3.5 Wet or oily skin highlights without normal maps

**Facts.**
- Skin specular is a dielectric with "F0 = 0.028", evaluated with the half vector. Real faces vary specular roughness and intensity by region ([d'Eon & Luebke, GPU Gems 3 ch. 14](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-14-advanced-techniques-realistic-real-time-skin)).
- Measured faces vary with "sweat, cold, or makeup" ([Weyrich et al. 2006](https://graphics.ucsd.edu/~henrik/papers/skin-analysis/)).
- Rough surfaces get darker when wet because the liquid film increases absorption ([Lekner & Dorf, *Why some things are darker when wet*](https://www.wgtn.ac.nz/scps/staff/pdf/darkerwhenwet.pdf)).
- Games adjust the dry BRDF: darker albedo, lower roughness ([fxguide on Lagarde](https://www.fxguide.com/?p=43362)).

**Phong recipe.**
- A per-vertex `wet` value (0–1): paint downward drip streaks, the mouth, eye rims, hair, and the lower body for the poolrooms.
- Albedo: `albedo *= mix(1, 0.65, wet)`.
- Specular strength: `mix(0.03, 0.08, wet)`.
- Shininess: `mix(25, 180, wet)`.
- Add a second, broad, weak lobe, and a Schlick Fresnel on h·v.
- Keep our existing rule from the bump height (wet in the creases, dull on the lumps). It breaks highlights into glints.
- With light coming from the camera, highlights land on convex points that face you: rib crests, shoulders, brow. That is exactly the read a wet, starved body needs.

### 3.6 Subsurface-scattering approximations

- **Wrap lighting.** `max(0,(N·L + w)/(1 + w))`, plus a dark-red tint band near the terminator ([Green, GPU Gems ch. 16](https://developer.nvidia.com/gpugems/gpugems/part-iii-materials/chapter-16-real-time-approximations-subsurface-scattering)). Use w = 0.2–0.35 for living skin and 0.05–0.1 for starved, waxy skin. Use 0 for rubber and cloth.
- **Pre-integrated skin** ([Penner & Borshukov, GPU Pro 2](https://gameenginegems.com/gemsdb/article.php?id=1103); [slides](https://www.slideshare.net/slideshow/penner-preintegrated-skin-rendering-siggraph-2011-advances-in-realtime-rendering-course/13966747)). It indexes scattering by N·L and curvature. Screen-space curvature gives artefacts at triangle edges, so "use curvature maps" ([gamedev.net](https://www.gamedev.net/forums/topic/658069-how-can-i-have-a-softer-skin-using-brdf/5165432/)). We can bake per-vertex curvature in Blender as one float.
- **Thin-part translucency** (ears, fingers, the sheet when a tube lights it from behind). Frostbite's local-thickness approach ([Frostbite](https://frostbite.com/frostbite/news/approximating-translucency-for-a-fast-cheap-and-convincing-subsurface-scattering-look)): bake per-vertex thickness by raycasting inward in Blender, and add light when the light is behind the surface.

### 3.7 Making creatures read at distance

**What others do:**
- TF2 used distinctive silhouettes plus luminance and hue variation so players identify characters quickly in any lighting, and added rim highlights ([Mitchell, Francke, Eng 2007](https://www.cs.princeton.edu/courses/archive/fall07/cos597B/papers/mitchell-team-fortress.pdf)).
- Valve's Left 4 Dead talk places the highest contrast at the area of interest, with a value gradient from dark feet to a light chest ([MIGS 2008 slides](https://cdn.fastly.steamstatic.com/apps/valve/2008/MIGS08_ConnectingVisualsToGameplay.pdf), per a search summary; the PDF text could not be extracted here).
- The silhouette test: fill the model black, and it should still be recognisable from one or two dominant shapes ([Game Tyrant](https://gametyrant.com/news/why-great-video-game-monsters-are-recognizable-at-a-glance)).
- Frictional's Amnesia creature: a humanoid form the player can read, disturbing detail concentrated on the head, and a body kept realistic ([Game Developer](https://www.gamedeveloper.com/art/amnesia-the-dark-descent-birth-of-a-monster-part-1-)).

**For us:**
- **One silhouette idea each.** Smiler: stoop, head forward, overlong arms. Hound: horizontal, with hair hanging. Watcher: a sheeted cone with a head knob. OZK figure: hood, mask snout, hose.
- **A fixed value relation to its surroundings.** The Smiler is darker than any lit wall. The Watcher's sheet is paler than the dark.
- **The eyes are the brightest pixels.**
- **Close-up detail lives on the head and hands.**

---

## 4. Reference imagery (public domain or CC0)

**Anatomy atlases.**
- Gray's Anatomy plates on Commons are tagged public domain ([category](https://commons.wikimedia.org/wiki/Category:Gray%27s_Anatomy_plates)).
- Albinus, *Tabulae sceleti et musculorum* (1747) ([category](https://commons.wikimedia.org/wiki/Category:Tabulae_sceleti_et_musculorum_corporis_humani)). The NLM scan is public domain ([File:Albinus title.jpg](https://commons.wikimedia.org/wiki/File:Albinus_title.jpg)), **but the Wellcome scans of the same plates are tagged CC BY 4.0**.
- Vesalius, *Fabrica* ([category](https://commons.wikimedia.org/wiki/Category:De_humani_corporis_fabrica)): also mostly Wellcome scans tagged CC BY 4.0. NLM's [Historical Anatomies on the Web](https://www.nlm.nih.gov/exhibition/historicalanatomies/home.html) is a US-government alternative.
- Paul Richer, *Artistic Anatomy* (1890; Richer died 1933 ([Wikipedia](https://en.wikipedia.org/wiki/Paul_Richer))): the best artist-oriented atlas of surface landmarks. Plates are excerpted at [Scott Eaton](https://www.scott-eaton.com/excerpts-from-female-anatomy-by-dr-paul-richer).

**Emaciation.** The CDC Public Health Image Library holds public-domain photos of marasmus, such as [PHIL 3991](https://phil.cdc.gov/Details.aspx?pid=3991) and [PHIL 20303](https://phil.cdc.gov/Details.aspx?pid=20303). These are real, mostly children. Use them only as private anatomy reference: never as texture, tracing, marketing or anything in the repo. Prefer the atlases. Stay away from Holocaust and Holodomor imagery entirely.

**OZK and GP-5** (licences read through the Commons API):
- [Противогаз ГП-5.jpg](https://commons.wikimedia.org/wiki/File:%D0%9F%D1%80%D0%BE%D1%82%D0%B8%D0%B2%D0%BE%D0%B3%D0%B0%D0%B7_%D0%93%D0%9F-5.jpg): **CC0**. A GP-5 with the ShM-62u helmet-mask, 3072×4080.
- [Противогаз ГП-5.png](https://commons.wikimedia.org/wiki/File:%D0%9F%D1%80%D0%BE%D1%82%D0%B8%D0%B2%D0%BE%D0%B3%D0%B0%D0%B7_%D0%93%D0%9F-5.png): **CC0**. A white GP-5.
- [Gas mask, 2009.07.17.jpg](https://commons.wikimedia.org/wiki/File:Gas_mask,_2009.07.17.jpg): **CC0**. "Soviet made gas mask in abandoned factory in Olaine, Latvia". Good mood reference.
- [GP-5 Gasmask nva.jpg](https://commons.wikimedia.org/wiki/File:GP-5_Gasmask_nva.jpg): **PD**, released by its author.
- [Máscaras Gas (Rastro).jpg](https://commons.wikimedia.org/wiki/File:M%C3%A1scaras_Gas_(Rastro).jpg): **PD**. A GP-5 beside an M9, an M17 and a British Light Service Respirator.
- [Soldier with NBC-suit.jpg](https://commons.wikimedia.org/wiki/File:Soldier_with_NBC-suit.jpg): **PD** (US Marines). A Soviet-style gas mask and chemical suit.
- [Underground Moscow museum - 2026-05 - p15.jpg](https://commons.wikimedia.org/wiki/File:Underground_Moscow_museum_-_2026-05_-_p15.jpg): **CC0**. An L-1 suit.
- [CBRN Soldiers … Rapid Trident 21 (6854239).jpg](https://commons.wikimedia.org/wiki/File:CBRN_Soldiers_from_the_Armed_Forces_of_Ukraine_take_part_in_Rapid_Trident_21_(6854239).jpg): **PD** (US National Guard). OZK-type kit in use.
- Most other photos in [Category:OZK suits](https://commons.wikimedia.org/wiki/Category:OZK_suits) and [Category:GP-5](https://commons.wikimedia.org/wiki/Category:GP-5) are CC BY-SA or CC BY, e.g. Vitaly Kuzmin's NBC-suit photos. Reference only.

**Soviet civil-defence posters.**
- Commons requires public domain in both Russia and the US.
- In Russia, authors who died before 1952 are free. Anonymous works must have been published before 1943, or under the 70-year rule for 1943–1955.
- The US (URAA) restored works still protected in Russia in 1996 and published after 1930 ([Commons: Russia](https://commons.wikimedia.org/wiki/Commons:Copyright_rules_by_territory/Russia)).
- So post-war civil-defence posters are almost all still copyrighted in the US. Use them as reference for layout and typography, and draw our own.

---

## Implications

### Recommended pipeline for the creature rebuild

**Inputs (vendored once into `assets/third_party/cc0/`, each with a `SOURCE.txt` holding the URL, date and quoted licence line):**
- the MPFB2 `base.obj`, the selected `.target.gz` files, and `rig.game_engine.json` + `weights.game_engine.json`;
- the HBM v1.4.1 `.blend`, or better, OBJ exports of the realistic body, skull and skeleton;
- the Poly Haven gas mask, linen and hessian;
- two or three ambientCG fabrics;
- one or two MakeHuman skins.

**Steps (all bpy, deterministic, outputs committed so the game build never runs a simulation):**
1. **Body.** Load MakeHuman `body` (delete the helpers) and apply targets per creature. Or use the HBM realistic body with Multires level 1. Use the MakeHuman joint cubes and HBM landmarks to pose the mesh onto our `humanRig()` rest pose. Smiler and Hound get their stoop or all-fours in the rest pose so the skinning stays clean.
2. **Skeleton.** Place the HBM skeleton bones on rig bone frames. Shrinkwrap the skin to bone under a landmark mask (§3.1). Add cord tubes. Voxel union at 5–7 mm.
3. **Detail.** Seeded Displace for loose skin. Decimate with a vertex-group factor to 12–15k triangles.
4. **Paint.**
   - Bake MakeHuman skin and any texture colour by selected-to-active into vertex colour (prefiltered).
   - Multiply by baked cavity.
   - Add per-creature tints (Smiler near-black with a grey sheen, Hound grey-yellow and bruised).
   - Write `wet` into a second attribute.
5. **Weights.** Transfer from the posed MakeHuman weights (mapped onto our bone names), or use our existing `weigh()`. Smooth them, and test the extreme poses.
6. **Cloth.** Run the sheet and OZK sims headless, and store per-pose results as morph targets.
7. **Hair.** Cards from hair curves, with an "across/along" attribute and alpha computed in the shader.
8. **Eyes.** A separate small mesh: pupil disc, emissive mask, and a per-creature shine colour and gain.
9. **Shader work in the renderer** (no new texture types needed):
   - wrap diffuse;
   - a wet/oil specular driven by an attribute, with Fresnel;
   - flashlight-driven eyeshine;
   - per-material rest-pose bump tiles (skin, linen, rubber);
   - optionally curvature/thickness attributes for skin scattering and translucency.

**Budgets (triangles):**

| Creature | Body | Extras | Total |
|---|---|---|---|
| Smiler | 13–15k (dense ribcage, face, hands) | eyes 0.4k, teeth 1.5–2k | ≈17k |
| Hound | 12–14k | hair cards 2–3k, eyes 0.4k | ≈17k |
| Watcher | 2–4k (only the feet show) | sheet 10–12k (morph targets ×4) | ≈15k |
| OZK figure (if used) | suit 12–14k | mask 3–4k, hose 0.5k, eyes or lenses 0.3k | ≈18k |

All fit the ≈25k ceiling, leaving room for a 2–3k LOD if profiling asks for one.

### Exact CC0 assets worth fetching

1. MPFB2 base mesh: https://raw.githubusercontent.com/makehumancommunity/mpfb2/master/src/mpfb/data/3dobjs/base.obj. Licence: "This asset was explicitly released as CC0 in september 2020."
2. MPFB2 targets, rigs and weights: https://github.com/makehumancommunity/mpfb2/tree/master/src/mpfb/data. Licence: "These assets have been released under CC0 1.0 Universal" ([LICENSE.md](https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.md)).
3. MakeHuman system assets (skins, proxies, eyes, hair): https://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip. Licence: every item marked "CC0".
4. MakeHuman skins02 (old male skins): https://files.makehumancommunity.org/asset_packs/skins02/skins02_cc0.zip. Licence: each skin "CC0".
5. Blender Human Base Meshes v1.4.1: https://mirror.blender.org/demo/asset-bundles/human-base-meshes/human-base-meshes-bundle-v1.4.1.zip. Licence: "49 MB – CC0" ([demo files](https://www.blender.org/download/demo-files/)), plus `license='CC0'` on every asset inside.
6. Poly Haven: [old_gas_mask](https://polyhaven.com/a/old_gas_mask), [rough_linen](https://polyhaven.com/a/rough_linen), [hessian_230](https://polyhaven.com/a/hessian_230), [cotton_jersey](https://polyhaven.com/a/cotton_jersey), [scuba_suede](https://polyhaven.com/a/scuba_suede). Licence: "Our assets are all licensed as CC0."
7. ambientCG: [Fabric001](https://ambientcg.com/view?id=Fabric001), [Fabric032](https://ambientcg.com/view?id=Fabric032), [Fabric062](https://ambientcg.com/view?id=Fabric062), [Fabric081B](https://ambientcg.com/view?id=Fabric081B), [Leather014](https://ambientcg.com/view?id=Leather014). Licence: "All ambientCG assets are provided under the Creative Commons CC0 1.0 Universal License."
8. Blender Essentials hair and dynamics node groups: bundled with Blender 5.2 (`datafiles/assets/nodes/`). Licence file: "CC0 1.0 Universal". Use them as tools; nothing needs vendoring.

### Risks

- **Licence drift.**
  - Quaternius moved to QAL in August 2026.
  - MakeHuman's community services went partly offline on 21 Oct 2026.
  - ShareTextures attaches redistribution limits to its "CC0".

  Snapshot the licence pages with dates when we vendor, and never take Quaternius or ShareTextures files.
- **Mixed-licence packs.** MakeHuman lists CC-BY packs next to CC0 ones, and Wellcome scans of public-domain atlases are tagged CC BY 4.0. Script a check: for every vendored file, require a CC0 or PD line.
- **Rig mismatch.** Neither base matches `humanRig()`. Fitting and weight transfer are the riskiest engineering. Budget time for test poses: the Hound on all fours and the Smiler's lunge will stretch the shoulders and hips.
- **UV loss.** Voxel remesh drops UVs, so bake colour by selected-to-active *before* the original mesh is thrown away. Unfiltered vertex sampling aliases.
- **Relief disappears under a coaxial flashlight.** Anatomy must be in silhouette, baked cavity and specular glints. Otherwise expensive geometry will look like a smooth mannequin in play. Judge every creature in-engine under the flashlight, not in Blender.
- **Sub-pixel eyes flicker, and eyeshine can turn into "glowing eyes".** Tie it strictly to flashlight illuminance and facing.
- **Determinism.**
  - Cloth and decimation results can change between Blender versions. Pin 5.2.2 LTS and commit the generated meshes.
  - Seed every noise.
  - Never run simulations at game build or load time.
- **Photo skins.** They carry baked light, and some carry tattoos or make-up. Choose neutral ones and high-pass them.
- **Respect.** Emaciation reference must stay anatomical. No real victims' photos in the repo or in marketing, and no tracing.
