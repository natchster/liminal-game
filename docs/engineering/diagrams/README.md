# The README's diagrams

The two pictures of the front page's *The harness* (`README.md`): the
game loop (a floor's build, one frame's order, the directors) and the
pipelines (assets, changes and playtests, the gates, the steering loop).
Each is a hand-written SVG with its own dark ground, so it reads the same
on a light or a dark page, with system fonts only, a `<title>` and a
`<desc>` for a reader that cannot see it, no script and nothing fetched.

What they draw is told at length in [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
(*One frame*, *Building a level*, *Creatures and pacing*),
[`../DEVELOPMENT.md`](../DEVELOPMENT.md) and [`../AGENTS.md`](../AGENTS.md).
Their monospaced lines name code: the game crate's modules and functions,
xtask's commands, the skills. `tests/structure.rs`
(`the_diagrams_name_code_that_exists`) fails when one of those names is
gone, so a renamed module sends you here.

Change one by editing its text, then look at it as a browser draws it (a
headless one, as the game's own pictures are rendered: never a capture of
the screen), at 1200 px wide: no line may run past its box.
