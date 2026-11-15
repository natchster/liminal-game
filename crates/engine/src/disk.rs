//! How much of a disk is free: the app looks before a recording begins and
//! while it runs (`game::replay::disk`, GitHub issue #9).

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// The share (0..1) of the volume holding `path` that this process may
/// still write (`statvfs`: the blocks free to an unprivileged process over
/// all the volume's), measured at the nearest folder of `path` that exists:
/// a folder not made yet will be made on that volume. None if it cannot be
/// measured.
pub fn free_share(path: &Path) -> Option<f64> {
    let mut at = path;
    while !at.exists() {
        at = at.parent()?;
    }
    let c = CString::new(at.as_os_str().as_bytes()).ok()?;
    // SAFETY: a plain C struct of counts, for the call to fill; all zeroes
    // is a valid value of it
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c` is a NUL-terminated path alive through the call, `s` a
    // statvfs it may write
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return None;
    }
    let total = s.f_blocks as f64 * s.f_frsize as f64;
    (total > 0.0).then(|| s.f_bavail as f64 * s.f_frsize as f64 / total)
}

#[cfg(test)]
mod tests {
    use super::free_share;
    use std::path::Path;

    #[test]
    fn a_disk_is_measured_where_a_folder_is_or_would_be() {
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let share = free_share(here).expect("the crate's own disk can be measured");
        assert!((0.0..=1.0).contains(&share), "{share}");
        // a folder not made yet is measured on the volume it would be made on
        let later = free_share(&here.join("not/made/yet")).expect("its nearest folder");
        assert!((later - share).abs() < 0.01, "{later} vs {share}");
        assert_eq!(free_share(Path::new("")), None, "no folder: nothing to measure");
    }
}
