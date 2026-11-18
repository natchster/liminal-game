# Realistic plants from open tools, on this Mac

*Gathered 2026-11-18 for the lab ([`../../lab/README.md`](../../lab/README.md)):
which open tools and models make realistic plants (the office's potted
plants, alive, dying and dead; weeds, moss, small trees) as meshes, run on an
M4 Pro with 64 GB, are deterministic from a seed and may be used for a game
that is sold. Gathered by a research agent and checked against the sources
it links; what the lab built from it is at the end.*

## The answer

A **seeded procedural generator**, written here, made of what botany and the
procedural-plant literature already describe: a skeleton per species
(a rosette, arching fronds, a branching crown), stems swept as tubes, leaves
as parametric blades bent by their own weight, and one health parameter that
drives what neglect does to a plant. No generative 3D model fits a build that
must repeat byte for byte and be licensed for sale; they are, at best,
references and blockouts.

## Procedural and botanical generators (no weights)

| Tool | What | Licence | On this Mac |
|---|---|---|---|
| [Infinigen](https://github.com/princeton-vl/infinigen) (Princeton) | seeded factories driving Blender geometry nodes: `small_plants/` (snake plant, spider plant, fern, succulent), `monocot/` (agave, banana, grasses), `leaves/` (broadleaf, ginkgo, maple), deformed and rotten trees, moss and ivy, a plant container | BSD-3 | single objects on ARM ([install](https://infinigen.cs.princeton.edu/docs/latest/Installation.html)); dense meshes to bake and decimate |
| [Sapling Tree Gen](https://extensions.blender.org/add-ons/sapling-tree-gen/) (Weber–Penn) | parametric trees in Blender | GPL-3.0+ (the tool; not, in the agent's reading, what it makes) | yes |
| [L-Py / PlantGL](https://github.com/openalea/lpy) | L-systems to meshes | CeCILL-C | osx-arm64 builds |
| [ez-tree](https://github.com/dgreenheck/ez-tree) | seeded trees in JavaScript for WebGL, exported as GLB | MIT | yes |
| [proctree](https://github.com/jarikomppa/proctree) | a small C++ tree generator | BSD | yes |

The algorithms are free to reimplement: L-systems, phyllotaxis and leaf
construction from *The Algorithmic Beauty of Plants*
([PDF](https://algorithmicbotany.org/papers/abop/abop.pdf)); space
colonization ([Runions et al.](https://algorithmicbotany.org/papers/colonization.egwnp2007.html)).

Research of 2025–2026, none ready for houseplants: NeuraLeaf (ICCV 2025, a
neural parametric leaf; no code licence stated,
[site](https://neuraleaf-yang.github.io)), PlantDreamer (L-systems to
Gaussian splats, a LoRA per species, [paper](https://arxiv.org/html/2505.15528v1)),
TreeStructor (forest point clouds to trees,
[site](https://lewkesy.github.io/treestructor)), 3D-L-plants (scans to
L-system parameters, [paper](https://arxiv.org/abs/2505.22337)), FloraForge
(an LLM writing plant generators; code promised,
[paper](https://arxiv.org/html/2512.11925)).

## Generative 3D models

| Model | Licence | On this Mac | For us |
|---|---|---|---|
| [TRELLIS](https://github.com/microsoft/TRELLIS) / [TRELLIS.2-4B](https://huggingface.co/microsoft/TRELLIS.2-4B) (Microsoft) | MIT, code and weights | officially Linux and CUDA; [trellis-mac](https://github.com/shivampkumar/trellis-mac) on MPS (about 5 min a mesh, 18 GB peak; it pulls in DINOv3, whose licence asks for a notice, and RMBG-2.0, **CC BY-NC**); [trellis2mlx](https://github.com/lyonsno/trellis2mlx) on MLX (21 min, experimental) | references and blockouts only |
| Hunyuan3D 2.1 (Tencent) | excludes the **EU, UK and South Korea**, a licence above 1 M users ([summary](https://www.eachlabs.ai/blog/hunyuan3d-things-to-know)) | an [MLX shape port](https://huggingface.co/AgenticVibes/hy3dmlx-shape-v2.1), 18 min, 687 k faces, not bit-reproducible | excluded |
| Stable Fast 3D / SPAR3D (Stability) | free under $1 M revenue only, outputs included ([card](https://huggingface.co/stabilityai/stable-point-aware-3d)) | experimental MPS | conditional |
| [TripoSR](https://github.com/VAST-AI-Research/TripoSR), [TripoSG](https://github.com/VAST-AI-Research/TripoSG) | MIT | MPS (TripoSR 30–60 s) | thin leaves come out poorly |
| [Step1X-3D](https://huggingface.co/stepfun-ai/Step1X-3D) | Apache-2.0 | CUDA only | no |

They make blobby shells of 200–700 k faces from one view, differ from run to
run and machine to machine, and their licences trap a sold game more often
than not; if one is ever used, what it makes is committed as a source and
never made again in a build.

## Leaves and references (CC0)

- [Poly Haven](https://polyhaven.com/license)'s scanned plants
  (`potted_plant_01/02/04`, `pachira_aquatica_01`, `calathea_orbifolia_01`,
  `anthurium_botany_01`, `fern_02`, `weed_plant_02`, `nettle_plant`,
  `moss_01`, dry branches; listed by [its API](https://api.polyhaven.com/assets?type=models)):
  the references a silhouette is checked against, or leaf-texture donors.
- [ambientCG](https://docs.ambientcg.com/license/)'s alpha leaf atlases
  (LeafSet001–030; [LeafSet030](https://ambientcg.com/a/LeafSet030) is dried
  brown leaves, LeafSet029 ivy).

## What neglect does to a plant (what the generator draws)

The health parameter (1 tended, about 0.5 left a season, 0 left for years)
moves what a botanist looks for: turgor lost (leaves and fronds droop),
drying (blades fold along the midrib and their margins curl), colour from
the tips and margins inward (chlorosis, a yellow line, then necrosis, papery
brown, then straw), the oldest leaves first (outer fronds, inner and shaded
leaves), leaf drop (the weeping fig's answer to any stress: a ring of yellow
and brown leaves on the carpet), collapse (a snake plant's leaves fold over
at a soft base and lie across the rim), and the pot itself (the soil pale,
cracked and shrunk from the wall; a salt crust on terracotta).

## What the lab built

`lab/viewer/src/plants/`: the snake plant (a banded rosette from two or
three rhizome clumps, *Laurentii* margins on some seeds), the Boston fern
(arching fronds of alternate pinnae shortening to both ends) and the weeping
fig (three braided stems, a weeping crown of drip-tipped leaves in two rows),
each in a plastic or terracotta pot, each a pure function of `(seed,
health)`: a SplitMix64 stream forked per leaf, the engine's correctly rounded
`dmath` for every angle, so a seed is the same plant on every Mac (its tests:
the same plant twice, budgets, colours in range, neglect visible in the mean
leaf colour). 3–29 k triangles a plant. In the agent's order of what to
build first (snake plant, fern, fig), then what is left: a pothos trailing
from a shelf, weeds through the carpet, moss, CC0 leaf atlases instead of
vertex colour, and a blind review against Poly Haven's scans.
