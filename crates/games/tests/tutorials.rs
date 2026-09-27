//! Tes tutorial (SPEC §7.7): setiap game terdaftar wajib punya tutorial,
//! dan setiap tutorial diputar terhadap mesin aturan asli.

use std::path::{Path, PathBuf};

use kyusin_core::help::man_page;
use kyusin_core::tutorial::validate;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn every_registered_game_has_a_valid_tutorial() {
    let registry = kyusin_games::with_fixture().expect("registry");
    assert!(!registry.is_empty());
    let mut failures = Vec::new();
    for c in registry.iter() {
        let id = &c.manifest.id;
        let path = repo_root().join(&c.manifest.tutorial);
        match std::fs::read_to_string(&path) {
            Ok(on_disk) => {
                if normalize_newlines(&on_disk) != normalize_newlines(c.tutorial_src) {
                    failures.push(format!(
                        "{id}: tutorial yang ditanam berbeda dengan {}",
                        c.manifest.tutorial
                    ));
                }
            }
            Err(_) => failures.push(format!("{id}: tutorial {} tidak ada", c.manifest.tutorial)),
        }
        if let Err(e) = validate(c.tutorial_src, c) {
            failures.push(format!("{id}: {e}"));
        }
        match man_page(c) {
            Err(e) => failures.push(format!("{id}: man: {e}")),
            // Kunci terjemahan yang hilang muncul apa adanya (mis. `man.rules`).
            Ok(page) => {
                for text in [&page.id, &page.en] {
                    for prefix in ["man.", "category.", "opponent.", "kind.", "rtp."] {
                        if text.contains(prefix) {
                            failures.push(format!("{id}: man memuat kunci mentah `{prefix}…`"));
                        }
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn no_orphan_tutorials() {
    let registry = kyusin_games::with_fixture().expect("registry");
    let Ok(entries) = std::fs::read_dir(repo_root().join("tutorials")) else {
        return;
    };
    let orphans: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".toml"))
        .filter(|name| registry.get(name.trim_end_matches(".toml")).is_none())
        .collect();
    assert!(
        orphans.is_empty(),
        "tutorial tanpa game terdaftar: {orphans:?}"
    );
}
