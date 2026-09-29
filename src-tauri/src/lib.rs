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
use tauri::{Manager, State, WebviewWindowBuilder};

mod casino;
mod play;
mod profile;
mod tutorial;

/// Argumen bawaan wry untuk WebView2 (menghapus menu mini dan SmartScreen);
/// dipertahankan bila argumen tambahan diset.
const WRY_DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

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
    /// Perkiraan rating tiap level dari kalibrasi (SPEC §8), bila ada.
    bot_ratings: Vec<Option<i64>>,
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
    /// Profil untuk sapaan boot (M3): nama dan game terakhir.
    profile: Option<profile::ProfileDto>,
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
        profile: profile::profile_dto(&state).ok(),
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
                    bot_ratings: kyusin_bots::ratings(&m.id),
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

/// Folder data. `KYUSIN_DATA_DIR` mengalihkannya, dipakai tes jendela asli
/// supaya tidak menyentuh data pemain (D-040).
fn data_dir(app: &tauri::App) -> Result<PathBuf, String> {
    match std::env::var_os("KYUSIN_DATA_DIR") {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => app.path().app_data_dir().map_err(|e| e.to_string()),
    }
}

fn open_store(app: &tauri::App) -> (Result<Mutex<Store>, String>, Option<PathBuf>) {
    let dir = match data_dir(app) {
        Ok(d) => d,
        Err(e) => return (Err(e), None),
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
            // Jendela dibuat di sini (bukan otomatis dari konfigurasi) supaya
            // folder data WebView2 mengikuti `WEBVIEW2_USER_DATA_FOLDER` bila
            // variabel itu diset. msedgedriver (lewat tauri-driver) mengesetnya
            // dan menunggu berkas DevToolsActivePort di folder itu (D-040).
            let config = app.config().app.windows[0].clone();
            let mut window = WebviewWindowBuilder::from_config(app.handle(), &config)?;
            if let Some(dir) =
                std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").filter(|d| !d.is_empty())
            {
                window = window.data_directory(PathBuf::from(dir));
            }
            // Argumen WebView2 yang dikirim wry lewat API bisa menimpa variabel
            // lingkungan ini (terbukti di runner CI), jadi digabung eksplisit
            // dengan argumen bawaan wry. Dipakai tes jendela asli untuk membuka
            // port DevTools (D-040).
            if let Some(extra) = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS")
                .ok()
                .filter(|a| !a.trim().is_empty())
            {
                window = window.additional_browser_args(&format!("{WRY_DEFAULT_ARGS} {extra}"));
            }
            window.build()?;

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
            play::match_flag,
            play::match_pause,
            play::match_unpause,
            play::match_suspend,
            play::match_resign,
            play::match_resume,
            play::suspended_forfeit,
            play::suspended_list,
            profile::profile_get,
            profile::profile_set_name,
            profile::stats,
            casino::chips_daily,
            play::replay_pgn,
            play::pgn_open,
            play::replay_list,
            play::replay_open,
        ])
        // Menutup jendela saat pertandingan berjalan otomatis menundanya
        // (SPEC §4 Rev. 9).
        .on_window_event(|window, event| {
            if matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) && let Some(state) = window.try_state::<AppState>()
            {
                play::suspend_running(&state);
            }
        })
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
            bot_ratings: vec![],
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
