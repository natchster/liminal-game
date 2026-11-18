//! What the place is made of, embedded as the game embeds its own assets:
//! the set file (`assets/set.bin`, written by `assets/src/jahamun/set.py`)
//! and its photographed surfaces (`assets/textures/`, written by
//! `assets/src/jahamun/textures.py` from CC0 photographs listed with their
//! SHA-1 in `assets/src/jahamun/sources.json`).

/// The set file: the manifest and the meshes, deflated.
pub const SET: &[u8] = include_bytes!("../assets/set.bin");

/// Every photograph and gloss mask, by its file name.
const TEXTURES: &[(&str, &[u8])] = &[
    ("floor.jpg", include_bytes!("../assets/textures/floor.jpg")),
    ("floor_gloss.jpg", include_bytes!("../assets/textures/floor_gloss.jpg")),
    ("paint.jpg", include_bytes!("../assets/textures/paint.jpg")),
    ("panel.jpg", include_bytes!("../assets/textures/panel.jpg")),
    ("plaza.jpg", include_bytes!("../assets/textures/plaza.jpg")),
    ("road.jpg", include_bytes!("../assets/textures/road.jpg")),
    ("stone.jpg", include_bytes!("../assets/textures/stone.jpg")),
    ("tile.jpg", include_bytes!("../assets/textures/tile.jpg")),
    ("tile_gloss.jpg", include_bytes!("../assets/textures/tile_gloss.jpg")),
    ("vault.jpg", include_bytes!("../assets/textures/vault.jpg")),
];

/// A texture's bytes (JPEG) by its file name.
pub fn texture(name: &str) -> Option<&'static [u8]> {
    TEXTURES.iter().find(|(n, _)| *n == name).map(|(_, b)| *b)
}

#[cfg(test)]
mod tests {
    /// Every file in `assets/textures/` is embedded, and nothing else is named.
    #[test]
    fn every_texture_is_embedded() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures");
        let mut on_disk: Vec<String> =
            std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
        on_disk.sort();
        let mut named: Vec<String> = super::TEXTURES.iter().map(|(n, _)| n.to_string()).collect();
        named.sort();
        assert_eq!(on_disk, named);
    }
}
