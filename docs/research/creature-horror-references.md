# Horror, not dolls: references for the Hound's and the Smiler's bodies

*Gathered 2026-11-19 for the creatures' remake ([`../design/ENTITIES.md`](../design/ENTITIES.md)), by a
research agent answering the brief below. Every claim links its source. What only a search excerpt
showed says so, and so does what was computed here or is judgement. Films and games are craft notes
only: we never rebuild their creatures. Not repeated here: the canon, gait and starvation
([`hound-references.md`](hound-references.md)); the uncanny valley, proportions, eyeshine and faces
([`folklore-and-imaging.md`](folklore-and-imaging.md)); the wet recipe, wrap lighting and budgets
([`modelling.md`](modelling.md)).*

**The brief.** The developer finds both creatures "look like a doll, not a horror monster" and
"glossy". What makes a real-time creature read as a monster rather than a mannequin? What do horror
games and films show for a starved, blind person on all fours, and for a very tall, starved figure
gone black as if tarred, its lips dried back off its teeth? What triangles and LODs do games use?

## The answer

The doll comes from one clean surface: one colour, one sheen, mirrored, smooth outline. Every
source below that turned a creature from a toy into a horror did the opposite: grubbier, mottled,
matte where dry and wet only where wet, darker in every fold, asymmetric, nearer to a real person.
Both bodies have a highlight over their whole skin today (below, §4), with living skin's gloss.

## 1. Doll or monster

