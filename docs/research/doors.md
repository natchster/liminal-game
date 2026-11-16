# References: the doors

Gathered on 2026-11-16 by research agents, one a door, for the doors' remake
by the asset pipeline ([`../plan/DOORS.md`](../plan/DOORS.md)): the
measurements, materials, palette, references (each with its licence class)
and where such doors are found. The dossiers that use them are
`assets/src/props/<door>/` (their `references.toml` number the references
as these notes do); what could not be verified is listed in each. Kept as
found.

## 1. The exit door (`exit_door`)
Subject: the flush, solid-core, wood-veneer fire door (20-90 min) from an office corridor to the exit
stair, in its pressed-steel frame, with its hinges, lever, closer and kick plate, and the lit EXIT sign
over it. Offices in North America or Europe, 1980s-2000s. Companion data: `exit_door.toml`. Reference
numbers below match the order of its `[[ref]]` entries.

Status markers: **VERIFIED** = read on the page or in the PDF text. **SEARCH-ONLY** = seen only in a
search-result summary. **DERIVED** = computed, working shown. **ESTIMATE** = no source.

Commons SHA-1s come from the imageinfo API. Three were also checked against downloaded originals
(Wooden_fire_door.JPG, Fire_door_label.jpg, ISO_7010_E001.svg) and all three matched.

### References

Free (11); every image *viewed*; Commons page links are in the TOML.

1. Wooden fire door, Palo Alto office (CC BY-SA 3.0, BrokenSphere): light maple/birch leaf, amber
   clear finish, lever at ~0.45 of leaf height, closer on the top rail by the hinge edge, grey frame.
2. Texas hotel stairwell fire door (CC BY 3.0, Achim Hering): walnut-look veneer, almond HM frame,
   3 hinges, round-rose lever, pull-side closer, vision lite, stair ID sign. A hotel, same assembly.
3. Door closer (PD, Wesha): aluminium closer and regular arm on a stair door; cream head, its stop.
4. Empty hallway with exit sign, Herndon VA (CC BY-SA 4.0, Yitzilitt): office-type corridor, white
   thermoplastic red-letter sign on a ceiling canopy, khaki-painted steel door and frame.
5. Faculty Office Building hallway, BYU (CC BY-SA 2.0, Ben P L): green-letter sign hung over an
   office corridor; small in frame (letter strokes ~3 px in the 960 px thumbnail).
6. Basement Exit Sign (CC0, MarkBuckawicki): grimy yellow-grey thermoplastic sign, cobwebs, askew.
7. Emergency exit UK 001 (CC BY 4.0, HarJIT): lit Thorn box sign (running man + arrow) just above a
   door head, closers on the leaves below.
8. Abandoned Buzz Bingo hall fire exit, Chatham UK (CC BY-SA 4.0, OathOn): box sign over the head,
   stickers, debris. Not an office and a steel push-bar door: use for neglect and sign placement only.
9. Fire door label (CC BY-SA 3.0, Ben Schumin): Steelcraft 1-1/2 h (B) label, chipped dirty paint.
10. ISO 7010 E001 vector (PD, MaxxL): fill `#237F52`; E002 mirror is File:ISO_7010_E002.svg
    (sha1 dd4b9544…). File downloaded.
11. OSHA 29 CFR 1910.37 (US government, PD), read only (`sampled = false`).

Limited (9): looked at and measured only.

12. LCN 4040XP cut sheet (Allegion). PDF text extracted with pypdf; drawings not seen as images.
13. Steelcraft Technical Data Manual, Frames (Allegion). PDF text extracted; profile drawing not
    seen as an image.
14. Hager commercial hinge catalogue 2016. PDF text extracted.
15. Schlage ND series data sheet (Allegion). PDF text extracted.
16. Rockwood K1050 10" kick plate, Doorways Plus listing. Page text read.
17. idighardware.com, "Decoded: allowable clearances for fire door assemblies" (NFPA 80, 2022).
    Page text read.
18. LSI EX thermoplastic exit sign spec sheet. PDF text extracted.
19. Premdor FD30 oak flush veneer door 2040x926x44, Jewson listing. Page text read.
20. Whitecroft Lighting, "Emergency lighting: a practical guide" (UK, era of BS 5266-1:1988). PDF
    text extracted.

### The leaf

- Nominal 3'0" x 7'0" x 1-3/4" = **914 x 2134 x 44 mm**. VERIFIED [14]: Hager names this as the
  reference door for three hinges.
- UK metric FD30: **2040 x 926 x 44 mm**, oak veneer. VERIFIED [19]: "veneered edge to edge with
  concealed vertical edge lipping's" and "factory lacquered".
- FD60 is 54 mm thick in the UK. SEARCH-ONLY (Halspan Prima entry on NBS Source).
- Core: mineral core for rated doors, stave core for non-rated. Also VERIFIED:
  - "solid core flush doors with transparent factory finish, generally installed in metal frames";
  - fire labels fixed on the hinge edge.

  Source: Columbia University design requirements, Wood Doors 08 14 00 (2011),
  https://cufo.columbia.edu/sites/cufo.columbia.edu/files/content/ARCH_SECTION_081400_-_WOOD_DOORS.pdf.
  Read but not given a ref number; its clearances duplicate [17].
