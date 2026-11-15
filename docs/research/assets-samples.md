# References: almond water and the battery

Gathered on 2026-11-15 by a research agent for GitHub issue #13 (the asset
pipeline's samples, [`../plan/ASSETS.md`](../plan/ASSETS.md) A4-A6): the
measurements, materials, palette, references (each with its licence class)
and where such things are found, for a plain 500 ml PET water bottle (the
canon's almond water) and an alkaline D cell. The dossiers that use it are
`assets/src/props/almond_water/` and `assets/src/props/battery/`; what
could not be verified is listed at the end. Kept as found.

Collected 2026-11-15 using web search and web fetch only. All figures carry a source link.

**Status markers used below**
- **VERIFIED**: read directly on the cited page or document.
- **SEARCH-ONLY**: seen only in a search-engine summary. The page itself was not opened or could not be fetched.
- **DERIVED**: calculated by me from verified figures. The working is shown.
- **ESTIMATE**: no source found. This is an artistic starting value; calibrate it against the reference photos.

The "viewed" note in the reference tables means I looked at a 330 px thumbnail of the image myself. "Not viewed" means the description comes only from the file page's caption.

**Colour values.** All physicallybased.info values are linear sRGB (I confirmed the `"colorSpace": "srgb-linear"` label in the v2 JSON at https://raw.githubusercontent.com/AntonPalmqvist/physically-based-api/main/deploy/v2/materials.json). I converted them to 8-bit sRGB with the standard sRGB transfer function (`1.055·L^(1/2.4) − 0.055`).

---

## 1. Almond water

Canon: a plain single-use 500 ml (16.9 fl oz) PET water bottle with a screw cap and nothing printed on it. The liquid is very pale or faintly cloudy. Some bottles are full and sealed; others are opened and partly drunk.

### 1.1 Measurements

| Feature | Value to model | Range / notes | Source | Status |
|---|---|---|---|---|
| Nominal fill | 500 ml (16.9 fl oz) | — | [Petainer 500 ml water bottle](https://petainer.com/pet-plastic-bottles/water-bottles/500ml-water-bottle-28mm-pco-1810) | VERIFIED |
| Overall height (incl. cap) | **205 mm** (chosen) | 195 mm (Sidel X-LITE Still, ultra-light still water) · 203 mm (8 in, generic 16.9 oz bottle) · 216.5 mm (Petainer 500 ml, PCO 1810 neck) | [Sidel X-LITE Still](https://www.sidel.com/services/x-lite-still/); [dimensions.com 16.9 oz bottle](https://www.dimensions.com/element/soda-bottle-16-9-oz); [Petainer](https://petainer.com/pet-plastic-bottles/water-bottles/500ml-water-bottle-28mm-pco-1810) | VERIFIED (each value); the 205 mm choice is mine |
| Body diameter | **65 mm** | 63.5 mm (2.5 in, dimensions.com) to 65 mm (Petainer) | same as above | VERIFIED |
| Empty bottle weight | 9–10 g for a modern lightweight still-water bottle | 6.5 g (X-LITE) · 7.95 g (Sidel RightWeight, 26/22 cap) · 8.8 g (Krones 2007) · 9.9 g (Krones) · 24 g (Petainer, heavy PCO 1810 neck) | [Sidel X-LITE](https://www.sidel.com/services/x-lite-still/); [Food Ingredients First, 8.8 g bottle](https://www.foodingredientsfirst.com/news/ultra-light-88-gram-pet-bottle-breaks-weight-record.html); RightWeight and 9.9 g from search summaries of [PlasticsToday](https://www.plasticstoday.com/plastics-processing/sidel-s-new-pet-water-bottle-sheds-some-weight) (403 on fetch) and [Packaging Europe](https://www.packagingeurope.com/sidel-introduces-x-lite-still/) | 6.5, 8.8 and 24 g VERIFIED; 7.95 and 9.9 g SEARCH-ONLY |
| Body wall thickness | **0.10 mm** (body); use 0.10–0.15 mm | Patent for thin-wall PET containers: 50–150 µm average | [Food Ingredients First, "0.1 millimetre" wall](https://www.foodingredientsfirst.com/news/ultra-light-88-gram-pet-bottle-breaks-weight-record.html); [US12420977B2](https://patents.google.com/patent/US12420977) | 0.1 mm VERIFIED; patent range SEARCH-ONLY |
| Neck (finish) wall thickness | ≈1.24 mm at the thread root | (E 24.20 − bore 21.72) / 2 | [ISBT PCO-1881 drawing](https://cdn.prod.website-files.com/646b9f8f61d2f66e1f5d5e20/65131b77ef381c1c6e2c0daa_3784253-21.pdf) | DERIVED |
| Neck finish standard | **PCO-1881, 28 mm** (as briefed) | Many still-water bottles use a lighter 26 mm finish instead: Krones 8.8 g bottle "26-millimetre modified PCF 26 P"; Sidel retrofits "from 29/25 to 26/22mm" | [ISBT Threadspecs index](https://isbt.com/threadspecs.asp); [Food Ingredients First](https://www.foodingredientsfirst.com/news/ultra-light-88-gram-pet-bottle-breaks-weight-record.html); [Sidel](https://www.sidel.com/services/x-lite-still/) | VERIFIED |
| PCO-1881 thread outer diameter (T) | Ø27.40 ±0.13 mm | — | ISBT drawing 3784253-21 (2007-02-14) | VERIFIED |
| PCO-1881 thread root diameter (E) | Ø24.20 mm | G Ø25.07 max; F Ø24.94 ±0.13 | ISBT drawing | VERIFIED |
| PCO-1881 bore | Ø21.72 ±0.13 mm | — | ISBT drawing | VERIFIED |
| PCO-1881 support ledge (neck ring) | Ø33.00 ±0.15 mm | Bottle just below the ring: Ø26.19 +0.40/−0.50, straight for ≥5.00 mm | ISBT drawing | VERIFIED |
| PCO-1881 bead / other diameters | Ø29.15 ±0.15 (labelled M); Ø28.00 ±0.15 (two features) | The role of each lettered feature (tamper bead vs. locking ring) was not checked against the ISBT nomenclature card | ISBT drawing | Values VERIFIED; feature names UNVERIFIED |
| PCO-1881 finish height | 17.00 ±0.25 mm overall; H = 15.24 ±0.15 | PCO 1810 is 21 mm, so 1881 is 4 mm shorter | ISBT drawing; [dercuano notes](https://dercuano.github.io/notes/bottlecaps.html) | VERIFIED |
| Thread | Pitch 2.70 mm; the drawing says "right hand thread, single start, 650° turn" | Conflict: Pelliconi calls its 1881 cap "2-start"; dercuano says the thread wraps "only twice" | ISBT drawing; [Pelliconi 28 mm 1881](https://www.pelliconi.com/product/plastic-caps-28mm-1881) | VERIFIED, but the sources conflict |
| Finish weight | 3.61 g (finish cut under the support ring) | Bericap: 3.74 g vs 5.05 g for PCO 28 | ISBT drawing; [Bericap news](https://www.foodingredientsfirst.com/news/new-bericap-neck-finish-according-to-new-isbts-pco-1881.html) | 3.61 g VERIFIED (small print); 3.74 g SEARCH-ONLY |
| Cap outer diameter | **31.5 mm** | — | [ALPLA PCO1881 GME32.04 "Mogador"](https://product-catalogue.alpla.com/product/caps/pco1881-gme3204-mogador) | VERIFIED |
| Cap height | **15.4 mm** | — | ALPLA (same page) | VERIFIED |
| Cap weight | ~2.15 g (ALPLA, HDPE); 2.55 g (Berry CompactGuard, HDPE) | — | ALPLA; [Berry/Amcor CompactGuard](https://packagingsolutions.amcor.com/en/product/product-item/28mm-compactguard-pco1881-47-vol-14401740) | ALPLA VERIFIED; Berry SEARCH-ONLY |
| Cap for a 26 mm water neck (alternative) | Not found | Sidel X-LITE pairs with "Novembal Novaqua RightCAP26"; Krones 8.8 g uses "Bericap SK 26/15FB lw" | Sidel; Food Ingredients First | Names VERIFIED; dimensions NOT FOUND |
| Shoulder | Sloped, rounded-conical shoulder from the neck ring to the body | "Tall, ribbed bottle with sloped shoulder" (8.8 g bottle); "wider midsection which narrows towards the tip" | Food Ingredients First; dimensions.com; the CC0 photo Botella_pet.JPG (ref B1) | VERIFIED (qualitative); no dimensioned profile found |
| Ribs and grip bands | 5–6 horizontal hoop ribs on the upper body; wavy/chevron grip bands on the lower body (as in ref B1) | Hoop ribs give hoop strength and keep thin walls round; vertical ribs act as hinges | Ref B1 (viewed); patent literature summarised in search: [US7032770B2](https://patents.google.com/patent/US7032770B2/cs) and hot-fill patents | Photo VERIFIED; patent rationale SEARCH-ONLY |
| Base | Still water: low-pressure "star" base with radial ribs or feet and a central push-up (Sidel StarLITE family). True petaloid feet are for carbonated drinks. Ref B1 shows a five-lobed footed base. | StarLITE HPC: "anti-rocking" base with "360-degree continuous contact surface", "can be combined with structural ribs and oval push-up" | [Beverage Industry, StarLITE HPC](https://www.bevindustry.com/articles/94328-sidel-introduces-starlite-hpc-base-to-increase-stability-of-pet-rpet-containers); [Sidel X-LITE](https://www.sidel.com/services/x-lite-still/) ("StarLITE Still base"); petaloid vs still from search summary ([vienna.at](https://www.vienna.at/ever-noticed-this-is-why-the-bottom-of-pet-bottles-looks-like-this/10397873)) | StarLITE VERIFIED; petaloid/still rule SEARCH-ONLY |
| Injection gate | Small round nub or dimple at the centre of the base, where the preform was injected | Visible in ref B1 | Ref B1 | VERIFIED (photo only) |
| Brimful capacity | ~520–560 ml for a 500 ml bottle | Example 500 ml bottles: 519, 521, 540, 559 ml brimful (mixed glass/PET) | Search summary of [Berry 500 ml Vita Round](https://berryglobal.com/en/product/product-item/500ml-vita-round-13823479), [Miron](https://www.miron.com/products/aquarius-bottle-500-ml-pp28-SM191004-204) and others | SEARCH-ONLY |
| Fill level (sealed, full) | Liquid line in the lower shoulder, about 35–45 mm below the top of the cap | O.Berk: at standard capacity the contents "usually fill up to its shoulder area" | [O.Berk](https://www.oberk.com/packaging-crash-course/quick-question-what-is-the-difference-between-standard-and-overflow-capacity) (certificate error on fetch) | SEARCH-ONLY; the mm figure is an ESTIMATE |
| Fill level (opened) | Free: anything from just below the shoulder to a few mm of liquid. Opened bottles should show cap-off or loose-cap states and a broken tamper ring left on the neck. | — | — | ESTIMATE (design choice) |

### 1.2 Materials and how they look

**Bottle: PET (polyethylene terephthalate)**
- Optics: IOR 1.575, density 1380 kg/m³, base colour white, transmission 1, roughness 0 ([physicallybased.info "Plastic (PET)"](https://api.physicallybased.info/materials)). VERIFIED.
- Tint: clear PET often gets a blue toner to cancel yellowing, especially with recycled PET. Ampacet's Blue Edge 78 gives "a lighter bluish tone" and is recommended for water bottles. Avient's ColorMatrix Optica keeps packaging "neutral or blue" ([Avient bulletin](https://avient.com/sites/default/files/2025-03/ColorMatrix%20Optica%20PET%20Recycle%20Blue%20Toners%20Product%20Bulletin.pdf), [Ampacet](https://www.ampacet.com/?p=1874)). SEARCH-ONLY. A faint blue-grey cast is therefore right, strongest where you look through two walls edge-on.
- Clarity and haze: the stretched body wall is clear. Haze targets in a PET container patent are "<15%, preferably <10%, ideally <5%" ([US5989661](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5989661)). The neck finish stays "substantially amorphous" at 0–2 % crystallinity ([US5829614](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/5829614)). SEARCH-ONLY. In practice the thick neck and base gate look glassier and slightly greener or bluer than the thin body.
- Light: thin curved walls give long, sharp, broken highlights along each hoop rib. Refraction and dispersion through a bottle are shown in ref B7. Water inside acts as a cylindrical lens: it magnifies and inverts what is behind it and throws a bright caustic line on the surface below.
- Wear: a 0.1 mm wall dents and creases easily. Squeezed bottles keep permanent crinkle facets, and sharp folds go white (stress whitening). Scuffs show as fine matte scratches on the ribs and base feet, which touch surfaces. ESTIMATE / observation: no technical source found for scuff or whitening appearance.
- Crushing: empty bottles with the cap on resist crushing (trapped air). With the cap off they collapse into a flat, faceted shape. ESTIMATE.

**Cap: HDPE (sometimes PP)**
- Material: HDPE ([ALPLA](https://product-catalogue.alpla.com/product/caps/pco1881-gme3204-mogador); [Pelliconi](https://www.pelliconi.com/product/plastic-caps-28mm-1881): "manufactured from HDPE material"). VERIFIED. dercuano says caps are "usually made of polypropylene", and ALPLA also sells a two-piece PP 1881 closure ([ALPLA XTRA-LOK](https://www.alpla.com/es/node/522), SEARCH-ONLY). For a modern one-piece water cap, use HDPE.
- Form: vertically knurled skirt (fine ribs for grip), flat top, and a tamper-evident band below the skirt held by thin bridges. ALPLA: "slitted and folded tamper-evident band". After opening, the band stays on the neck as a loose ring sitting on the support ledge (see ref B8). VERIFIED (ALPLA text and photo).
- Look: HDPE is waxy and semi-matte, translucent at the edges (subsurface scattering), with moulded (not printed) top-surface texture. For "nothing printed on it", use a plain white or natural (milky translucent) cap. The colour is ESTIMATE.

**Liquid**
- Water: IOR 1.3325. "Pure water has a slight blue color" ([physicallybased.info "Water"](https://api.physicallybased.info/materials)). VERIFIED.
- Almond water: canon wants it very pale or faintly cloudy. Use water IOR with a very low-density volume scatter tinted like dilute milk (Milk albedo in the palette). ESTIMATE: no source for the canon drink.

### 1.3 Palette

| Material | sRGB (8-bit) | Hex | Other parameters | Source | Status |
|---|---|---|---|---|---|
| PET bottle wall, base colour | 255, 255, 255 | #FFFFFF | IOR 1.575, transmission 1, roughness 0 | [physicallybased.info Plastic (PET)](https://api.physicallybased.info/materials) | VERIFIED |
| PET blue toner tint (transmission tint for a thin shell) | ≈ 244, 248, 252 | #F4F8FC | Keep it subtle. Calibrate against refs B1 and B4. | Direction from [Avient](https://avient.com/sites/default/files/2025-03/ColorMatrix%20Optica%20PET%20Recycle%20Blue%20Toners%20Product%20Bulletin.pdf) / [Ampacet](https://www.ampacet.com/?p=1874) ("lighter bluish tone") | ESTIMATE (value), SEARCH-ONLY (direction) |
| Water | 255, 255, 255 | #FFFFFF | IOR 1.3325, transmission 1 | [physicallybased.info Water](https://api.physicallybased.info/materials) | VERIFIED |
| Almond-water scatter albedo (dilute) | 233, 233, 215 | #E9E9D7 | Linear (0.815, 0.813, 0.682), the Milk entry. Use at very low density so the liquid only looks faintly milky. | [physicallybased.info Milk](https://api.physicallybased.info/materials) | VERIFIED value; its use for almond water is ESTIMATE |
| Cap, white HDPE | ≈ 238, 238, 234 | #EEEEEA | Roughness ≈ 0.4, some subsurface | none found | ESTIMATE |
| Cap, natural HDPE (milky) | ≈ 225, 228, 222 | #E1E4DE | High subsurface, slight transmission | none found | ESTIMATE |
| PP (if a PP cap is preferred) | 255, 255, 255 | #FFFFFF | IOR 1.492 (base data for clear PP) | [physicallybased.info Plastic (PP)](https://api.physicallybased.info/materials) | VERIFIED |

### 1.4 References

| # | URL (direct file) | Page | Shows | Licence class | Licence / author | SHA-1 | Notes |
|---|---|---|---|---|---|---|---|
| B1 | https://upload.wikimedia.org/wikipedia/commons/0/02/Botella_pet.JPG | https://commons.wikimedia.org/wiki/File:Botella_pet.JPG | **The thing.** An empty, completely unlabelled ~500 ml PET water bottle with a white knurled cap. Shows hoop ribs on the upper body, wavy grip bands below, a footed base with the gate dimple, and the sloped shoulder. Studio shot on beige. 1045×2431. | free | CC0, MdeVicente | d287786292d5ac9f374cdfa817d48ef242324dc8 | Viewed. Best single shape reference and already matches canon (no label). |
| B2 | https://upload.wikimedia.org/wikipedia/commons/8/8a/Quiet_time_in_Hotel_Ibis.JPG | https://commons.wikimedia.org/wiki/File:Quiet_time_in_Hotel_Ibis.JPG | **In a place (desk).** A labelled 500 ml still-water bottle with a blue cap on a built-in desk/shelf against a textured wall, beside a plastic cup and room key. Hotel room, cool daylight. | free | CC BY-SA 4.0, Philippe Alès | be8da2267a3637a0cf7024079a9a3a2d298cf90c | Viewed. Good for scale against a cup and for liquid-level look. Has a brand label, so do not copy it. |
| B3 | https://upload.wikimedia.org/wikipedia/commons/0/04/202410_%E6%8A%8A%E7%A9%BA%E5%AF%B6%E7%89%B9%E7%93%B6%E7%95%B6%E9%96%80%E6%AA%94%E9%98%BB%E6%AD%A2%E9%97%9C%E9%96%80.jpg | https://commons.wikimedia.org/wiki/File:202410_%E6%8A%8A%E7%A9%BA%E5%AF%B6%E7%89%B9%E7%93%B6%E7%95%B6%E9%96%80%E6%AA%94%E9%98%BB%E6%AD%A2%E9%97%9C%E9%96%80.jpg | **In a place (floor).** An empty PET bottle lying on its side on a tiled floor, wedged against a door frame as a doorstop. Shows how an empty bottle rests and catches light at floor level. | free | CC BY 4.0, Junyu-K | 44c2f91b1851296597325746cc99db27fa1df126 | Viewed. Has a label. |
| B4 | https://upload.wikimedia.org/wikipedia/commons/e/ea/SZ_Shenzhen_Futian_Shennan_Middle_Road_Shenzhen_Museum_Ancient_Art_Museum_vending_machine_plastic_bottle_water_March_2025_R12S.jpg | https://commons.wikimedia.org/wiki/File:SZ_Shenzhen_Futian_Shennan_Middle_Road_Shenzhen_Museum_Ancient_Art_Museum_vending_machine_plastic_bottle_water_March_2025_R12S.jpg | **In a place (vending machine).** Rows of small PET water bottles standing on lit shelves behind glass. Strong cool-blue LED cast through the PET and water, with white caps. | free | CC0, AIOLIAN 2025 | 2c30f1fe2320f82684eb89db8c023102da1cbdc7 | Viewed. Best reference for how stacked water bottles glow under machine lighting. |
| B5 | https://upload.wikimedia.org/wikipedia/commons/5/51/Mira_Sintra-Mele%C3%A7as_Train_Station_Vending_Machines.jpg | https://commons.wikimedia.org/wiki/File:Mira_Sintra-Mele%C3%A7as_Train_Station_Vending_Machines.jpg | **In a place (vending machine, exterior).** A bottled-water vending machine next to a coffee and snack machine in a bare station hall with a tiled floor. | free | Public domain, Ivo Emanuel Gonçalves | b94d5b55314a0c727a83e9eb7e36646b4d87e195 | Viewed. Shows the machine in a utilitarian interior. The water machine has a branded front, so do not copy it. |
| B6 | https://upload.wikimedia.org/wikipedia/commons/4/46/%E5%AE%87%E6%B2%BB%E8%8C%B6%E3%83%9C%E3%83%88%E3%83%AB%E8%87%AA%E8%B2%A9%E6%A9%9F.jpg | https://commons.wikimedia.org/wiki/File:%E5%AE%87%E6%B2%BB%E8%8C%B6%E3%83%9C%E3%83%88%E3%83%AB%E8%87%AA%E8%B2%A9%E6%A9%9F.jpg | **In a place (vending machine).** PET drink bottles (tea) displayed in a lit two-row sample window of an indoor vending machine. | free | CC0, おいでやす千年の都 | 01789a75f37dc479d9a30b5f9a2ab358ea23a671 | Viewed. Tea, not water, but the same bottle class and display layout. |
| B7 | https://upload.wikimedia.org/wikipedia/commons/9/91/Brechung_und_Dispersion_PET.JPG | https://commons.wikimedia.org/wiki/File:Brechung_und_Dispersion_PET.JPG | Refraction and dispersion of light through a PET bottle (caption). | free | CC BY-SA 3.0, McZusatz | 1ce2c7e1e60ab706ebc55f596a676b47ffdcd0e6 | Not viewed. For checking caustics and refraction. |
| B8 | https://upload.wikimedia.org/wikipedia/commons/2/24/Screw_cap%2C_plastic_bottle.jpg | https://commons.wikimedia.org/wiki/File:Screw_cap,_plastic_bottle.jpg | Close-up of a knurled plastic screw cap on a PET bottle neck. Shows the tamper band with its bridges, the support ring and the start of the shoulder. | free | CC BY-SA 4.0, Андрей Романенко | 12a24fb0a5f75f91307fb231a2d99ba005ff2a22 | Viewed. Brown beer-style PET and a gold cap, so use it for geometry only. |
| B9 | https://upload.wikimedia.org/wikipedia/commons/8/87/PET_bottles_in_a_trash_can_%28Prague%29.jpg | https://commons.wikimedia.org/wiki/File:PET_bottles_in_a_trash_can_(Prague).jpg | **In a place (bin).** PET bottles in a trash can (caption). | free | Public domain (author not given in metadata) | 82cbf4959738f91681c24be2ad4391cd739f8efa | Not viewed. |
| B10 | https://cdn.prod.website-files.com/646b9f8f61d2f66e1f5d5e20/65131b77ef381c1c6e2c0daa_3784253-21.pdf | https://isbt.com/threadspecs.asp | Document: ISBT PCO-1881 28 mm finish engineering drawing (3784253-21, 2007-02-14), with all neck dimensions used in 1.1. | limited | ISBT voluntary guideline; all rights reserved | — | Read it for dimensions only. |
| B11 | https://product-catalogue.alpla.com/product/caps/pco1881-gme3204-mogador | (same) | Document: ALPLA HDPE PCO1881 cap spec (Ø31.5 × 15.4 mm, ~2.15 g, slitted folded tamper band). | limited | ALPLA, all rights reserved | — | For dimensions. |
| B12 | https://pixel-shot.com/en/showcase/i-4252231-0/workplace-with-bottle-of-water--lamp-and-notebook-on-table-in-modern-office | (same) | **In a place (office desk).** Stock photo titled "Workplace with bottle of water, lamp and notebook on table in modern office". | limited | Stock licence (all rights reserved for our purpose) | — | Not viewed; the title comes from search. Mood only. |

Free count: 9 (B1–B9). In-place count: B2 desk, B3 floor, B4/B5/B6 machine, B9 bin, B12 office desk (limited).

### 1.5 Where found

- **Office and workplace emergency kits.** Ready.gov lists "Water (one gallon per person per day for several days, for drinking and sanitation)" for a basic kit. For work it says: "Be prepared to shelter at work for at least 24 hours", with water in a "grab and go" case ([ready.gov/kit](https://www.ready.gov/kit), VERIFIED). A Pennsylvania workplace checklist says to store water "in plastic containers such as soft drink bottles" (SEARCH-ONLY, [PA ready](https://prdready.pwpca.pa.gov/BePrepared/BuildKit/Pages/For-The-Workplace.aspx), host unreachable). A university office-kit list includes "Bottled Water" (SEARCH-ONLY, [BYU](https://emergencymanagement.byu.edu/office-emergency-preparedness-kits)).
- **Vending machines and break rooms.** Water machines appear in public halls (refs B4–B6). Federal and worksite vending guidance covers water in vending ([CDC/GSA guidelines archive](https://webarchive.library.unt.edu/web/20141206000655/http://www.cdc.gov/chronicdisease/resources/guidelines/food-service-guidelines.htm), SEARCH-ONLY; I did not read the beverage table).
- **Delivered by the case.** Offices buy 16.9 oz water in 24-bottle cartons (e.g. an office-supply listing "16.9 oz., 24/Carton", [Quill](https://Quill.com/ice-mountain-100-natural-spring-water-169-oz-24-carton-12119419/cbs/216448.html), SEARCH-ONLY). Expect shrink-wrapped flats of 24 on storage-room floors and shelves, and single bottles on desks (ref B2), in desk drawers, on meeting-room tables and by printers.
- **Never (sourced).** US workplace rules forbid storing or consuming food and drink "in toilet rooms or in an area exposed to a toxic material" ([OSHA 29 CFR 1910.141(g)(2),(g)(4)](https://www.osha.gov/laws-regs/regulations/standardnumber/1910/1910.141), VERIFIED). So no bottles should appear in restrooms, chemical or janitorial stores, or battery-charging rooms.
- **Never (unsourced, convention).** Inside electrical and server rooms, on top of running equipment, or inside ceiling plenums. Treat as design convention, not fact.
- **Lore note (UNVERIFIED).** Commercially sold bottled water carries a mandatory label (FDA food labelling), so a sealed bottle with *nothing* printed on it is itself unnatural. I did not fetch the regulation. Unprinted stock does exist as samples (Myron Promos: samples "sent either randomly printed or blank", SEARCH-ONLY).

---

## 2. Battery (D cell, IEC LR20, alkaline)

Canon: the cell that "recharges" the big hand torch. Real LR20 alkaline cells are primary (non-rechargeable). Energizer warns "Don't recharge a battery unless it is specifically marked 'rechargeable'". A rechargeable D would be NiMH (IEC HR20). This does not affect the model.

### 2.1 Measurements

| Feature | Value to model | Range / notes | Source | Status |
|---|---|---|---|---|
| Designation | IEC LR20, ANSI 13A, 1.5 V, Zn/MnO₂ | Other names: Mono, UM-1, #1, BA-30 | [Energizer E95 datasheet (2023)](https://files.batteryjunction.com/frontend/files/energizer/datasheet/ENERGIZER-E95-UpdatedDataSheet.pdf); [Wikipedia D battery](https://en.wikipedia.org/wiki/D_battery) | VERIFIED |
| Body diameter | **33.2 mm** (nominal) | 32.30 min – 34.20 max | Energizer E95 drawing; [VARTA 4020 datasheet via Novapart](https://novapart.co/datasheet/farnell:3604394/); Wikipedia "33.2 ± 1 millimeters" | VERIFIED |
| Overall height (incl. nub) | **61.0 mm** (chosen) | 59.50 min – 61.50 max | Energizer E95 drawing; VARTA (A 61.5 max, B 59.5 min) | VERIFIED range; 61.0 is my choice |
| Positive nub diameter | **9.0 mm** (chosen) | ≤ 9.50 mm | Energizer E95 drawing; VARTA F 9.5 max | VERIFIED limit |
| Positive nub projection | **1.8 mm** (chosen) | ≥ 1.50 mm above the shoulder | Energizer E95 drawing; VARTA G 1.5 min | VERIFIED limit |
| Negative flat contact diameter | **≥ 18.0 mm**; model about 20 mm | ≥ 18.00 mm | Energizer E95 drawing ("18.00 (0.709) Minimum"); VARTA C 18.0 min | VERIFIED |
| VARTA dimension E | ≤ 1.0 mm | Probably the maximum recess of the negative contact below the crimp rim. The letter is not defined on the page. | VARTA via Novapart | Value VERIFIED; meaning UNVERIFIED |
| Mass | 139–144 g | 139 g (E95, 2023); 144.0 g (E95, 2014); ~140.4 g (VARTA 4020) | E95 2023; [E95 2014 (ELV copy)](https://media.elv.com/file/120089_mono_d_data.pdf); VARTA | VERIFIED |
| Volume | 51–56 cm³ | 56 cm³ (Energizer); 51.3 cm³ (VARTA) | same | VERIFIED |
| Terminal type | Flat contact (no snap or solder tabs) | — | E95 datasheets ("Terminal: Flat Contact") | VERIFIED |
| Steel can | Steel can, also the cathode collector. Positive end is the closed can bottom with a welded positive cover (the nub). | Can wall thickness NOT FOUND | [Energizer Alkaline Handbook](https://data.energizer.com/pdfs/alkaline_appman.pdf) fig. 4 and text ("Steel can confines active materials and serves as the cathode collector") | VERIFIED; thickness missing |
| End covers | "Positive and negative covers provide contact surfaces of nickel-plated steel" | — | Energizer Alkaline Handbook | VERIFIED |
| Negative end, crimp and seal | Open end of the can closed by an annular nylon seal. The can lip is crimped inward over the seal, which also insulates the negative cover from the can. Seen from outside: a flat nickel disc (negative cover) inside a narrow rolled rim, with the jacket film turned over the shoulder. | Handbook fig. 4 labels: Seal, Sealant, Plastic Bushing, Negative Cover, Spot Weld, Current Collector (brass) | Energizer Handbook (VERIFIED); crimp details from patent summaries, e.g. [US6270919](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6270919), [US7122270](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/7122270) (SEARCH-ONLY) | Mixed |
| Vent | Internal: "Nylon seal provides a safety venting mechanism". A thinned section of the seal ruptures under pressure, and gas escapes through small openings in the negative cover. Not visible on a fresh cell. Vented or leaking cells show white crystals at the negative end. | — | Energizer Handbook (VERIFIED); vent openings from patent summaries (SEARCH-ONLY) | Mixed |
| Label jacket | Thin plastic film label, not paper: "Non-conductive plastic film label electrically insulates the battery"; "Jacket: Plastic Label". Typically shrunk hard PVC, sometimes PET, often with a vapour-deposited metal layer under the print. It wraps over both shoulders. | Patents: metallised shrink film ([US5952122](https://patents.google.com/patent/US5952122)); label switched from steel jacket to "thinner metalized plastic film" ([US6248473](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/6248473)); "HPVC" film ([US5292566](https://patentimages.storage.googleapis.com/f3/92/8d/194707e7e58903/US5292566.pdf)) | Energizer Handbook; E95 2014 datasheet (VERIFIED); patents (SEARCH-ONLY) | Mixed |
| Jacket thickness | Not found | — | — | NOT FOUND |
| IEC dimensional figure | IEC 60086-2:2021 §8.1.6, Figure 6 "Dimensional drawing: LR20, R20P, R20S" | The free preview stops before the figure | [IEC 60086-2 ed.14 preview](https://assets.vde-verlag.de/iec-normen/preview-pdf/info_iec60086-2%7Bed14.0%7Den.pdf) | Figure NOT READ (paywalled); manufacturers' drawings used instead |

### 2.2 Materials and how they look

- **Nickel-plated steel ends.** A warm, slightly yellow-grey metal, brighter and warmer than bare steel. The nub top is usually polished or bright. The negative disc is often duller, with faint concentric tooling rings and sometimes a stamped ring or dimple. Metallic, roughness about 0.15–0.3 (ESTIMATE). Ref D1 shows both ends.
- **Steel can.** Hidden under the jacket. It shows only where the jacket is torn or scraped. Old or wet cells rust at the crimp and negative end first (ref D3; those are zinc-carbon cells, but the rust pattern is the same idea).
- **Jacket.** A glossy shrink film with a clear top layer. Many jackets have a metallised silver ground showing through translucent coloured ink, so colour blocks look slightly metallic. The film wraps over both shoulders and wrinkles a little at the turn-over (one patent notes labels "can have wrinkles or distortion", SEARCH-ONLY). Wear: scuffing at the shoulders and along the length, nicks where it rubbed a torch tube. On abused cells the film can split along the seam.
- **Common layouts (no brand).** Based on refs D1, D2 and D4 and the Energizer datasheet thumbnail:
  1. Two colour blocks split across the length: a contrasting band about ⅓ of the length at the positive end, the rest a second colour.
  2. A metallic silver body with one coloured band near the positive end, and the size letter **"D"** in a box or roundel on that band.
  3. An industrial/bulk style: one dark colour with a light stripe, sparse text.
- **Text placement.** A **"+"** near the positive shoulder (and often **"−"** near the negative end). The size letter **D** large on the band. **"LR20"**, **"1.5V"** and **"ALKALINE"** (often in several languages) in a small block running lengthwise (ref D1 shows "LR20 1.5 V" lengthwise). A lengthwise panel of small warning text in capitals, of the kind "CAUTION: … do not recharge … do not dispose of in fire …" (ref D4 shows this panel). Usually also a date or expiry code printed or laser-marked near one end. IEC 60086-1 marking reportedly requires designation, nominal voltage, polarity and manufacturer ID ([batterydesign.net summary](https://www.batterydesign.net/legislation-rules-and-regulations/iec-60086-1/), SEARCH-ONLY). The date/expiry requirement is UNVERIFIED.
- **Leakage.** Potassium hydroxide electrolyte creeps out at the negative-end seal and reacts with air into white, fluffy or crusty crystals (a Commons caption calls it "possibly potassium carbonate"). It sometimes stains the jacket brownish and corrodes torch contacts (refs D8 and D12). The handbook says to "inspect devices battery compartment every few months to be sure batteries are not leaking", and that seal failure becomes more likely with temperature cycling. An abandoned torch's cells would plausibly have leaked.

### 2.3 Palette

| Material | sRGB (8-bit) | Hex | Other parameters | Source | Status |
|---|---|---|---|---|---|
| Nickel plating (ends, nub) | 217, 209, 198 | #D9D1C6 | Metallic. Linear (0.697, 0.641, 0.563); F82/specular tint linear (0.855, 0.805, 0.675). | [physicallybased.info Nickel](https://api.physicallybased.info/materials) | VERIFIED, DERIVED conversion |
| Bare steel (scraped can) | 192, 190, 187 | #C0BEBB | Metallic. Linear (0.53, 0.513, 0.494), Iron entry. | [physicallybased.info Iron](https://api.physicallybased.info/materials) | VERIFIED, DERIVED conversion |
| Bare steel, alternative | 214, 209, 203 | #D6D1CB | Metallic. Linear (0.669, 0.639, 0.598). | [physicallybased.info Stainless Steel](https://api.physicallybased.info/materials) | VERIFIED, DERIVED conversion |
| Metallised label layer (under clear film) | 245, 246, 246 | #F5F6F6 | Metallic aluminium under a dielectric coat (clear coat IOR ≈1.54, the PVC value) | [physicallybased.info Aluminum](https://api.physicallybased.info/materials) + PVC IOR 1.542 (same source) | VERIFIED values; the layering is my interpretation of US5952122 |
| White print / paper-label variant | 230, 235, 242 | #E6EBF2 | Roughness 0.8 for paper. Use a glossy coat for printed film. | [physicallybased.info Office Paper](https://api.physicallybased.info/materials) | VERIFIED, DERIVED conversion |
| Black print areas | 63, 63, 63 | #3F3F3F | Linear (0.05, 0.05, 0.05) | [physicallybased.info Toner (Black)](https://api.physicallybased.info/materials) | VERIFIED value; its use on battery ink is ESTIMATE |
| Coloured band inks (generic, no brand) | e.g. muted yellow ≈ 220, 180, 40; mid blue ≈ 30, 60, 140 | — | Translucent over the metallised ground | Compare refs D1, D2 | ESTIMATE |
| Leaked electrolyte crystals | ≈ 238, 238, 232 | #EEEEE8 | Rough, fluffy, subsurface | Appearance from ref D8 | ESTIMATE |
| Rust at crimp (aged) | ≈ 120, 70, 40 | #784628 | Rough, non-metal | Appearance from ref D3 | ESTIMATE |

### 2.4 References

| # | URL (direct file) | Page | Shows | Licence class | Licence / author | SHA-1 | Notes |
|---|---|---|---|---|---|---|---|
| D1 | https://upload.wikimedia.org/wikipedia/commons/d/d0/D_matchstick-1.jpg | https://commons.wikimedia.org/wiki/File:D_matchstick-1.jpg | **The thing, on a desk.** Two alkaline D cells on squared paper with a matchstick for scale. One stands showing the nub and shoulder; one lies showing the negative disc, crimp rim and jacket turn-over. Silver body with a yellow band, boxed "D", "+", and "LR20 1.5 V … ALKALINE" text lengthwise. | free | CC BY-SA 3.0, Lead holder | eabf673801be36d9d90fffa906b46e041b28e289 | Viewed. Best layout and end-detail reference. Store-brand look, but still do not copy it 1:1. |
| D2 | https://upload.wikimedia.org/wikipedia/commons/a/a1/Panasonic_Powerline_Industrial_D-type_batteries.JPG | https://commons.wikimedia.org/wiki/File:Panasonic_Powerline_Industrial_D-type_batteries.JPG | **The thing, on a table.** Two industrial alkaline D cells (dark blue body, silver band) on a white surface. Negative end and nub visible. | free | CC BY-SA 3.0, Dmitry G | 2a4a2b0a49feea77acdd5db133319e17bbde094b | Viewed. Example of the industrial layout. Branded. |
| D3 | https://upload.wikimedia.org/wikipedia/commons/e/e2/Korrodierte_sowjetische_Orion_Batterien_R20.jpg | https://commons.wikimedia.org/wiki/File:Korrodierte_sowjetische_Orion_Batterien_R20.jpg | **Aged cells, on a surface.** Six old R20 (D-size) cells on a plastic sheet. Negative ends heavily rusted and pitted; jackets faded. | free | CC0, Zeugvondamals | 22a4db81ddcad83d686190c5829880cfbbb1ee60 | Viewed. These are zinc-carbon, not alkaline. Use for weathering only. |
| D4 | https://upload.wikimedia.org/wikipedia/commons/e/e9/DuracellBatteryBHD-2.jpg | https://commons.wikimedia.org/wiki/File:DuracellBatteryBHD-2.jpg | **In a device.** A D cell seated in a black PCB battery holder. Shows the lengthwise caution-text panel and where the size letter and type code sit. | free | CC BY-SA 3.0, Tom Blaha | 6233768feb9cd118b7f28a6973197376da0e6d5f | Viewed. Strong brand trade dress: use only to place text blocks. Small (338×296). |
| D5 | https://upload.wikimedia.org/wikipedia/commons/d/d7/3x_AA_to_D_battery_converter%2C_MY_DAY_vintage_flashlight%2C_Sofirn_SP36_flashlight.jpg | https://commons.wikimedia.org/wiki/File:3x_AA_to_D_battery_converter,_MY_DAY_vintage_flashlight,_Sofirn_SP36_flashlight.jpg | **The torch.** A D-size cell adapter next to a 1970s metal D-cell hand torch with a large reflector, plus a modern torch. | free | CC BY-SA 4.0, Francis Flinch | 824b47e2e64c94932d61cd9bc99bd7c0ff8ec476 | Viewed. The adapter is D-sized; good scale against the torch head. |
| D6 | https://upload.wikimedia.org/wikipedia/commons/2/2d/MagLiteFlashlight.jpg | https://commons.wikimedia.org/wiki/File:MagLiteFlashlight.jpg | **The torch.** A black 2-D-cell aluminium hand torch with a knurled body (cells not visible). | free | CC BY-SA 3.0, KMJ (German Wikipedia) | 5ad8bb8461cdb61b200cecc5f8933b5b60e0d0df | Viewed. Branded; use for scale and form only. |
| D7 | https://upload.wikimedia.org/wikipedia/commons/7/75/%E5%B8%B8%E5%82%99%E7%81%AF_%288196410377%29.jpg | https://commons.wikimedia.org/wiki/File:%E5%B8%B8%E5%82%99%E7%81%AF_(8196410377).jpg | **In a place (building wall).** A standby emergency torch (常備灯) in its wall bracket beside a door in a Japanese building. | free | CC BY 2.0, anna Hanks | 35a840ff2ce0eebf343a7f32825ef2dc596ee18a | Viewed. The cell size inside is **UNVERIFIED** (not confirmed as D). Similar: [File:ナショナル 常備灯 (54753502577).jpg](https://commons.wikimedia.org/wiki/File:%E3%83%8A%E3%82%B7%E3%83%A7%E3%83%8A%E3%83%AB_%E5%B8%B8%E5%82%99%E7%81%AF_(54753502577).jpg), CC BY 4.0, nagi usano, SHA-1 ba82bda3c6a6594cb905c50476b61a0cae76cb45. |
| D8 | https://upload.wikimedia.org/wikipedia/commons/7/7d/LeakedBattery_2701a.jpg | https://commons.wikimedia.org/wiki/File:LeakedBattery_2701a.jpg | Close-up of white crystallised electrolyte growing from the end of a leaked alkaline cell. | free | CC BY-SA 3.0 DE, Túrelio | a8009ba981bdf5118ac0a5888e5b1be564c8d3dc | Viewed. Smaller cell (AA-class), not D. Use for the crystal look. |
| D9 | https://upload.wikimedia.org/wikipedia/commons/e/e5/6_most_common_battery_types-1.jpg | https://commons.wikimedia.org/wiki/File:6_most_common_battery_types-1.jpg | D, C, AA, AAA, AAAA and 9 V cells side by side. | free | CC BY-SA 3.0, Lead holder | 141ad434563134f544032e91e6cae8eeb5565b46 | Not viewed. Scale check. |
| D10 | https://files.batteryjunction.com/frontend/files/energizer/datasheet/ENERGIZER-E95-UpdatedDataSheet.pdf | (same) | Document: Energizer E95 D datasheet (Form E95NA0323) with the dimensioned drawing (34.20/32.30 Ø, 61.50/59.50 H, nub ≤9.50 Ø × ≥1.50, negative ≥18.00 Ø), 139 g. Older version (EBC-1105M, "Jacket: Plastic Label", 144 g): https://media.elv.com/file/120089_mono_d_data.pdf | limited | Energizer, all rights reserved | — | Read. |
| D11 | https://data.energizer.com/pdfs/alkaline_appman.pdf | (same) | Document: Energizer Alkaline Manganese Dioxide Handbook (2018). Fig. 4 cutaway with labelled seal, sealant, plastic bushing, negative cover, spot weld, current collector, label; materials text; care and leakage warnings. | limited | Energizer, all rights reserved | — | Read. The cutaway figure labels the can "Zinc Can" while the text says steel can; follow the text. |
| D12 | https://softsolder.com/wp-content/uploads/2009/10/cimg4145-corroded-alkaline-c-cells.jpg | https://softsolder.com/2009/11/01/d-cell-corrosion-prepare-for-liftof/ | **From a drawer.** Corroded alkaline cells taken from a torch that had been put back in a drawer with flat cells. | limited | Ed Nisley blog, no licence stated | — | Not viewed. The size is disputed (post says D, caption says C, commenter says G). |

Free count: 9 (D1–D9). In-place count: D1 desk/paper, D2 table, D3 surface, D4 in a device, D7 building wall (torch), D12 drawer (limited).

### 2.5 Where found

- **With torches and in emergency kits.** D cells were long called "flashlight batteries" and are "typically used in high current drain applications, such as in large flashlights, radio receivers …" ([Wikipedia](https://en.wikipedia.org/wiki/D_battery), VERIFIED). Ready.gov's basic kit lists "Flashlight" and "Extra batteries", and work kits are meant to let staff "shelter at work for at least 24 hours" ([ready.gov/kit](https://www.ready.gov/kit), VERIFIED). Expect them in grab-and-go kits under reception desks, in facilities and security offices, and in maintenance rooms.
- **Desk and kitchen drawers.** A blog inventory of one drawer found "7 D cell batteries, 2 of which are in an unopened package" among dozens of loose cells ([An Eclectic Mind](https://aneclecticmind.com/2009/12/06/the-junk-drawer/), VERIFIED). A forum user keeps "D alkalines in original duracell pkg inside drawers" ([Candlepower Forums](https://www.candlepowerforums.com/threads/how-do-you-store-your-batteries.225412/post-2871275), SEARCH-ONLY). Expect loose cells, part-used blister packs and shrink-wrapped 2-, 4- and 12-packs in supply cupboards.
- **Inside torches left in drawers, leaking.** Energizer: "Remove batteries from devices that will be stored for extended periods" and "Do inspect devices battery compartment every few months to be sure batteries are not leaking" ([Handbook](https://data.energizer.com/pdfs/alkaline_appman.pdf), VERIFIED). Ref D12 is exactly this case.
- **Wall-mounted standby torches** by doors in Japanese buildings and hotels (ref D7). The cell type is UNVERIFIED.
- **Storage conditions (sourced).** "Do store batteries in a cool, dry place at normal room temperature". "Don't carry loose batteries in a pocket or purse with metal objects like coins, paper clips". "Don't put batteries … in hot places". The handbook also cautions that sealed waterproof torches (e.g. "diving lights") can accumulate hydrogen ([Handbook](https://data.energizer.com/pdfs/alkaline_appman.pdf), VERIFIED). A damp office is therefore a place where cells would leak sooner. Temperature cycling "can cause a failure of the battery seal integrity".
- **Never or unlikely (unsourced, convention).** Inside exit signs and fixed emergency luminaires (these use built-in rechargeable packs), in smoke detectors (9 V or AA), or in wall clocks and remotes (AA/AAA). These are UNVERIFIED but match common practice. Primary cells should never be in a charger (Handbook warning).

---

## 3. What I could not verify or find

- The **IEC 60086-2 LR20 figure** itself (paywalled). All LR20 dimensions come from Energizer and VARTA drawings that state the IEC envelope.
- **D-cell can wall thickness and jacket film thickness.** Not found.
- **Exact still-water bottle profile drawings** (shoulder radius, rib depth and pitch, base geometry). No free stock-bottle drawing found. Shape guidance comes from photos (B1) and qualitative sources.
- The **cap dimensions for 26 mm still-water closures** (RightCAP26, SK 26/15). Only names found. The PCO-1881 cap (ALPLA) is dimensioned.
- **Fill height in mm** and **brimful capacity of a PET 500 ml water bottle**. Only search-summary figures from other 500 ml bottles.
- **Thread start count for PCO-1881.** The drawing says single start; Pelliconi says 2-start. Unresolved.
- **sRGB for HDPE caps, PET tint, inks, leak crystals and rust.** No database or measured-photo values found. Those palette rows are marked ESTIMATE. I did not sample pixel values from photos.
- **IEC 60086-1 marking list** (date and expiry code). Only a secondary summary.
- The **cell size in the Japanese standby torches** (ref D7).
- **Images marked "not viewed"** (B7, B9, B12, D9, D12) are described from captions only.
- **Pexels/Unsplash/Openverse.** Not searched: the Openverse API returned 403, and Pexels/Unsplash licences are not CC0-like. No such images are included.
