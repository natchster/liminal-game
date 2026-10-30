# Research: real windows, their glass, and a view beyond them

Research note, 2026-10-30, for the brief "remake windows in Backrooms zones, making them more realistic, and show a creepy landscape beyond the window" (the entity that crosses the landscape is researched separately). Every claim links a page I opened. **[est]** marks my own estimate, arithmetic or reading of a photo or drawing, with no source. **[unverified]** marks something I looked for and could not confirm. Photos from Wikimedia Commons are listed with their licence; the CC BY‑SA ones are references only, not assets (the manifest allows MIT, CC0 and CC BY 4.0).

**What the game has today** (`core_props/builders.rs` `build_window`, `core_props/tail.rs` `place_window_at`): a 1.5 × 1.4 × 0.08 m box frame, a 1.6 × 0.06 × 0.14 m sill below it, a cross of muntins 40 mm wide and 30 mm deep, and an unlit plane in front painted with a grey gradient, rain streaks, smears and sometimes four warm dots. The whole group stands 5 cm off a wall, centred 1.55 m up (sill at about 0.80 m, head at about 2.25 m), with a cool point light 0.6 m into the room. Up to 3 a level. There is no opening in the wall, no reveal, no reflection and no parallax. (The working tree as of today also has an uncommitted `Patch::WindowView`, a per-pixel GroundedSkybox, and a `Patch::Glass`. This note does not depend on them.)

---

## 1. The windows the rooms imitate

### 1.1 Types, where they occur, and their numbers