- Veneer colour: VT Industries' bulletin (VERIFIED, not a ref) says natural birch and maple mix white
  sapwood with red/brown heartwood, and that this "cannot be controlled". "Select White" (all
  sapwood) maple or birch gives the uniform pale look. Plain-sliced red oak is the other common
  1980s-90s face. ESTIMATE: no oak photo was sampled.
  https://www.vtindustries.com/webres/File/architectural-doors/Technical%20Bulletins/VT-Technical-Bulletin-Natural-Veneer-and-Variations.pdf
- Edges: in North America the vertical edges are same-species hardwood (the face veneer runs to a
  visible edge band). ESTIMATE from photos [1][2]. UK leaves have concealed lippings [19].
- Ageing of the finish: maple and white veneers yellow to amber under light, and cherry darkens.
  SEARCH-ONLY (Tabu and Wolf guidance in a search summary). Photo [1] shows the amber cast.

### The frame (pressed steel, "hollow metal")

All VERIFIED from [13] (Steelcraft F series) unless marked.

- Face **2" (50/51 mm)** on jambs and head; optional **4" (102 mm)** head in masonry.
- Stop **5/8" (16 mm)**. Door rabbet **1-15/16" (49 mm)** for a 1-3/4" leaf. Single-rabbet side 1-9/16" (40 mm).
- Return/backbend **1/2" (13 mm)**, or 7/16" (11 mm) on the 5-3/4" depth.
- Double rabbet is standard. Jamb depths run 4-1/2"-20" (114-508 mm).
- Typical stud-wall depth is 5-3/4" (146 mm). SEARCH-ONLY for the wall pairing: a 6-3/4" jamb fits a
  5-3/4" wall, per a Door Closers USA listing.
- Steel 16 ga 0.053" (1.3 mm), 14 ga 0.067" (1.7 mm), 12 ga 0.093" (2.3 mm). Factory primer to
  ANSI A250.10.
- Hinge reinforcement for a 4-1/2" (114 mm) hinge is standard, 5" optional; 7 ga.
- Strike prep 4-7/8" ASA. Strike centreline **40-5/16" (1024 mm)**, labelled in the elevation
  drawing. The drawing also carries "9-3/4" (248 mm)", "10-3/8" (264 mm)" and "Equal" near the
  hinges, but the text extraction does not show what they are measured from. Read them as the hinge
  layout only as an ESTIMATE: top hinge about 250 mm below the head, bottom hinge about 250-265 mm
  above the floor, middle hinge equally spaced.
- Top of the head above the floor: 2134 + 51 = **2185 mm**. DERIVED (frame opening = nominal leaf
  height is an ESTIMATE of common practice; the leaf is shortened by the gaps below).
- Net leaf height: 2134 − 3 (head gap) − 19 (undercut) ≈ **2112 mm**. DERIVED from [17]. Pre-fit
  doors are ordered this way. ESTIMATE that a given door was.
- Painted colours seen: dark grey [1], almond/beige [2], cream [3], khaki [4]. VERIFIED by viewing.
  Frames are usually painted to match the walls or the door hardware. ESTIMATE.
- Wood frame with casing: possible in some offices, but no source was found. Every photo found shows
  a steel frame.

### Clearances

- Head and jambs: **1/8" (3 mm) max** for flush wood doors rated over 1/3 h, and 1/8" ±1/16" for
  20-minute doors in HM frames. Bottom: **3/4" (19 mm) max** to the finished floor. All VERIFIED [17].
- Columbia's standard says the same: 1/8" at the edges and top, 3/4" above the finished floor.
  VERIFIED.
- UK: about 3 mm (2-4 mm) leaf-to-frame gaps; threshold 6-10 mm for fire-only doors, 3 mm with smoke
  control. SEARCH-ONLY.

### Hardware

- **Hinges** [14], all VERIFIED:
  - three per leaf for doors over 60" and up to 90";
  - 4-1/2" high for 1-3/4" doors up to 36" wide;
  - BB1279 is 4-1/2 x 4-1/2" (114 x 114 mm), 0.134" (3.4 mm) leaf, 5 knuckles, non-rising
    removable pin with button tip;
  - "Use ball bearing hinge when door closer is installed on door".

  Finish US26D (652, satin chrome plated) is SEARCH-ONLY (dealer listing). Photos [1][2] show
  satin-grey hinges.
- **Lever lockset** [15]: rose **3-7/16" (87 mm)** across; default latch 2-3/4" (70 mm) backset with
  a 1-1/8 x 2-1/4" faceplate; ANSI strike 1-1/4 x 4-7/8"; 1-3/4" door. Finish 626 satin chrome or
  630 satin stainless. All VERIFIED.
  - Rhodes lever length 4-7/8" (124 mm), projection about 2-1/2". SEARCH-ONLY (Ferguson/Build.com
    listings).
  - The sheet only states "lever lengths vary".
  - Lever centreline height ≈ strike centreline ≈ **1024 mm**. DERIVED: a cylindrical lock's latch
    sits on the lever axis. In photo [2] it sits at about 0.43 of the leaf height (DERIVED, with
    perspective).
  - A stair door may be a passage set or a keyed storeroom set. ESTIMATE.
