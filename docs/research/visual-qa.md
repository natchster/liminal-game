# Research: automated visual QA for a first-person game

Gathered on 2026-11-11 by a research agent for the visual playtest
([`../engineering/VISUAL_PLAYTEST.md`](../engineering/VISUAL_PLAYTEST.md)) and kept whole, as the
record of what was found; [`SOURCES.md`](SOURCES.md) section 18 sums
up what the design took from it and where it shows up. Every link is a page
the agent opened; "not found" marks what it searched for and could not
source, and the pages that refused it (Valve's "Leak" wiki, the Springer
z-fighting paper, the UFO:AI wiki, Sea of Thieves' pages) are not cited.
Read a source before relying on one of its details.

2026-11-11. Every link is a page I opened; "not found" means I searched and found no primary source.

## 1. How studios automate visual QA

- **Frostbite (EA).** Screenshot tests use "multiple cameras per level": switch camera, run setup, capture, send to the harness, tear down, repeat. The image score is the maximum per-pixel difference, held to "extremely stringent thresholds". Frostbite keeps per-platform and per-IHV references (32,849 of them), waits on events rather than delays, and has a "deterministic mode". [Frostbite][fb]
- **Unreal.** Screenshot actors placed in the level; a person approves the ground truth. Tolerances cover local and global error, plus "Disable Noisy Rendering Features" and per-hardware alternative references. [UE][ue-ss] Performance tests run static cameras, flythroughs or replays. [UE forum][ue-perf]
- **Unity.** Tests run a fixed number of frames at a fixed timestep against per-platform golden images. Each regression people find becomes a new test. [Aras][aras] Thresholds are per-pixel deltaE, average error, RMSE and the share of incorrect pixels. [Unity][ugtf]
- **Flythrough videos.** A camera "traversing the environment" records video for a person to watch. EA SEED calls this "extremely time consuming", and glitches get overlooked. [SEED][seed]
- **Bots.** Ubisoft's *Division* bots mimic player input and report issues [80.lv][div]; *FF7 Remake*'s replay system "can explore a level on its own" [GDC][ff7]; EA's RL agents raise coverage and find exploits [arXiv][drl] [SEED][cog23]. None says bots capture screenshots for art QA. Activision: not found.

## 2. Z-fighting and flicker

- **Cause and offset.** Round-off gives coplanar polygons different depths, and switching to LEQUAL or EQUAL "won't resolve the problem". Polygon offset pushes the filled primitive back. It can wrongly pass depth tests and distort intersections. [OpenGL FAQ][glfaq]
- **Precision.** The near plane has a "profound" effect on depth precision. The advice: "use a floating-point depth buffer with reversed-Z!" [NVIDIA][nv]
- **Image-based detection.** An "image-based coplanarity detector" built on depth peeling colour-codes the coplanar fragments per pixel (red = 2, blue = 3, and so on). [Vasilakis & Fudos][fudos]
- **Geometric detection.** Brute-force tests of plane proximity plus facing dot products. [Unity forum][ufz]
- **Store requirement.** Meta requires no visible z-fighting, checked by 45 minutes of manual play. [Meta][meta]
- **Rendering twice with a camera jitter and diffing:** not found.

## 3. Holes, cracks, seams and leaks

- **Seeing the void.** The hall-of-mirrors effect appears where nothing overwrites the previous frame. Causes include BSP holes and translucent walls with nothing behind them. [BeyondUnreal][hom] Unity's load/store debug mode highlights undefined (uncleared) pixels. [Unity][uls] A sentinel-colour clear as a documented test: not found.
- **T-junctions.** Mesh repair fixes T-junctions "even if the vertices don't coincide", which "prevents visible gaps". It also removes duplicate overlapping polygons. [Unity Asset Transformer][repair]
- **Watertightness.** trimesh checks that "every edge is included in two faces". [trimesh][tm]
- **Inside or outside.** The generalized winding number is "a perfect segmentation for watertight input and is well-behaved otherwise". [Jacobson][gwn]
- **Light leaks.** Walls should be "no thinner than 10 centimeters", and one-sided meshes seen from behind leak. [UE Lumen][lumen]

## 4. Near-plane clipping and the camera inside geometry

- **Glimpses through walls.** These happen when "the wall geometry penetrates the near face of the view frustum". The fix is to cast the near face, or four padded corner rays, instead of a single ray. [Undersander][cam] Pulling the near plane closer costs depth precision instead. [NVIDIA][nv]
- **VLMs on clipping.** In VideoGameQA-Bench's clipping task the best model scored 87.8%, missing slight overlaps and sometimes hallucinating clipping. [VGQA][vgqa] Sony (2026) found VLMs give "substantial false positives" on near-contact geometry and partial occlusion, and should serve as "high-recall candidate filters". [Sony][sony]
- **A production camera-inside-mesh test:** not found.

## 5. Out-of-place assets

- **Error colours.** Unity draws a magenta error shader for a missing material or failed compile, and cyan while a shader compiles. [Unity][uerr]
- **EA SEED's classifier** (normal, stretched, low-res, missing, placeholder): 86.8% accuracy, 88.1% recall, 6.7% false positives; low-res and stretched hardest. A white placeholder taught it to flag white objects; a patterned one improved every class. Averaging views of an object helps. [SEED][seed]
- **Texel density.** Unreal's Lightmap Density view colour-codes density against an ideal. [UE views][uevm] Its texture-streaming views show red for 2 or more mips missing and green for 2 or more extra. [UE streaming][uestr]
- **Lighting.** Unlit, Lighting Only and Detail Lighting separate albedo from light. [UE views][uevm] Unity's Material Validator flags non-metal albedo outside 50–243 sRGB. [Unity][mv]
- **Style.** "A single highly detailed object in a low-poly scene will stand out for the wrong reasons." [Unity blog][style]
- **Automatic detection of emissive or unlit objects in a lit scene:** not found.

## 6. Triage and VLMs

- **Review tools.** Frostbite's Underling is in-memory, runs at 60 fps, is keyboard-driven and updates in bulk. [Frostbite][fb] Unreal shows ground truth, difference and incoming side by side. [UE][ue-ss] FLIP maps "the difference perceived by humans when alternating between two images". [FLIP][flip] Contact sheets and severity scales: not found.
- **GlitchBench (CVPR 2024).** GPT-4V found 43.4% of glitches and was 91.6% correct on clean images. Extra context frames lowered its accuracy. [GlitchBench][gb]
- **VideoGameQA-Bench (NeurIPS 2025).** Image glitch detection reached 82.8%, but visual regression only 45.2%. Some models fell below 2% specificity, and the authors call autonomous detection "not yet feasible". [VGQA][vgqa]
- **Bug-frame retrieval (EA, University of Alberta).** With the bug report, GPT-4o ranked the bug frame first 89% of the time; without one it classified glitches at 46%. Only 183 of 350 videos were right in all three runs; majority voting helped; bloom and motion blur caused false positives. [arXiv][bfr]

## Rules this suggests

1. **Deterministic captures:** fixed poses, seeds and frame counts; capture on events; references per GPU/driver. [Frostbite][fb] [Aras][aras]
2. **Golden passes without TAA, motion blur or bloom**, at strict per-pixel thresholds. [UE][ue-ss] [Frostbite][fb] [bug-frame retrieval][bfr]
3. **Loud, patterned error and placeholder materials** (never white), found by pixel count. [SEED][seed] [Unity][uerr]
4. **Geometry before pixels:** two-faces-per-edge, duplicate/coplanar faces, T-junctions, wall thickness. [trimesh][tm] [Asset Transformer][repair] [Unity forum][ufz] [UE Lumen][lumen]
5. **Count coplanar fragments per pixel**; fix geometry, keep polygon offset for decals. [Vasilakis & Fudos][fudos] [OpenGL FAQ][glfaq]
6. **Guard the near plane:** reversed-Z float depth, near plane as far out as possible, camera collision covering its corners, an inside test at every pose. [NVIDIA][nv] [Undersander][cam] [Jacobson][gwn]
7. **Debug views per pose:** unlit, lighting-only, texel density. A sentinel clear colour is my inference, not documented practice. [UE views][uevm] [UE streaming][uestr] [BeyondUnreal][hom]
8. **Aggregate several views** of an object or pose before flagging. [SEED][seed]
9. **VLMs as high-recall filters, never the pass/fail gate:** structured output, repeated runs or majority vote, a person on every flag. [Sony][sony] [VGQA][vgqa] [bug-frame retrieval][bfr]
10. **Fast human loop:** reference, diff and flip view, keyboard-driven bulk accept; every human-found regression becomes a fixed-pose test. [Frostbite][fb] [Aras][aras] [FLIP][flip]

[fb]: https://enginearchitecture.org/downloads/Testing_Rendering_Code_at_Frostbite-REAC.pdf
[ue-ss]: https://dev.epicgames.com/documentation/en-us/unreal-engine/screenshot-comparison-tool-in-unreal-engine
[ue-perf]: https://forums.unrealengine.com/t/automated-performance-testing-plugin-guidance/2666136
[aras]: https://aras-p.info/blog/2011/06/17/testing-graphics-code-4-years-later/
[ugtf]: https://docs.unity3d.com/Packages/com.unity.testframework.graphics@8.5/api/UnityEngine.TestTools.Graphics.ImageComparisonSettings.html
[seed]: https://arxiv.org/pdf/2406.08231
[div]: https://80.lv/articles/gdc-using-ai-controlled-players-to-test-the-division/
[ff7]: https://gdcvault.com/play/1027049/-Final-Fantasy-VII-Remake
[drl]: https://arxiv.org/abs/2103.15819
[cog23]: https://www.ea.com/seed/news/cog23-challenges-deploying-rl-agents-game-testing
[glfaq]: https://opengl.org.ru/docs/faq/polygonoffset.htm
[nv]: https://developer.nvidia.com/content/depth-precision-visualized
[fudos]: https://www.cs.uoi.gr/~fudos/coplanarity.pdf
[ufz]: https://discussions.unity.com/t/how-to-check-for-z-fighting-ie-two-planes-directly-on-top-of-each-other/475692
[meta]: https://developers.meta.com/horizon/resources/vrc-pc-performance-8/
[hom]: https://beyondunrealwiki.github.io/pages/hall-of-mirrors.html
[uls]: https://docs.unity.com/en-us/engine/6000.0/script-reference/unityengine/rendering/loadstoreactiondebugmodesettings
[repair]: https://docs.unity.com/en-us/asset-transformer-sdk/2026.4/manual/functions/repairmeshes
[tm]: https://trimesh.org/trimesh.base.html
[gwn]: https://users.cs.utah.edu/~ladislav/jacobson13robust/jacobson13robust.html
[lumen]: https://dev.epicgames.com/documentation/unreal-engine/lumen-technical-details-in-unreal-engine
[cam]: https://gamedeveloper.com/programming/accurate-collision-zoom-for-cameras
[vgqa]: https://arxiv.org/html/2505.15952v2
[sony]: https://www.alphaxiv.org/abs/2607.25921
[uerr]: https://docs.unity.com/en-us/engine/current/manual/materials-and-shaders/shaders/shader-troubleshooting/shader-error
[uevm]: https://dev.epicgames.com/documentation/en-us/unreal-engine/viewport-modes-in-unreal-engine
[uestr]: https://dev.epicgames.com/documentation/en-us/unreal-engine/building-texture-streaming-data-in-unreal-engine
[mv]: https://docs.unity3d.com/2020.1/Documentation/Manual/MaterialValidator.html
[style]: https://unity.com/blog/game-art-styles-guide
[flip]: https://research.nvidia.com/publication/flip
[gb]: https://arxiv.org/html/2312.05291v2
[bfr]: https://arxiv.org/html/2508.04895v1
