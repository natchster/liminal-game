# Brief for entity research (shared by all research agents)

The game: "The Backrooms" (native), a first-person liminal-horror game for macOS
in Rust. One player, a flashlight (a lamp that drains, batteries), stamina,
"sanity" (nerve), Almond Water, paper to mark a path, crouch, run, a "cry out"
verb, hide spots (under beds, in wardrobes) in some places, cells with hermetic
doors on the Building's floors. Levels generated from a seed.

The world (canon):
- Levels 1-3 are "the Building": an endless Soviet-era concrete megastructure
  (inspired by the Russian imageboard myth "Samosbor" / самосбор, 2018): panel-block
  galleries round a void, stone stair halls, dead factory floors. Nothing walks
  there; the danger is the "self-assembly": a siren, then a violet fog rising from
  below; shelter in hermetic cells; "cleaners" known only by their log and red
  marks on doors that did not answer; residents gone behind their doors (sometimes
  something knocks back). A grey thin figure stands far across the void and turns
  away when looked at (after SCP-3333).
- Levels 4+ are "the Backrooms" proper: the Lobby (yellow wallpaper, wet carpet,
  hum), offices, utility halls, electrical station, habitable zone, poolrooms,
  blackout zones; deeper levels rot. A survivor "M." left a trail of folded paper
  and camps; her notebook pages hint, never explain.
- Current roaming creatures: Smilers (black starved stooping thing, eyes + grin in
  the dark, drawn to light, lunge if you turn your back), Hounds (starved people on
  all fours, wet hair over the face, hear footsteps far off, stare-down stops them
  briefly), the Watcher (a figure under a dust sheet that only moves when unseen).
  Anomaly-only: a pale face in an endless stairwell (SCP-087), faceless store staff
  by night (SCP-3008), shadows that are deadly only while you are lit (SCP-1983), a
  faceless seated man who nears when unseen (SCP-970).
- Pillars: 1 emptiness is the threat; 2 familiar but wrong; 3 the sound never stops
  (except where it does); 4 you are never shown everything (threats heard and
  half-seen first; rules learned, never told; no tutorials); 5 dread, then release
  (a pacing director: calm, build, peak, relax).
- Style: "real buildings, photographed badly"; worn materials, weak fluorescent
  light, real darkness; at a distance a creature is a silhouette and its eyes; close
  up the detail is for horror; eyes have eyeshine, never glow like lamps unless canon
  says so. The Building's surfaces are now photographs of real Soviet-era places.
- Creature meshes are built procedurally with Blender Python (bpy) scripts, as
  skinned bodies on a rig, painted with vertex colours; the renderer has skinning,
  morph targets, bump maps, emissive maps, alpha, fog. The whole game must stay
  beatable, fair, and learnable.

The goal of the research: redesign the game's entities from scratch (look and
behaviour) to match the game's updated style and lore (Soviet megastructure, the
samosbor, the yellow rooms, M.'s story) and a much higher quality bar, with more
complex but well-defined, learnable behaviour.

Rules for you:
- Use WebSearch / WebFetch. Cite a URL for every factual claim (as markdown links).
- Note the licence of every source design (CC BY-SA 3.0 for SCP Wiki and the
  Backrooms wikis; copyrighted for commercial games, Trevor Henderson, Slender Man
  (owned), Kane Pixels; public domain for old folklore). We may only take general
  motifs from copyrighted works; CC BY-SA concepts need attribution.
- Do not modify any files outside your own output file. Do not download binaries
  unless needed; if you download anything, put it in its own new empty directory
  under a working folder, and never execute it.
- Quote at most a sentence or two from any source; summarise in your own words.
- Write in plain, concrete English. No marketing tone.