- **Closer** [12] (LCN 4040XP; the 4041 was its 1980s-2000s predecessor, ESTIMATE):
  - Body 12-1/4" (311 mm) long. VERIFIED.
  - 3-1/2" (89 mm) high. VERIFIED as a drawing label beside the body; Build.com and FHC listings
    give the same height (SEARCH-ONLY).
  - 2-1/4" (57 mm) projection. VERIFIED as a label; the axis is inferred, and Build.com's
    "Projection 2.25 in" agrees.
  - Hinge (pull) side mount needs 3-3/4" (95 mm) of top rail, otherwise a -18 plate is added.
    Opens to 120°. Both VERIFIED.
  - Stock finish 689 aluminium powder coat. VERIFIED.
  - Leaks: the sheet sells "full complement bearings … to reduce potential leaks" and positive-stop
    valves that "eliminate leaking". VERIFIED. That closers weep oil down the leaf as they age is
    ESTIMATE, inferred from that wording.
  - Placement: on the corridor (pull) side of a stair door the body sits on the top rail near the
    hinge edge, the arm to a shoe on the head [1][2][3]. Seen in photos. On the push side, a
    parallel arm lies flat along the head [7].
- **Kick plate** [16]: Rockwood K1050, **10" (254 mm)** high, **0.050" (1.27 mm)** stainless,
  US32D/630 satin, four edges bevelled, #6 x 5/8" screws. "Typically sized 2 inches less than the
  door width" gives **34" (864 mm)** on a 36" leaf. VERIFIED, then DERIVED.
  - It sits on the push side, flush with the leaf's bottom edge. ESTIMATE. That NFPA 80 lets plates up to
    16" high go on rated doors without a label is from memory, also ESTIMATE.
  - None of the free photos show a kick plate.

### EXIT sign: North America

- Letters at least **6" (152 mm)** high, principal strokes at least **3/4" (19 mm)**. Face lit to at
  least **5 fc (54 lux)**. VERIFIED [11].
- LSI EX: 6" letters, 3/4" stroke. VERIFIED [18].
- NFPA 101 adds letters at least 2" wide (except I) with 3/8" spacing. ESTIMATE, from memory; the
  Wikipedia article does not give it.
- Red is traditional in the US; either red or green is allowed. Canada's 2010 code moved to the green
  running man. VERIFIED (Wikipedia "Exit sign", read, not a ref).
- Housing:
  - LSI EX: dimension labels 11.81 / 7.25 / 1.81 in, plus 8.25 / 4.63 in for the canopy. VERIFIED.
    Read as **300 W x 184 H x 46 D mm**, a DERIVED axis assignment.
  - Lithonia LQM: 11-3/4 x 7-5/8 x 2 in. SEARCH-ONLY.
  - Injection-moulded thermoplastic, white or black. VERIFIED [18].
  - Lithonia's LHQM sheet stresses "UV-stable resin resists discoloration". VERIFIED (Lowe's-hosted
    PDF, not a ref), which implies older housings yellowed.
- 1980s-90s signs were often incandescent or fluorescent in steel or die-cast aluminium boxes,
  deeper than LED ones (about 2.5-4 in). ESTIMATE. Wikipedia: "Early exit signs generally either
  were made of metal…". VERIFIED.
- Mounting: canopy for top, end or back (wall) mount [18].
  - Over a stair door it is usually back-mounted on the wall, centred over the opening.
  - Its bottom sits about 100-250 mm above the frame head, about **2.29-2.44 m** above the floor.
    DERIVED from 2185 mm + an ESTIMATE of the gap.
  - In corridors it is often ceiling-hung across the path instead [4][5].
  - No code mounting height was found for standard signs.
- Ageing: grime film, yellowing, cobwebs, hanging askew from the conduit [6]; red letters still lit.

### EXIT sign: Europe (ISO 7010 E001/E002)

- Pictogram geometry from [10]. Green `#237F52` = (35,127,82) as drawn. VERIFIED from the file.
  E001 and E002 are mirror images, chosen by the escape direction.
- EU Safety Signs Directive 92/58/EEC: exit signs green. VERIFIED (Wikipedia, read).
- Whitecroft [20], VERIFIED:
  - the guide dates from the change from BS 2560 "EXIT" text signs and BS 5499 to the Directive
    pictogram;
  - viewing distance = **200 x legend height** for internally lit signs, **100 x** for externally
    lit ones;
  - the new format's larger green area "will significantly reduce luminaire light output".
- 24 m box: legend about 24 / 200 = **120 mm** high. DERIVED.
- Box sizes:
  - Thorn Voyager IP65 24 m: 381 x 98 x 190 mm. SEARCH-ONLY.
  - Voyager E: 392 x 190 x 60 mm. SEARCH-ONLY.
  - The brief's 350 x 150 mm is typical of printed or photoluminescent "Fire exit" plates
    (Commons "Emergency exit UK 007" shows one over a light veneer door; viewed but not listed).
- In photo [7] the box is about **1.8:1** (width:height) and sits about 150-250 mm above the head.
  DERIVED, with perspective.
- Mounting: about 1.7-2.0 m to the sign base for wall signs; avoid more than 20° above the line of
  sight (EN 1838). Over-door boxes sit just above the head, as in [7][8]. SEARCH-ONLY for the numbers.

### Ageing in a neglected building

Neglect, as seen in the photos:
- dirt film and chipped paint on pressed steel, worst at screw heads and edges [9];
- grimy, yellowed sign housing hanging off level [6];
- stickers, debris and propped doors [8].

ESTIMATE, no source found:
- veneer worn grey and glossy at the latch edge between 0.9 and 1.2 m, where hands push;
- the clear finish worn through around the rose;
- scuffs, shoe marks and dents along the bottom 300 mm, on the push side where there is no kick plate;
- a dark oil streak down the leaf below the closer, from failed seals (see the [12] leak note);
- frame paint chipped at the strike, at hinge knuckles and at floor level (mop and cart damage);
- the sign's red or green diffuser faded, with dead pixels or lamps.

