//! Jembatan Tauri: UI hanya membaca katalog dari registry dan mengirim
//! perintah teks; semua aturan ada di crate `kyusin-*` (SPEC §5).

use std::sync::Mutex;

use kyusin_core::help::{man_page, rtp_line};
use kyusin_core::tutorial::{Feedback, Step, TutorialRun};
use kyusin_core::{ActionSpec, Manifest, Registry, Tutorial};
use serde::Serialize;
use tauri::State;

/// Batas jumlah perintah konkret per templat yang dijadikan tombol.
const BUTTON_LIMIT: usize = 12;

struct AppState {
    registry: Registry,
    tutorial: Mutex<Option<TutorialRun>>,
}

#[derive(Serialize)]
struct CategoryDto {
    key: &'static str,
    label: &'static str,
    games: Vec<GameDto>,
}

#[derive(Serialize)]
struct GameDto {
    #[serde(flatten)]
    manifest: Manifest,
    rtp_line: Option<String>,
}

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "KyuSin",
        version: env!("CARGO_PKG_VERSION"),
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
            label: cat.label(),
            games: games
                .into_iter()
                .map(|m| GameDto {
                    rtp_line: rtp_line(m),
                    manifest: m.clone(),
                })
                .collect(),
        })
        .collect()
}

#[tauri::command]
fn man(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let cartridge = state
        .registry
        .get(&id)
        .ok_or_else(|| format!("man: tidak ada game `{id}`"))?;
    man_page(cartridge).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct ActionDto {
    spec: ActionSpec,
    usage: String,
    /// Perintah konkret untuk tombol, bila jumlahnya kecil.
    concrete: Option<Vec<String>>,
}

#[derive(Serialize)]
struct TutorialDto {
    game: String,
    title: String,
    index: usize,
    total: usize,
    finished: bool,
    step: Option<Step>,
    view_text: String,
    actions: Vec<ActionDto>,
    feedback: Option<Feedback>,
}

fn snapshot(run: &TutorialRun, feedback: Option<Feedback>) -> TutorialDto {
    let session = run.session();
    let learner = run.learner();
    let actions = if run.step().is_some_and(|s| s.action.is_some()) {
        session
            .legal_actions(learner)
            .into_iter()
            .map(|spec| ActionDto {
                usage: spec.usage(),
                concrete: spec.concrete(BUTTON_LIMIT),
                spec,
            })
            .collect()
    } else {
        Vec::new()
    };
    TutorialDto {
        game: run.tutorial().game.clone(),
        title: run.tutorial().title.clone(),
        index: run.index(),
        total: run.len(),
        finished: run.is_finished(),
        step: run.step().cloned(),
        view_text: session.view_text(learner),
        actions,
        feedback,
    }
}

#[tauri::command]
fn tutorial_start(id: String, state: State<'_, AppState>) -> Result<TutorialDto, String> {
    let cartridge = state
        .registry
        .get(&id)
        .ok_or_else(|| format!("tutorial: tidak ada game `{id}`"))?;
    let tutorial = Tutorial::from_toml(cartridge.tutorial_src).map_err(|e| e.to_string())?;
    let run = TutorialRun::start(tutorial, cartridge).map_err(|e| e.to_string())?;
    let dto = snapshot(&run, None);
    *state.tutorial.lock().unwrap() = Some(run);
    Ok(dto)
}

#[tauri::command]
fn tutorial_act(command: String, state: State<'_, AppState>) -> Result<TutorialDto, String> {
    let mut guard = state.tutorial.lock().unwrap();
    let run = guard.as_mut().ok_or("tidak ada tutorial yang berjalan")?;
    let feedback = run.submit(&command).map_err(|e| e.to_string())?;
    Ok(snapshot(run, Some(feedback)))
}

#[tauri::command]
fn tutorial_next(state: State<'_, AppState>) -> Result<TutorialDto, String> {
    let mut guard = state.tutorial.lock().unwrap();
    let run = guard.as_mut().ok_or("tidak ada tutorial yang berjalan")?;
    run.advance().map_err(|e| e.to_string())?;
    Ok(snapshot(run, None))
}

#[tauri::command]
fn tutorial_stop(state: State<'_, AppState>) {
    *state.tutorial.lock().unwrap() = None;
}

pub fn run() {
    let registry = kyusin_games::builtin().expect("registry cartridge bawaan tidak sah");
    tauri::Builder::default()
        .manage(AppState {
            registry,
            tutorial: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            catalog,
            man,
            tutorial_start,
            tutorial_act,
            tutorial_next,
            tutorial_stop,
        ])
        .run(tauri::generate_context!())
        .expect("KyuSin gagal dijalankan");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bentuk JSON yang dibaca `ui/src/lib/backend.ts`.
    #[test]
    fn tutorial_dto_matches_ui_shape() {
        let registry = kyusin_games::with_fixture().unwrap();
        let c = registry.get("fixture").unwrap();
        let mut run = TutorialRun::start(Tutorial::from_toml(c.tutorial_src).unwrap(), c).unwrap();
        run.advance().unwrap();
        let feedback = run.submit("take 1").unwrap();
        let json = serde_json::to_value(snapshot(&run, Some(feedback))).unwrap();

        assert_eq!(json["game"], "fixture");
        assert_eq!(json["index"], 1);
        assert_eq!(json["total"], 4);
        assert_eq!(json["finished"], false);
        assert_eq!(json["step"]["aksi"], "take 3");
        assert_eq!(json["step"]["sorot"][0], "aksi:take 3");
        assert_eq!(json["feedback"]["kind"], "wrong");
        assert!(json["feedback"]["hint"].as_str().unwrap().contains("tiga"));
        assert_eq!(json["actions"][0]["usage"], "take <n>");
        assert_eq!(json["actions"][0]["spec"]["kind"], "template");
        assert_eq!(json["actions"][0]["concrete"][2], "take 3");
        assert!(json["view_text"].as_str().unwrap().contains("7"));
    }

    #[test]
    fn game_dto_flattens_manifest_with_spec_keys() {
        let registry = kyusin_games::with_fixture().unwrap();
        let m = &registry.get("fixture").unwrap().manifest;
        let json = serde_json::to_value(GameDto {
            manifest: m.clone(),
            rtp_line: None,
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
        ] {
            assert!(json.get(key).is_some(), "kunci `{key}` hilang");
        }
        assert_eq!(json["perintah"][0]["pola"], "take <n>");
    }
}
