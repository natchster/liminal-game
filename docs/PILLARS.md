# Pillars

What the game is for, as feelings, broken down after Raph Koster's
pillars → goals → rules. Every change to content quotes the rule it serves
(in the commit message, or the skill's checklist); a change that serves none
needs a reason. The canon behind these is [`LORE.md`](LORE.md); how the look
is held to them is [`STYLE.md`](STYLE.md).

Each rule ends with **its test**: how you know it still holds, and who or what
checks it. *Gate* means a check in `cargo xtask check`; *review* means a blind
reviewer; *people* means fresh players.

## 1. Emptiness is the threat

*The player should feel that the place is too big and too empty for them, and
that the emptiness is watching.*

- **Goal: most of the level is nothing.**
  - Rule: the Lobby and the `empty` purpose stay sparse. Clutter belongs to
    purposes deeper down. *Test:* the range report's prop density for `empty`
    zones stays in its band (gate, `cargo xtask range`).
  - Rule: zones without a threat are legitimate. Not every room needs a
    creature or a scare; Pools has none. *Test:* the director's calm phase
    takes a real share of play (gate, `tests/pacing.rs`).
- **Goal: what is absent says more than what is present.**
  - Rule: show what a place was for by what is left, never by a label beyond
    the zone caption. *Test:* `lore-keeper` review of new text.

## 2. Familiar but wrong

*The player should recognise every place, and still not trust it.*

- **Goal: real places, slightly off.**
  - Rule: architecture follows real offices, halls and tunnels: real ceiling
    heights, real doorways, real materials, then one thing wrong (a doorway
    to nowhere, a ceiling that steps for no reason). *Test:* the zone views in
    the goldens; `art-critic` review against `STYLE.md`.
  - Rule: props look like real objects, not toys: physically plausible
    albedo, muted colour, wear. *Test:* the material lint and the look
    anchors (gate, `tests/materials.rs`, `cargo xtask shots`).
- **Goal: wrongness accumulates.**
  - Rule: deeper levels rot: more grime, darker regions, thicker fog. *Test:*
    luminance and deep-shadow share per level depth in the range report.
  - Rule: deeper down, not every trail is hers. From her fifth floor page's
    floor, flat sheets that look like hers lead into a dead end where the
    place waits; the tell (hers are folded twice) is only ever hinted.
    *Test:* `trails_that_are_not_hers_lead_nowhere` (gate, `tests/story.rs`).

## 3. The sound never stops (except where it does)

*The player should never be in silence, so that silence means something.*

- **Goal: the hum and the room are always there.**
  - Rule: every sound comes from somewhere: a prop, a fixed signature spot, a
    fixture. *Test:* `tests/ambient.rs`, `tests/listen.rs` (gate).
  - Rule: the mix holds its level; nothing piles up at a level's start.
    *Test:* `tests/loudness.rs`, `tests/listen.rs` (gate).
- **Goal: silence is a warning.**
  - Rule: the director's hush thins and softens the ambience before a peak.
    *Test:* `the_hush_quiets_the_ambience` (gate); whether it reads as a
    warning (people).

## 4. You are never shown everything

*The player should always suspect there is more, and never be sure what it is.*

- **Goal: threats are heard and half-seen before they are seen.**
  - Rule: a creature announces itself through the place (steps, failing
    lights, silence) before it arrives. *Test:* the director's build phase
    precedes every peak (gate, `tests/pacing.rs`).
  - Rule: at a distance a creature is a silhouette and a pair of eyes; its
    detail is for close range. *Test:* the creature goldens at 8 m and 2 m;
    `art-critic` review.
- **Goal: rules are learned, not told.**
  - Rule: no tutorials, maps or explanations of creatures or anomalies;
    notes hint, never explain. *Test:* `lore-keeper` review.
  - Rule: the way down is found, not shown. No compass and no objective
    line; the door is lit into the light map and calls from where it stands.
    *Test:* the HUD has no compass (`tests/hud.rs`); `the_door_is_heard`
    (gate, `tests/listen.rs`); the exit view in `--dump-level … --views`.

## 5. Dread, then release

*The player should be afraid in waves, never numb.*

- **Goal: tension has a shape.**
  - Rule: calm, build, peak, relief, in that order; after a close call the
    threat backs off. *Test:* `tests/pacing.rs` phase bands (gate).
  - Rule: the player can choose when the dread starts: taking her page at
    her camp puts out her lamp and ends the quiet within seconds; the first
    floor waits for it. *Test:* `her_page_wakes_the_place` (gate,
    `tests/story.rs`).
  - Rule: the same scare never repeats soon, and the same cue does not always
    mean the same thing. *Test:* one-shot repetition in `tests/loudness.rs`
    (gate); run-log analysis (planned, roadmap 4.2).
- **Goal: it stays scary.**
  - Rule: scares are judged by people who have not played the build before.
    The developer and the agent are both used to them. *Test:* the playtest
    kit (people, roadmap 4.4).
