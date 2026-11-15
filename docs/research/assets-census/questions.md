# What earns its place

Each sheet (`sheetNNN.png`) shows twelve things the game put on a floor of
its rooms, one a cell, framed as a player standing or crouching near it
would see it (in a dark place the player's lamp is on). The cells are
numbered 1-12: the top row left to right, then the middle row, then the
bottom row. `sheetNNN.txt` names each cell's thing by the code's name for
it (`place_briefcases` places briefcases; `core_baby_doll` is a baby doll).
A sheet may have fewer than twelve; the rest are black.

**The place.** Read `docs/design/STYLE.md` (*The look in one paragraph*,
*Purposes*) and `docs/design/LORE.md` (*Premise*, *Places*): an endless
abandoned office building, damp and fluorescent, almost empty near the
top; each area had a purpose (an office wing, living quarters, a play
area, a parlour, washrooms, maintenance, a shrine, a party room, storage)
and what is left in it tells that; deeper, the place keeps what it has
collected from people's lives.

**Why you are asked.** The game's owner finds its props too varied and of
uneven quality: about a third of the kinds will be taken out. Judge each
cell for that, as a player who comes on it here would.

For each cell, first write the evidence: what it is, and how it looks and
sits here (one or two sentences). Then three answers:

- `belongs`:
  - `yes`: someone who worked, lived or was trapped here could have left it
    here, or the place could have kept it; a player would not ask why it
    is here;
  - `odd`: it could be, but a player would ask why it is here;
  - `no`: it does not belong in this place at all (a theme-park or stage
    prop, another genre's thing, a joke).
- `interest`:
  - `high`: it catches the eye and says something (a use, a person, a
    story, a scare);
  - `some`: dressing that reads and fits;
  - `none`: filler: too small, too generic or too like the others to notice
    or remember.
- `made`: `real` (it could be a photograph of a real, worn thing), `plain`
  (simplified but plausible), `cartoon` (a toy, a cartoon or a
  placeholder: primitive shapes, toy colours, wrong proportions, no detail
  a real one has), or `unseen` (it cannot be made out in the cell: too
  small, too dark, hidden, off the frame; then `belongs` and `interest`
  from what the name says it is, and `interest` is at most `some`).

Flag anything broken (floating, sunk in a wall or the floor, black or
magenta surfaces, stretched textures).

End with your answers as JSON, by sheet and cell:
`{"sheet001": {"1": ["yes", "some", "plain"], "2": ["odd", "none", "unseen"], ...}, ...}`
