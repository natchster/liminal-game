# The sets

Places built apart from the game, each runnable on its own and made to join
the game later as a location. A workspace of its own (like `lab/`): the
game's build, its check, its fixtures and its `target/` never see it, and
nothing here changes the game's crates (the sets read `crates/engine`).

| Set | What |
|---|---|
| [`jahamun/`](jahamun/README.md) | the Jahamun Tunnel, Seoul, at night: the approach to the portal and its long stairs, then the 485 m walkway inside the bore behind the glass noise barrier |

```sh
cd sets
cargo run --release -p jahamun                 # walk it
cargo run --release -p jahamun -- --shots DIR  # its named views, headless
cargo test --release                           # the sets' own tests
```

A set keeps the game's rules, so moving in is moving code, not rewriting it:

- **the engine only.** A set's library reads `engine` and nothing else; its
  window and Metal device live in its binary (the game's layering: `engine`
  ← `game` ← `backrooms`).
- **pure.** No random draw and no clock in what is built or drawn; the
  game's own math (`engine::dmath`), never std's `sin` and kin (the
  workspace takes the game's `clippy.toml` from the root).
- **light and fog belong to places.** Light is baked into the surfaces
  from the place's own lamps; fog is chosen by the light map (where each
  surface is), never by where the eye is.
- **photographs are CC0** (or MIT, CC BY 4.0), listed with their SHA-1 in
  the set's `sources.json`, as the game's `assets/src/textures/sources.json`
  lists its own; references (other photographs, the developer's video) are
  looked at, never put in.
