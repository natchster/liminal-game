# r14b: small things a person dropped or left (Props P2)

Six placers the census graded `cartoon`, remodelled from the real things
(`docs/design/PROPS.md`, "How a prop is remodelled"). The parts are placed as
modelled (`prop_part_as_modelled`, a scale of one), each in its prop's own
frame; each placer keeps its draws call for call. Checked with
`--dump-level` on each prop's census floor and one more where it appears:
the same draw count, and every object of every other placer the same
(placer, position, bounds, turn, meshes), compared with the old code built on
the same `prop_meshes.bin`; each remodelled thing at its old position and
turn.

| Placer | Floors checked |
|---|---|
| `place_safety_glasses` (p19) | (99, 7) open, (7, 13) folded |
| `place_wristwatches` (p11) | (4242, 11), (4242, 7) |
| `place_keychain_keys` (p20) | (4242, 14), (99, 5) |
| `place_reading_glasses_and_books` (p09) | (1999, 10) on the books, (4242, 11) beside them |
| `place_small_ribbons` (p26 `party.rs`) | (1999, 7) the knot, (1999, 6) the single length |
| `place_alarm_clocks` (p27) | (4242, 7), (4242, 11) |

## What each is, and where its size comes from

| Prop | The real one | Size used | Source |
|---|---|---|---|
| Safety glasses | Clear wraparound safety spectacles of the 3M Virtua AP class: one polycarbonate lens with its side shields and a nose notch, a soft nose piece, nylon temples on hinge blocks | 125 mm across, the lens 40 mm deep, temples 132 mm; the lens 2.2 mm | https://safetyglassesusa.com/products/3m-virtua-ap-safety-glasses-with-clear-lens (frame 125 mm, lens height 40 mm, temple 132 mm) |
| Wristwatch | A 38 mm quartz watch of the Timex Weekender class: a plated brass case on its caseback, four lugs 20 mm apart, the crown at 3, a domed acrylic crystal, a printed dial (12- and 24-hour numerals), a two-piece strap (leather, or woven webbing) with a tang buckle and two keepers | case 38 mm, 9 mm high, lugs 20 mm; the strap 20 mm tapering to 18, 75 mm (buckle side) and 115 mm (seven holes, 6 mm apart) | https://timex.com/products/weekender-38mm-leather-strap-watch-tw2r63100 (38 mm, 20 mm strap), https://timex.com/products/weekender-38mm-fabric-strap-watch-tw2v46200 (case height 9 mm) |
| Keys | House keys of the Kwikset KW1 and Schlage SC1 kinds (five pin cuts, the keyway milled along both faces, the shoulder stop), a padlock key, a small wafer cabinet key; cut brass, some nickel-plated; on a 25 mm split ring of flat spring steel (two turns); a metal-rimmed paper key tag, 1 1/4 in, on a jump ring | KW1 54 mm long, blade 8.5 mm (0.335 in), 2 mm thick; SC1 57 mm, bow 27 mm; padlock 44 mm; cabinet 38 mm; ring 25 mm; tag 32 mm | https://makerworld.com/models/1569331 (KW1 blade 0.335 in), https://patents.justia.com/patent/20170370124 (SC1 bow 27 mm), https://clksupplies.com/collections/vendors/products/kwikset-kw1-space-depth-keys (KW1: five spaces, six depths), https://www.clksupplies.com/collections/cylinder-keys/products/kw1-kwikset-key-blank-nickel-plated (brass, nickel-plated), https://tandyleather.ca/products/split-key-rings-nickel-plated (25 mm split rings) |
| Reading glasses | Plastic (acetate) readers, the eyewear size 50-20-135 (lens 50 mm, bridge 20, temples 135), a soft-rectangle lens 30 mm deep, a keyhole bridge with moulded pads; folded, lying on their temples | 50-20-135, lens 30 mm deep, rims 4.5 mm | https://www.zennioptical.com/p/plastic-rectangle-eyeglass-frames/R0020 (lens / bridge / temple marking), https://www.yesglasses.com/shop/reading-glasses/r13552 (a 50 mm lens 35 mm deep on a 135 mm frame) |
| Books | The book stack's hardback and paperback (batch `r02`, its sources in `docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md`), reused | demy octavo 222 x 146, royal 240 x 164, crown 192 x 128; the paperback 216 x 140 | `docs/research/props-remodelling/book_stack_suitcase_coat_stand_and_pile.md` |
| Ribbon | Crimped polypropylene curling ribbon, 3/16 in, cut from round a present: the knot and four tails rolled in ringlets (the curl along the ribbon's length, the ringlet's axis along its width), or one cut length curled at both ends | 4.8 mm wide; ringlets 6-12 mm across | https://www.walmart.com/ip/LaRibbons-3-16-Crimped-Curling-Ribbon-500-Yard-Spool/1493508190 (3/16 in, polypropylene), https://www.mh-chine.com/blog/products/bulk-curling-ribbon |
| Alarm clock | A twin-bell wind-up alarm clock: a painted steel drum on two splayed feet, two spun steel bells on posts with the hammer between them, a carrying handle from bell to bell, a chromed bezel and domed glass, a printed dial (Arabic numerals, a dot minute ring, luminous dots, the alarm hand), two butterfly wind keys and a set knob at the back | drum 104 mm across, 46 deep; bells 57 mm; 156 mm tall over the handle, 125 mm across the bells | https://www.kohls.com/product/prd-8118698/westclox-twin-bell-metal-alarm-clock.jsp (6.5 x 4.5 x 2.5 in), https://clockhistory.com/0/model-2334-1.html (the small Babybelle, 4 1/2 x 3 1/4 x 2 in) |

