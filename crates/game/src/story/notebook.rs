//! M.'s notebook: thirteen pages, one at her camp on each floor of the
//! descent to level 13, in the order she wrote them. The eight floor pages
//! (`FLOOR_PAGES`) lie on the run's ordinary floors in turn (the first four
//! always on the Building's three floors and the Lobby, levels 1–4, where no
//! anomaly comes: `docs/design/MEGASTRUCTURE.md`); an anomaly's
//! page (`ANOMALY_PAGES`) lies on that anomaly's floor, wherever the run put
//! it (the Stairwell's and the Loop's are written on the floor whose door
//! leads in, before she went through); the last page (`LAST_PAGE`) lies at the last camp, on
//! level 13, the first floor below every anomaly (they all come by level 12).
//!
//! Her pages hint, as a survivor would, and never explain (`docs/design/LORE.md`,
//! *How canon reaches the player*); a thread is a line she wrote, never an
//! instruction. E. is Elias (his loose notes are among the torn pages); M.
//! is the M. of *M.'s Rules*. Neither is ever said, nor what they are to
//! each other, nor whether the man in the Loop is E. (Reviewed by
//! `lore-keeper`, 2026-10-26: no zone or level is named, since a floor page
//! lands on a different level each run; the first four, on fixed floors since
//! the Building came above the rooms, may show their place and still name
//! none of it.)

use crate::level::scp::{SCP_HORIZON, scp_for_level};

/// One page of her notebook.
#[derive(Clone, Copy, Debug)]
pub struct Page {
    /// Stable id (saved): `f1`…`f8`, `stairwell`, `store`, `house`, `loop`, `last`
    pub id: &'static str,
    pub body: &'static str,
    /// The line that stays on screen once it is read (the thread)
    pub thread: &'static str,
}

/// The floor pages, in order.
pub static FLOOR_PAGES: [Page; 8] = [
    // the Building's three floors and the Lobby: always levels 1-4
    Page {
        id: "f1",
        body: "E. told me about the wall at the back of the storage room, the one that gives when you lean on it. I laughed at him. Then he stopped answering, and I went and leaned on it.\n\nGalleries round a hole with no bottom, and windows, hundreds of them, and nobody at any of them. A siren went somewhere far off. I got into one of the little rooms with a wheel on its door, turned it until it stopped, and waited a long time. I'm leaving paper as I go, folded twice, an arrow on each, so I can find my way back. Folded twice, so I know it's mine.",
        thread: "Folded twice, so I know it's mine.",
    },
    Page {
        id: "f2",
        body: "I found his writing on a wall today: an E., and a row of tallies under it. He was counting days. There is no night here to count by, so I count sleeps and call them days.\n\nThe fog rose while I was on the stairs. I didn't know a colour could be that cold. Something knocked at my door while it was up. I didn't open it. After a while it stopped.",
        thread: "There is no night here to count by.",
    },
    Page {
        id: "f3",
        body: "I found where he stayed. A long time, by the look of it: a bedroll worn through, a wall of tallies I couldn't finish counting. No E.\n\nUnder the pillow, a page asking whoever found it to remember him. I kept it. His arrows went on down from there, past the machines, to a small door that was warm behind it. So I went down.",
        thread: "A wall of tallies I couldn't finish counting.",
    },
    Page {
        id: "f4",
        body: "Through his door: yellow, everywhere, and the hum. After the concrete it felt like a waiting room. It isn't.\n\nWhen the lights die along a stretch I walk faster, and I don't run. Something down here listens.",
        thread: "Something down here listens.",
    },
    // the rooms' ordinary floors, wherever the run's anomalies leave them
    Page {
        id: "f5",
        body: "I wrote once that if something is waiting round a corner you should close your eyes and count to ten. Not the tall ones. One stood at the end of a hall, its grin going on too far round its face in my beam, and I put the lamp out and didn't close my eyes, and it didn't move. I backed off a long way in the dark before I let myself breathe.\n\nI don't close my eyes any more.",
        thread: "I don't close my eyes any more.",
    },
    Page {
        id: "f6",
        body: "Not all the paper down here is mine.\n\nI followed someone else's sheets for most of a day once. They didn't go down. They went into a room and stopped, and the room was dark, and I didn't stay to see why.",
        thread: "Not all the paper down here is mine.",
    },
    Page {
        id: "f7",
        body: "Today his arrows ran beside mine, fresh, as if he had walked that way after me. That can't be. I put my thumb on one.\n\nThe pencil came off on my thumb.",
        thread: "The pencil came off on my thumb.",
    },
    Page {
        id: "f8",
        body: "The hum is lower down here, like a voice going hoarse. My face in the black of a dead screen; it took me a while to know it. He wrote that he forgot what he looked like. I say my name under my breath now, and then his.\n\nThe paper's nearly gone. Whoever is reading these: you came the way I came. I'm sorry about that.",
        thread: "You came the way I came.",
    },
];

