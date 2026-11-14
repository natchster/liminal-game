# Replays: recording, seeking, determinism, and a save from any point

Gathered on 2026-11-14 by a research agent for the record/replay engine and kept whole, as the record of what was found. The brief: replays that can start and end at any point of play; adequate file size; no randomness the seed does not fix; no floating-point drift; no notable CPU or RAM cost; a documented replay format; recording the player starts and stops; replay selection and playback in the game menu; and a save format aligned with replays, so that a save can be made from any point of a replay and a replay's start equals a save. The questions: (1) approaches and how named games did it; (2) starting at an arbitrary point; (3) determinism, including floating point and Rust specifics; (4) file size; (5) desync detection and verification; (6) player-facing UX. The note ends with recommendations for this game: a first-person horror game in Rust, macOS on Apple Silicon only, levels a pure function of (run seed, level), one seeded per-frame random stream, a variable time step clamped to 50 ms, and a version-1 JSON recorder.

Every claim links the page it rests on. *(inference)* marks my own conclusions. **Not verified** marks something I searched for and could not source; section 7 lists them all. Facts about this repository come from reading its code and files on 2026-11-14, cited by path; nothing was built or run except reading files, `nm`/`otool` on the existing release binary, and a Python size estimate on an existing recording. Read a source before relying on one of its details.

## 0. What the game does today (read from the code, 2026-11-14)

These facts set up the recommendations; each is from the file named.