Makers' names are invented (HALDEN), none real.

## The parts (`r14b.py`)

- Safety glasses: `Real__safetyGlassesOpenLens|Frame` (standing open as put
  down: on the lens's foot and the temples' tips), `…FoldedLens|Frame`
  (folded, lying on their back on the temples). The lens is a gridded slab
  cut to its outline and wrapped round a 74 mm cylinder, curved a little top
  to bottom and tilted in at its foot; the frame is the temples (their
  earpieces bent down 24 deg), hinge blocks and screws, and the rubber nose
  saddle moulded into the notch. Both poses are modelled whole and settled
  onto the floor (`settle`: dropped on the lowest point, tipped till they
  rest on three), centred on their footprint.
- Reading glasses: `Real__readersFrame` (rims, bridge, endpieces, pads and
  the folded temples, lying on their temples) and `Real__readersLenses`
  (plus lenses, a shade thicker at the middle).
- Wristwatch: `Real__watchCase` (the case lathed, the lugs, spring bars,
  stem and knurled crown, and the strap's tang buckle and tongue lying
  flat at its end), `Real__watchCrystal`, `Real__watchStrap` (both pieces
  swept along their length: padded, thinner toward the tip, the long piece's
  tip curled up from its wrist shape, its seven holes cut through, the two
  keepers on the buckle piece), `Real__watchHour|Minute|Second` (each
  pointing to 12, -z, about the pivot).