/// The anomalies' pages, by anomaly key (`level::scp`).
pub static ANOMALY_PAGES: [Page; 4] = [
    Page {
        id: "stairwell",
        body: "A steel door on this floor that says STAIRS. Through it, very far down, someone crying. A child.\n\nI came back to it three times before I went in. I'm sure he went through here. If it locks behind me, it locks.",
        thread: "Someone is crying down there. A child.",
    },
    Page {
        id: "store",
        body: "A shop with music in it, and beds made up for nobody. Staff who look at you and have nothing to look with.\n\nHis handwriting on a price tag, under a display bed: a little drawing of a man lying down, and a clock at nine.",
        thread: "Staff who look at you and have nothing to look with.",
    },
    Page {
        id: "house",
        body: "Every room from somewhere else. One of them was ours: the kitchen, the chipped tile by the sink, the burn on the counter where I put the pan down. I didn't stay in it.\n\nThe lights in this house are not on your side.",
        thread: "The lights in this house are not on your side.",
    },
    Page {
        id: "loop",
        body: "His arrow on an ordinary office door with no sign over it. On the other side a latch clicks, and clicks again.\n\nHe went in. I think he's still going round.",
        thread: "I think he's still going round.",
    },
];

/// The last page, at the last camp.
pub static LAST_PAGE: Page = Page {
    id: "last",
    body: "This is as far as the paper goes. Below here there are no arrows of his, only the hum, and I'm too tired to go on finding nothing.\n\nI said his name again. Then mine. Keep the pages.",
    thread: "I said his name again. Then mine.",
};

/// The level her last camp is on: the first below every anomaly.
pub const LAST_LEVEL: i32 = SCP_HORIZON + 1;

/// Her page on level `n` of the run `run_seed` whose anomaly (if any) is
/// `scp`: the anomaly's page on its floor, the last page on `LAST_LEVEL`, and
/// otherwise the next floor page, counting the run's ordinary floors from
/// the first. None past the last camp.
pub fn page_on(run_seed: u32, n: i32, scp: Option<&str>) -> Option<&'static Page> {
    if !(1..=LAST_LEVEL).contains(&n) {
        return None;
    }
    if n == LAST_LEVEL {
        return Some(&LAST_PAGE);
    }
    if let Some(k) = scp {
        return ANOMALY_PAGES.iter().find(|p| p.id == k);
    }
    let k = (1..=n).filter(|&m| m == n || scp_for_level(run_seed, m).is_none()).count();
    FLOOR_PAGES.get(k - 1)
}

/// Her page on level `n` of the run `run_seed` (its anomaly from the run's plan).
pub fn page_for_level(run_seed: u32, n: i32) -> Option<&'static Page> {
    page_on(run_seed, n, scp_for_level(run_seed, n))
}

/// Her pages in the order of the descent: (level, page), levels 1 to `LAST_LEVEL`.
pub fn notebook(run_seed: u32) -> Vec<(i32, &'static Page)> {
    (1..=LAST_LEVEL).filter_map(|n| page_for_level(run_seed, n).map(|p| (n, p))).collect()
}

/// A page by its id.
pub fn page(id: &str) -> Option<&'static Page> {
    FLOOR_PAGES.iter().chain(ANOMALY_PAGES.iter()).chain(std::iter::once(&LAST_PAGE)).find(|p| p.id == id)
}
