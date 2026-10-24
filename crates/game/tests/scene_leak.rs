//! Changing level must not leave objects behind in the scene.
use game::Game;

fn ids(g: &Game) -> Vec<u64> {
    g.scene.children().iter().map(|o| o.id()).collect()
}
/// scene children that are not the current entities' bodies / faces (their number follows the level)
fn non_entity(g: &Game) -> Vec<engine::scene::Obj> {
    let ent: Vec<u64> = g.entities.iter().flat_map(|e| [Some(e.g.id()), e.face.as_ref().map(|f| f.id())]).flatten().collect();
    g.scene.children().into_iter().filter(|o| !ent.contains(&o.id())).collect()
}
fn show(o: &engine::scene::Obj) -> String {
    let kids: Vec<String> = o
        .children()
        .iter()
        .take(3)
        .map(|c| format!("{:?}/light={}/mesh={}/kids={}", c.name(), c.is_light(), c.is_mesh(), c.children().len()))
        .collect();
    format!(
        "name={:?} light={} group={} mesh={} visible={} kids={} first={:?}",
        o.name(),
        o.is_light(),
        o.is_group(),
        o.is_mesh(),
        o.visible(),
        o.children().len(),
        kids
    )
}

#[test]
fn level_changes_do_not_leak_scene_objects() {
    let mut g = Game::new_bare();
    g.boot();
    g.state = "playing".into();
    g.init_level(1);
    let n1 = non_entity(&g).len();
    for n in 2..12 {
        let before = ids(&g);
        g.init_level(n);
        let now = non_entity(&g);
        if now.len() != n1 {
            let added: Vec<String> = now.iter().filter(|o| !before.contains(&o.id())).map(show).collect();
            panic!(
                "level {n} (scp {:?}): {} non-entity scene children vs {} after level 1; new since the last level: {:#?}",
                g.level_scp,
                now.len(),
                n1,
                added
            );
        }
    }
}
