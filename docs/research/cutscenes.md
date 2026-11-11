# Research: cutscenes in a first-person horror game

Gathered on 2026-11-11 by a research agent for the cutscene engine
([`../CUTSCENES.md`](../CUTSCENES.md)) and kept whole, as the record of
what was found; [`../SOURCES.md`](../SOURCES.md) section 17 sums up what
the design took from it and where it shows up. Every claim links a page
the agent opened; *(inference)* marks its own conclusions and **Not
found** what it searched for and could not source. Read a source before
relying on one of its details.

Bracketed numbers link to pages I opened. *(inference)* = my conclusion; **Not found** = searched, no openable source.

## 1. Data model of sequencers

**Unreal Sequencer.** Tracks hold sections, and sections hold keys. Level Sequence Actor settings [1]:
- Loop.
- *Restore State*: "restores all actors to their previous state before the sequence started".
- *Pause at End*: actors keep their final positions.
- *Disable Movement Input* / *Disable Look At Input*.
- *Hide Player*, *Hide HUD*.
- *Disable Camera Cuts*: the sequence never takes the camera.

Each section's *When Finished* defaults to Restore State, or can be *Keep State*. The actor-level Restore State overrides the sections [2].

Camera Cut track: *Can Blend* is off by default. Once on, dragging a section's corner blends in from gameplay or out to it. *Lock Previous Camera* "locks the last used camera as the view to blend from" [3].

Epic's hand-back recipe [4]:
- Add blend-out regions on the camera-cut and animation sections, so the character doesn't pop.
- Set the transform section to Keep State.
- To return control early, fire an Event-track key that calls `Set Cinematic Mode` (Affects Movement/Turning).
- Fix the camera's facing with `Set Control Rotation`.
- Turn off *Constrain Aspect Ratio* to avoid "abrupt field-of-view changes".

Sequencer's frame rate does not throttle the engine tick. *Lock to Display Rate at Runtime* forces the engine to tick at that rate [5].

**Unity Timeline.** Wrap modes [6]:
- *Hold*: plays once and holds the last frame.
- *Loop*: repeats until interrupted.
- *None*: "resets all animated properties to the values they held before playback".

Update methods are DSP, Game Time, Unscaled and Manual [6]. Signals: a reusable *Signal Asset* links an *Emitter* marker to a *Receiver*, so the timeline never references gameplay code [7]. Clips ease in and out on a curve by default [8].

Timeline overrides the Cinemachine Brain while it runs. Afterwards "control returns to the Cinemachine Brain", and a shot's Ease In/Out blends against the live gameplay camera [9]. Blend styles include Ease In Out, Hard In/Out and Linear ("mechanical-looking") [10]. The default in the source code is `EaseInOut, 2f` [11].

**Valve VCD.** A scene holds actors, actors hold channels, and channels hold events with start and end times (-1 marks a one-shot event). Event types [13]:
- Performance: Expression, LookAt, MoveTo, Speak, Gesture, Sequence, FireTrigger, SubScene, Camera.
- Flow control: Loop, Interrupt, StopPoint, PermitResponses.

Events carry flags (ResumeCondition, LockBodyFacing) and ramp curves of (time, value, curve type) keys, including ease, Catmull-Rom and Hold. Scenes store `fps` (default 60) in text and binary forms, splitting authoring from runtime [13].

**Frictional HPL2 (custom engine).** It uses script primitives instead of a timeline [14]:
- `SetPlayerActive`
- `StartPlayerLookAt(entity, speedMul, maxSpeed, atTargetCallback)`
- `MovePlayerHeadPos(x, y, z, speed, slowDownDist)`
- `FadePlayerRollTo(angle, speedMul, maxSpeed)`
- `FadeOut/FadeIn(time)`

**Not found:** text write-ups from Naughty Dog, Guerrilla, Remedy or id/Bethesda (GDC Vault: abstracts only).

## 2. First-person takeovers and seamless transitions