- Keys: `Real__keysKey0..4` (house KW1 kind, house SC1 kind, padlock key,
  a second house key cut differently, cabinet key: each its outline with
  the bitting cut as V cuts with flat bottoms, the hole, the keyway milled
  along both faces), lying fanned and stacked by 2 mm at their bows, tipped
  so the tip lies on the floor; `Real__keysRing` (the split ring's two turns
  leaning on the bows, the tag's rim, eyelet and jump ring),
  `Real__keysTagPaper`. The game places keys 0..n-1 for the old 3-5 draw
  (the lower keys of the pile first, so none floats).
- Ribbon: `Real__ribbonKnot`, `Real__ribbonLength`: a strip 4.8 mm wide,
  its middle a shade proud (the crimp), drawn both sides; flat where it lies,
  ringlets squashed where trodden on.
- Alarm clock: `Real__alarmCase` (the drum and domed back with its stamped
  ring), `Real__alarmChrome` (bezel, bells, posts and nuts, hammer, handle,
  legs and pads, wind keys, set knob, hub), `Real__alarmGlass`,
  `Real__alarmHour|Minute|Set` (pointing to 12, +y).

## The game side

- **Safety glasses** (`place_safety_glasses`): the old draws (the 8% roll,
  the tile, `scatter`'s place and turn) are made as before; open or folded
  (3 in 5 open), a grey lens now and then and the nylon's colour come from
  where it lies (`r14b_kit::seed_of`, no draw). The lens is `Patch::Glass`
  as the windows' panes and the wine glass: the room in it by Fresnel, its
  dirt from a map painted per pair (haze, dust blotches, a thumb's whorls,
  fine scratches wiped one way, a few long ones from a fall); a grey lens a
  body of its own (`milk`).
- **Wristwatch** (`place_wristwatches`): the strap, case and dial draws pick
  black, dark brown, tan or brown leather or grey or navy webbing; gold-tone,
  chrome, brushed or black plating; aged cream, black, grey-white or navy
  dials. The hands' three draws set the time: the minute from its draw, the
  hour from its draw's hour carried on by the minutes (no more independent
  hands); the ticking draw keeps its animator (the second hand steps once a
  second) and the position for the tick sound. The dial is printed (minute
  track, hours in Helvetica Neue, the 24-hour ring in red, HALDEN, QUARTZ,
  WATER RESISTANT), dulled at its rim, freckled where damp got in, sometimes
  a tide mark; the strap is painted from above along its length (grain or
  webbing ribs, creased and dark at the hole it was worn on, the sweat band
  behind the case, scuffs); the plating rubbed through to brass in specks;
  the crystal hazed and scratched (`Patch::Glass`).
- **Keys** (`place_keychain_keys`): the old `n_keys` and per-key brass draws
  pick how many keys and which are bare brass or nickel; both metals dark in
  diffuse with their colour in the highlight, tarnished in blotches, rubbed
  bright in patches, green or rust in specks. The tag's paper is painted:
  card gone yellow, a printed ring, a word in felt pen (SPARE, BOILER,
  STORE 2, B-14, BACK DOOR …), rubbed and stained.
- **Reading glasses and books** (`place_reading_glasses_and_books`): the
  cover draw picks the hardback (A, B or C of `r02`) and its cloth; the
  frame draw picks black, dark tortoiseshell, burgundy, honey tortoiseshell
  or navy acetate (the tortoiseshell's mottle painted); the glasses' turn
  draw turns them; the bookmark draw (40%) is now `r02`'s slip of paper in
  the hardback, its colour draw a card, a ticket, manila or a green slip. A
  paperback lies askew on the hardback. The glasses lie folded on the
  paperback by its spine, or on the floor beside the books (from where it
  lies). The books are painted as `r02` paints its stack (its painters are
  private to the core props, so done again here for two books: cloth with
  its weave, faded toward the fore-edge, a blind rule, a gilt title, bumped
  corners, a cup's ring; the paperback's print, creases and worn edges; the
  spines' titles; the page edges yellowed and foxed). Titles and authors are
  made up.
- **Ribbon** (`place_small_ribbons`): the old colour draw picks the colour,
  faded (rose, blue, gold, green, purple, pink); shiny polypropylene
  (phong, both sides), bleached in patches and grimy where trodden. The knot
  (7 in 10) or the single length from where it lies.
- **Alarm clock** (`place_alarm_clocks`): the old draws (the roll, the tile,
  the place and turn) as before; its paint (cream, oxblood, black, green,
  grey), the time it stopped at and the alarm's setting from where it
  stands. The dial is printed: cream-painted tin yellowed toward the rim,
  the double ring and minute dots, the hours in Futura, the luminous dots
  gone brown, HALDEN, UNADJUSTED and ONE JEWEL, the paint crazed, a tide
  line, fly specks. The paint is mottled, chipped to the steel and rusted in
  the chips; the plating pitted; the glass dusty (`Patch::Glass`). The ring
  sound's position is kept.

Every painter draws from its own stream and mixes in where its thing lies,
so no two floors paint the same pair of glasses, watch, keys, books, ribbon
or clock, and no level draw is added. No emissive anywhere.
