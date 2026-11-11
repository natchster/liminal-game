# Cutscene sources

One file a cutscene, `ID.toml`, its `id` the file's name. The format, the
anchors and the rules a cutscene keeps are in `docs/CUTSCENES.md`; the way
to make one is the `add-cutscene` skill.

```sh
cargo xtask cutscene check    # the sources, the bundle, the game's rules
cargo xtask cutscene write    # compile into crates/game/assets/cutscenes.json (and MANIFEST.json)
cargo xtask cutscene list     # what there is
```

The game never reads these files: it embeds the compiled bundle, every
field written out. Edit a source, then write it again.
