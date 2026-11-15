# The README's screenshots

The four pictures of the front page's grid (`README.md`, *The game*):
places, and what walks them seen from afar, never close and never a page
of text (no spoilers). Each is rendered headless (never a capture of the
screen) at 1280×800 with `--fresh`, so the same command makes the same
picture on the same build, and saved as JPEG (quality 82, `sips`).

| Picture | What | Made by (`cargo run --release -- …`) | Frame |
|---|---|---|---|
| `the-well.jpg` | the Well (level 1), its galleries round the void | `--fresh --dump-level DIR --seed 4242 --levels 1 --screenshot x --far-figure --width 1280 --height 800` | `figure_4242_L1_view_none.png` |
| `the-stair-halls.jpg` | the Stair Halls (level 2), the way to its door (built after level 1, as a run comes to it: its caption gone) | `--fresh --dump-level DIR --seed 4242 --levels 1,2,3 --screenshot x --views --width 1280 --height 800` | `rs_4242_L2_exit.png` |
| `the-rooms.jpg` | the rooms (level 6), the Hound 8 m off | `--fresh --screenshot OUT --level 6 --seed 4242 --play --frames 90 --showcase hound:8 --width 1280 --height 800` | the shot |
| `the-pillar-hall.jpg` | a pillar hall (level 5), the Smiler 10 m off | `--fresh --screenshot OUT --level 5 --seed 4242 --play --frames 90 --showcase smiler:10 --width 1280 --height 800` | the shot |

Make them again when the look of these places changes (a new golden
frame accepted for them is the sign), look at them, and keep the
composition: places, and creatures only from afar.