### Palette notes

Every value is a mean over a box in a 960 px Commons thumbnail, decoded with macOS `sips` to BMP.
No white balance was applied.
- Casts: [1] is warm (wall 192,181,169); [3] and [9] are warm; [4] is neutral to slightly warm.
- Metals (lever, kick plate) take the room's colour. Treat them as roughness/metalness cues, not
  albedo.
- The lit sign colours [5][7] include glow and overexposure. For an emissive texture, use the hue and
  raise brightness; for the unlit printed green, use the ISO value [10].
- If a neutral wall is wanted, a WB-corrected maple sample from [1] is about (156,126,66). DERIVED
  by scaling to the wall.

### Could not verify

- The exact meaning of the Steelcraft hinge labels (9-3/4", 10-3/8"), and therefore exact hinge
  heights. A "5, 10 and equal" rule (top hinge 5 in below the head, bottom 10 in above the floor) was
  seen only in a search summary of an Allegion KB page, attributed to NAAMM/HMMA (403 on fetch).
- LCN arm length: labels 11" (279 mm) on the hinge-side drawing and 14-1/2" (368 mm) on the
  parallel-arm drawing were read, but their meaning was not confirmed.
- Schlage Rhodes lever length (4-7/8"): listings only.
- North American sign mounting height over the door: no code text found. Lithonia LQM official sheet
  (403).
- European box dimensions from a maker's own PDF: Thorn and Acuity PDFs returned 403. BS 5266-1 and
  EN 1838 were not read.
- NFPA 101 sign letter width and spacing (2", 3/8"): not read.
- A free photograph of an abandoned *office* exit-stair door: none found on Commons. [8] is an
  abandoned bingo hall. HABS "Arlington Realty Building" (PD; an office-building lobby, date not checked,
  with the stair fire door and closer; B&W, sha1 2bd5ed67b9fa5a4a6e39047aa1a4936dafbc5d3b) is a usable extra.
- A free photo of a kick plate on such a door. The Canaan Valley NWR door (PD, USFWS) shows a new
  plate still in its protective film.
- Veneer greying, oil weeping and kick-mark patterns: no documentary source; all ESTIMATE.
- US26D hinge finish, UK FD60 54 mm, UK gap figures: SEARCH-ONLY.

## 2. The steel door (`steel_door`)
Researched 2026-11-16. Companion to `steel_door.toml`; the [n] numbers below are its `[[ref]]` order.

Status markers:
- **VERIFIED**: read on the page or in the PDF text, or seen in a rendered drawing.
- **SEARCH-ONLY**: seen only in a search-engine summary.
- **DERIVED**: computed; the working is shown.
- **ESTIMATE**: no source.

Commons SHA-1s come from the Commons API (`prop=imageinfo&iiprop=sha1|url|extmetadata`). I downloaded the original of Fire door.jpg [5], and its SHA-1 matched. When I tried the same for the Taiwan file [8], upload.wikimedia.org rate-limited me (HTTP 429), so that one is API-only.

### References

Free (8 images, all viewed, plus 1 document read):
1. [Ramada stair, bottom-landing door](https://commons.wikimedia.org/wiki/File:2020-11-05_20_33_20_Door_to_the_lobby_on_the_bottom_landing_of_the_central_stairwell_within_the_Ramada_by_Wyndham_Rochelle_Park_in_Rochelle_Park_Township,_Bergen_County,_New_Jersey.jpg). CC BY-SA 4.0, Famartin. Viewed at 960 and 1920 px, with crops. Seen from inside the stair: red-brown leaf, beige frame, wired-glass narrow lite in black trim, closer, brass kick plate and a "Lobby" plaque. The walls are painted CMU.
2. [Ramada stair, top landing](https://commons.wikimedia.org/wiki/File:2020-11-05_20_30_32_The_top_landing_of_the_central_stairwell_within_the_Ramada_by_Wyndham_Rochelle_Park_in_Rochelle_Park_Township,_Bergen_County,_New_Jersey.jpg). CC BY-SA 4.0, Famartin. Viewed, with crops of the door's top and bottom. A beige roof door on its push side, with a rim push bar, a parallel-arm closer, signs and a black sweep. Next to it is a grey leaf in a dark red frame. The landing is grey concrete with nosing tape. The same uploader posted the whole stair, floor by floor, if more angles are needed.
3. [Texas hotel stair door](https://commons.wikimedia.org/wiki/File:Texas_hotel_fire_door_stairway.png). CC BY 3.0, Achim Hering. Viewed. Shows the corridor side: 3 hinges, a beige frame, a narrow lite, a closer and a lever, plus a tactile/Braille stair sign. The leaf is wood-grain. It could be veneer or a printed finish; I could not tell whether it is steel.
4. [Set of Crash Bar Doors](https://commons.wikimedia.org/wiki/File:Set_of_Crash_Bar_Doors.jpg). CC BY-SA 4.0, Scott Brody. Viewed. Pairs of school stair doors with push bars, vertical rods, top latches, closers and narrow lites. The frame head carries an "S3" label.
5. [Fire door](https://commons.wikimedia.org/wiki/File:Fire_door.jpg). Public domain, Interiot. Viewed at full size. Off-white leaf, wired-glass (diamond mesh) narrow lite, lever on a long escutcheon.
6. [Janitor's Closet](https://commons.wikimedia.org/wiki/File:Janitor%27s_Closet.JPG). CC BY-SA 4.0, Ukelady123. Viewed. An engraved "JANITORS CLOSET" plaque with black letters on a silver face. The leaf is stainless, not painted hollow metal, and it is dented and scratched.
7. [Dark Exit](https://commons.wikimedia.org/wiki/File:Dark_Exit_(16536230456).jpg). CC BY 2.0, darkday. Viewed. Exit door of an abandoned milk factory in Brisbane, used for grime and scuff patterns. It is not a stair door, and the photo has an HDR look.
8. [Heavy steel doors, Taiwan](https://commons.wikimedia.org/wiki/File:%E8%BB%8D%E4%BA%8B%E7%94%A8%E9%80%9A%E8%A8%8A%E6%89%80%E7%9A%84%E5%8E%9A%E9%87%8D%E9%8B%BC%E9%96%80.jpg). CC BY-SA 4.0, Astraea1020. Viewed. Used only for the colour and pattern of rust on painted steel.
9. [2010 ADA Standards, ch. 7](https://www.access-board.gov/ada/chapter/ch07). U.S. federal work, public domain. Read, not sampled (`sampled = false`).

Limited (looked at and measured only):
10. [ANSI/SDI A250.8-2017 (SDI-100)](https://steeldoor.org/wp-content/uploads/2020/02/A250_8-1.pdf). I read the full PDF text.
11. [Steelcraft Technical Data Manual sec. 2, F Series frames](https://allegion.ca/content/dam/allegion-us-2/web-files/steelcraft/technical-documents/Section__2__Frames_variations_&_options_and_anchoring_systems_110553.pdf). I read the text and viewed manual p. 26, rendered as an image.
12. [Von Duprin 98/99 rim exit device](https://cdn.shopify.com/s/files/1/0562/3493/4478/files/Von_Duprin_98.99.pdf), catalogue pp. 8-9 on a reseller's CDN. Read.
13. [LCN 4040XP catalogue](https://cdn.shopify.com/s/files/1/0562/3493/4478/files/LCN_4040XP_Catalog.pdf). I read the text and viewed p. 37 (the mounting drawing) rendered.
14. [Rockwood K1050 kick plate cut sheet](https://cdn.shopify.com/s/files/1/0562/3493/4478/files/Rockwood_K1050_Cut_Sheet.pdf). Read.
15. [NGP L-FRA100 vision lite kit (Doorways Plus listing)](https://doorwaysplus.com/shop/ngp-l-fra100-steel-door-window-lite-kit-with-diamond-wire-glass-fire-rated-2284). Read through a page summary.
16. [ICC Building Safety Journal, "Stairway signage demystified"](https://www.iccsafe.org/?p=171866) (2017). Read through a page summary.
17. [Schlage ND series data sheet](https://allegion.ca/content/dam/allegion-us-2/web-files/schlage/information-documents/Schlage_ND_Series_Data_Sheet_106486.pdf). Read.
18. [HC Brands 2 x 8 in engraved sign](https://www.hcbrands.com/engraved-sign-horizontal-2-x-8). Read through a page summary.

### The leaf
- Standard opening 3'0 x 7'0 (914 x 2134 mm), 1-3/4 in (44.5 mm) thick: **VERIFIED** [10] Table 1 and sec. 1.4.
- Minimum face thickness by level: L1 20 ga 0.032 in (0.8 mm), L2 18 ga 0.042 in (1.0 mm), L3 16 ga 0.053 in (1.3 mm), L4 14 ga 0.067 in. **VERIFIED** [10] Table 2. The A250.8 appendix gives "nominal" 16 ga as 0.059 in; mills order to the low end of the tolerance (**VERIFIED** [10]).
- Cores that pass: kraft honeycomb, polystyrene, polyurethane, mineral board, vertical steel stiffeners. **VERIFIED** [10] 2.3.2.
- Full flush: one sheet per face, no seams on the faces, a full-height seam allowed on the edges. Seamless: edge seams filled, or welded and dressed smooth. **VERIFIED** [10] 2.3.1.1-2.
- Standard bevel is 1/8 in in 2 in; the narrow side of the edge meets the stop. **VERIFIED** [10] 2.3.1.3.
- Top and bottom are closed with flush or inverted channels at least 0.042 in thick. **VERIFIED** [10] 2.3.1.4. This matches the black sweep and bottom channel seen in [2].
- Steelcraft's "B" series is steel-stiffened with visible seams on the hinge and lock edges. **SEARCH-ONLY** (Steelcraft Architectural Guide; the PDF I downloaded was corrupt).
- Clearances: at most 1/8 in to the frame, undercut at most 3/4 in, 1/16-3/32 in from the face to the stop. **VERIFIED** [10] 2.1.8.
- Actual leaf size **DERIVED**:
  - Width: 36 - 2 x 1/8 = 35-3/4 in (908 mm).
  - Height: 84 - 1/8 - 3/4 = 83-1/8 in (2111 mm).
- Finish: factory rust-inhibiting primer, baked or air-dried, then a finish coat. The finish coat is applied in the field unless the doors are factory finish-painted. **VERIFIED** [10] 2.1.3-2.1.5.
  - The appendix notes that welds and grinding can "show through" the paint, more so at higher gloss. **VERIFIED** [10].
  - It also says thermal bow is reduced by a light colour on the exposed face. **VERIFIED** [10].
  - The lite-kit primers are named beige, gray and tan, plus dark bronze. **VERIFIED** [15]; names only, no colour values.

### The frame
- 16 ga (min 0.053 in) for Level 2/3 doors and 18 ga for Level 1. **VERIFIED** [10] Table 3. Steelcraft F16 is 16 ga, 0.053 in (1.3 mm). **VERIFIED** [11].
- F16 standard profile: face 2 in (50 mm), stop 5/8 in (16 mm), return 1/2 in (13 mm), except 7/16 in (11 mm) on a 5-3/4 in depth. **VERIFIED** [11], profile table and drawing.
- Double rabbet: 1-15/16 in (49 mm) on the door side and 1-9/16 in (40 mm) on the other side. Depths of 4-1/2 in and up are double rabbet; below that, single rabbet. **VERIFIED** [11].
- Soffit at the stop for a 5-3/4 in depth: 5.75 - 1.9375 - 1.5625 = 2.25 in (57 mm). **DERIVED**.
- Head: 2 in face. A 4 in head is an option for masonry. **VERIFIED** [11]. Corners are die-mitred with 4 concealed tabs (knock-down), or welded. **VERIFIED** [11]. A250.8 makes knock-down the default unless welded frames are specified. **VERIFIED** [10] 2.4.1.1.
- A 5-3/4 in depth suits a 4-7/8 in stud-and-drywall wall. An 8 in CMU stair wall would take roughly an 8-3/4 in frame or a butted profile. **ESTIMATE**: I did not check this against a sheet. It matters if the stair is block, as in [1][2].
- The frame is factory-primed with a baked-on rust-inhibiting primer to A250.10. **VERIFIED** [11].

### Hinges, lock and strike
- 3 hinges on a 7'0 leaf. Doors up to 7'6 need at least 3; taller doors need 4. **VERIFIED** [10] 3.1.5. [3] shows 3 hinges.
- Steelcraft frame hinge centrelines: top 9-3/4 in (248 mm) below the head rabbet, bottom 10-3/8 in (264 mm) above the floor, middle hinge equally spaced. Strike CL 40-5/16 in (1024 mm). **VERIFIED** [11] elevation.
- Spacing: (84 - 9.75 - 10.375) / 2 = 31.9375 in = 31-15/16 in (811 mm). **DERIVED**. This matches Steelcraft's tech manual sec. 1, which gives "2 @ 31-15/16" for 7'0 (**VERIFIED**, sec. 1 p. 19; not a numbered ref).
- A250.8 limits: top hinge CL up to 11-3/4 in from the rabbet, bottom up to 13 in from the frame bottom. Locks, latches and panic devices: strike CL 38-42 in. **VERIFIED** [10] Table 5. Text extraction joined the "Rim and Mortise Panic Devices" row to the 38-42 in row; the joining is my reading of the table layout.
- Hinges are 4-1/2 in standard, 5 in optional. Leaf thickness is 0.134 in (standard weight) or 0.180 in (heavy). **VERIFIED** [11]. Hager BB1279 4-1/2 x 4-1/2: 5-knuckle, ball bearing, 0.134 in, US26D. **SEARCH-ONLY**.
- Lever lock (Schlage ND, cylindrical): rose 3-7/16 in (87 mm), backset 2-3/4 in (70 mm), strike 1-1/4 x 4-7/8 in. **VERIFIED** [17]. Rhodes lever length 4-7/8 in: **SEARCH-ONLY** (retailer listings disagree on projection).
- On the door: cylindrical lock CL 40-5/16 - 3/4 = 39-9/16 in above the door bottom (**DERIVED**). Steelcraft sec. 1 states the same figure (**VERIFIED**, not a numbered ref).

### Exit device (push bar), stair side
- Von Duprin 99 rim device. **VERIFIED** [12] for all of these:
  - Length: 3 ft for 2'4-3'0 doors.
  - Centreline: 39-13/16 in (1011 mm) above the floor.
  - Centre case: 8 x 2-3/4 x 2-3/8 in (203 x 70 x 60 mm).
  - Mechanism case: 2-1/4 x 2-1/4 in (57 x 57 mm).
  - Projection: 3-13/16 in (97 mm) at rest, 3-1/16 in (78 mm) pushed.
  - Latch: 3/4 in throw. Strike: 299, dull black.
  - The 99 has a grooved case; the 98 is smooth.
  - Finishes include US26D, US28, 313 and 315.
- Photo check, **DERIVED** from [2] (1920 px): door top y=260, bottom y=1050, so 790 px for about 83-1/4 in. Bar centre at y=662 is 388 px above the bottom, which gives about 40.9 in above the door bottom and about 41.6 in above the floor. That is consistent with 39-13/16 in, given the upward camera angle.
- The bar goes on the side people push to leave. The photos show levers on the pull side ([1], [3], [5]) and bars on the push side ([2], [4]). I did not check the code text for which doors need panic hardware.

### Closer
- LCN 4040XP, regular arm on the hinge (pull) side. **VERIFIED** [13] drawing:
  - Body 12-1/4 in (311 mm) long, projecting 2-1/4 in (57 mm) from the door face.
  - Arm reaches 11 in (279 mm) at 90 deg.
  - The body starts 5/8 in from the hinge-side line.
  - The shoe dimension reads "10-7/16 in / 625 mm". 10-7/16 in is 265 mm, so the metric figure is probably a typo.
- Body height: in the drawing, 12-1/4 in = 283 px and the body is 80 px tall, so about 3.46 in. **DERIVED**. This matches the 3-1/2 in (89 mm) callout.
- Finishes: 689 Aluminum is standard; 690 statuary bronze, 691 light bronze, 693 black, 695 dark bronze and 696 brass are options. **VERIFIED** [13].
- Mountings: hinge side (pull), top jamb (push), parallel arm (push). **VERIFIED** [13]. [2] shows a parallel arm on the push side. [1] and [3] show regular arms on the face where the hinge knuckles show (the pull side). So the door in [3] swings into the corridor.

### Kick plate, vision lite, signs, plaque
- Kick plate (Rockwood K1050). **VERIFIED** [14]:
  - 0.050 in thick, in aluminium, brass, bronze or stainless.
  - Push side: door width less 2 in. Pull side: door width less 1-1/2 in.
  - All edges bevelled; #6 x 5/8 in oval-head screws.
  - NFPA 80 note: no label is needed when the plate's top is 16 in or less above the door bottom.
  - A 34 in wide plate on a 36 in push side is **DERIVED**. Retailers list 10 x 34 and 12 x 34 in plates (**SEARCH-ONLY**).
  - In [1] the plate measures about 10 in high: 113 px against about 11.2 px/in vertically (930 px of door for 83 in). **DERIVED**, roughly +-1 in.
- Vision lite kit (NGP L-FRA100). **VERIFIED** [15]:
  - Visible glass 4x25, 5x20, 6x30 or 10x10 in.
  - 18 ga cold-rolled steel frame with mitred, welded corners.
  - Diamond wire glass, 3/16, 1/4 or 5/16 in.
  - The listing itself says the kit does not meet IBC 2006+ rules for doors.
  - 4 x 25 = 100 sq in (**DERIVED**). That is the usual cap for fire-protective glazing in 60/90-min doors: CDF Distributors' blog (VERIFIED, not numbered) says "100 square inches (maximum dimension 10 inches)". The 10 in figure contradicts the 4x25 and 5x20 kits that are sold, so it is doubtful.
  - [5]: the glass is 114 x 700 px, an aspect of 6.1:1, which fits 4 x 25 in (6.25:1). The trim is about 35 px a side, about 1-1/4 in. The wire diamonds repeat about every 29 px, about 1 in across. All **DERIVED** on the 4 in assumption.
  - [1]: about 5 x 23 in glass and 8 x 27 in over the trim. **DERIVED**, +-15%, from mixed horizontal and vertical scales (13.2 and 11.2 px/in).
- Stairway ID sign inside the stair: at least 18 x 12 in, at each floor landing of exit stairs connecting more than 3 stories. Content:
  - the floor level (largest text);
  - the level of exit discharge and the direction to it;
  - the stair designation ("Stair A");
  - the stair's extent;
  - whether there is roof access.

  The bottom edge is about 5 ft above the floor (the article's guidance, not a code limit). Self-luminous or photoluminescent where luminous path markings are required. **VERIFIED** [16]. The section number is 1023.9 in recent IBCs (**SEARCH-ONLY**; one checklist cites 1023.10).
- Tactile floor sign beside the door, on the latch side: **VERIFIED** [16] ("VRB adjacent to each door on the latch side").
- ADA tactile sign rules. **VERIFIED** [9]:
  - The lowest tactile baseline is at least 48 in (1220 mm) above the floor; the highest at most 60 in (1525 mm).
  - An 18 x 18 in clear floor space beyond the door swing.
  - Characters uppercase, raised at least 1/32 in, 5/8-2 in tall.

  A common tactile level-sign size of 2.875 x 7.875 in is **SEARCH-ONLY**.
- Engraved plaque: 2 x 8 in, 1/16 in (1.6 mm) plastic. Engraving cuts away the top layer to show the core colour. The most popular pair is black with white letters. **VERIFIED** [18]. 2 x 8 and 2 x 10 sizes, "2-ply" and 0.063 in ABS with an adhesive back are **SEARCH-ONLY** (Staples, Quill, McMaster). Brown with white letters was not confirmed.
  - [6]: the plaque is about 4.4:1 (355 x 80 px at 1920). **DERIVED**.
  - [1]: the brass "Lobby" plaque is about 10 x 4 in. **DERIVED**, +-15%.
- Fire label: on the hinge edge or the frame. Paint must not cover it. **VERIFIED**: SDI-124 (not numbered) says "Certification labels present shall not be painted over during any repainting."

### Palette (all photo-sampled except one)
Method: ffmpeg crops of the 1920 px Commons thumbnails, then an area-average to 1 px in sRGB. Values include the scene's lighting and white balance. [2] is warm (its white CMU reads 235,219,193), [5] has a yellow-green cast, and [8] is overcast. Correct for white balance before using any value as an albedo.
- **Leaves:**
  - red-brown 50,23,14 [1]
  - beige 161,133,90 [2]
  - grey 76,68,52 [2] (about 76,73,63 after scaling by the wall's white)
  - near-black brown 42,42,41 [4]
  - putty 172,162,127 [5]
  - grimy off-white 93,85,72 [7]
- **Frames:** beige 194,162,120 [1] and 235,203,151 [3]; dark red 42,18,12 [2]; slate 49,46,43 [4].
- **Hardware:**
  - push bar 148,133,108 (warm light) [2] and 172,181,181 (cool light) [4]
  - brass kick plate 164,129,73 [1]
  - bronze closer 152,126,76 [1]
  - satin chrome escutcheon 79,78,57 [5] (cast, reads dark; prefer a standard US26D value)
  - black sweep 29,23,17 [2]
  - grimy kick plate 21,17,11 [7]
- **Signs:** stair sign grey 175,170,172 [3]; plaque silver 155,149,122 [6]. The plaque black 24,24,24 [18] is an **ESTIMATE**: the source names the colour but gives no value.
- **Wear:** worn sill paint 132,113,81 [2].
- **Rust** [8]:
  - dark rust on the frame 64,45,36
  - bloom through paint 149,105,87 (mean of the 10% most-orange pixels; the single most-orange pixel is 187,122,84)
  - mixed scale 121,97,78
  - streaked grey paint 135,133,148
- Not sampled: a neutral grey enamel or primer. Every grey sample I found was under a warm cast.

### Ageing (what the photos show, and where)
- **Bottom of the door** [2]:
  - a black sweep strip with screws
  - a dirt line along the bottom
  - dark scuffs on the lower 12 in of the leaf
  - the sill or threshold below worn through to lighter layers, with chipped edges
- **Lower leaf** [7]: hand smears, vertical drips and boot scuffs, and a kick plate dulled to near-black. [6]: dents and long scratches on the leaf face.
- **Rust** [8] starts at the frame edges and the bottom, then blooms through the paint in spots and runs down in streaks. For corridor doors, rust at the bottom edge and the hinge side is plausible: CDF Distributors (not numbered) says chips should be repaired "to prevent rust formation". I have no photo of a rusted hollow-metal stair door bottom. **ESTIMATE** for the exact placement.
- **Closer oil**: SDI-124 lists "fluid leakage" among closer defects (**VERIFIED**). I found no photo of an oil streak.
- **Sticker residue and cart scuffs**: no photo found. The cart and kick-plate link is from CDF ("Kick plates ... protect the bottom rail from foot traffic and cart damage"). **ESTIMATE** for the look.
- **Taped notices and stickers** near the door: seen on the wall in [1]. Signs fixed straight to the leaf: [2].

### Other sources read (not numbered)
- [Steelcraft TDM sec. 1](https://allegion.ca/content/dam/allegion-us-2/web-files/steelcraft/technical-documents/Tech_Data_Manual_Section__1__General_Information_110552.pdf):
  - Door nomenclature: 18 ga = 0.042 in, cylindrical lever prep "61L", narrow lite "N".
  - 3 hinges standard on 6'8-7'6.
  - A comparison table of hinge and strike locations across Steelcraft, Ceco, Curries, Mesker, Republic and others. Curries puts the strike at 40 in, Ceco at 41-13/16 in, Pioneer at 38 in.
- [SDI-124](https://steeldoor.org/wp-content/uploads/2020/02/SDI_124.pdf): maintenance (closer fluid leakage, repainting, labels).
- [CDF: vision panels](https://www.cdfdistributors.com/blog/post/vision-panel-fire-rating-requirements-commercial-doors) and [CDF: maintaining metal doors](https://www.cdfdistributors.com/blog/post/cleaning-maintaining-commercial-metal-doors).
- [Commons: Fire door label](https://commons.wikimedia.org/wiki/File:Fire_door_label.jpg) (CC BY-SA 3.0, Ben Schumin, viewed). A Steelcraft 1-1/2 h (B) label, oval with 2 rivets, on the frame head, painted over at the edges. A good decal reference; left out to keep the count down.
- Commons, viewed but not used: Panic bar.jpg (rim device close-up, CC BY-SA 3.0); Door closer.jpg (PD, closer on a stair door); LumAware stairwell (red stair door, CC BY-SA 4.0, promotional).

### Could not verify / doubtful
- Grey enamel: no free photo of a grey hollow-metal stair door in neutral light. No maker publishes RGB values for its primer or prefinish colours (I checked none beyond the NGP names).
- No free photo of rust at the bottom of a hollow-metal stair door or frame, and no closer oil streak, sticker residue or cart-scuff close-up.
- No free photo of an abandoned building's stair door. [7] is an exit door in an abandoned factory.
- Panic-device height: VD gives 39-13/16 in, and A250.8 puts it in the 38-42 in strike range (my reading of the table layout). The photo derivation gives about 41-42 in, with perspective error.
- The "max dimension 10 in" for 100 sq in lites (CDF) conflicts with the 4x25 and 5x20 kits that are sold. NFPA 80 text not read.
- Wired glass is restricted for human-impact locations in new work ([15], CDF). It is period-correct for older stairs only.
- Engraved plaque "two-ply" and brown/white colour: SEARCH-ONLY. The 2 x 8 x 1/16 in size and black/white pair are VERIFIED [18].
- Leaf finish in [1] and [3]: [3] is wood-grain (veneer or print). [1] looks painted steel but is not confirmed.
- Frame depth for CMU stair walls (8-3/4 in): ESTIMATE.
- Steelcraft B-series edge seams, Hager hinge data, Rhodes lever length and the 2.875 x 7.875 in tactile level sign: SEARCH-ONLY.
- Reference count is 18 (9 free, 9 limited), above the 10-14 aim, because each hardware item has its own maker sheet. Refs 7, 15 and 17 are the easiest to drop.