| Type | Where and when | Frame and sash | Glass | Sources |
|---|---|---|---|---|
| **Wood double-hung, historic** (two sashes sliding past each other, the upper one outboard, a parting bead between them) | Houses, hotels, schools, mill offices, up to the mid‑20th c. | Mill sash in Pennsylvania: **48 × 96 in (1.22 × 2.44 m), 12 lights in each sash**. Historic sash **1‑3/8 in (35 mm)** thick, historic meeting rail **1‑1/4 in (32 mm)** face, muntins **3/4 in (19 mm)** wide. The frames were set into the masonry, which hid most of the frame from outside. | Single glazing | [NPS Tech Note 21](https://www.nps.gov/orgs/1739/upload/tech-note-windows-21-replacement-wood-sash.pdf). Parts (sash, stiles, rails, meeting rail, muntins, putty, parting bead, interior stop, pulleys, weights, weight pocket, sill, stool): diagram in [NPS Brief 9](https://www.nps.gov/orgs/1739/upload/preservation-brief-09-wood-windows.pdf) |
| Wood sash, UK Georgian | 18th c. on | Glazing bars **≤ 18 mm**, meeting rails **≤ 22 mm**. "Horns" on the meeting-rail corners are Victorian, absent on early multi-pane sashes. After 1709 London windows had to be **set back 4 in (100 mm)** from the wall face. | Small panes, fewer as glass got cheaper | [Bassetlaw guidance](https://www.bassetlaw.gov.uk/media/3570/guidance-note-sash.PDF) |
| Wood double-hung, modern | Replacement and new units, 1990s on | Marvin Ultimate: frame depth **4‑9/16 in (116 mm)**, crescent cam lock with sash release lever, **block-and-tackle balances**, optional sash lift. **[est]** From the unlabelled section drawing, about 82 mm from the frame edge to the glass at each jamb. Replacement sash: 1‑3/4 in (44 mm), meeting rail 1‑3/4 in. Joinery forum: rails and stiles 45–55 mm, sashes 55–57 mm. Common sizes: **24–48 in wide, 36–72 in tall**, unit ½ in smaller than its rough opening. | Insulating unit **11/16 in (17 mm)**, triple 7/8 in. Options: obscure, frost, reed, rain, glue chip, and bronze, gray or green tints | [Marvin detail manual](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-12/Marvin%20Ultimate%20Wood%20Double%20Hung%20Specifications.pdf), [Tech Note 21](https://www.nps.gov/orgs/1739/upload/tech-note-windows-21-replacement-wood-sash.pdf), [UK Workshop](https://www.ukworkshop.co.uk/threads/making-sash-windows-rail-and-stile-dimensions.115947/), [Feldco](https://www.4feldco.com/articles/standard-double-hung-window-sizes) |
| **Rolled-steel windows**: casement, pivot, **projected (hopper/awning)**, "austral" | US from about 1890 (fire codes after city fires) to after WWII. Projected windows were "perfected in the 1920s for industrial and institutional buildings", upper panels opening out and lower ones in. Austral windows "often used in schools, libraries". Pivots in warehouses and power plants. | **Sections 1/8 in (3 mm) thick, 1–1‑1/2 in (25–38 mm) wide** for sash, frame and subframe. "Almost exclusively found in masonry or concrete buildings." Crittall's Universal section (1909) was replaced by W20 (1965, weather-stripped); galvanised after 1945. | **Wire glass** made them fire-resistant. Glazed with compound and clips; paint laps slightly onto the glass. | [NPS Brief 13](https://www.nps.gov/orgs/1739/upload/preservation-brief-13-steel-windows.pdf), [Designing Buildings](https://www.designingbuildings.co.uk/wiki/Crittall_window) |
| **Aluminium**: double-hung, casement plus hopper, **horizontal sliders** | 1930s–50s on, dominant after WWII | 1920–50 mostly **"mill finish"** (bare metal that darkens to grey). Anodised oxide 0.05–1.5 mil, clear or gold, grey, golden brown or green. Locks and lifts in white metal or chromed bronze. Outside putty or inside glazing beads. A 1960s Chicago tower: "light residential" sliders, "thin aluminum members", single glazed. | Single, later insulating | [NPS Tech Note 22](https://www.nps.gov/orgs/1739/upload/tech-note-windows-22-historic-aluminum-windows.pdf) |
| **Storefront / office framing** (fixed lights between mullions) | Offices, lobbies, 1960s on | Kawneer Trifab 451: **2 in sightline × 4‑1/2 in deep (50.8 × 114.3 mm)**, infill up to 1‑1/8 in (28.6 mm), anodised (4 colours) or painted | Insulating units, up to 1‑1/8 in | [Kawneer](https://www.kawneer.com/products/framing-systems/trifab-versaglaze-451-451t-framing-system) |
| Post-war office "crystal" tower | 1940s–70s | Equitable Building (Portland): "sea green" heat-absorbing sealed double glazing (40% less solar transmission). UN Secretariat: tinted glass, **venetian blinds inside** and 4,000 induction units **beneath the sills**. Induction units are "one unit per window or every other window module". | Tinted | [ASHRAE Journal 1999](https://www.ashrae.org/File%20Library/About/Mission%20and%20Vision/ASHRAE%20and%20Industry%20History/Air-Conditioning-in-Office-Buildings-After-World-War-II.pdf) |

**Glass in the pane**

| Glass | Thickness | Notes | Sources |
|---|---|---|---|
| Single strength | 3/32 in (2.4 mm) | Ordinary sheet | [Brennan](https://brennancorp.com/blog/how-thick-is-window-glass/) |
| Double strength | 1/8 in (3.2 mm) | | [Brennan](https://brennancorp.com/blog/how-thick-is-window-glass/) |
| Polished wired glass | 1/4 in (6.8 mm) | "Square patterned wire mesh", diamond also made; fire-rated to 45 min | [General Glass](https://www.generalglass.com/?p=720) |
| Patterned (obscure) glass | 4–10 mm by design | Rolled glass with a pattern pressed into one face; privacy graded 1–5 | [Pilkington](https://www.pilkington.com/en/gbl/architectural-and-technical-glass/product-categories/decoration/pilkington-texture-glass) |

- **Wired glass.** The wire keeps a cracked pane in its frame, but the glass "is actually weaker than unwired glass" and breaks into worse cuts. The US IBC "effectively banned" it in 2006, so it lingers in old schools, institutional doors and stair halls ([Wikipedia](https://en.wikipedia.org/wiki/Wired_glass)).
- A 1923 PPG catalogue page shows wire glass in nine finishes, mostly with a **hexagonal mesh**: Polished, Rough, Ribbed, Maze, Syenite, Romanesque, Muranese, Pentecor and Factrolite ([Commons, public domain](https://commons.wikimedia.org/wiki/File:Various_styles_of_wire_glass,_from_Glass_Paints_(p._141).jpg)). Being public domain, it could be used directly as a source for patterns.

### 1.2 Heights, reveal and trim

**Heights**
- **Sill (code limits).**
  - A window used for emergency escape may have its sill at most **44 in (1118 mm)** above the floor (IRC R310).
  - In dwellings, the opening of an operable window more than **72 in (1829 mm)** above the ground outside must have its lowest part at least **24 in (610 mm)** above the floor, or carry a fall-prevention device (IRC R312.2.1). Minnesota notes that its rule changed "the sill height of 36" to 24"" ([MN DLI](https://www.dli.mn.gov/sites/default/files/pdf/fs_window.pdf)).
- **Sill in offices.** Offices of the 1950s–70s put a perimeter induction unit **beneath the sill** ([ASHRAE](https://www.ashrae.org/File%20Library/About/Mission%20and%20Vision/ASHRAE%20and%20Industry%20History/Air-Conditioning-in-Office-Buildings-After-World-War-II.pdf)), so the stool sits on top of a cabinet. **[est]** About 0.8–0.9 m, from the [North Window 8 photo](https://commons.wikimedia.org/wiki/File:North_Window_8.jpg) (CC BY‑SA 4.0), where a convector cabinet stands under the stool.
- **Head height: [unverified].** I found no source. **[est]** In houses the head lines up with door heads, 2.0–2.1 m. In offices and halls it runs up to a bulkhead just under the ceiling.

**Reveal and wall depth**
- **Wood unit.** The frame is 116 mm deep (Marvin, above). Where the wall is thicker, the gap is closed by a wood jamb extension or by a **drywall return**: drywall turned into the opening with an L‑bead, leaving 1/16 in clearance at the frame. Trim-Tex sells L‑beads with return legs of 2‑1/8–5‑5/8 in. The look is "clean and modern", with "less room for error" ([Trim-Tex](https://www.trim-tex.com/blog/ultimate-guide-drywall-window-returns)).
- **Steel windows** sit in masonry or concrete (Brief 13 above), so the reveal is the wall's own depth.

**Interior trim (stool, apron, casing)**
- Stool length = window width + both casings + **2 in for the horns** (so about 1 in past each casing).
- **1/4 in reveal** on the casing (the casing stands back from the jamb's inner edge by that much).
- Apron = outside edge to outside edge of the casings.
- Residential casing **2½–3½ in (64–89 mm)** wide ([This Old House](https://www.thisoldhouse.com/windows/how-to-trim-an-interior-window)).

### 1.3 Hardware

- **Sash weights.** Historic sashes hang on cords over pulleys to iron weights in a **weight pocket** behind the jamb; the pocket is reached through a removable plate. Broken cords are "a common repair", replaced with new cord or chain ([Brief 9](https://www.nps.gov/orgs/1739/upload/preservation-brief-09-wood-windows.pdf)).
- **Modern locks and balances.** A crescent cam lock and keeper on the meeting rail, block-and-tackle balances, and an optional sash lift ([Marvin](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-12/Marvin%20Ultimate%20Wood%20Double%20Hung%20Specifications.pdf)).
- **Andersen 400.** Bar lifts, hand lifts or finger lifts. An "Estate" lock that takes 9/16 in off the clear opening. An opening-control device that stops the sash at under 4 in. A 3‑1/4 in "pocket" insert for replacing sashes in old frames ([Andersen 400 brochure](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-03/Andersen%20400%20Tilt%20Wash%20Brochure.pdf)).
- **Hardware finishes** (Andersen): white, stone, black, satin nickel, brass, oil-rubbed bronze and others ([same brochure](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-03/Andersen%20400%20Tilt%20Wash%20Brochure.pdf)).

### 1.4 Finish

- **Old wood joinery** "should always be painted"; staining is a misconception. In the 18th c. it was "Georgian white", an off-white; from the 1780s, dark greys, greens and browns ([Bassetlaw](https://www.bassetlaw.gov.uk/media/3570/guidance-note-sash.PDF)).
- **Steel windows** carry lead paint ([Brief 13](https://www.nps.gov/orgs/1739/upload/preservation-brief-13-steel-windows.pdf)).
- **Aluminium**: mill finish weathering to grey, or anodised ([Tech Note 22](https://www.nps.gov/orgs/1739/upload/tech-note-windows-22-historic-aluminum-windows.pdf)).
- **Modern wood units** often arrive with the interior unfinished pine ([Andersen](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-03/Andersen%20400%20Tilt%20Wash%20Brochure.pdf)).

### 1.5 How they age

**Wood**
- Brief 9's repair example (a sound window) shows the usual state: "many layers of paint, some cracked and missing putty, slight separation at the joints, broken sash cords, and one cracked pane".
- Decay starts where water stands: **the sill, the sill-to-jamb joints, the corners of the bottom rails and the muntin joints**. Paint blistering, cracking and peeling marks those places ([Brief 9](https://www.nps.gov/orgs/1739/upload/preservation-brief-09-wood-windows.pdf)).
- "Sticky windows" come from paint build-up. Rot is usually on the **bottom rail**. Failed putty and broken glass are common ([Bassetlaw](https://www.bassetlaw.gov.uk/media/3570/guidance-note-sash.PDF)).

**Steel**
- Corrosion is light (surface flaking), medium (a "bubbling texture") or heavy (delaminated metal).
- Sections bow; a frame bottom bows out under pressure.
- Glass cracks and glazing compound deteriorates ([Brief 13](https://www.nps.gov/orgs/1739/upload/preservation-brief-13-steel-windows.pdf)).

**Aluminium**
- Surfaces become "dull, soiled, and oxidized", with "discoloration and varying degrees of corrosion and pitting".
- Over 400 hoppers in one building were removed or disabled to fit **ventilation louvers or air-conditioning units**. Caulk was smeared over replacement glass.
- Survey of 120 windows in a 1960s Chicago tower:

  | Finding | Share |
  |---|---|
  | Original glass gone, **replaced with Plexiglas** or similar | 61% |
  | Missing screens | 71% |
  | Operating sash damaged | 31% |
  | **Boarded up**, with frame damage | 29% |
  | No visible defects | 15% |

  ([Tech Note 22](https://www.nps.gov/orgs/1739/upload/tech-note-windows-22-historic-aluminum-windows.pdf))

**Insulating glass and condensation**
- A failed seal leaves a **"milky-white film on the interpane surfaces"** ([Window & Door](https://www.windowanddoor.com/article/insulating-glass-failures)).
- Condensation forms inside failed units. Persistent humidity brings **mould** and can rot the window and sill timbers. Heavy net curtains hide all this ([RICS isurv](https://isurv.com/directory_record/2799/condensation_windows)).
- Photo: condensation running down inside a double-glazed unit, beads and streaks ([CondensInDubbelGlas, CC BY‑SA 3.0](https://commons.wikimedia.org/wiki/File:CondensInDubbelGlas.jpg)).

**Insects**
- Cluster flies leaving hibernation "will often go to the nearest light source such as a window" and "tend to die rather quickly": dead flies along the sill ([Montana State](https://urbanipm.montana.edu/entomology/resources/fact-sheets/nuisance_insect_pests/cluster.flies.html)).

**Blinds**
- Vinyl mini blinds deteriorate "from exposure to sunlight and heat" and leave dust on the slats. About 25 million a year were imported in the mid‑1990s ([CPSC 1996](https://www.cpsc.gov/content/cpsc-finds-lead-poisoning-hazard-for-young-children-in-imported-vinyl-miniblinds)).

**Reference photos**

| Photo | Licence | What to take from it |
|---|---|---|
| [North Window 8](https://commons.wikimedia.org/wiki/File:North_Window_8.jpg) | CC BY‑SA 4.0 | Closed industrial office: wood double-hung in a painted brick wall, a **mini blind raised crooked** (bottom rail tilted), a wood stool, a steel convector cabinet below, an outlet and a cable on the wall. The view is another mill's grid windows **[est]** about 15 m away. |
| [Office Windows in abandoned Industrial Facility](https://commons.wikimedia.org/wiki/File:Office_Windows_in_abandoned_Industrial_Faciity.jpg) | CC BY‑SA 4.0 | 6‑over‑6 double-hung: dirty panes, a cracked light, paint lost from the frame, a deep plaster reveal with a projecting stool, peeling whitewashed brick outside |
| [WestendOfficeWindow](https://commons.wikimedia.org/wiki/File:WestendOfficeWindow.jpg) | CC BY 3.0 | Glass gone and a 1 in **mini blind with bent and kinked slats**, flaking paint on frame and stool |
| [Office blinds at Nottingham Hall](https://commons.wikimedia.org/wiki/File:2011-02-10_Office_blinds_at_Nottingham_Hall.jpg) | CC BY‑SA 4.0 | From outside: three lights in white aluminium framing, mini blinds closed at slightly different tilts, so each pane reads a different grey |
| [CondensInDubbelGlas](https://commons.wikimedia.org/wiki/File:CondensInDubbelGlas.jpg) | CC BY‑SA 3.0 | Condensation and drip runs between panes |

### 1.6 Blinds and curtains

**One-inch mini blind**
- "Long, narrow slats held together by string"; slats 15 or **25 mm**, often aluminium.
- A tilt wand turns a drive rod; lift cords run through the slats to a cord lock.
- "The '70s began the era of the mini-blind." By **1981 they were 70–80% of the US window-covering market** ([Wikipedia](https://en.wikipedia.org/wiki/Mini_blind)).

**A contract mini blind's cut sheet** (Mariak 1½ × 1 in Estate, [cut sheet](https://sweets.construction.com/swts_content_files/53604/763268.pdf))

| Part | Detail |
|---|---|
| Headrail | Steel, **1½ in wide × 1 in high**, 0.024 in wall, baked polyester enamel |
| Slats | "8‑gauge" aluminium |
| Bottom rail | **C‑shaped steel**, clear plastic end caps |
| Tilt | **Clear hexagonal wand** (tilter left, cords right as standard) |
| Lift cords | **1.4 mm** polyester; a crash-proof cord lock |
| Ladders | Braided polyester, rout holes at most 20 in apart |
| Ladder spacing | **22.5 / 21.5 / 19.5 / 18.0 mm** (13.5–16.9 slats per foot) |
| Sizes | 9–96 in wide and tall |

**Vertical blinds** (an institutional standard, [Missouri State University 2008](https://design.missouristate.edu/_Files/Standards/Division12/122116VerticalLouverBlinds.pdf))
- PVC vanes **3½ in (89 mm)** wide, at least 0.027 in thick, concave; the "typical University color is white".
- Aluminium headrail 1 × 1.5 in; vanes rotate 185°.
- **Metal bead chain**: one 13½ in cycle self-aligns the vanes.
- Light gap at most 0.25 in; overlap at least 0.375 in.
- Mounted no closer than **1 in (25 mm)** to the glass.

**Net curtains** hide condensation and failed seals from a surveyor ([isurv](https://isurv.com/directory_record/2799/condensation_windows)).

---

## 2. How glass looks

**Reflection strength**
- Fresnel at normal incidence: R₀ = ((n₁−n₂)/(n₁+n₂))². For glass (n ≈ 1.5) that is **about 4% per surface**. It rises to total reflection at grazing incidence, and Brewster's angle is about 56° ([Wikipedia](https://en.wikipedia.org/wiki/Fresnel_equations)).
- Schlick's approximation: **R(θ) = R₀ + (1−R₀)(1−cos θ)⁵** ([Wikipedia](https://en.wikipedia.org/wiki/Schlick%27s_approximation)). The engine's BRDF already uses a Schlick-type term (`shaders.metal`).
- **[est]** Counting the inner reflections between surfaces (no absorption), N panes reflect R = 2Nr / (1 + (2N−1)r). With r = 0.04:
  - single pane: **≈ 7.7%**
  - double glazing: **≈ 14.3%**

**Why a lit room's window is a mirror at night**
- Glass passes about 95% and reflects about 5%. When the far side is darker, the reflection of the near side dominates: "This is why you can't see out of windows at night" ([Naked Scientists](https://www.thenakedscientists.com/get-naked/experiments/ghostly-windows)).
- Typical illuminances ([Wikipedia, Lux](https://en.wikipedia.org/wiki/Lux)):

  | Scene | lux |
  |---|---|
  | Office lighting | 320–500 |
  | Full moon | 0.05–0.3 |
  | Dark limit of civil twilight | 3.4 |
  | Moonless overcast night | 0.0001 |

- Typical luminances ([Wikipedia, Orders of magnitude (luminance)](https://en.wikipedia.org/wiki/Orders_of_magnitude_(luminance))):

  | Scene | Luminance |
  |---|---|
  | Night sky | 1 mcd/m² |
  | Cloudy night sky, no moon | 100 µcd/m² |
  | Scene at sunrise or sunset | 25 cd/m² |
  | Average cloudy sky | 2 kcd/m² |

- Surface luminance is illuminance × reflectance / π ([Toet & Alferdinck](https://publications.tno.nl/publication/105310/3J08H8/toet-2013-effects.pdf)). **[est]** A wall at 400 lx with reflectance 0.5 is about 64 cd/m². Its reflection in a single pane is about 5 cd/m², and in double glazing about 9 cd/m².
- **[est] What that means for the window:**
  - A moonlit field (0.1 lx × 0.2 / π ≈ 0.006 cd/m²) is about **1,000× dimmer than the room's reflection**. At night you see the room, plus the outside's own lights, plus the outside only where the reflected room is dark or where your head shades the glass.
  - At sunset (about 25 cd/m²) the landscape outshines the reflection and the room is a ghost over it.

**Double glazing**
- A flame held to the glass shows **one reflection per surface**: four for double glazing and six for triple, all the same colour for units of about 1975–1995. In low‑E units (mainly after 1995) the second reflection has **"a slight violet tint"** ([autarc](https://hilfe.autarc.energy/en/articles/6940917-how-do-i-determine-the-type-of-window-glazing)).
- **[est]** The pairs are offset by the pane gap (a 17 mm unit), so a lamp shows as a short chain of fainter copies.

**Colour**
- Float glass's greenish tint "is from iron impurities", seen "especially on larger and thicker sizes". Low-iron glass has about 0.01% ferric oxide; ordinary glass is said to have about ten times that ([Wikipedia](https://en.wikipedia.org/wiki/Low-iron_glass)).
- **[est]** The green shows edge-on, through the cut edge, a crack or a bevel, where light crosses centimetres of glass rather than 3 mm.
- **Tints** are offered as bronze, gray and green ([Marvin](https://detroitmi.gov/sites/detroitmi.localhost/files/hdc-submitted-materials/2023-12/Marvin%20Ultimate%20Wood%20Double%20Hung%20Specifications.pdf)); "sea green" heat-absorbing glass was used in post-war towers ([ASHRAE](https://www.ashrae.org/File%20Library/About/Mission%20and%20Vision/ASHRAE%20and%20Industry%20History/Air-Conditioning-in-Office-Buildings-After-World-War-II.pdf)).

**Old glass**
- Cylinder glass was blown into a cylinder, cut and flattened. It was made into the mid‑19th c. and then replaced by plate glass ([Wikipedia](https://en.wikipedia.org/wiki/Cylinder_blown_sheet)).
- Drawn sheet (Fourcault process, early 1900s) has **"waves, seeds (small gas bubbles) or stones"** that distort the view. Float glass largely replaced it ([Wikipedia](https://en.wikipedia.org/wiki/Fourcault_process)).
- Panes thicker at the bottom are crown glass set thick side down. The glass does not flow: about **1 nm in a billion years** ([American Ceramic Society](https://ceramics.org/news/glass-viscosity-calculations-definitively-debunk-the-myth-of-observable-flow-in-medieval-windows/)).

**Dirty glass and veiling glare**
- Dirt and scratches add a **veiling luminance** that lowers contrast: Lv = f(θ)·E, where E is the illuminance on the glass from a bright source and θ is the angle between the view direction and that source.
- For an average dirty windscreen:
  - **f(θ) = 6.6·θ^−1.73** for 1° < θ < 9°
  - **f(θ) = 44·θ^−2.6** for 9° < θ < 110°
  - A clean windscreen is about 0.2 of that.
- The same authors measured dirty windscreens at **14×** clean ([Toet & Alferdinck 2013](https://publications.tno.nl/publication/105310/3J08H8/toet-2013-effects.pdf)).
- **[est] For a window:**
  - Each ceiling fitting near the glass adds a haze over the view, strongest next to the fitting's reflection.
  - Each street light outside adds a glow round itself, larger on a dirty pane.

**Smears and fingerprints** (photo-based ambientCG masks; licence below)
- Photo-based: [Smear001](https://ambientcg.com/view?id=Smear001), [Smear002](https://ambientcg.com/view?id=Smear002), [Smear004](https://ambientcg.com/view?id=Smear004), [Smear005](https://ambientcg.com/view?id=Smear005), [SurfaceImperfections004](https://ambientcg.com/view?id=SurfaceImperfections004)
- Procedural: [Fingerprints002](https://ambientcg.com/view?id=Fingerprints002), [SurfaceImperfections015](https://ambientcg.com/view?id=SurfaceImperfections015) (dust)
- Method per the ambientCG API: "PBRApproximated" for the photo-based ones, "PBRProcedural" for the procedural ones.

**Obscure glass**
- Patterned glass is graded **1 (least) to 5 (most) obscuration** ([Pilkington](https://www.pilkington.com/en/gbl/architectural-and-technical-glass/product-categories/decoration/pilkington-texture-glass)). Through grade 4–5, only light and moving shapes come through.

---

## 3. Showing a view beyond a window: techniques

**[est] How much parallax matters.** Take a viewer who walks **s = 3 m** along a window wall. A point at distance D should swing by atan(s/D):

| D | Swing |
|---|---|
| 10 m | 16.7° |
| 50 m | 3.4° |
| 200 m | 0.9° |

A view at infinity (a skybox) shows no swing at all, so it is fine for the sky and far land and wrong for the ground near the building.

| Technique | How it works (source) | Holds up when you walk along the window | Cost |
|---|---|---|---|
| **Skybox at infinity** | Fixed relative to the viewer, so its contents seem infinitely far; "the lack of parallax motion may be perceived as unrealistic or confusing" ([Wikipedia](https://en.wikipedia.org/wiki/Skybox_(video_games))) | Only beyond about 200 m **[est]** | One texture read |
| **Ground-projected environment map** | three.js `GroundedSkybox(map, height, radius)`. `height` "is how far the camera that took the photo was above the ground. A larger value will magnify the downward part". The sphere's lower vertices are flattened onto the plane y = −height, with a smooth blend from y = −1.5·height to the horizon ([docs](https://threejs.org/docs/pages/GroundedSkybox.html), [source](https://github.com/mrdoob/three.js/blob/dev/examples/jsm/objects/GroundedSkybox.js)). Babylon.js has had `enableGroundProjection`, `projectedGroundRadius` and `projectedGroundHeight` since 6.27 ([docs](https://doc.babylonjs.com/features/featuresDeepDive/environment/skybox)). | **[est]** Flat ground is exact from any eye position if `height` is right; the sky and far land are fine on the dome. Vertical things nearer than the dome (trees, pylons, a bin) smear across the ground or the dome. The patched nadir blurs within a few metres of the projection centre. | **[est]** Per pixel: a ray-sphere test, a ray-plane test, atan2, acos and one read, on window pixels only |
| **Interior mapping** | Ray-cast in the pixel shader against virtual floors, ceilings and walls spaced regularly: ceiling height = ceil(y/d)·d (looking up) or (ceil(y/d)−1)·d (looking down); the nearest of the three planes wins. A "furniture plane" parallel to the facade, with alpha, holds furniture and characters. It fitted in **64 SM2.0 instructions** together with a diffuse and a reflection map. In a 2008 test it beat modelled interiors past 100 buildings; z‑prepass gave +44% ([van Dongen, CGI 2008](https://www.proun-game.com/Oogst3D/CODING/InteriorMapping/InteriorMapping.pdf)). Used in SimCity (2013) and, it is believed, Spider‑Man (2018) ([80.lv](https://80.lv/articles/interior-mapping-rendering-real-rooms-without-geometry)). | Exact for box rooms by construction. Weak points: flat furniture up close; different rooms seen through two windows of a corner room ([80.lv](https://80.lv/articles/interior-mapping-rendering-real-rooms-without-geometry)); "only in really narrow angles" does it look wrong ([Simon Schreibt](https://simonschreibt.de/gat/windows-ac-row-ininite)) | Low |
| **Parallax-corrected cubemap** | Intersect the view (or reflection) ray with a proxy box and look up the direction from the cubemap's centre to the hit point: `FirstPlaneIntersect = (BoxMax − PositionWS) / ReflDirectionWS; … Distance = min(min(FurthestPlane.x, FurthestPlane.y), FurthestPlane.z)` ([Lagarde 2012](https://seblagarde.wordpress.com/2012/09/29/image-based-lighting-approaches-and-parallax-corrected-cubemap/)) | Exact only on the proxy's surfaces; anything else distorts | On PS3, **0.25–0.75 ms** per extra cubemap at 25–75% of the screen (per object); 0.08 ms in a separate mixing pass |
| **Layered parallax planes** | BioShock Infinite: a room of planes with two bump-offset layers and a background texture that fills gaps; the glass becomes total reflection at grazing angles, hiding the distortion. Assassin's Creed 3 shifts one smooth tiling texture by bump offset ([Simon Schreibt](https://simonschreibt.de/gat/windows-ac-row-ininite)) | Good across the front; the glass's grazing reflection covers the edges | Low |
| **Portal / stencil** (render real geometry seen only through the opening) | Render from a second viewpoint and clip what lies in front of the boundary plane. Lengyel moves the projection's near plane onto the portal plane ("oblique near-plane clipping"): no added clipping cost, at the price of the far plane and depth precision ([Lengyel 2005](https://terathon.com/lengyel/Lengyel-Oblique.pdf)) | Exact | Scales with what is beyond: a second scene draw. Valve's own Portal method and Prey's (2017) Looking Glass: **[unverified]**, no developer source found |
| **Source 3D skybox** | Distant geometry built at about **1/16 scale**, moving slower than the level, so it "appear[s] far away" without being infinite ([Wikipedia](https://en.wikipedia.org/wiki/Skybox_(video_games))). The Valve wiki refused the fetch (403). | Good parallax for mid-distance shapes | A small second scene |

Other leads (Spider-Man 2's window shader, Cities, Forza): **[unverified]**. I found no developer source that could be read (ArtStation returned 403).

**What the game's case asks for [est].** The landscape is a photograph; a creature walks across it (another note); the viewer walks along a window inside a room. Two techniques fit:
- **A ground-projected panorama** (exact flat ground, a far dome), traced per pixel and seen only through the opening.
- **The creature as real geometry beyond the glass**, standing on that same ground plane, clipped to the opening and fogged with the photograph's haze at its distance. Its feet then agree with the photo's ground at every eye position.

Interior mapping is the tool for **interior glass**, an office sidelight into a dark next room, not for the landscape.

---

## 4. CC0 landscapes for the view

**Licences**
- **Poly Haven:** "Our assets are all licensed as CC0, which is effectively Public Domain"; "You can redistribute them", "include them … in a product you sell"; no credit needed ([licence](https://polyhaven.com/license)). Each asset page carries "CC0" (checked on [kloppenheim_04](https://polyhaven.com/a/kloppenheim_04)).
  - Their live API is free, but "building on the live API specifically requires a small 'Powered by Poly Haven' credit" ([Public-API](https://github.com/Poly-Haven/Public-API)). **[est]** A build script that fetches fixed file URLs and checks their SHA‑1, as `sources.json` already does, is not built on the API.
  - Their published HDRIs must be photographs, **≥16k wide, unclipped**, linear, with the **"Tripod & shadow removed from bottom"** ([requirements](https://docs.polyhaven.com/en/technical-standards/hdris)). The nadir is filled with PTGui's patch tool, or from a shot of the ground taken after moving the camera away ([Poly Haven blog](https://blog.polyhaven.com/?p=1975)).
- **ambientCG:** CC0 1.0; "You can include the raw files in your project, for example a video game"; no restrictions listed ([licence](https://docs.ambientcg.com/license/)).
- **Wikimedia Commons:** a search for CC0 equirectangular field and night panoramas returned only Poly Haven mirrors, for example [Kloppenheim 04 – Panorama](https://commons.wikimedia.org/wiki/File:Kloppenheim_04_-_Panorama_(Greg_Zaal_via_Poly_Haven).jpg). Nothing new there.

**Camera height:** no Poly Haven page, API record or capture guide states it. **[unverified]** **[est]** About 1.6–1.8 m on a tripod. Tune `height` against a feature of known size: parking bays, farm-track width.

**How to read the table**
- Metadata (time, weather, EVs, white balance, resolution) is from each asset's `api.polyhaven.com/info/<id>` record.
- "Ground for projection" is my reading of each preview **[est]**. "Flat" means usable; "objects near camera" means they will smear.
- Each Poly Haven asset page is `https://polyhaven.com/a/<id>`.

| # | Asset (page) | Max res | What it shows | Time / weather (attributes) | EVs, WB | Ground for projection |
|---|---|---|---|---|---|---|
| 1 | [kloppenheim_04](https://polyhaven.com/a/kloppenheim_04) | 24576 × 12288 | Misty grass field at night, a row of trees, **distant artificial lights glowing in fog** | night, fog, low contrast; 2019‑11‑01 | 12, 3392 K | Flat grass; fog hides the far land; slight smear at the very bottom |
| 2 | [kloppenheim_07](https://polyhaven.com/a/kloppenheim_07) | 24576 × 12288 | Open grass field, moonlit overcast sky, **warm lights on a far tree line** | night, overcast, low; 2019‑11‑03 | 22, 3776 K | Flat, clean |
| 3 | [kloppenheim_01](https://polyhaven.com/a/kloppenheim_01) | 24576 × 12288 | Misty field at sunset under overcast, rocks, a thin tree line | sunset, overcast | 8 | Flat, but **rocks near camera** |
| 4 | [preller_drive](https://polyhaven.com/a/preller_drive) | 16384 × 8192 | Night field with tyre ruts, trees, houses, **a power pylon**, street lights, stars | night, partly cloudy, high contrast; 2016‑03‑02 | 18 | Flat-ish; **tall grass near camera** smears |
| 5 | [misty_farm_road](https://polyhaven.com/a/misty_farm_road) | 21712 × 10856 | Gravel farm tracks converging into fog, ploughed fields | afternoon, fog, low; 2022‑11‑12 | 11, 6500 K | **Flat, strong track lines**: the best parallax cue |
| 6 | [spruit_dawn](https://polyhaven.com/a/spruit_dawn) | 16384 × 8192 | **Lattice pylons and power lines**, mist over a river, grass bank | sunrise, fog | 8, 5400 K | Slopes to the river (bank errors) |
| 7 | [misty_dawn](https://polyhaven.com/a/misty_dawn) | 16384 × 8192 | Rough field, a line of pylons with lines overhead, mist | sunrise, fog | 10, 5400 K | Flat-ish rough grass |
| 8 | [industrial_sunset](https://polyhaven.com/a/industrial_sunset) | 16384 × 8192 | Bare ploughed field to the horizon, **factory chimneys** far off | sunset, partly cloudy, low | 13 | Flat furrows |
| 9 | [overcast_soil](https://polyhaven.com/a/overcast_soil) | 16384 × 8192 | Dark ploughed soil to a flat horizon, heavy overcast, two tree clumps | afternoon, overcast, low | 12, 6900 K | Flat, featureless |
| 10 | [farmland_overcast](https://polyhaven.com/a/farmland_overcast) | 29696 × 14848 | Flat farmland with grassy strips, textured cloud | afternoon, overcast, low; 2025 | 12, 5400 K | Flat |
| 11 | [beach_parking](https://polyhaven.com/a/beach_parking) | 16384 × 8192 | **Empty asphalt parking lot in fog at dawn**, poles, low buildings | sunrise, fog, low | 12 | Flat asphalt; a kerbed island and **a bin near camera** |
| 12 | [learner_park](https://polyhaven.com/a/learner_park) | 16384 × 8192 | **Empty rooftop parking** with painted bays, low walls, dusk | sunset, partly cloudy, low | 12 | Flat; the bays help set `height` |
| 13 | [rooftop_night](https://polyhaven.com/a/rooftop_night) | 16384 × 8192 | Rooftop parking at twilight, **floodlights**, a moon, purple sky | dusk, partly cloudy, high | 16 | Flat concrete; a cable on the deck near camera |
| 14 | [moonless_golf](https://polyhaven.com/a/moonless_golf) | 16384 × 8192 | Very dark field, tree silhouettes, warm lamps, stars | night, clear, medium | 23, 3942 K | Flat (barely visible) |
| 15 | [missile_launch_facility_02](https://polyhaven.com/a/missile_launch_facility_02) | 16500 × 8250 | Clearing in pine and birch woods, overcast | afternoon, overcast, low | 8, 5600 K | **Uneven**, logs near camera: poor |
| 16 | ambientCG [EveningSkyHDRI045A](https://ambientcg.com/view?id=EveningSkyHDRI045A) | 1K–16K (stitched HDR panorama) | Gloomy overcast dusk over farmland and a dirt road (tags "dramatic, moody, gloomy") | evening; released 2026‑05‑20 | n/a | Flat |
| 17 | ambientCG EveningSkyHDRI043A | 1K–16K **[est]** (as its siblings) | Dusk or twilight over a field (tags "dusk, twilight, field") | evening | n/a | Flat |

- ambientCG's **"B" variants** replace the ground with flat grey (seen in the 045A and 045B previews), so they are sky-only and no use for ground projection.
- **File sizes** (kloppenheim_04): 4K EXR 22.56 MB, 8K tonemapped JPG 4.98 MB ([page](https://polyhaven.com/a/kloppenheim_04), [files API](https://api.polyhaven.com/files/kloppenheim_04)). The tonemapped JPG carries Poly Haven's own tonemapping; grade from the EXR instead.

---

## 5. What looks fake about game windows

**From developer sources**
- **Painted views with no parallax.** Most games either made windows "fully reflective, not allowing a look inside", or drew "some details very close to the window … into the diffuse texture and the rest … black". Plain textures look "as if pictures of details have been glued on to a flat surface" ([van Dongen 2008](https://www.proun-game.com/Oogst3D/CODING/InteriorMapping/InteriorMapping.pdf)). A skybox's missing parallax reads as "unrealistic or confusing" ([Wikipedia](https://en.wikipedia.org/wiki/Skybox_(video_games))).
- **Sky light leaking into the room.** A skybox light is "less useful for interior scenes … where the skybox is only visible through windows" because it "lights the entire room, this creates an unnatural effect" ([Stride docs](https://doc.stride3d.net/4.0/en/Manual/graphics/lights-and-shadows/skybox-lights.html)).
- **Breaking at grazing angles.** Interior mapping goes wrong "only in really narrow angles". BioShock Infinite hides this by turning the glass into a full reflection at a graze ([Simon Schreibt](https://simonschreibt.de/gat/windows-ac-row-ininite)). Corner rooms show different interiors through their two windows, and desks are flat up close ([80.lv](https://80.lv/articles/interior-mapping-rendering-real-rooms-without-geometry)).
- **Too clear a view.** One developer frosts the glass on purpose ("a blurry/milky effect … so they are not completely clear like real life") so the city beyond can stay low-detail ([Samcpfc devlog](https://samcpfc.itch.io/reloaded/devlog/517959/devlog-3-windows)).

**Against the real numbers above [est]**

| Fake | Real |
|---|---|
| **A sticker on the wall**: a frame 8 cm deep standing 5 cm proud of a wall with no opening (the game today) | The glass sits 100–200 mm into the wall: a 116 mm wood frame or a 114 mm storefront mullion in a drywall or masonry reveal, a stool that projects past the casing, horns about 1 in past it |
| **A cross of 40 mm muntins** making four equal lights (the game today) | Historic muntins are 19 mm; a double-hung has a **meeting rail at mid-height where two sashes in different planes overlap** (the upper one outboard); steel sections are 25–38 mm on a grid of small lights |
| **A view brighter than the room** | At night the outside is about 10⁻³ of the room's luminance and the glass is a mirror (§2); only dusk outshines the reflection |
| **No reflection** | 8–14% square on, all of it at a graze, a chain of faint copies in double glazing |
| **Perfectly clean, uniform glass** | Smears, dust lit by the room, a milky failed unit, condensation runs, a Plexiglas pane among glass ones (61% in one survey), one cracked light, a pane boarded or swapped for a louvre |
| **A light spilling from a night window** (today's cool point light) | At night the window gives the room nothing; at dusk, a faint cool fill near the glass at most |

---

## 6. For this game

**Window types by zone**
- **The Lobby** (mono-yellow wallpaper, wet carpet, ceilings 2.7–3.05 m): **wood double-hung, 1‑over‑1 or 2‑over‑2**.
  - Painted off-white or cream in many layers; the lower sash painted shut; a stool and apron; casing 64–89 mm.
  - An aluminium mini blind, half raised and crooked.
- **Office wing / Abandoned Office** (drywall, carpet tiles, ceilings 2.5–2.75 m): **aluminium strip windows**.
  - Fixed lights about 1.2 m wide between 50 mm mullions 114 mm deep, clear or bronze anodised, a 25 mm insulating unit, some panes **milky with failed seals**.
  - A drywall return and a laminate stool on a **convector or induction cabinet**.
  - **[est]** Mini blinds, or white vertical blinds on bead chains.
  - Optional: an **interior sidelight** into a dark office next door, rendered by interior mapping.
- **Utility halls, maintenance, service tunnels, Electrical Station** (concrete, brick, ceilings 2.35–2.65 m): **rolled-steel projected (hopper) windows**, high in the wall.
  - **[est]** Sill about 1.5–1.7 m, so the view is mostly sky and the tops of things.
  - Square-mesh **wired glass** 6 mm or ribbed glass; putty failing; rust bubbling at the bottom rail.
  - One hopper replaced by a louvre or plywood.
- **Washrooms:** obscure patterned glass (privacy 4–5). The landscape becomes light and blur, and anything moving outside becomes a shape.
- **Pool hall:** not researched.

**Sizes to build at** (metres, from §1; **[est]** where the sources gave no figure)

| | Double-hung (Lobby) | Office strip | Steel hopper (utility) |
|---|---|---|---|
| Opening | 0.91 × 1.37 (36 × 54 in) | 1.2 × 1.3 per light, in runs | 0.9 × 0.6 **[est]** |
| Sill / head | 0.75–0.80 / about 2.10 | 0.80–0.90 / 2.10–2.30 (under a bulkhead) | 1.5–1.7 / 2.2–2.3 **[est]** |
| Frame depth | 0.116 | 0.114 | 0.03 sections in a 0.2–0.3 m masonry reveal **[est]** |
| Sash, rails, bars | Sash 0.035–0.044; stiles about 0.045; meeting rail 0.032–0.044; muntins 0.019 | Mullion face 0.050 | Sections 0.025–0.038 × 0.003; lights about 0.30 × 0.40 **[est]** |
| Glass | 0.0024–0.0032 single, or a 0.017 unit | 0.025 unit **[est]** | 0.0068 wired |

**Finish and wear** (pick from §1.5, a few per window, from the level's stream or a salted one, as `ROOMS.md` asks):
- paint build-up and flaking at the sill and bottom rail, cracked putty
- one cracked light (green at the crack's edge)
- dead flies along the sill
- black mould in the bottom corners
- one milky failed unit, one yellowed Plexiglas pane
- tape or cardboard over one light **[est]**: no source shows it, only plywood

**Blinds**
- Size: 25 mm aluminium slats, 18–22.5 mm apart on two to four ladders, a steel headrail 38 × 25 mm, a C‑shaped bottom rail, 1.4 mm cords, a clear hexagonal wand.
- Placement: hung 25 mm or more inside the glass.
- Wear: often raised unevenly (one cord let out), some slats kinked, white or off-white gone grey with dust.
- Effect: blinds are a cheap creepy device. Tilted slats cut the view into strips, and the entity can be seen between them.

**Glass behaviour to render**
1. **Fresnel** at 4% per surface (8% single, 14% double), rising to all-reflection at a graze.
2. At night **the room reflected over a dark view**: the view shows only where the reflection is dark or the player's head is close. The landscape is otherwise there only as its own lights.
3. Dust and smears lit by the room (photo-based ambientCG masks), and the **veiling haze** round bright fittings and street lights per Toet's f(θ).
4. **Obscure, wired, waved or milky glass** as a UV distortion or blur of the view.
5. Each failed unit gets its own milky layer.
6. Keep the view's texture read inside its own patch, never in the global one (shader-cost rule).
7. No light from the window at night. At dusk, a faint cool tint on the light map near the glass, not a lamp.

**The view**
- **Use** a ground-projected panorama: the ground plane at the photo's camera height below the projection centre, the centre a few metres outside the glass so the patched nadir is never seen, the dome radius about the distance of the main tree line.
- **Grade** it to the STYLE bands: desaturated, with the night view at about 1–5% of the room's wall brightness **[est]**.
- **Texture budget [est]:** an 8K source gives 22.8 px/° against 34 px/° on a 3440-wide screen at 100°. A window sees at most the half-sphere in front and roughly −50° to +60° of elevation. Crop to about 4096 × 2500, about 40 MiB as RGBA8, against 128 MiB for the full 8K.
- **Bring them in** through `assets/src/textures/` from the EXR, by URL and SHA‑1.
- **Lore:** `LORE.md` says the Building has "no outside and no sky". The view is then an outside that cannot be there, which fits *familiar but wrong*, but the lore-keeper should rule on it.

**Panoramas, ranked**
1. **kloppenheim_04**, night fog field with distant lights. The fog hides every error of the projection beyond 50 m, the near ground is flat, and the lights read as "lights that shouldn't be there" (as today's painted dots meant to). The entity can emerge from the fog.
2. **kloppenheim_07**, overcast night over an open field with a lit tree line. A clean, flat expanse across which something can be seen walking.
3. **misty_farm_road**, tracks converging into fog. Flat ground, the strongest parallax lines, a path that leads to the window. Grade it toward dusk.
4. **preller_drive**, a night field at a suburb's edge with a pylon, lines and street lights. The most "liminal" of them, though the near grass is tall and it is high in contrast.
5. **beach_parking** (or **learner_park** for a dusk version), an empty parking lot in fog. It suits the office wing's windows.

Alternates: **spruit_dawn** and **misty_dawn** (pylons in mist) for utility zones, and **overcast_soil** or **industrial_sunset** for "nothing out there".