- **Valve.** Half-Life 2 never takes the camera; it steers the eye with the world (scripted birds, an obelisk drawing eyes to the Citadel) [16]. "Players never have control wrested away", and conversations keep "free player movement and camera control" [17].
- **Frictional (SOMA).** "We do not force players to notice events by use of cutscenes". The player keeps "full control" even when that "might break the experience a bit" [18]. HPL2's middle ground: a speed-capped look-at with a callback when on target, a head offset that slows near its end, and a roll fade [14].
- **God of War (2018).** The one-shot camera used "six to eight tricks" [19] and was built in the cinematic pipeline [20]. **Not found:** blend durations or hand-back details.
- **Hand-back** *(inference from [3][4][9])*: capture the live gameplay camera at the start, blend out to the *current* gameplay camera, set the player's yaw and pitch to match the final view, and keep the FOV unchanged.
- **Title screens.**
  - God of War (2018) and Ragnarök title screens "lead directly into a new game". Xenoblade 2 goes "directly into the opening". Bayonetta Origins starts when the player opens an in-world storybook [21].
  - Braid's menu sits in its world; Journey's blends into the intro. [22] advises skipping the menu on first launch and, if play starts immediately, to "leave a safe zone or wait for input".
  - **Not verified:** Halo, TLOU2, Inside, Death Stranding, Firewatch, RE7, Titanfall 2.

## 3. Signalling that controls are unavailable

- **Engine switches.** Hide HUD, Hide Player and the input locks [1]; `Set Cinematic Mode` with Affects HUD/Movement/Turning [4].
- **Letterbox.** The Evil Within letterboxed its gameplay as "intended", then after complaints added "Letterbox bar removal" [29].
  - On a 16:9 screen, 2.35:1 gives bars of about 12.2% each and 2.39:1 about 12.8% *(arithmetic)*.
  - **Not found:** slide-in timing, the history of the convention, desaturation cues.
- **Frustration.**
  - Losing control mid-play "completely negated" immersion, like "an actor looking at the camera" [44].
  - Non-interactive death scenes turn players into spectators [33].
  - Players skip repeats to "get back to the action"; studios use hold-to-skip, some block first-view skips [30].
  - Dragon Quest XI S shows "Hold to skip" and has an Autoplay Cutscenes toggle [24].
- **Accessibility.**
  - XAG 105: players should be able to pause "cinematics with audio"; events under 3 s are exempt [23].
  - XAG 104: subtitles for all speech, enabled "before the first cinematic plays"; at most 40 characters per line and 2 lines; scalable to at least 200% [26].
  - GAG [27]:
    - "Allow all narrative and instructions to be replayed".
    - Make subtitles available "before any sound is played".
    - "Avoid any sudden unexpected movement or events".

## 4. Motion comfort

- **Forsyth (Oculus, 2013).** "Generally bad": changing head orientation "without user input", changing FOV, "unprovoked translation", freezing the view "during cinematics" [31]. VR rules; flat screens share the mismatch, weaker [25].
- **Meta.**
  - Keep accelerations brief and infrequent.
  - Predictable motion is more comfortable, and losing sight of the horizon has the same effect as reading in a moving car.
  - Reference speeds: 1.4 m/s walking, 2.8 m/s running [32].
- **GAG VR.** Keep the camera "in full control of the player", avoid acceleration, keep "a constant horizon", snap-turn in 30° steps, and keep latency under 20 ms [28].
- **XAG 117.** Shake, head bob and blur should be avoidable or able to be turned off. Also offer an adjustable FOV and a way to "disable automatic camera movement". Sea of Thieves exposes its auto-centre speed (180) [25].
- **Engine practice.** HPL2 caps rotation with `maxSpeed`; roll fade defaults to "really slow" [14]. Cinemachine defaults to 2 s Ease In Out [11].
- **Not found:** a sourced maximum turn speed in °/s for flat-screen cutscenes.