- **Clean and pale glows.** *The Descent*'s crawlers began with "pure white" skin; "as soon as we
  put the crawlers in a dark environment, they were far too bright, too reflective. They'd look
  almost phosphorescent", so they got "a grubbier, 'underground' look. They needed that filth"; and
  the design "evolved away from" "wide-eyed and creature-like" "to something more human"
  ([American Cinematographer](https://theasc.com/articles/the-descent-creepy-crawlers)).
- **Cartoony, then real; one hue, then many.** Frictional's first Amnesia monster was "too funky";
  Grip asked for "something more realistic, something that you almost could find in real life".
  Bluish skin "made it feel too much
  like an alien", so the palette became "warmer colors with elements of purple, blue and yellow"
  ([Game Developer](https://www.gamedeveloper.com/art/amnesia-the-dark-descent-birth-of-a-monster-part-1-)).
- **Built from the bone out.** Dead Space's remake modelled each creature "starting from the
  skeletal system", then "organs, flesh, tendons, layers of subcutaneous fat and skin"; "somewhat
  plausible" and "strangely grounded" ([EA](https://www.ea.com/seed/news/inside-dead-space-2-new-necromorph-nightmare)).
  RE7 photographed clay rigs and real meat for its monsters ([Kill Screen](https://killscreen.com/capcom-using-photogrammetry-creat);
  [80.lv](https://80.lv/articles/resident-evil-7-the-use-of-photogrammetry-for-vr/)).
- **Gloss is a map, not a number.** Roughness is "the most creative map", where you "tell a visual
  story about the surface" ([Adobe, *PBR Guide* 2](https://www.adobe.com/learn/substance-3d-designer/web/the-pbr-guide-part-2)).
  Epic: "skin has different shading properties in different areas"; a greyscale map should "darken
  the base color and the specular (caused by occlusion)"; and when filtering loses the fine relief
  "the surface appears much more shiny than it should be", so distant mips are made rougher
  ([Creating Human Skin](https://dev.epicgames.com/documentation/unreal-engine/creating-human-skin-in-unreal-engine)).
  A character artist: skin that "looks too wet and waxy" is mostly "the Specular Value", and it
  should be lower still "in crevices" ([80.lv, Alday 2025](https://80.lv/articles/sculpting-texturing-a-broken-orc-bust-in-zbrush-3ds-max-substance-3d-painter)).
- **Mirrored reads as made.** Faces made perfectly symmetric from mirrored halves were rated
  significantly less attractive than the natural ones ([Zaidel & Deblieck 2007](https://dwz.psych.ucla.edu/wp-content/uploads/sites/114/2016/11/ZaidelDeblieck.pdf)).
- **Dry and rough looks flat under a lamp at the eye.** A rough diffuse surface brightens as the
  view nears the light, and a very rough sphere looks like a flat disc, as the full moon does
  ([Oren & Nayar](https://cs.columbia.edu/CAVE/projects/oren/oren.php)); rough materials show "a
  retro-reflective peak" ([Burley 2012, slides](https://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_slides_v2.pdf),
  search excerpt). Judgement: under the flashlight a smooth Lambert-and-gloss body shades round and
  even like a polished doll; ash and dust go flat, pale-edged and dull.
- **An effect seen as an effect is cut.** *The Exorcist*'s spider-walk was left out in 1973 because
  its wires showed, and restored in 2000 with them erased ([Bloody Disgusting](https://bloody-disgusting.com/news/3413649/exorcist-just-recreated-iconic-spider-walk-watch-scene/),
  search excerpts; accounts differ).

## 2. The Hound's references

- **The Descent's crawlers** (above): blind, hunting by sound, "blind malformed eyes", "mottled
  with dirt and slime" ([Fangoria](https://www.fangoria.com/original/revisiting-the-descent/)).
  *Take:* filth and mottling, made more human, not more creature. *Not:* the albino cave-humanoid,
  its slime, its ears.
- **The Clickers** (Naughty Dog): from a "slumped figure", "just covered in fungus, you couldn't
  even see their face anymore"; in the show "we keep them out of focus because it was creepier not
  to see them" ([Naughty Dog](https://naughtydog.com/blog/the_last_of_us_part_i_hbo_tv_show_clickers)).
  *Take:* the face replaced by a mass (for us, matted hair); the head's outline is the wrongness.
  *Not:* the bloom, the clicks.
- **Gollum** (Weta): a starved person on all fours. His skin was modelled on "the paint-on-silicone
  process" of prosthetics, its "translucent silicone base layer" by subsurface scattering
  ([TheOneRing, Weta's SIGGRAPH demo](https://www.theonering.net/torwp/2003/08/06/25471-weta-digital-sfx-details-2/),
  search excerpt). *Take:* paint over a thin translucent base: thin ears and fingers redden against
  light (`modelling.md`'s thickness). *Not:* his face, eyes, or proportions.
- **Mama** (Botet; below): one "tendril" of hair chosen to cover the face in a frame
  ([MPA](https://www.motionpictures.org/2013/01/mommy-issues-making-monsters-with-mama-visual-effects-supervisor-aaron-weintraub/)).
  *Take:* the hair over the face is placed, not left to chance.
- **Not found:** sourced notes on Silent Hill 2's Lying Figure and Mannequin, [REC]'s Medeiros or
  Kane Pixels' entities.

## 3. The Smiler's references

- **Javier Botet in *Mama*:** "very frail and can get his body into all these weird positions";
  monofilament pulled at random made his "arms ... jut in these really strange, unnatural ways";
  upper and lower body shot apart so the spine "could bend at a right angle"; takes sped up and run
  backwards ([MPA](https://www.motionpictures.org/2013/01/mommy-issues-making-monsters-with-mama-visual-effects-supervisor-aaron-weintraub/)).
  *Take:* a real frail body, jerked. *Not:* Mama, her floating hair.
- **Alien's performer:** Bolaji Badejo (208 cm) was cast "due to his height and 'very long legs'"
  ([Wikipedia](https://en.wikipedia.org/wiki/Bolaji_Badejo)). *Take:* long real limbs are what stop
  a figure reading as a man in a suit.
- **SCP-096** (Dr Dan, CC BY-SA 3.0): 2.38 m, "very little muscle mass", arms "approximately 1.5
  meters each", "grossly out of proportion" ([SCP Wiki](https://scp-wiki.wikidot.com/scp-096)).
  Computed: 0.63 of its height against a person's 0.44 (Drillis & Contini, `hound-references.md`),
  43% too long. *Take:* a yardstick: ours stays nearer 10–25% (`folklore-and-imaging.md` P1). *Not:*
  its pallor, its jaw.
- **The Pale Man** (DDT): "stick-thin with hanging skin"; fingers painted black "so they'd look like
  eyelashes" ([Lit Hub, Martí 2022](https://lithub.com/how-iconic-movie-monster-the-pale-man-was-created)).
  *Take:* loose skin where the starved body folds. *Not:* the eyes in the hands.
- **The Babadook:** stop-motion, five frames at a time; "black elements in every frame just to
  create contrast" ([American Cinematographer](https://theasc.com/articles/the-babadook-cinematography)).
  *Take:* a black figure needs something blacker beside it, and lit edges. *Not:* the hat, the coat.
- **Grins.** *Smile*'s were "human performances", with no effects on them
  ([Looper](https://www.looper.com/1036742/smile-director-parker-finn-reveals-the-secrets-to-smiles-ghoulish-grins-exclusive-interview)).
  A felt smile needs the orbicularis oculi (cheeks raised, eyes narrowed); without it a smile is
  false or masks something ([Ekman, Davidson & Friesen 1990](https://ffri.hr/~ibrdar/komunikacija/seminari/Ekman,%201990%20-%20Duchenne%20smile.pdf),
  abstract-level). Mummies show "lips pulled back as if they are screaming"; "rapid desiccation ...
  can fix facial muscles into their tensed state", and such faces are "post-mortem artifacts"
  ([Archaeology, Aufderheide](https://archive.archaeology.org/online/features/screaming_mummy/)).
  Risus sardonicus, tetanus' fixed grin, is a spasm, not a smile
  ([Wikipedia](https://en.wikipedia.org/wiki/Risus_sardonicus)). *Take:* the rictus is the mouth
  alone; the upper face is dead. *Not:* any performed grin.
- **Not found:** sourced craft notes on Kane Pixels' figures, Little Nightmares' Thin Man, Outlast's
  Walrider, the Rake.

## 4. Skin and surface

**Living skin:** UE roughness "around 0.4", Specular 0.35 (measured), white specular, F0 0.028
([Epic](https://dev.epicgames.com/documentation/unreal-engine/creating-human-skin-in-unreal-engine);
[GPU Gems 3, ch. 14](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-14-advanced-techniques-realistic-real-time-skin)).

**To our shininess.** Blinn-Phong's exponent is 2/α² − 2 with α = roughness²
([Karis](http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html)). Our BRDF is that
normalised lobe with Schlick Fresnel (`BRDF_BlinnPhong`, `crates/engine/src/shaders.metal`), so:

| UE roughness | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 |
|---|---|---|---|---|---|---|
| shininess (computed) | 245 | 76 | 30 | 13 | 6 | 3 |

**What the bodies have (read in the code).** Smiler skin shininess 40 (≈0.47), Hound 20 (≈0.55):
living skin. Both use `with_skin_detail`: shine 1, cavity 0, so the shader keeps 0.3–1.15× the
highlight over all the skin and darkens no crease. `with_skin`'s own comment: "a sheen over all of
it read as a doll's vinyl".

**Dark is not black.** Base colour: charcoal 0.02, fresh asphalt 0.02, worn asphalt 0.08, bare
soil 0.13 ([Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/physically-based-materials-in-unreal-engine));
"you should not go under 30-50 sRGB" ([Adobe](https://www.adobe.com/learn/substance-3d-designer/web/the-pbr-guide-part-2)).
Asphalt goes from about 0.05 new to 0.15 aged and soiled; the binder lightens as it oxidises
([Cool Pavements review](https://arxiv.org/pdf/2409.12242), search excerpt).

**Tar.** Bitumen's refractive index is about 1.58 ([JKU](https://research.jku.at/de/publications/investigating-and-correcting-refractive-index-effects-in-bitumen-/))
to 1.64–1.66 (MFA's CAMEO), both search excerpts. Computed: F0 = ((n−1)/(n+1))² ≈ 0.05–0.06, twice
skin's. Fresh tar glints harder than skin; old tar is matte, paler, dusty.

**Dried flesh.** Mummified skin is "hard and leathery", "light to dark brown" to "bright yellow to
dark brownish-to-black", "parchment-like" ([Mileva et al. 2023](https://pmc.ncbi.nlm.nih.gov/articles/PMC10588963),
CC BY); "stretched tightly across anatomical prominences such as the cheekbones, chin, costal
margin, and hips", "darkened, hard, forming a leathery shell" ([Campobasso et al., AAFS 2007](https://www.aafs.org/sites/default/files/media/documents/AAFS-2007-G4.pdf)).

**Teeth.** Retracted lips showed "a remarkable dental attrition" of the upper incisors and calculus
on the lower ([Piombino-Mascali et al. 2022](https://pmc.ncbi.nlm.nih.gov/articles/PMC9799007),
CC BY 4.0). Receding gums bare the root, whose cementum and dentin are yellower than enamel
([Wikipedia](https://en.wikipedia.org/wiki/Gingival_recession); patient sources, weaker).

## 5. Triangles and LODs

**Characters (LOD0, reported).** Killzone Shadow Fall: 40k, 7 LODs of 40k / 20k / 10k / 3.2k / 800
/ 350 / 150; Killzone 3: 10k and 3 LODs ([Guerrilla, slide 54](https://www.slideshare.net/slideshow/killzone-shadow-fall-demo-postmortem/21167452)).
God of War's Kratos 80k, 32k of it the face ([PlayStation Blog](https://blog.playstation.com/2019/12/16/the-polygonal-evolution-of-5-iconic-playstation-characters/)).
Cloud: 110k, then 220k; hair half ([PlayStation Blog](https://blog.playstation.com/?p=390251)).
The Order: 1886: background 12–16k, main 40–60k ([ZBrushCentral](https://www.zbrushcentral.com/t/the-order-1886-team-post/205980?page=4)).
Not found: The Last of Us, RE Engine, Dead Space or Silent Hill 2 counts.

**MetaHuman** (vertices, not triangles): face 24,000 / 12,000 / 6,000 / 2,500 / 1,300 / 560 / 270 /
130, influences 12 → 4; body 30,500 / 7,600 / 3,350 / 1,507, influences 8 → 4; "This doesn't mean
that each MetaHuman will use exactly these numbers" ([Epic](https://dev.epicgames.com/documentation/en-us/metahuman/platform-support-and-lod-specifications-for-metahumans)).

**Practice.** Switch by screen size: Unreal uses "the projected diameter of the bounding sphere",
with hysteresis "to avoid 'flickering'" ([Epic](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/SkeletalMeshLODInfo?application_version=5.3)).
Simplygon "strongly suggest[s]" on-screen size over triangle ratio, each LOD made from the last
([Simplygon](https://simplygon.com/posts/0179d2c5-a440-49d7-850a-0a9a94152d1b)). Ryse cut the face
from 8 influences and 260 joints to 4 and 96, and faded blendshapes after two metres
([fxguide](https://www.fxguide.com/?p=49486)).

**Small triangles.** An edge through a 2×2 quad shades it twice ([Humus](https://humus.name/index.php?ID=228)).
PowerVR, a tile-based GPU like Apple's: avoid "dipping below 32 pixels per primitive"
([Imagination](https://docs.imgtec.com/performance-guides/graphics-recommendations/html/topics/triangle-size.html)).
Apple: "often times the vertex cost is almost entirely hidden by fragment processing"
([WWDC20 10632](https://developer.apple.com/videos/play/wwdc2020/10632)).

**Computed for our camera** (74° vertical; a body of 1.8 m² shows about a quarter of it, half its
triangles facing): pixels per visible triangle ≈ 462,000 / (N·d²) at 1080 lines (×3.4 at 2,000).
At 23k: 20 px at 1 m, 5 at 2 m, 1.3 at 4 m; at 45k: 10 px at 1 m, 2.6 at 2 m. The added
triangles are seen only up close, so they need a chain. The screen-height fraction is D / (1.51·d) for a bounding diameter D.

## 6. Licences

| Source | Licence | Use |
|---|---|---|
| SCP-096 | CC BY-SA 3.0 | numbers only |
| Mileva 2023; Piombino-Mascali 2022 | CC BY | facts, quoted |
| AAFS abstract, papers, docs, talks | their own | facts and numbers only |
| Films, games, their artists' words | copyrighted | craft notes only; no likeness, no tracing |

## 7. What could not be verified

Weyrich's per-region skin roughness (the PDF was too large); Burley's and Cool Pavements' texts;
CAMEO (refused); the spider-walk's reasons; the Polycount wiki; any official TLOU, RE Engine, Silent
Hill 2 or Dead Space counts.

## What this asks of the two bodies

Our pipeline: Blender geometry, vertex colour, one specular colour and shininess per material, the
shared tiling bump's per-pixel multiplier. Numbers in brackets are judgement.

**The Hound**

1. Skin through `with_skin`, not `with_skin_detail`: shine [0.15–0.25], cavity [0.4] (Epic,
   Alday).
2. Skin shininess [6–13] (roughness 0.6–0.7), specular F0 ≈ 0.028 or below (§4).
3. A per-vertex `wet` (`modelling.md`) only at mouth corners, nostrils and the hair's lowest cords;
   zero elsewhere.
4. Bake ambient occlusion and curvature in Blender into the vertex colour: creases [30–50%] darker
   (ribs, blades, armpits, behind knees, between fingers).
5. Mottle at two scales [20–40 cm blotches, 5–10 cm purpura], warm grey-yellow with purple and
   yellow, never one hue (Amnesia); albedo ≥ 0.03.
6. The floor's grime darkest on what faces it: palms, knees, shins, belly, the vest's hem (The
   Descent's "filth").
7. Paint each side on its own: one knee torn, one blade higher, purpura on one forearm, the dragged
   leg thinner, hair cords uneven (Zaidel).
8. Spine knobs, shoulder blades, elbows and knuckles in the silhouette's geometry, since a coaxial
   lamp casts no relief (`modelling.md`).
9. Hair that hides the face in every view, one strand placed across it (Clickers, Mama).
10. Calloused heels of the palms and kneecaps a little smoother [shininess 20–30] than the dusty
    skin: polished by the floor.
11. Ears and fingers thin enough to redden against light (Gollum; `modelling.md`'s thickness).
12. Cotton matte [shininess ≤ 6, specular ≤ 0.02], greasy and slightly shinier only at knees and
    cuffs.
13. LOD0 [40k], spent on hands, feet, spine, ribs and hair; then 20k, 10k, 3k, switching near [45%,
    20%, 8%] of screen height (2.5, 5.6, 14 m for a 1.7 m bound), 10% hysteresis, 4 influences from
    LOD2.

**The Smiler**

1. Base colour near-black brown, not black: linear [0.025–0.05] (sRGB 45–60) on the body, warmer
   on tendons, never below 0.02 (Epic, Adobe).
2. Two surfaces: crust over most [shininess 4–10, F0 0.03]; tar glints [shininess 120–250, F0
   0.05–0.06] only in cracks and where it bends (insides of elbows, knuckles, neck folds), drawn
   by a per-vertex `tar` attribute (§4).
3. Grey-brown dust [albedo 0.06–0.10] on what faces up (shoulders, crown, tops of forearms) and in
   folds, flattening it under the lamp (Oren & Nayar).
4. Cracks across the joints: paler, dusty lips with dark glossy floors; in geometry where long, in
   vertex colour where 3 cm or more (`modelling.md`).
5. Shrinkwrap to the skeleton at cheekbones, chin, costal margin and hips; hollows between the
   ribs; cords at wrist, neck and behind the knees (AAFS).
6. Folds of hanging skin only where the stoop compresses it: belly, insides of elbows (Pale Man).
7. Lips dried back above and below the gum line, uneven, one side higher (Aufderheide).
8. Teeth not white: dull ivory crowns [sRGB 150–175], yellow-brown roots, brown calculus at the
   lower gum line, worn upper incisors, one or two gaps or broken (Piombino-Mascali); gums dried
   to the skin's brown; enamel [shininess 60–120] with only small glints.
9. A dead upper face: no cheek raise, no narrowed lids, no brow (Ekman); deep sockets; the eyes
   only catch the lamp (already ours).
10. Long real limbs, [10–25%] past a person's, not SCP-096's 43% (Badejo, Botet).
11. Asymmetry in the silhouette: one shoulder lower, the head tilted, fingers curled unevenly.
12. Something blacker beside it: the mouth's inside and the deep sockets darker than the crust
    (Babadook).
13. LOD0 [40–50k] on face, teeth, hands, ribs and tendons; then halves and quarters, switching
    near [45%, 20%, 8%] (2.9, 6.6, 17 m for a 2 m bound), with morphs on LOD0–1 only (Ryse).
