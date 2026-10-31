# The prop census

This time the frames are not pairs: each sheet (`sheetNNN.png`) shows six
props of the game, each in the place the game put it, framed by a camera
as a player would stand or crouch to look at it (in a dark place the
player's lamp is on). The cells are numbered 1–6: the top row left to
right, then the bottom row. `sheetNNN.txt` names each cell's prop by the
code's name for it (`place_radiators` places radiators; `core_trash_can`
is a trash can), and the file the cell was cut from.

**The question.** For each cell, does the prop look like a real object,
worn, as `docs/STYLE.md` asks — or like a toy, a cartoon or a
placeholder?

For each cell, first write the evidence: what the object is, and what
makes it read as real or not (its shape and proportions, the detail a
real one has or lacks, its material and colour, its wear). Then grade it:

- `real`: it could be a photograph of a real, worn object;
- `plain`: simplified but plausible; it reads as the thing and nothing
  jars;
- `cartoon`: it reads as a toy, a cartoon or a placeholder: primitive
  shapes (bare boxes, cylinders, spheres), slab or blobby forms, toy
  colours, wrong proportions, parts that do not join, no detail a real
  one has;
- `unseen`: the prop is not visible in the cell (too small, too dark,
  hidden behind something, off the frame), so it cannot be judged.

A decal (a stain, a crack, a poster, graffiti) is judged the same way:
does it read as a real mark on a real surface?

Flag anything broken on either side of the question: floating, sunk in a
wall or the floor, black or magenta surfaces, stretched textures.

End with your grades as JSON, by sheet and cell:
`{"sheet001": {"1": "cartoon", "2": "plain", ...}, ...}`
