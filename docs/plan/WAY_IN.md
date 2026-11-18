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
| W3 | The tunnel as the run's start: the mode (`modes/tunnel.rs`), the ring of modules with no end, the door that changes, her note, the door into level 1; New descent from the title; its tests | |
| W4 | The look you take: `tunnel_in`, from the street above into the walkway, its contract | |
| W5 | Your voice: the persona `you_tunnel`, its lines made and judged, played in order and at what you pass | |
| W6 | The place's sound: the bars' hum, the road behind the glass going quiet, steps in its reverb, the wind behind the door | |
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
found it longer at night, sent a photograph of a door at 3:12 and was not
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