## 5. Death in first-person horror

- **Grip (Frictional).**
  - After a death, "the situation has lost its sense of fear", and retries "spoil those initial memories".
  - His fix is that "death is postponed", and repetition is broken: "in Amnesia we changed the map a bit after each death" [33].
  - Amnesia's late "waking up" mechanic "made people more immersed" [34].
  - Rebirth places a caught player "slightly ahead of the encounter" [35].
- **Red Barrels (Outlast 2).** "The fifth time you replay the same sequence, it's just not scary anymore" [36].
- **The Callisto Protocol.** A 2022 patch made death animations skippable, apparently because they "go on for just a little bit too long" [37].
- **Alien: Isolation.**
  - Distinct death animations, such as a single bite [39].
  - Creative Assembly rejected keeping the alien in place on reload, which causes "an infinite loop of dying and respawning", and also rejected removing it [38].
- **Engine support (HPL2).** The `CheckPoint` callback receives a death count (0 on the first load), which makes shorter variants on repeat easy. There are also `SetDeathHint` and `FadeOut` [14].
- **Not found:** sources on camera-fall or roll-to-floor staging.

## 6. Testing

- **Sample by time.**
  - Variable delta time makes behaviour depend on it; use fixed steps, an accumulator and interpolation [41].
  - Unity's Manual mode has you "call PlayableGraph.Evaluate with your own deltaTime" [12].
  - Unreal notes that the time step grows when frames take longer [5].
- **Golden images.** Unreal's Screenshot Comparison checks each capture against a "ground truth" image [40]:
  - Default tolerance is Low (TAA adds per-pixel variation); global and local error limits.
  - "Disable Noisy Rendering Features" turns off AA, motion blur, SSR and auto-exposure.
- **Skip equals full play.**
  - Fast-forwarding with timeScale made objects tunnel through colliders.
  - Setting "everything to the final keyframe state" makes skipping "trivial" [42].
  - The When Finished setting defines the end state [2].
- **Not found:** studio write-ups on automated cutscene tests.

## Design rules this suggests

1. **Cutscene = data, evaluated as a pure function of time.** Keyed curves with explicit interpolation, events and bindings [6][13].
2. **Every track declares an end policy** (restore or keep), with a sequence-level override [1][2].
3. **Skip = evaluate t_end, apply the end policies, fire any events not yet fired, in order.** Never fast-forward the simulation [42].
4. **Hand back to the live gameplay camera.** Set the player's yaw and pitch to the final view; keep the FOV [3][4][9][31].
5. **Prefer partial takeovers.** A capped-speed look-at with an on-target callback beats a full cut [14][16][18].
6. **Ease and cap every camera move.** About 2 s Ease In Out for hand-backs; no linear blends; brief accelerations; no roll outside deliberate death beats [10][11][28][32].
7. **Signal only full takeovers** by hiding the crosshair and HUD; never letterbox gameplay [1][29].
8. **Hold-to-skip; pause anything ≥ 3 s; subtitles before the first cinematic** [23][24][26][30].
9. **Honour comfort settings in cutscenes:** FOV, auto-camera, shake and bob [25][27].
10. **Keep deaths short.** Show the full version once, then a shorter one keyed on a death counter; change something after each death; never respawn the threat on the player [14][33][34][36][37][38].
11. **Fixed-step tests.** The same t at 30, 60 and 144 fps must give identical poses [5][12][41].
12. **Golden frames at named keys** with noise features off, plus non-image checks *(inference)*: the hand-back pose equals the gameplay pose, input is restored, and state after a skip equals state after full play [2][40][42].

## Sources