- **The recorder** (`crates/game/src/debug/replay.rs`): a `Recording` holds `native_seed`, then per frame `(dt, now_ms, FrameInput)`. A frame's input is a list of events: key down/up by name (`"KeyW"`), mouse look `(f64, f64)`, a HUD click by element name, pointer lock, blur, wheel `f64`, menu hover by id. It also holds once-a-second telemetry rows and the director's log. It is written as JSON `"version": 1`. `replay_frame` applies the input, sets `g.now_ms`, and calls `g.frame(dt)`.
- **A state hash already exists**: `Game::state_hash` gives four FNV-1a 64-bit parts, `player`, `entities`, `director` and `world`, over chosen fields. `StateHash::diff` names the parts that differ. The module comment cites Factorio's per-tick check.
- **The time step** (`crates/game/src/session/frame.rs`): `Game::frame(real_dt)` clamps to `real_dt.min(0.05)`, and uses `fixed_step_dt` instead when that is non-zero, so a fixed-step mode already exists for tests. `g.now_ms` is set from wall time (`self.started.elapsed()`) in `crates/backrooms/src/main.rs`. The `set_timeout` timers in `crates/game/src/base/game.rs` fire on `now_ms`. So gameplay reads two clocks, the clamped `dt` and the unclamped wall clock, and both are recorded.
- **Mouse look** (`crates/engine/src/app.rs`, `crates/game/src/session/frame.rs`): winit's `DeviceEvent::MouseMotion { delta: (f64, f64) }` is forwarded as is. `mouse_move` scales it by `0.0022 * settings.look`, honours `settings.invert_y`, and also accumulates the raw deltas into `mouse_dx`/`mouse_dy`. Settings therefore change what the simulation does with the same recorded input.
- **The save** (`crates/game/src/session/flow.rs`): `SaveData`, under the key `opencarole.backrooms.save.v2`, carries the run between levels: `level_num` and `run_seed`, sanity, battery, the counts, lore, notebook, the creatures' and asides' run memory, and what was taken. It has no mid-level state and no random-stream state, plus a wall-clock `savedAt`.
- **Hidden state**: `crates/game/src` holds 33 `thread_local!` declarations, the random streams among them (`crates/game/src/base/rng.rs`), and 164 mentions of `HashMap`/`HashSet` from `std::collections`. Those use the default, randomly seeded hasher (section 3.2).
- **JSON floats round-trip exactly**: the workspace `Cargo.toml` enables `serde_json`'s `float_roundtrip`. serde_json's manifest says this "makes f64 -> JSON -> f64 produce output identical to the input" ([serde_json Cargo.toml](https://github.com/serde-rs/json/blob/master/Cargo.toml)). Today's JSON therefore loses nothing; it is only large.
- **Size today**: `target/replays/recorded_77_L3.json` is 1,800 frames over 29.7 s of game time. It looks machine-generated: exactly 1,800 frames, with a look event on every frame. Its `frames` array is 139,581 bytes of compact JSON, 77.5 B per frame or about 276 KB per minute. Re-encoded by a scratch script, with these figures in KB per minute:

  | Encoding | Raw | deflate -9 | xz |
  |---|---|---|---|
  | JSON, as today | 276 | 50 | 43 |
  | Same data, binary f64 | 123 | 48 | 42 |
  | Quantized: dt in µs, look in 1/1000 units, zigzag varints, key ids | 31 | 18 | 15 |

  *(inference)* A real session has idle frames and fewer look events, so these numbers are an upper bound on the mouse part.
- **Simulation speed**: `perf/history.csv` holds rows dated 2026-11-14 (commit `8d84453+`, Mac16,11, macOS 27.0) for the twelve benched levels. Game update p50 is 0.023–0.083 ms and p99 0.87–0.94 ms per frame; level build takes 24–574 ms.
- **The math library is the OS's.** `nm -u target/release/backrooms` (built 14 Nov 12:56) lists `_sin`, `_cos`, `_tan`, `_asin`, `_acos`, `_atan`, `_atan2`, `_exp`, `_exp2`, `_log`, `_log10`, `_pow`, `_hypot`, `_cosh`, `_tanh`, `_fmod` and `___sincos_stret` as undefined, resolved from `/usr/lib/libSystem.B.dylib` (per `otool -L`). `___powidf2` is defined inside the binary: `powi` comes from Rust's compiler-builtins, not from Apple. Apple "does not support statically linked binaries" and promises compatibility only "in each dynamically linked system library" ([Apple QA1118](https://developer.apple.com/library/archive/qa/qa1118/_index.html)). The transcendental results a player gets come from whatever `libsystem_m` their macOS ships.
- **Drawing takes `&mut Game`** (`crates/backrooms/src/main.rs`, `draw(&mut self.g, …)`). Whether any of it writes simulation state is not checked here (section 3.3).
- **Compression is already a dependency**: `miniz_oxide = "0.8"` (deflate, pure Rust) is in the workspace `Cargo.toml`.

## 1. Approaches, and how named games did it

### 1.1 Input-only (lockstep) demos

The file holds the starting conditions plus every input; playback re-runs the simulation. It needs bit-exact determinism. The files are tiny, and they break when the simulation code changes.

**Doom.** Each tic stores a `ticcmd_t`, "the data sampled per tick": `forwardmove`, `sidemove`, `angleturn`, buttons, plus a `consistancy` field that "checks for net game" ([d_ticcmd.h](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/d_ticcmd.h)).
- **The demo record is quantized to 4 bytes**: forward, side, `(angleturn+128)>>8` and buttons. `G_WriteDemoTiccmd` then reads the bytes straight back into the live command, with the comment "make SURE it is exactly the same" ([g_game.c L1506–L1523](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1506-L1523)). The game consumes exactly what was recorded, so there is never a recorded-versus-played mismatch.
- **The header** is the game version and the new-game parameters: skill, episode, map, deathmatch, respawn, fast, nomonsters, consoleplayer, playeringame ([L1549–L1566](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1549-L1566)).
- **Playback** refuses another version ("Demo is from a different game version!"), then calls `G_InitNew(skill, episode, map)` ([L1589–L1611](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1589-L1611)). A demo starts at a level start, rebuilt from parameters, as this game's levels are.
- **Rate and size** *(inference)*: at `TICRATE` 35 ([doomdef.h](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/doomdef.h)), that is 140 B/s, about 8.2 KiB per minute per player. The default buffer is `0x20000` bytes ([L1530–L1545](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1530-L1545)), about 15 minutes of single-player play.
- **Quantizing at the source with carry**: Chocolate Doom rounds the turn to the nearest 256-unit step before the game uses it, and "carr[ies] forward the error from the reduced resolution to the next tic, so that successive small movements can accumulate". "Longtics" demos store the full 16-bit turn instead ([chocolate-doom g_game.c L633–L651, L2048–L2058](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/g_game.c#L633-L651)).
- **Net games** compare a per-tic consistency value, the player's x or the RNG index, and stop with "consistency failure" ([L680–L693](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L680-L693)).

**Age of Empires.** Every machine ran the same simulation on the same commands. The team "check-summed the world, the objects, the pathfinding, targeting and every other system"; "any simulation that ran differently was tagged as 'out of sync' and the game stopped". Random numbers were seeded and synchronized, and the simulation had to make the same number of random calls. Randomness outside the simulation, such as terrain sounds, saved and re-seeded the generator. Recorded games, added in Age of Kings, replay "because our simulation is deterministic", and "a game recording gave us a great way of passing around repro cases for bugs" ([1500 Archers on a 28.8, Bettner and Terrano](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).

**Supreme Commander** (Gas Powered Games, as told by Forrest Smith). "The replay file only needs to store player inputs." This is "why replays stop working when the game is patched". "The entire game state is hashed once per second"; on a mismatch "Desync Detected" appears and "there is no recovery mechanism" ([Synchronous RTS Engines and a Tale of Desyncs](https://www.forrestthewoods.com/blog/synchronous_rts_engines_and_a_tale_of_desyncs/)). Re-simulation for reconnection "would be similar to playing a replay file" and can run the sim layer as fast as possible, skipping the user layer ([Sync Harder](https://www.forrestthewoods.com/blog/synchronous_rts_engines_2_sync_harder/)).

**StarCraft II.** Replays are tied to the build. The game "has always automatically loaded replays in the appropriate version of the game behind the scenes", and an old version "has to disconnect from Battle.net" ([Blizzard, patch 2.0.10](https://news.blizzard.com/en-us/article/10495761/update-der-replay-funktion-in-2-0-10)). Blizzard's decoder is "a base-build-specific library" ([s2protocol](https://github.com/Blizzard/s2protocol)).

**Factorio.** A replay is a file stored within the save, "containing any information needed to repeat (replay) a game from start". "Replays for worlds created in older versions cannot be played", and adding or removing mods breaks them ([Factorio wiki, Replay system](https://wiki.factorio.com/Replay_system)). The developers save the game and replay it; "the replay also takes the CRC check every tick", and a mismatch throws the desynchronization exception ([FFF #47](https://factorio.com/blog/post/fff-47)).

**Halo 3 Theater.** Bungie's Max Dyckhoff confirmed that films are captured inputs plus a starting seed. The seed drives "not just every enemy, but every projectile, physics object, sound played, etc." Network co-op forced the determinism: "every operation inside the game is deterministic" ([HBO interview](https://halo.bungie.org/misc/kamikazegrunts.html)).

**Trackmania.** A replay is "a list of inputs and their timestamps", physics runs at 100 updates per second, and the authors state the engine is "completely deterministic". The in-game Validate "replays inputs of a player from start to finish to determine if the simulation matches the ghost of the replay" ([donadigo, TMX replay investigation](https://donadigo.com/tmx1)). The file carries both the inputs and a ghost, and the ghost serves as the check.

**osu!** The `.osr` header holds game version, beatmap MD5, replay MD5, mods and a life-bar graph. The body is an LZMA stream of `w|x|y|z` frames: milliseconds since the previous action, cursor x and y as floats, and a key bitmask. Since 20130319 a last frame `-12345|0|0|seed` stores the RNG seed ([osu! wiki, osr format](https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29)).

**Celeste TAS** (community tool). Each line is `(Frames),(Actions)`; `123,R,J` holds Right and Jump for 123 frames. Inputs are held states, so the format is run-length coded by construction. Analog input is `F,(Angle),(Magnitude)` ([CelesteTAS Input File](https://github.com/EverestAPI/CelesteTAS-EverestInterop/wiki/Input-File)). `console load` starts a TAS at a level, room, or exact position and speed, and `Assert` aborts the TAS when a condition fails, which the wiki describes as a way to detect desyncs ([CelesteTAS Commands](https://github.com/EverestAPI/CelesteTAS-EverestInterop/wiki/Commands)).

**N++ and Super Meat Boy.** **Not verified.** I found no developer or format source for how N++ stores its leaderboard replays, or whether Super Meat Boy's all-attempts replay stores inputs or positions. Searches turned up only reviews and third-party remakes.

### 1.2 State streams (network-demo style)

The file holds what a client received from the server, or snapshots of replicated state. Determinism is not needed, and a recording can start anywhere. Files are larger, and the decoder is tied to the protocol.

**Quake.** `CL_WriteDemoMessage` "dumps the current net message, prefixed by the length and view angles" ([cl_demo.c L59–L80](https://github.com/id-Software/Quake/blob/master/WinQuake/cl_demo.c#L59-L80)).

**Quake III.** `record` "begins recording a demo from the current position". It writes a full gamestate message (config strings and entity baselines) first, and "don't start saving messages until a non-delta compressed message is received". The file name carries the protocol version (`.dm_%d`) ([cl_main.c L248–L330](https://github.com/id-Software/Quake-III-Arena/blob/master/code/client/cl_main.c#L248-L330)). The server deltas each snapshot against the last one the client acknowledged ([Fabien Sanglard, Quake 3 network model](https://fabiensanglard.net/quake3/network.php)). A Quake III demo can begin anywhere because it begins with a full state.

**Unreal Engine.** The replay system "works by using a `DemoNetDriver` to read data drawn from the built-in replication system". Playback has `demopause`, `demospeed` and `demoscrub`. Backward compatibility exists "starting in version 4.13", via `EngineNetVer`/`GameNetVer` and version overrides ([Using the Replay System](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-the-replay-system-in-unreal-engine)).
- **Checkpoints** are "snapshots of the net changes to the world", taken every 30 s by default (`demo.CVarCheckpointUploadDelayInSeconds`). Longer intervals make backward scrubbing slower but replays smaller ([DemoNetDriver and Streamers](https://dev.epicgames.com/documentation/en-us/unreal-engine/demonetdriver-and-streamers-in-unreal-engine)).
- **Seeking** rebuilds a moment from the baseline, the checkpoints, and the changes after the last checkpoint.
- **The Memory streamer** "keeps a user-configurable running length of replay data in memory", for instant replays.

**Fortnite** (on Unreal). Replays "become unusable because the context changes too much" after updates, since they store a simulated network stream ([Fortnite Creative replays](https://dev.epicgames.com/documentation/en-us/fortnite/replays-feature-in-fortnite-creative)).

**Rocket League.** The replay holds `keyframes` and `network_frames`. Keyframes are "the main frames that are derived from in the following frame data" ([boxcars KeyFrame](https://docs.rs/boxcars/latest/boxcars/struct.KeyFrame.html)), and "the network data will often change with each Rocket League patch" ([boxcars](https://github.com/nickbabcock/boxcars)). One example is "a 1.5 MB replay" ([rattletrap README](https://hackage.haskell.org/package/rattletrap-13.0.3/docs/README.md)).

**Overwatch.** The GDC 2017 talk covers "how to develop a replay system in parallel with your network model" ([GDC Vault, Orwig](https://gdcvault.com/play/1024053/Replay-Technology-in-Overwatch-Kill)). In-game, "replays will be available for the duration of a patch" ([Blizzard](https://overwatch.blizzard.com/en-us/news/23000187)). **Not verified**: the talk's internals (snapshot and delta layout) sit behind the Vault paywall.

### 1.3 Hybrids: keyframes plus inputs

**MotoGP** (Shawn Hargreaves). MotoGP "stored the entire physics simulation state at periodic intervals" and "controller inputs for the gaps in between these keyframes", which "allowed us to jump back and forth between keyframes … while keeping the overall data rate quite low" ([MotoGP: replays](https://www.shawnhargreaves.com/blog/motogp-replays.html)).
- **A fixed budget**: a 1 MB buffer. Keyframe spacing was computed from the expected race length, so a 20-lap full-pack race got none and a 1-lap race got one every couple of seconds.
- **Only the core bike physics** went into keyframes. Animation, sound, camera and particles only needed to look similar.
- **On a jump** the game deleted particles and skidmarks, restored the keyframe, and "ran a single update before rendering".

**Photon Quantum.** A replay needs "an application build with the same version of game and Quantum libraries", the assets, the session's configuration, and its input history (`InputHistoryDeltaCompressed`). Checksums "can be recorded with and verified during playback". `InitialTick`/`InitialFrameData` start a replay mid-game, and a snapshot export "saves a snapshot of the current game state with the asset db into the same `QuantumReplayFile` format" ([Quantum 3 Replay](https://doc.photonengine.com/quantum/v3/manual/replay)). A snapshot and a replay share one file format.

**Emulator movies.** A Dolphin `.dtm` header has "Starts from savestate", a "SHA-1 hash of Dolphin Git revision", the "MD5 hash of game file", the emulation settings, a tick count and an input count ([TASVideos, DTM](https://tasvideos.org/EmulatorResources/Dolphin/DTM)). libTAS's `.ltm` stores the game's MD5 and a `savestate_frame_count` for savestate movies. With variable frame rate on, a per-frame `|T num:den` holds the frame time as an exact rational, and mouse input is stored as signed integers ([libTAS format](https://clementgallet.github.io/libTAS/guides/format)).

### 1.4 State recording for rewind

**Braid.** The design required rewinding "a large amount of gameplay (30 to 60 minutes)" within "a small space (40 megabytes on consoles)" ([GDC Vault, Blow 2010](https://gdcvault.com/play/1012210/The-Implementation-of-Rewind-in)). **Not verified**: how Blow stored it (per-frame state, deltas, sampling rates). The talk is behind the Vault paywall, and the secondhand accounts disagree.

**Prince of Persia: The Sands of Time.** Rewind rolls the world back "seven seconds", and "one frame equalled one tick of the physics engine" ([GOG, Preserving Sands of Time](https://www.gog.com/blog/preserving-sands-of-time/)). **Not verified**: its buffer design.

**2K Marin** (Mark Wesley, GDC 2013). A rewindable state-recording replay "is far superior for debugging and tuning than a deterministic replay system" ([GDC Vault](https://gdcvault.com/play/1018138/Implementing-a-Rewindable-Instant-Replay)).

**Halo 3 Theater** shows the cost of input-only playback with a big state. Campaign films could not rewind; Reach's "Theater 2.0" added rewinding in campaign and Firefight ([Halopedia, Theater](https://www.halopedia.org/Theater)). Dyckhoff tied the difficulty to the campaign's streaming and the larger campaign state ([HBO interview](https://halo.bungie.org/misc/kamikazegrunts.html)).

### 1.5 When each makes sense, and what it costs

| Approach | Examples | Needs bit-exact determinism | Size | Start anywhere / seek | Survives a patch |
|---|---|---|---|---|---|
| Inputs from a known start | Doom, AoE, SupCom, SC2, Factorio, Halo, Trackmania, osu!, Celeste TAS | yes | tiny (Doom about 8 KiB/min; SC2 about 100 KB per 15 min, see 4.4) | start: only where a full state exists; seek: re-simulate | no (SupCom, SC2, Factorio, AoE) |
| State stream (network demo) | Quake, Quake III, Unreal/Fortnite, Rocket League, Overwatch | no | large (CS2 demos are hundreds of MB; Rocket League's example 1.5 MB) | start: yes (Q3 writes a gamestate first); seek: via checkpoints (Unreal) | sometimes, with versioned fields (Unreal); often not (Fortnite, Rocket League, Overwatch) |
| Keyframes + inputs | MotoGP, Quantum, emulator movies | yes | small plus the keyframes | yes, to and from keyframes | no |
| Per-frame state ring | Braid, PoP, 2K Marin's debug replay | no | big; budgeted (Braid: 30–60 min in 40 MB) | yes, within the buffer | n/a (in memory) |

*(inference)* A single-player game with an already deterministic simulation and no replication layer fits the first row, plus the third when seeking must be fast. A state stream needs a serialized-world-delta layer this game does not have, and it buys only patch tolerance, which Fortnite, Rocket League and Overwatch show is often not delivered anyway.

## 2. Starting a replay at an arbitrary point

**Keep a full state at the start.** Quake III begins every demo with a gamestate and waits for a non-delta snapshot ([cl_main.c](https://github.com/id-Software/Quake-III-Arena/blob/master/code/client/cl_main.c#L248-L330)). Quantum starts from `InitialFrameData` ([Quantum](https://doc.photonengine.com/quantum/v3/manual/replay)). Dolphin and libTAS movies can start from a savestate ([DTM](https://tasvideos.org/EmulatorResources/Dolphin/DTM), [libTAS](https://clementgallet.github.io/libTAS/guides/format)). CelesteTAS starts at a room or at an exact position and speed with `console load` ([Commands](https://github.com/EverestAPI/CelesteTAS-EverestInterop/wiki/Commands)). Doom can only start at a level start, because its header holds new-game parameters ([g_game.c](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1589-L1611)).

**Re-simulate from the last checkpoint.**
- **Unreal** applies the baseline, the checkpoints, then the changes after the latest checkpoint; checkpoints default to 30 s ([DemoNetDriver](https://dev.epicgames.com/documentation/en-us/unreal-engine/demonetdriver-and-streamers-in-unreal-engine)).
- **MotoGP** jumped only to keyframes and sized their spacing from a memory budget ([Hargreaves](https://www.shawnhargreaves.com/blog/motogp-replays.html)).
- **Factorio** has none of this: "there is no way to skip ahead, the entire replay must be watched up until the point of interest", and rewinding is not possible ([wiki](https://wiki.factorio.com/Replay_system)).
- **Age of Empires IV**: a player guesses its replays "must be sequential and you cannot just skip large portions" ([AoE forum thread](https://forums.ageofempires.com/t/cant-rewind-scrub-through-replays/179514)). This is not official, and **not verified** for AoE II.
- **StarCraft II**: Rewind "will immediately convert the game into a fully loaded replay", "allowing you to rapidly jump to any point in the match" ([Blizzard, patch 3.13 preview](https://news.blizzard.com/en-gb/article/20719765/patch-3-13-0-preview-rewind)). **Not verified**: whether SC2 keeps snapshots taken during play or re-simulates.

**How fast re-simulation is.** Forrest Smith notes a re-sim can skip the user layer and run as fast as possible ([Sync Harder](https://www.forrestthewoods.com/blog/synchronous_rts_engines_2_sync_harder/)). *(inference, from `perf/history.csv` above)* For this game, a minute of play at 60 fps is 3,600 updates. At the benched p50 of 0.023–0.083 ms that is 0.08–0.3 s of CPU, and at a pessimistic 0.9 ms per update it is 3.2 s. Re-simulating ten minutes of a level from its start would take 1–3 s typically, with a level build (24–574 ms) on top. This bench follows a fixed path; real play with creatures hunting may cost more. The creature pack's first row had update p50 0.07 ms and p99 0.89 ms (PERF.md in the engineering folder, read locally). Re-simulating from the level start may be fast enough to seek without keyframes at first. Measure it before building keyframes.

**Making a save and a replay start the same thing.**
- **Quantum**: a snapshot is a replay file with initial frame data ([Quantum](https://doc.photonengine.com/quantum/v3/manual/replay)).
- **Factorio**: the replay lives in the save, and "you can save during playback and later play again from that point" ([wiki](https://wiki.factorio.com/Replay_system)).
- **StarCraft II**: Heart of the Swarm lets players "jump to a certain part of the game via the replay and resume from there" and "take control of the game themselves at any point of time" ([GosuGamers, 2012](https://www.gosugamers.net/news/22662-resume-from-replay-coming-to-beta-tomorrow)). It shipped as "Take Command". Patch 2.0.10 notes that old-version replays cannot be used to recover games ([Blizzard](https://news.blizzard.com/en-us/article/10495761/update-der-replay-funktion-in-2-0-10)).
- **Doom** keeps them apart: the savegame is a state dump with its own version string, rejected silently on mismatch ([g_game.c L1200–L1218](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1200-L1218)), while demos start only from new-game parameters.

*(inference)* Two designs follow:
- **(a) A save is "level-start save + inputs up to frame N".** Loading rebuilds the level and re-simulates. It is exact, tiny, and cheap to build, but bound to the build.
- **(b) A save is a full-state snapshot.** It loads instantly and can survive builds through versioned migrations, but every system's state, including the thread-locals, must be serialized. GGPO calls such hidden state the thing to move into saveable state ([GGPO Developer Guide](https://github.com/pond3r/ggpo/blob/master/doc/DeveloperGuide.md)).

## 3. Determinism

### 3.1 Fixed versus variable time step

Glenn Fiedler warns that "the behavior of your physics simulation depends on the delta time you pass in", and proposes an accumulator where "the renderer produces time and the simulation consumes it in discrete dt sized steps", interpolating the render state. Fixed steps give "exact reproducibility from one run to the next given the same inputs"; his semi-fixed variant is "not exactly the same due to the limited precision of floating point arithmetic" ([Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/)). Lockstep must use "the same delta time" each step ([Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/)). Hargreaves lists variable-timestep timing differences among replay breakers and fixed MotoGP's with a fixed-timestep mode ([MotoGP](https://www.shawnhargreaves.com/blog/motogp-replays.html)). libTAS shows the variable alternative: it records each frame's time as an exact rational `num:den` ([libTAS](https://clementgallet.github.io/libTAS/guides/format)).

*(inference)* A variable step is deterministic if the exact `dt` sequence is recorded and fed back, which is what this game does today. What a variable step costs is behaviour that depends on frame rate, a fairness and tuning problem, not drift; and bytes per frame for `dt`. Quantizing `dt` at the source to a power-of-two grid makes it exactly representable in f64 and makes sums of steps exact for 2^53 ticks.

### 3.2 Random streams, iteration order, hidden state

- **Seed every generator from the recording.** Age of Empires required the same number of random calls, and made non-simulation randomness save and re-seed the generator ([1500 Archers](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)). Hargreaves lists time-seeded or shared generators as replay breakers ([MotoGP](https://www.shawnhargreaves.com/blog/motogp-replays.html)). GGPO says to include the RNG state in the game state, keep wall time out of game state, and move static locals into saveable state ([GGPO](https://github.com/pond3r/ggpo/blob/master/doc/DeveloperGuide.md)). osu! stores its RNG seed in the replay ([osr](https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29)), and Halo's films store the starting seed ([HBO](https://halo.bungie.org/misc/kamikazegrunts.html)).
- **Iteration order.** Rust's `HashMap` "uses a hashing algorithm … randomly seeded", and `keys()` visits "in arbitrary order" ([std HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)). Clippy's restriction lint `iter_over_hash_type` exists because "hash types are unordered … on redundant systems this may cause inconsistencies" ([clippy source](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/iter_over_hash_type.rs)). Box2D notes that object-pool free lists can change body order, so recreating a world is sometimes needed ([Box2D, Determinism](https://box2d.org/posts/2024/08/determinism/)).
- **Uninitialized and dangling state.** SupCom names "an uninitialized variable" as one of the most common desync causes, and a dangling pointer in Demigod ([Tale of Desyncs](https://www.forrestthewoods.com/blog/synchronous_rts_engines_and_a_tale_of_desyncs/)). Halo names "threads sharing data and uninitialised data" ([HBO](https://halo.bungie.org/misc/kamikazegrunts.html)).

### 3.3 Threads, audio, and drawing

- **Factorio.** A chunk-generation desync "that has been in the game since … July 22, 2017" appeared only when two computers had a different number of CPU cores ([FFF #415](https://factorio.com/blog/post/fff-415)).
- **Box2D.** Atomics and mutexes "prevent data races but don't make the order deterministic". It merges per-thread bit arrays to get a fixed order ([Box2D](https://box2d.org/posts/2024/08/determinism/)).
- **MotoGP.** Hargreaves names race conditions and "drawing code modifying simulation state", and the fix of separating Update and Draw ([MotoGP](https://www.shawnhargreaves.com/blog/motogp-replays.html)).
- **Audio.** Halo's seed drives the "sound played" ([HBO](https://halo.bungie.org/misc/kamikazegrunts.html)), so audio decisions are part of the simulation there. *(inference)* Here, sound code that draws the per-frame stream must run identically during a muted fast-forward: mute at the output, never skip the update.

### 3.4 Floating point across builds, compilers and OS versions

- **Same binary, same instructions.** Bruce Dawson: "many games have been shipped based on this", because "every user is executing exactly the same instructions" ([Floating-Point Determinism](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/)).
- **What still varies.** Dawson lists x87's per-thread precision; per-thread rounding and denormal modes ("flushing denormals to zero changes results"); FMA; evaluation order; "the precise results of functions like sin, cos, tan, etc. are not defined by the IEEE standard"; estimate instructions; and CPU dispatch in libraries. sin can differ "depending on whether sin() was calculated at run-time or compile-time". He advises asserting the FP settings "once per frame … (on all threads!)" ([Dawson](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/)).
- **Debug versus release.** Fiedler's experts note replays "cannot be played back on machines with different CPU architectures, compilers, or optimization settings" (Hargreaves), and Apple Developer Support said fsin/fcos "could produce slightly different answers across implementations" ([Floating Point Determinism](https://gafferongames.com/post/floating_point_determinism/)). Fiedler adds his simulation is "probably not even deterministic between debug and release builds" ([Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/)).
- **Platform libms differ, and versions of one libm differ.**
  - Zimmermann et al.: "users might get different results with different libraries, or different versions of the same library". Some libraries dispatch different code per CPU, and the same binary can differ by hardware through instructions like rsqrt/rcp ([Accuracy of Mathematical Functions, 2024](https://homepages.loria.fr/PZimmermann/papers/glibc239-20240215.pdf)).
  - The largest known double errors for the Apple Math Library 14.0 (Darwin 23.0.0, M1): sin 0.944, cos 0.948, exp 0.521, pow 0.757, atan2 0.747, hypot 1.21 ulp. None is correctly rounded (0.5 ulp). For musl 1.2.4: sin 0.831, cos 0.834, exp 0.511, pow 0.525, atan2 1.55, hypot 1.04 ulp (Tables 6 and 7 of the same paper).
  - A Rust user saw `ln_1p` differ between Linux and macOS, and found the `libm` crate gave reproducibility ([users.rust-lang.org](https://users.rust-lang.org/t/f64-ln-1p-platform-specific-behavior/77113)).
  - Microsoft's C++ team: library maintainers changing a function "can shift the least significant bits". Under correct rounding "the output bit pattern for any given input is mathematically unique" ([Bringing Correctly Rounded Math to Production with LLVM-libc, 2026-09-16](https://devblogs.microsoft.com/cppblog/bringing-correctly-rounded-math-to-production-with-llvm-libc/)).
  - **Not verified**: a documented case of Apple's libm changing results between macOS versions. *(inference)* Since the functions are not correctly rounded and the game resolves them from `libSystem` at run time (section 0), an OS update may legally change their low bits. That would shift replays, and, because level generation uses the same functions, possibly the levels a (seed, level) pair builds.
- **FMA contraction.** Since clang 14 the C/C++ default is `-ffp-contract=on`, and one poster saw `x1*y2 - x2*y1` become `fma(x1, y2, -x2*y1)` on an Apple M1 ([LLVM discourse](https://discourse.llvm.org/t/fp-contraction-fma-on-by-default/64975)). This matters for any C compiled into the game, not for Rust (3.5). Box2D disables it with `-ffp-contract=off` ([Box2D](https://box2d.org/posts/2024/08/determinism/)).
- **Fast-math.** Catto: "never use this setting" ([Box2D](https://box2d.org/posts/2024/08/determinism/)).
- **x87.** It matters only on 32-bit x86. Rust's RFC notes x87 return values and i586 internal precision ([RFC 3514](https://rust-lang.github.io/rfcs/3514-float-semantics.html)). Apple Silicon has no x87.
- **Denormals.** Rust's float spec assumes "default exception handling without traps, without abruptUnderflow/flush-to-zero" ([RFC 3514](https://rust-lang.github.io/rfcs/3514-float-semantics.html)). Unity's Burst deterministic mode instead flushes "subnormal … numbers … to zero on all platforms" ([Unity Burst](https://docs.unity.com/en-us/engine/6000.7/manual/programming-environment/script-compilation/burst/compilation/float-precision-determinism)). **Not verified**: the default FPCR.FZ state of a macOS arm64 process.

### 3.5 Rust specifics

- **Basic operations are exact and not contracted.**
  - RFC 3514: `+ - * / % sqrt mul_add`, casts and comparisons "produce results that exactly match IEEE 754-2008", with non-determinism only in NaN bits ([RFC 3514](https://rust-lang.github.io/rfcs/3514-float-semantics.html)).
  - On whether rustc may turn `x*y + z` into an FMA: "Currently no" (scottmcm, [users.rust-lang.org](https://users.rust-lang.org/t/floating-point-does-same-formula-with-same-inputs-give-same-result/100398)). An RFC to "allow floating-point operations to provide extra precision … as an optimization" was closed ([rust-lang/rfcs#2686](https://github.com/rust-lang/rfcs/pull/2686)).
  - Reordering is opt-in only, through unstable "algebraic" intrinsics that emit `reassoc … contract`. The plain loop in that PR ran in 84 µs against 10 µs for the algebraic one ([rust-lang/rust#136457](https://github.com/rust-lang/rust/pull/136457)).
- **Library functions are documented as non-deterministic.**
  - The docs for `f64::powi`, `sin`, `cos`, `exp`, `hypot`, `atan2` and others say: "The precision of this function is non-deterministic. This means it varies by platform, Rust version, and can even differ within the same execution from one invocation to the next." `hypot` and `atan2` "currently correspond to the … from libc on Unix". By contrast, `mul_add` and `sqrt` are "guaranteed to be the rounded infinite-precision result … guaranteed not to change" ([std f64](https://doc.rust-lang.org/std/primitive.f64.html)).
  - The cause is optimization. `2f64.powi(-1074)` printed `0e0` in debug and `5e-324` in release, because "LLVM performs constant folding of this intrinsic using the host libm" ([rust-lang/rust#71355](https://github.com/rust-lang/rust/issues/71355)). Two consecutive `f32::sin` calls on one value differed when one was folded at compile time ([#109118](https://github.com/rust-lang/rust/issues/109118)). PR #124609 then documented "arbitrary nondeterministic precision" ([#124609](https://github.com/rust-lang/rust/pull/124609)).
- **The `libm` crate.**
  - It is "a port of MUSL's libm to Rust" ([README 0.2.4](https://docs.rs/crate/libm/0.2.4/source/README.md)), now maintained in compiler-builtins as "fallback implementations for Rust's float math functions in `core`" ([compiler-builtins/libm](https://github.com/rust-lang/compiler-builtins/tree/main/libm)).
  - Its changelog shows bit-changing fixes and ports between releases, such as the CORE-MATH `cbrt` in 0.2.12 and an `fma` zero-sign fix ([CHANGELOG](https://docs.rs/crate/libm/0.2.16/source/CHANGELOG.md)). Pin the version.
  - Bevy's `libm` feature exists "to ensure consistent outputs across platforms at the cost of losing hardware-level optimization using intrinsics" ([bevy_math Cargo.toml](https://github.com/bevyengine/bevy/blob/main/crates/bevy_math/Cargo.toml)). Bevy adds `squared()`/`cubed()` to cover "the lack of a `libm` counterpart to `f32::powi`" ([bevy_math ops](https://docs.rs/bevy_math/0.17.3/src/bevy_math/ops.rs.html)).
  - Speed: the only measurement I found is on a Cortex-M4F, where `libm::sin` was about 2× slower than newlib's (45.1 s against 21.3 s for a million calls) ([rust-lang/libm#246](https://github.com/rust-lang/libm/issues/246)). A maintainer noted CORE-MATH and LLVM libc offer correctly rounded functions "with competitive performance", and that Rust libm's `expf` and `log2f` lag upstream musl ([compiler-builtins#850](https://github.com/rust-lang/compiler-builtins/issues/850)). **Not verified**: `libm` against Apple's libm on Apple Silicon. I found no benchmark and ran none (the brief forbids builds).
- **Correctly rounded options.** The `core-math` crate "provides a Rusty API to the CORE-MATH library", "a correctly rounded mathematical library in C". It covers sin, cos, tan, atan2, exp, pow, hypot and more, and claims its speed "is competitive with the most popular C libraries, even faster most of the time" ([docs.rs core-math](https://docs.rs/core-math)). CORE-MATH offers binary64 sin, cos, atan2, pow, exp and hypot under MIT, with seven binary64 functions integrated into glibc 2.43 ([CORE-MATH](https://core-math.gitlabpages.inria.fr/)). *(inference)* Correct rounding is the only property that makes results independent of the implementation's version: any correct future version returns the same bits.
- **Enforcement.** Clippy's `disallowed-methods` takes fully qualified paths with an optional `reason` and `replacement` ([Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)). It can ban `f64::sin` and its relatives in simulation crates.

### 3.6 What engines and studios recommend

- **Factorio.** CRC every tick in a special mode, a human-readable save every tick to diff, and "heavy mode", which "saves and reloads the game each tick, and compares a CRC", run on test servers after each commit in about 400–600 s ([FFF #47](https://factorio.com/blog/post/fff-47), [FFF #315](https://factorio.com/blog/post/fff-315)). Wube found sin and cos differed across Windows, Linux and macOS, and wrote "their own custom software trigonometry routines" ([Microsoft C++ blog](https://devblogs.microsoft.com/cppblog/bringing-correctly-rounded-math-to-production-with-llvm-libc/); the Factorio forum post it rests on refused automated access, so this is secondhand).
- **Box2D v3.** No random numbers; no fast-math; FMA contraction off; custom sin/cos/atan2; `sqrtf` used directly. A CI test on MSVC, Clang, GCC, x64 and ARM checks the sleep step (310) and a transform hash (0x5e70e5fe). Catto reports no noticeable performance cost, and chose floats over fixed point ([Box2D](https://box2d.org/posts/2024/08/determinism/)).
- **Rapier.** "Locally deterministic" by default. Cross-platform determinism needs the `enhanced-determinism` feature, IEEE 754-2008 targets, and nalgebra's `ComplexField`/`RealField` for trig instead of native methods ([Rapier determinism](https://rapier.rs/docs/user_guides/rust/determinism/)).
- **Unity Burst.** `FloatMode.Deterministic` uses "deterministic implementations of math functions", disables "certain floating-point optimizations", flushes subnormals, and is "only supported on 64-bit architectures". "Disabling some optimizations may negatively impact the performance" ([Unity Burst](https://docs.unity.com/en-us/engine/6000.7/manual/programming-environment/script-compilation/burst/compilation/float-precision-determinism)).
- **Photon Quantum.** The same build, assets and config; optional recorded checksums; input history ([Quantum](https://doc.photonengine.com/quantum/v3/manual/replay)). **Not verified**: Quantum's fixed-point `FP` type, because the manual page I tried returned 404.
- **GGPO.** The simulation "must be fully deterministic", the state fully serializable, and the game able to "load, save, and step one frame without rendering". A SyncTest session runs "a 1 frame rollback for every frame" and compares the states ([GGPO](https://github.com/pond3r/ggpo/blob/master/doc/DeveloperGuide.md)).

## 4. File size

### 4.1 Quantize at the source

Record exactly what the simulation consumed:
- **Doom** writes the 4-byte command, then reads it back into the live command ([g_game.c](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1506-L1523)).
- **Chocolate Doom** rounds the turn before use and carries the residual into the next tic ([chocolate-doom](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/g_game.c#L633-L651)).
- **libTAS** stores the mouse as signed integers and the frame time as `num:den` ([libTAS](https://clementgallet.github.io/libTAS/guides/format)).
- **Trackmania** stores steering as raw integers in a ±65536 range, per the investigators' conversion ([donadigo](https://donadigo.com/tmx1)).

### 4.2 Delta, run-length and variable-length coding

- **Fiedler's lockstep** inputs are "6 bits"; a repeat costs 1 bit and a change 7 ([Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/)).
- **Celeste TAS** lines are run-length by construction ([Input File](https://github.com/EverestAPI/CelesteTAS-EverestInterop/wiki/Input-File)).
- **osu!** stores the time since the previous action ([osr](https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29)).
- **Protobuf** varints take "between one and ten bytes, with small values using fewer bytes", and ZigZag maps signed values with `(n << 1) ^ (n >> 31)` ([protobuf encoding](https://protobuf.dev/programming-guides/encoding/)).
- **Snapshot compression.** Fiedler shrank state from 17.37 Mbps by bounding and quantizing (512 values per metre, "smallest three" quaternions in 29 bits), delta-coding against an acknowledged baseline, and sending only changed objects, down to about 15 kbps for a resting scene ([Snapshot Compression](https://gafferongames.com/post/snapshot_compression/)). This is relevant to keyframes, should the game ever store them.

### 4.3 General-purpose compression

zstd's README, on the Silesia corpus with an i7-9700K, lists zstd -1 at ratio 2.896, 510 MB/s compression and 1550 MB/s decompression; zlib -1 at 2.743, 105 and 390 MB/s; lz4 at 2.101, 675 and 3850 MB/s. Small data compresses poorly without a trained dictionary ([zstd](https://github.com/facebook/zstd)). On this game's test recording (section 0), deflate cut quantized varints from 31 to 18 KB/min and xz to 15. Most of the win came from quantizing, not from the compressor.

### 4.4 Typical sizes

- **Doom**: about 8.2 KiB/min per player, computed from the code (section 1.1).
- **StarCraft II**: "average about 100kb for a 15 minute game" (Liquipedia, per a search snippet; [the page](https://liquipedia.net/starcraft2/Replay_Features) refused automated access, so **not verified** directly).
- **Rocket League**: "a 1.5 MB replay" ([rattletrap](https://hackage.haskell.org/package/rattletrap-13.0.3/docs/README.md)), match length not stated.
- **Counter-Strike 2**: one write-up started from a "300MB demo file" and reduced its extracted positions from 355 MB of naive JSON to 58 MB delta-encoded and 27 MB as protobuf ([healeycodes](https://healeycodes.com/compressing-cs2-demos)).
- **MotoGP**: 1 MB covered a 3-lap race with keyframes every 10 s ([Hargreaves](https://www.shawnhargreaves.com/blog/motogp-replays.html)).
- **Braid**: 30–60 minutes in 40 MB of state ([GDC Vault](https://gdcvault.com/play/1012210/The-Implementation-of-Rewind-in)).
- **Not verified**: per-minute sizes for Halo films, osu!, Trackmania and Overwatch.

## 5. Desync detection and verification

**Checksums in the stream.**
- Doom checks a consistency value every tic in net games ([g_game.c](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L680-L693)).
- Age of Empires checksummed every system ([1500 Archers](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).
- SupCom hashes the whole state once per second ([Tale of Desyncs](https://www.forrestthewoods.com/blog/synchronous_rts_engines_and_a_tale_of_desyncs/)).
- Factorio's replay checks a CRC every tick ([FFF #47](https://factorio.com/blog/post/fff-47)).
- Quantum's checksums are optional and logged on mismatch ([Quantum](https://doc.photonengine.com/quantum/v3/manual/replay)).
- Trackmania's ghost doubles as the check ([donadigo](https://donadigo.com/tmx1)).
- CelesteTAS uses `Assert` ([Commands](https://github.com/EverestAPI/CelesteTAS-EverestInterop/wiki/Commands)).

**Finding the cause.**
- SupCom: "a binary search of printf-ing the current memory hash as the state is walked" ([Tale of Desyncs](https://www.forrestthewoods.com/blog/synchronous_rts_engines_and_a_tale_of_desyncs/)).
- Factorio: compare human-readable saves at the first bad tick; "the difference is usually very small - typically just a value of one variable" ([FFF #47](https://factorio.com/blog/post/fff-47)).
- Age of Empires: "Giant (50MB) message traces and world object dumps", and per-system sync macros compiled out of release builds ([1500 Archers](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).
- GGRS (a Rust GGPO-style library) made SyncTest "report all mismatched frames, not just the first one" ([ggrs#66](https://github.com/gschup/ggrs/pull/66)).

**CI.** Box2D checks its determinism hash on every pull request across compilers and architectures ([Box2D](https://box2d.org/posts/2024/08/determinism/)). Factorio runs heavy mode after each commit on dedicated machines ([FFF #315](https://factorio.com/blog/post/fff-315)). This repository already has `crates/backrooms/tests/replay.rs`, which records a bot run and replays it (per the module comment in `crates/game/src/debug/replay.rs`).

**Versioning to the build.**
- Doom: a version byte, refused on mismatch ([g_game.c](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1589-L1611)).
- Quake III: the protocol version in the file name ([cl_main.c](https://github.com/id-Software/Quake-III-Arena/blob/master/code/client/cl_main.c#L248-L330)).
- Dolphin: the git revision SHA-1 and the game MD5 ([DTM](https://tasvideos.org/EmulatorResources/Dolphin/DTM)).
- osu!: the game version and beatmap MD5 ([osr](https://osu.ppy.sh/wiki/en/Client/File_formats/osr_%28file_format%29)).
- libTAS: the game MD5 ([libTAS](https://clementgallet.github.io/libTAS/guides/format)).
- StarCraft II: per-build decoders ([s2protocol](https://github.com/Blizzard/s2protocol)).

**What to do on mismatch.**
- SupCom and Age of Empires stop the game ([Tale of Desyncs](https://www.forrestthewoods.com/blog/synchronous_rts_engines_and_a_tale_of_desyncs/), [1500 Archers](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).
- StarCraft II loads the matching old build ([Blizzard](https://news.blizzard.com/en-us/article/10495761/update-der-replay-funktion-in-2-0-10)).
- Unreal adapts versioned fields ([Unreal](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-the-replay-system-in-unreal-engine)).
- Overwatch and Fortnite expire replays with the patch ([Overwatch](https://overwatch.blizzard.com/en-us/news/23000187), [Fortnite](https://dev.epicgames.com/documentation/en-us/fortnite/replays-feature-in-fortnite-creative)).
- Factorio refuses replays from older versions ([wiki](https://wiki.factorio.com/Replay_system)).
- Forrest Smith on recovery: "Man months of time could be devoted to developing a recovery system, or you could just fix the desyncs" ([Sync Harder](https://www.forrestthewoods.com/blog/synchronous_rts_engines_2_sync_harder/)).

## 6. Player-facing UX

**Recording controls.**
- Fortnite records when **RECORD REPLAYS** is on in settings ([Fortnite Creative](https://dev.epicgames.com/documentation/en-us/fortnite/replays-feature-in-fortnite-creative)).
- Factorio's "Record replay" must be checked when the game is started ([wiki](https://wiki.factorio.com/Replay_system)).
- Quake III prints "recording to %s." and warns when its sync setting is off ([cl_main.c](https://github.com/id-Software/Quake-III-Arena/blob/master/code/client/cl_main.c#L248-L330)).
- Doom ends recording with `q` ([g_game.c](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/g_game.c#L1506-L1510)).
- StarCraft II turns a game into a replay after the fact, with "Rewind" on the Victory/Defeat dialogue or "Quit and Rewind" mid-match ([Blizzard](https://news.blizzard.com/en-gb/article/20719765/patch-3-13-0-preview-rewind)).
- Unreal's memory streamer keeps a running length for instant replays ([DemoNetDriver](https://dev.epicgames.com/documentation/en-us/unreal-engine/demonetdriver-and-streamers-in-unreal-engine)).
- **Not verified**: a sourced convention for an on-screen recording indicator in games.

**The replay browser.**
- Fortnite's list sits under Career > Replays, with **PLAY**, **DELETE**, **RENAME AND SAVE**, **OPEN REPLAY FOLDER** and **REFRESH** ([Fortnite Creative](https://dev.epicgames.com/documentation/en-us/fortnite/replays-feature-in-fortnite-creative)).
- Overwatch keeps "your 10 most recent matches" in the Player Profile ([Overwatch](https://overwatch.blizzard.com/en-us/news/23000187)).
- Halo 3 keeps up to 100 films, and Reach 1,000 of each file type ([Halopedia](https://www.halopedia.org/Theater)).
- Factorio plays a replay from Load game > "Start replay" ([wiki](https://wiki.factorio.com/Replay_system)).

**Playback controls.**
- Factorio: pause/play, slower, faster, and "the top bar shows the position in the replay out of the total time" ([wiki](https://wiki.factorio.com/Replay_system)).
- Overwatch: pause with Ctrl+P, speed with the wheel, first/third-person, free and overhead cameras, camera bookmarks ([Overwatch](https://overwatch.blizzard.com/en-us/news/23000187)).
- Fortnite: a progress bar to scrub, speed, five camera modes, and a hide-controls toggle ([Fortnite Creative](https://dev.epicgames.com/documentation/en-us/fortnite/replays-feature-in-fortnite-creative)).
- Insurgency: Sandstorm: F toggles a panel with a timeline scrubber and pause, Home/End change speed, Space cycles cameras, H hides the interface ([Saber support](https://support.saber.games/hc/en-us/articles/22304028093073-Understanding-the-Replay-System)).
- Unreal: `demopause`, `demospeed`, `demoscrub` ([Unreal](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-the-replay-system-in-unreal-engine)).
- StarCraft II: "Take Command" from any point ([GosuGamers](https://www.gosugamers.net/news/22662-resume-from-replay-coming-to-beta-tomorrow)).
- Halo: a free camera on Y ([Halopedia](https://www.halopedia.org/Theater)).

**While seeking.**
- MotoGP cleared particles and skidmarks, restored the keyframe and "ran a single update before rendering" ([Hargreaves](https://www.shawnhargreaves.com/blog/motogp-replays.html)).
- Halo warns of loading delays when speeding through campaign areas ([HBO](https://halo.bungie.org/misc/kamikazegrunts.html)).
- Unreal accepts "slight visual errors" when checkpoint saving is time-sliced ([DemoNetDriver](https://dev.epicgames.com/documentation/en-us/unreal-engine/demonetdriver-and-streamers-in-unreal-engine)).
- **Not verified**: a documented convention for the screen shown during a seek.

## 7. What I could not verify

- **Braid.** The rewind storage scheme beyond the GDC abstract's "30 to 60 minutes" in "40 megabytes".
- **Prince of Persia.** The rewind buffer's design.
- **Super Meat Boy and N++.** How their replays are stored.
- **Overwatch.** The replay internals beyond the talk abstract.
- **Rocket League.** Why its replays are network streams, and its per-minute size.
- **Halo.** Film sizes, and how Reach implemented rewind.
- **StarCraft II.** Liquipedia's 100 KB per 15 minutes (the page refused automated access), and whether Rewind re-simulates or keeps snapshots.
- **Age of Empires II.** How its recorded games seek; the one thread I could open was about AoE IV and held only a player's guess.
- **Factorio's own math functions.** Known only via Microsoft's blog; the Factorio forum refused automated access.
- **The Doom wiki** (refused automated access). Doom facts here come from id's and Chocolate Doom's source instead.
- **The `libm` crate's speed against Apple's libm on Apple Silicon.** No benchmark found, none run.
- **Apple's libm across macOS versions.** Any documented change of results.
- **macOS arm64.** The default FPCR flush-to-zero state of a process.
- **winit on macOS.** Whether `MouseMotion` deltas are integer counts or fractional points.
- **Photon Quantum.** The fixed-point `FP` type page (404).

## What this suggests for this game

1. **Keep input replay as the core, from a full state.** Doom, Age of Empires, SupCom, StarCraft II, Factorio, Halo, Trackmania and osu! all replay inputs over a deterministic simulation (1.1). The game already fits: levels rebuild from (seed, level), one seeded stream, and a per-frame state hash. A state stream (Quake, Unreal, Rocket League) would need a replication layer the game lacks, and it still breaks across patches in Fortnite, Rocket League and Overwatch (1.2). The recorder and `state_hash` stay; what changes is the format, the time and input quantization, and the start state.

2. **Make a save and a replay one container (Quantum, Factorio, StarCraft II, section 2).**
   - A **level-start save** is a replay with zero frames.
   - A **mid-level save**, from play or from any frame of a replay ("take control here"), is the level-start save plus the input frames up to N. Loading it rebuilds the level, applies the save, seeds the stream, and re-simulates N frames headless with the output muted.
   - **A replay's start is literally a save.** On a build mismatch, a replay or mid-level save opens as its level-start save and says what was dropped (Doom refuses silently; tell the player).
   - **Phase 2**, only if cross-build mid-level saves or instant seeks are wanted: a full snapshot serializer, covering `Game`, the 33 thread-locals and the RNG stack (GGPO's hidden-state rule). Its versioned snapshots become the `SNAP` chunk in recommendation 11.

3. **Record always; let the player mark in and out.** Keep the input log from each level start in memory, the way Unreal's memory streamer keeps a running length and StarCraft II's "Quit and Rewind" converts a finished game (2, 6). It is about 15–30 KB a minute, so an hour costs under 2 MB. "Start recording" stores an in-point, which can be retroactive ("save the last 2 minutes"), and "Stop" an out-point. The file carries the inputs from level start, and playback fast-forwards silently to the in-point. A recording that spans levels is a sequence of level segments, each starting from that level's save.

4. **Seek by re-simulating; add keyframes only if measured seeks are slow.** At the benched update p50 of 0.023–0.083 ms, a minute of play re-simulates in about 0.1–0.3 s, so seeking from the level start may be enough (2, *inference*). Add a `--bench` measure of "headless replay frames per second", held in `perf/history.csv` like the others. If a seek ever exceeds about 1 s, keep in-memory snapshots during playback, spaced from a memory budget as MotoGP did and every 30 s as Unreal's default. On a seek: freeze the last frame dimmed with a progress bar, mute, restore, run one update, then show (MotoGP).

5. **Quantize time at the source and derive both clocks from it.** Measure each frame's interval as an integer count of 1/65,536 s ticks. That grid is exact in f64 and its sums are exact (3.1, *inference*). Feed `dt = min(interval, 50 ms)` and a game clock `now_ms = Σ interval` to the simulation, in live play too, so that `set_timeout` no longer reads `Instant` and live play equals its replay by construction. One integer per frame replaces today's two f64s, and delta-coded it takes about 1 byte. A fixed step with interpolation (Fiedler) stays an option for frame-rate-independent behaviour, not a requirement for determinism (`fixed_step_dt` already exists).

6. **Quantize inputs where they enter, as Doom does.** Round each mouse delta to a fixed grid with Chocolate Doom's carry, feed the rounded value to everything (`mouse_move` and the `mouse_dx` accumulators), and record the integer. Do the same for the wheel. Replace key, HUD and menu strings with ids from a table in the header. Write every simulation-affecting setting into the header (`look`, `invert_y`, anything else that changes simulation), and log setting changes as events, because `mouse_move` multiplies by `settings.look` (section 0). Viewport size changes reach `g.resize()`, so record them as events too, or prove they never touch simulation state.

7. **Own the math the simulation and level generation use.**
   - Route the ~3,000 `sin/cos/atan2/powi/powf/exp/hypot` calls, plus `tan`, `asin`, `acos`, `atan`, `exp2`, `log`, `log10`, `cosh`, `tanh` and `fmod` (all in `nm -u`, section 0), through one module.
   - Back that module with a statically linked implementation, so results no longer depend on the macOS `libSystem` a player has (3.4) or on LLVM folding (3.5). Prefer correctly rounded functions (`core-math`/CORE-MATH): their bits cannot change with any correct implementation version. Otherwise use a pinned `libm` crate (Bevy's choice).
   - Replace `powi` with explicit products (Bevy's `squared`/`cubed`).
   - Use `mul_add` only explicitly; rustc never contracts on its own.
   - Ban the std methods in the game crates with clippy `disallowed-methods`.
   - Speed against Apple's libm is unverified. Switch behind a cargo feature and let `cargo xtask bench` judge build time and update p50/p99 before making it the default.

8. **Audit hidden non-determinism once, then lint it.**
   - **Iteration order.** Turn on `clippy::iter_over_hash_type` for simulation code, or replace `HashMap`/`HashSet` with ordered maps where iteration feeds state. The default hasher is randomly seeded per process (3.2).
   - **Thread-locals.** List all 33 `thread_local!`s and mark each as derived from (seed, level), part of the snapshot, or presentation-only.
   - **Drawing.** Prove `draw(&mut Game)` writes no simulation state: replay headless and rendered, and compare hashes (MotoGP's Update/Draw rule).
   - **Sound.** Sound code that draws the stream must run identically when muted.
   - **FP mode.** Assert the FP control register at each level start and in debug every frame (Dawson).

9. **Put checksums in the file and verify while playing (SupCom, Factorio, Quantum, section 5).**
   - Store the four-part `StateHash`, extended with the level-stream trace and `native_count` (a cheap catch-all: most divergences change how many draws happen), every 60 frames. That is about 2 KB a minute.
   - During playback, compare. At the first mismatch, pause and name the frame and the parts: "Diverged at 03:12 (entities). Stop / keep watching unverified / take control from 03:11".
   - A debug flag hashes every frame and dumps state at the first bad frame, the way Factorio diffs saves.
   - In CI, alongside `crates/backrooms/tests/replay.rs`: replay in a fresh process (the hasher seed and thread-locals differ); a golden replay per level that later commits must reproduce, rerecorded on purpose when the simulation changes; and a GGPO-style sync test (save, load, step, compare) once a serializer exists.

10. **Bind replays to the build and say so.** The header carries:
    - the format version;
    - the game version;
    - the git commit and a dirty flag;
    - a simulation-content hash;
    - the rustc version;
    - the macOS version, for diagnosing libm drift until recommendation 7 lands.

    Same build: play verified. Different build: warn, play with verification, and fall back to opening the start save on a mismatch. StarCraft II, Factorio, Overwatch and Fortnite all bind replays to the build (5). Keep the save part of the format migratable, as saves already are (`save.v2`).

11. **Use a documented, chunked binary format, about 15–20 KB a minute** (section 4; measured 276 KB a minute for today's JSON on the test file). Document it in an engineering spec, with a decoder test and a `--dump-replay` that prints it as JSON for debugging. Proposed layout, all little-endian:

    ```
    "OCREPLAY"            8-byte magic
    u16 format, u16 flags
    u32 header_len, u32 header_crc32
    header (UTF-8 JSON, uncompressed, small):
      build {version, commit, dirty, sim_hash, rustc, macos}
      title, created, level, run_seed
      start {kind: "level_start" | "snapshot", save: <SaveData, the same serializer as the save file>,
             native_rng: [s0, s1], frame: 0}
      settings {look, invert_y, ...}       tick = 1/65536 s
      tables {keys: [...], hud: [...], menu: [...]}
      in_frame, out_frame, frames, hash_every: 60
    chunks, each: 4-byte tag, u32 first_frame, u32 count, u32 len, u32 crc32, body
      FRMS  deflate(per frame: zigzag-varint Δinterval_ticks, varint n_events,
                    events: u8 op + payload; look = zigzag-varint dx, dy in grid units)
      HASH  u32 frame + 4 × u64 StateHash parts + u64 rng draws    (every 60 frames)
      LEVL  a level boundary: the next level's start save            (multi-level recordings)
      SNAP  deflate(snapshot)                                          (Phase 2 only)
      TELE  deflate(today's telemetry rows / director log)             (optional, playtests)
    INDX  (tag, first_frame, offset) per chunk; final StateHash; u32 index offset as the last 4 bytes
    ```

    - **Chunks** of about 600 frames (10 s) can be appended and flushed while recording, so a crash loses at most one chunk, and a seek can find its chunk through the index.
    - **deflate** comes from `miniz_oxide`, already a dependency.
    - **A save file** is this container with zero frames, or with frames up to its resume point.

12. **Player UX** (section 6).
    - **Recording.** A rebindable key and a pause-menu item "Start/Stop recording", with a small unobtrusive REC mark in a HUD corner and a toast on start, stop and save. *(inference)* A diegetic camcorder-style mark may suit a horror game better than a red dot.
    - **The browser.** A **Replays** screen off the main menu (Fortnite, Overwatch), listing title, level, date, length and a compatibility badge for this build or another, with Play, Rename, Delete and Show in Finder.
    - **Playback.** Pause, speed 0.25×–8×, ±10 s skips, and a timeline with markers from the director's log (deaths, encounters, level changes) that can be dragged to seek. Add "Take control here", which is the save-from-replay of recommendation 2.
    - **Camera.** Locked to the recorded view by default. A free camera is a design choice: it can reveal how the horror is staged.
