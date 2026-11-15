"""The selection (issue #13, A1): every kind's grades joined with how often
it is placed; the 30% to remove by the rule written before the result."""
import collections, glob, json, os, sys

C = sys.argv[1]
kinds = json.load(open(f"{C}/sheets/kinds.json"))
draws = json.load(open(f"{C}/draws.json"))
placed, nfloors = draws["placed"], draws["floors"]

# cell -> kind
cell_kind = {}
for t in glob.glob(f"{C}/sheets/sheet*.txt"):
    sh = os.path.basename(t)[:-4]
    for line in open(t):
        i, k = line.rstrip("\n").split("\t")
        cell_kind[(sh, i)] = k
grade = {}
for r in range(1, 6):
    a = json.load(open(f"{C}/rev{r}/answers.json"))
    for sh, cells in a.items():
        for i, v in cells.items():
            grade[cell_kind[(sh, i)]] = (v, r, sh, i)
info = {row[0]: row for row in kinds["rows"]}
missing = [k for k in info if k not in grade]
assert not missing, missing

# the census camera failed (framed the wrong thing, a pillar or a creature in
# the way, nothing there): no evidence about the thing itself
CAMERA = {"place_cracked_wall_holes", "core_wet_floor_sign", "core_weather_extras", "place_watercolors",
          "place_wedding_photos", "core_wall_clock", "place_ceiling_stains", "place_emergency_phones",
          "place_feathers", "place_broken_wall_clocks", "place_ceiling_pipes", "place_half_wall_partitions",
          "place_floor_stains", "place_sprinkler_heads"}
# what other systems stand on (never removed): the game's own props and
# placers, and the themed ones a system reads
SYSTEM = {"place_secrets", "place_story", "place_store", "place_house", "place_gameplay_r1_trigger",
          "place_metal_lockers"}
# the duplicates the reviewers named: in each group the weaker loses a point
DUPES = [
    ["place_vent_grates", "place_wall_vents"], ["core_traffic_cone", "place_safety_cones", "place_traffic_cones"],
    ["place_diplomas", "place_formal_diplomas"], ["place_small_remote_controls", "place_remotes"],
    ["place_small_hairbrushes", "place_hair_brushes", "place_hairbrushes"], ["place_keychain_keys", "place_keyring_with_keys", "place_small_keys", "place_car_keys"],
    ["place_wall_coat_hooks", "place_coat_hook_rows", "place_coat_hooks", "place_coat_hooks_pair", "place_coat_racks", "core_coat_rack"],
    ["place_vintage_typewriters", "place_small_typewriters"], ["place_marbles", "place_marble_scatters"],
    ["place_moon_globes", "place_small_globes"], ["place_newspapers", "place_floor_newspapers", "place_framed_newspapers", "place_newspaper_stacks"],
    ["place_family_portraits", "place_miniature_portrait_clusters"], ["core_fire_extinguisher", "place_extinguisher_cabinets", "place_fire_extinguishers"],
    ["place_nail_polish", "place_nail_polishes", "place_small_nail_polishes"], ["place_yoyos", "place_small_yo_yos"],
    ["place_wall_clock", "place_kitchen_clocks", "place_analog_clocks", "core_wall_clock"], ["place_wall_calendars", "place_kitchen_calendars"],
    ["place_floor_puddles", "place_water_puddles"], ["place_trash_cans", "place_overturned_wastebaskets", "core_trash_can"],
    ["core_wall_mirrors", "place_wall_mirrors", "place_ornate_wall_mirrors", "place_smudge_mirrors", "place_cracked_mirrors"],
    ["place_dropped_shoes", "place_single_boots", "place_shoe_pairs", "place_single_child_shoes", "place_children_shoes"],
    ["place_scrap_papers", "place_crumpled_receipts", "place_scattered_pages", "place_receipts"],
    ["core_crt_tv", "place_old_crtt_vs", "core_tv_on_floor"], ["place_cassettes", "place_cassette_tapes"],
    ["place_opened_suitcases", "core_suitcase", "place_suitcases", "place_packed_suitcases"],
    ["place_ashtrays", "place_smoking_ashtrays"], ["place_overturned_buckets", "place_metal_buckets"],
    ["place_ceramic_mugs", "place_teacups", "place_coffee_mugs"], ["place_cross_stitches", "place_embroidery_hoops", "place_cross_stitch_samplers"],
    ["place_thermostats", "place_wall_thermostats"], ["place_kids_drawings", "place_child_drawings"],
    ["place_dead_potted_plants", "place_dead_plants", "place_dead_flower_vases"],
    ["place_barometers", "place_wall_barometers"], ["place_vinyl_records", "place_vinyl_record_discs", "place_record_stacks"],
    ["place_rotary_phones", "place_small_rotary_phones"], ["place_cell_phones", "place_dropped_cell_phones"],
    ["place_lipsticks", "place_small_lipsticks"], ["place_perfume_bottles", "place_small_perfumes"],
    ["place_hand_mirrors", "place_small_hand_mirrors"], ["place_sewing_machines", "place_small_sewing_machines"],
    ["place_teddy_bears", "place_plush_bears", "place_worn_teddy_bears"], ["place_wallets", "place_opened_wallets"],
    ["place_jewelry_boxes", "place_small_jewelry_boxes"], ["place_wristwatches", "place_forgotten_wristwatches"],
    ["place_dice_clusters", "place_small_dice_pairs"], ["place_crayon_graffiti", "place_small_crayons", "place_crayons"],
]
B = {"yes": 2, "odd": 1, "no": 0}
I = {"high": 2, "some": 1, "none": 0}
M = {"real": 2, "plain": 1, "cartoon": 0, "unseen": 0}