[1]: https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-sequencer-movie-tool-overview
[2]: https://dev.epicgames.com/documentation/unreal-engine/what-happens-when-my-cinematic-ends-in-unreal-engine
[3]: https://dev.epicgames.com/documentation/en-us/unreal-engine/cinematic-camera-cut-track-in-unreal-engine
[4]: https://dev.epicgames.com/documentation/en-us/unreal-engine/blend-gameplay-animation-to-cinematic-animation-in-unreal-engine
[5]: https://dev.epicgames.com/documentation/en-us/unreal-engine/setting-your-display-rate
[6]: https://docs.unity3d.com/cn/2018.4/Manual/class-PlayableDirector.html
[7]: https://docs.unity3d.com/Packages/com.unity.timeline@1.8/manual/wf-signals.html
[8]: https://docs.unity3d.com/Packages/com.unity.timeline@1.2/manual/clp_ease.html
[9]: https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/concept-timeline.html
[10]: https://docs.unity3d.com/Packages/com.unity.cinemachine@3.1/manual/CinemachineBlending.html
[11]: https://raw.githubusercontent.com/Unity-Technologies/com.unity.cinemachine/main/com.unity.cinemachine/Runtime/Behaviours/CinemachineBrain.cs
[12]: https://docs.unity.com/en-us/engine/current/script-reference/unityengine/playables/directorupdatemode/manual
[13]: https://srctools.readthedocs.io/en/latest/modules/choreo.html
[14]: https://oldwiki.frictionalgames.com/hpl2/amnesia/script_functions
[16]: https://gamedeveloper.com/design/how-designers-turn-heads
[17]: https://www.bfi.org.uk/features/half-life-2-20-years-valve-shooter
[18]: https://frictionalgames.blogspot.com/2013/12/the-five-foundational-design-pillars-of.html
[19]: https://gamedeveloper.com/production/barlog-i-god-of-war-i-s-single-shot-camera-trick-was-a-tough-sell-for-devs
[20]: https://www.gdcvault.com/play/1026025
[21]: https://www.superjumpmagazine.com/the-unappreciated-beauty-of-the-title-screen/
[22]: https://www.gamedeveloper.com/design/how-to-create-immersive-game-intros
[23]: https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/105
[24]: https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/116
[25]: https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/117
[26]: https://devdocs.xbox.com/gaming/accessibility/xbox-accessibility-guidelines/104
[27]: https://gameaccessibilityguidelines.com/full-list/
[28]: https://gameaccessibilityguidelines.com/avoid-vr-simulation-sickness-triggers/
[29]: https://www.kitguru.net/gaming/jon-martindale/bethesda-fixes-up-evil-within-pc-fps-black-bar-issues/
[30]: https://fredzed.substack.com/p/why-do-players-skip
[31]: https://developers.meta.com/vr/blog/vr-sickness-the-rift-and-how-game-developers-can-help/
[32]: https://developers.meta.com/horizon/design/locomotion-comfort-usability/
[33]: https://frictionalgames.blogspot.com/2014/05/alien-isolation-and-two-hardest_5.html
[34]: https://gamedeveloper.com/business/gdc-europe-i-amnesia-i-s-grip-delivers-terrifying-tales-of-immersion
[35]: https://dreadcentral.com/editorials/457052/the-changing-face-of-death-in-the-amnesia-series
[36]: https://www.vice.com/en/article/how-a-horror-game-about-death-expanded-its-audience-by-letting-them-live
[37]: https://www.nme.com/news/the-callisto-protocol-now-lets-you-skip-its-gruesome-death-animations-3370695
[38]: https://gamedeveloper.com/design/game-design-deep-dive-the-save-system-of-i-alien-isolation-i-
[39]: https://www.destructoid.com/alien-isolation-is-haunting-and-uncompromisingly-scary/
[40]: https://dev.epicgames.com/documentation/en-us/unreal-engine/screenshot-comparison-tool-in-unreal-engine
[41]: https://gafferongames.com/post/fix_your_timestep/
[42]: https://discussions.unity.com/t/ive-given-up-on-allowing-player-to-skip-certain-cutscenes/739274
[44]: https://gamedeveloper.com/design/emotion-frustration-loss-of-control
