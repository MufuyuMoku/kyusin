//! Jembatan Tauri: UI hanya membaca katalog dari registry dan mengirim
//! perintah teks; semua aturan ada di crate `kyusin-*` (SPEC §5).
//!
//! Semua teks untuk pemain dikirim dalam dua bahasa (`{ id, en }`), jadi UI
//! bisa berganti bahasa tanpa bertanya ulang ke sini (D-027).

use std::path::PathBuf;
use std::sync::Mutex;

use kyusin_core::help::{man_page, rtp_line};
use kyusin_core::i18n::core;
use kyusin_core::{ActionSpec, Localized, Manifest, Registry};
use kyusin_store::Store;
use serde::Serialize;
use tauri::{Manager, State};

mod play;
mod tutorial;

/// Batas jumlah perintah konkret per templat yang dijadikan tombol.
const BUTTON_LIMIT: usize = 12;

pub(crate) struct AppState {
    registry: Registry,
    tutorial: Mutex<Option<kyusin_core::TutorialRun>>,
    play: Mutex<Option<play::Running>>,
    store: Result<Mutex<Store>, String>,
    db_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct CategoryDto {
    key: &'static str,
    label: Localized,
    games: Vec<GameDto>,
}

#[derive(Serialize)]
struct GameDto {
    #[serde(flatten)]
    manifest: Manifest,
    rtp_line: Option<Localized>,
    /// Jumlah level bot (0 = belum ada).
    bot_levels: u8,
}

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    /// Folder data aplikasi menurut sistem; ditampilkan di boot Verbose.
    data_dir: Option<String>,
    /// Berkas basis data dan apakah berhasil dibuka.
    database: Option<String>,
    database_ok: bool,
}

#[derive(Serialize)]
pub(crate) struct ActionDto {
    spec: ActionSpec,
    usage: String,
    /// Perintah konkret untuk tombol, bila jumlahnya kecil.
    concrete: Option<Vec<String>>,
}

pub(crate) fn action_dtos(specs: Vec<ActionSpec>) -> Vec<ActionDto> {
    specs
        .into_iter()
        .map(|spec| ActionDto {
            usage: spec.usage(),
            concrete: spec.concrete(BUTTON_LIMIT),
            spec,
        })
        .collect()
}

pub(crate) fn unknown_game(id: &str) -> Localized {
    core().localized("error.unknown_game", &[("id", id)])
}

#[tauri::command]
fn app_info(app: tauri::AppHandle, state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: "KyuSin",
        version: env!("CARGO_PKG_VERSION"),
        data_dir: app
            .path()
            .app_data_dir()
            .ok()
            .map(|p| p.display().to_string()),
        database: state.db_path.as_ref().map(|p| p.display().to_string()),
        database_ok: state.store.is_ok(),
    }
}

#[tauri::command]
fn catalog(state: State<'_, AppState>) -> Vec<CategoryDto> {
    state
        .registry
        .by_category()
        .into_iter()
        .map(|(cat, games)| CategoryDto {
            key: cat.key(),
            label: Localized::build(|lang| cat.label(lang)),
            games: games
                .into_iter()
                .map(|m| GameDto {
                    rtp_line: rtp_line(m),
                    bot_levels: kyusin_bots::levels(&m.id),
                    manifest: m.clone(),
                })
                .collect(),
        })
        .collect()
}

#[tauri::command]
fn man(id: String, state: State<'_, AppState>) -> Result<Localized, Localized> {
    let cartridge = state.registry.get(&id).ok_or_else(|| unknown_game(&id))?;
    man_page(cartridge).map_err(|e| e.message())
}

fn open_store(app: &tauri::App) -> (Result<Mutex<Store>, String>, Option<PathBuf>) {
    let dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => return (Err(e.to_string()), None),
    };
    let path = dir.join("kyusin.sqlite");
    let store = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|_| Store::open(&path).map_err(|e| e.to_string()))
        .map(Mutex::new);
    (store, Some(path))
}

pub fn run() {
    let registry = kyusin_games::builtin().expect("registry cartridge bawaan tidak sah");
    tauri::Builder::default()
        .setup(move |app| {
            let (store, db_path) = open_store(app);
            app.manage(AppState {
                registry,
                tutorial: Mutex::new(None),
                play: Mutex::new(None),
                store,
                db_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            catalog,
            man,
            tutorial::tutorial_start,
            tutorial::tutorial_act,
            tutorial::tutorial_next,
            tutorial::tutorial_stop,
            play::match_start,
            play::match_act,
            play::match_step,
            play::match_leave,
            play::replay_list,
            play::replay_open,
        ])
        .run(tauri::generate_context!())
        .expect("KyuSin gagal dijalankan");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_dto_flattens_manifest_with_spec_keys() {
        let registry = kyusin_games::with_fixture().unwrap();
        let m = &registry.get("fixture").unwrap().manifest;
        let json = serde_json::to_value(GameDto {
            manifest: m.clone(),
            rtp_line: None,
            bot_levels: 0,
        })
        .unwrap();
        for key in [
            "id",
            "nama",
            "kategori",
            "pemain_min",
            "pemain_maks",
            "jenis",
            "lawan",
            "kompetitif",
            "lan",
            "agen",
            "rtp",
            "tutorial",
            "perintah",
            "rtp_line",
            "bot_levels",
        ] {
            assert!(json.get(key).is_some(), "kunci `{key}` hilang");
        }
        assert_eq!(json["perintah"][0]["pola"], "take <n>");
        assert_eq!(json["perintah"][0]["ringkas"]["en"], "Take 1 to 3 sticks");
        assert_eq!(json["nama"]["en"], "Fixture: Sticks");
    }
}