order = kinds["order"]
import re
_t = open("/Users/nohwan/Documents/Personal_Projects/OpenCarole/native/crates/game/src/world/tables.rs").read()
theme_of = dict(re.findall(r'\("(place_[a-z0-9_]+)",\s*"([a-z]+)"\)', _t))
tof = lambda k: theme_of.get(k) or (info[k][1] if k in info else "core")
themed = sorted(set(order))
allk = sorted(set(info) | set(themed))


def floors_of(k):
    # how many of the 272 floors had it: by its draws if a themed placer,
    # else by the census's shots
    seen = info[k][2] if k in info else 0
    return max(seen, placed.get(k, 0))


score = {}
for k in allk:
    if k in grade:
        (b, i, m), r, sh, cell = grade[k]
        mm = 1 if (m == "unseen" and k in CAMERA) else M[m]
        score[k] = [B[b] + I[i] + mm, B[b], I[i], mm, f"{b}/{i}/{m}", f"{sh}-{cell}"]
    else:
        score[k] = [None, None, None, None, "never seen", ""]
dupe_lost = set()
for grp in DUPES:
    present = [k for k in grp if k in score and score[k][0] is not None]
    if len(present) < 2:
        continue
    keep = max(present, key=lambda k: (k.startswith("core_"), score[k][0], floors_of(k)))
    for k in present:
        if k != keep:
            score[k][0] -= 1
            dupe_lost.add(k)

protected = lambda k: k.startswith("core_") or k in SYSTEM
cands = [k for k in allk if not protected(k)]
target = round(0.30 * len(allk))
dead = [k for k in themed if floors_of(k) == 0 and not protected(k)]
rest = [k for k in cands if k not in dead and score[k][0] is not None]
rest.sort(key=lambda k: (score[k][0], score[k][1], floors_of(k), k))
remove = dead + rest[: max(0, target - len(dead))]
cut = score[remove[-1]][0] if len(remove) > len(dead) else None
out = {"total": len(allk), "target": target, "dead": dead, "remove": remove, "score": score,
       "floors": {k: floors_of(k) for k in allk}, "dupe_lost": sorted(dupe_lost), "nfloors": nfloors,
       "theme": {k: tof(k) for k in allk}}
json.dump(out, open(f"{C}/selection.json", "w"), indent=1)
print("kinds", len(allk), "candidates", len(cands), "target", target, "dead", len(dead), "last score", cut)
th = collections.Counter(tof(k) for k in remove)
tot = collections.Counter(tof(k) for k in allk)
for t in sorted(tot):
    print(f"  {t:12s} remove {th[t]:3d} of {tot[t]:3d}")
# how the boundary sits
edge = [k for k in rest if score[k][0] == cut]
print("at the boundary score", cut, ":", len(edge), "kinds,", sum(1 for k in edge if k in remove), "removed")
