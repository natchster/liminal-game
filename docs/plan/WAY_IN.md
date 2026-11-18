# The way in: the plan

The work behind [`../design/WAY_IN.md`](../design/WAY_IN.md): the Jahamun
Tunnel, built apart from the game as a set (`crates/jahamun`), made the
run's way in. Asked for by the developer on 2026-11-18: the entrance and
the set as the game's start; the story of a person who went missing while
researching the tunnel; a cinematic into the tunnel, then your hands, the
tunnel looped without end; after a minute or two of walking the door
changes and leads into the Building's first floor; your own voice telling
it as you walk; the first note pinned by that door; the lore and the notes
to match; done in steps, each committed and pushed, with the performance
and the tests' running time checked.

## Milestones

| Step | What | State |
|---|---|---|
| W1 | The place joins the game's workspace: `crates/jahamun`, its set file and photographs embedded, its pipeline in `assets/src/jahamun/` | done |
| W2 | The canon: the way in (`LORE.md`), her research and the door, your voice on the way in as a channel of its own, E.'s *Day 1* and her first page told again, her note by the door; reviewed by `lore-keeper` | done |
| W3 | The tunnel as the run's start: the mode (`modes/tunnel.rs`), the ring of modules with no end, the door that changes, her note, the door into level 1; New descent from the title; its tests | done |
| W4 | The look you take: `tunnel_in`, from the street above into the walkway, its contract | done |
| W5 | Your voice: the persona `you_tunnel`, its lines made and judged, played in order and at what you pass | done |
| W6 | The place's sound: the bars' hum, the road behind the glass going quiet, steps in its reverb, the wind behind the door | done |
| W7 | Performance (`--bench` of the way in, the game's bench) and the tests' running time, before and after | |

## Progress

**W1** (2026-11-18, `9a45fe7`). The set's crate moved from its own
workspace (`sets/`) to `crates/jahamun`, a member of the game's; its file
(10 MB) and photographs (4.4 MB) embedded with `include_bytes!`, its
pipeline and sources beside the game's other asset sources. Nothing in the
game used it yet. The fast check before any of the way in: 598 tests in
230.7 s, 246 s in all.

**W2** (2026-11-18). Who went missing, and how you came: she (M.) walked
the tunnel every night with a measuring wheel after E. stopped answering,
found it longer at night, sent a photograph of a door at 4:40 and was not
seen again; you came after her. The premise no longer has you noclip
through a storage room: E.'s *Day 1* and her first page now say the
tunnel. A new channel, *your own voice, on the way in*, is the one
narration the canon allows, and it ends at the door. `lore-keeper` read it
all against the canon and the pillars; taken from its review: her note no
longer explains the door nor gives away her paper's tell (folded twice:
only ever hinted) and opens without the stock address to the reader; its
thread is a line of hers that is no order ("I'm going after him."); the
voice's numbers agree with the canon's ("485, or a metre off it"; "no
door like that"); her last message moved off the Loop's ten past three;
the words the Loop owns (comes back round, drifted) and lines that echo
asides were changed; the voice waits for a third pass before it notices
the ring, so the player can first; the old premise's last traces (the
README, an unused overlay line) were rewritten.

**W3** (2026-11-18). The place got its two doors (`set.py` F and G, baked
with the rest: 766k vertices, 411k triangles, 11.6 MB, 322 s on an M4
Pro) and a second layout, the ring (`crates/jahamun/src/ring.rs`: twenty
modules a period, 42 modules either side seen, 106 built; the haze in the
bores 0.85 of the night's density, where the set alone has 0.42, so 1% is
left at 252 m). The game's mode (`modes/tunnel.rs`) is the Loop's kind of
glue: the first floor built behind it and hidden, your feet on the set's
ground, the ring's carry, the door's time (60-120 s of moving, the run
seed's own `SplitMix64`; it changes only while every copy is behind you
or 40 m off where you look), her note into the notebook (shown first,
`HER NOTE · BY THE DOOR`), the door onto level 1 as `How::Door` (built,
saved, its card). New descent goes through it, but for a run on a chosen
seed or level (`prologue=0|1` overrides): the tools', the bots' and the
tests' runs keep the old look from the title. Found on the way: the HUD
set the prompt from the hidden floor's interactables every frame, measured
from where you stand in the tunnel; in the tunnel it no longer looks there.
Evidence: `--screen tunnel`, `tunnel-door`, `tunnel-open`, `tunnel-deep`;
`tests/tunnel.rs` (the whole way in to the first floor, by keys; no end
and no way back; the door's time and that it changes unseen; which runs
skip it; a new floor leaves it clean).

**W4** (2026-11-19). The look into the tunnel, `tunnel_in` (29.6 s, held,
skippable; `assets/src/cutscenes/tunnel_in.toml`): from the street above,
down the stone wall's stairs, across the plaza, under the arch and along
the walkway into your eyes a few bays in, every key over ground you could
stand on. The cutscene engine had refused the self-contained places; it
plays in the way in, whose frame branch runs `cutscene_pre`/`post` round
its own camera, and reads the place for what it read of the floor: the
`eyes` anchor's ground (`cut_floor_y`) and walls for its keys and its
contract (`cut_hits_wall`: off the set's walkable ground,
`Walk::open`). The contract first failed twice, rightly: a first frame
that jumped 85 m from the title's camera (the look now begins 0.3 s after
the place is up behind the black, from the camera as it is, on the
street), and 4.0 m/s along the walkway at the end (its last keys moved).
Stages `tunnel:4242`, `tunnel:7`; its contract in `tests/cutscenes.rs`
(played, skipped at 20% and 60%, repeated, still, paused, at 30, 60 and
120 frames a second); goldens `tunnel_in_mid` (on the stairs) and
`tunnel_open` (the changed door, her note).

**W5** (2026-11-19). Your voice on the way in: the persona `you_tunnel`
(his voice, `same_as = "you"`, the asides' calm emotions, -19 LUFS) and
its 26 lines, `use = "way_in"`, which the voice tool and the game now
know. Made on Chatterbox (104 takes, 7.6 min), judged (5.3 min); two
lines had no take that passed and were said again (`VOICES.md`, *Your
voice on the way in*), then all 26 passed (112 takes). The chosen takes
are 8.1 MB of WAVs. `modes::tunnel_voice` says them: the look's four on
its clock (let go if a skip passes their time), the walk's thirteen in
order once your hands are back, each after the last and 2.4 s, and the
nine reactions once each when their moment comes and nothing else is
said; it ends at the door. Tests: its order, a skip, a reaction waiting
its turn (game unit tests); the whole way in said in order, nothing
twice, her note's word last (`tests/tunnel.rs`); its canon
(`no_line_on_the_way_in_names_what_the_canon_keeps_unsaid`); heard over
the tunnel (`listen`: 34 dB over the tunnel's 300-3000 Hz band, peak
-3.9 dBFS; `target/listen/way_in_voice_4242.wav` for a person's ears).

**W6** (2026-11-19). The place's sound, every sound from where it is
(`modes::tunnel_sound`): the bars' hum from the nearest bar and your steps
on the paint in a long hard reverb (the zone `tunnel`, 2.8 s); the cars on
the road behind the glass, a burst a car, low-passed by the glass, about
every five seconds at first and every fourteen as the road empties, none
after 95 s (the voice says it once they have stopped); and once the door
has changed, cold air breathing out of it every seven seconds, heard
within 60 m, before it is seen. No draws: when a car comes is a function
of its count. The bed measured before the first car: -36 dBFS (a floor of
the rooms' -32).
