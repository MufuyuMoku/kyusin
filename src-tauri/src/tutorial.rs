//! Perintah tutorial (SPEC §7.6).

use kyusin_core::i18n::core;
use kyusin_core::tutorial::{Feedback, Step, TutorialRun};
use kyusin_core::{Localized, Tutorial};
use serde::Serialize;
use tauri::State;

use crate::{ActionDto, AppState, action_dtos, unknown_game};

#[derive(Serialize)]
pub(crate) struct TutorialDto {
    game: String,
    title: Localized,
    index: usize,
    total: usize,
    finished: bool,
    step: Option<Step>,
    view_text: Localized,
    /// Data tampilan untuk kontrol visual game (papan Reversi dan sebagainya).
    view_data: serde_json::Value,
    actions: Vec<ActionDto>,
    feedback: Option<Feedback>,
}

fn no_tutorial() -> Localized {
    core().localized("error.no_tutorial", &[])
}

pub(crate) fn snapshot(run: &TutorialRun, feedback: Option<Feedback>) -> TutorialDto {
    let session = run.session();
    let learner = run.learner();
    let actions = if run.step().is_some_and(|s| s.action.is_some()) {
        action_dtos(session.legal_actions(learner))
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
        view_text: Localized::build(|lang| session.view_text(learner, lang)),
        view_data: session.view_data(learner),
        actions,
        feedback,
    }
}

#[tauri::command]
pub(crate) fn tutorial_start(
    id: String,
    state: State<'_, AppState>,
) -> Result<TutorialDto, Localized> {
    let cartridge = state.registry.get(&id).ok_or_else(|| unknown_game(&id))?;
    let tutorial = Tutorial::from_toml(cartridge.tutorial_src).map_err(|e| e.message())?;
    let run = TutorialRun::start(tutorial, cartridge).map_err(|e| e.message())?;
    let dto = snapshot(&run, None);
    *state.tutorial.lock().unwrap() = Some(run);
    Ok(dto)
}

#[tauri::command]
pub(crate) fn tutorial_act(
    command: String,
    state: State<'_, AppState>,
) -> Result<TutorialDto, Localized> {
    let mut guard = state.tutorial.lock().unwrap();
    let run = guard.as_mut().ok_or_else(no_tutorial)?;
    let feedback = run.submit(&command).map_err(|e| e.message())?;
    Ok(snapshot(run, Some(feedback)))
}

#[tauri::command]
pub(crate) fn tutorial_next(state: State<'_, AppState>) -> Result<TutorialDto, Localized> {
    let mut guard = state.tutorial.lock().unwrap();
    let run = guard.as_mut().ok_or_else(no_tutorial)?;
    run.advance().map_err(|e| e.message())?;
    Ok(snapshot(run, None))
}

#[tauri::command]
pub(crate) fn tutorial_stop(state: State<'_, AppState>) {
    *state.tutorial.lock().unwrap() = None;
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
        assert_eq!(
            json["step"]["teks"]["en"],
            "Take 3 sticks so that 4 are left."
        );
        assert_eq!(json["title"]["id"], "Batang: dasar");
        assert_eq!(json["feedback"]["kind"], "wrong");
        assert!(
            json["feedback"]["hint"]["id"]
                .as_str()
                .unwrap()
                .contains("tiga")
        );
        assert!(
            json["feedback"]["hint"]["en"]
                .as_str()
                .unwrap()
                .contains("three")
        );
        assert_eq!(json["actions"][0]["usage"], "take <n>");
        assert_eq!(json["actions"][0]["spec"]["kind"], "template");
        assert_eq!(json["actions"][0]["concrete"][2], "take 3");
        assert!(
            json["view_text"]["id"]
                .as_str()
                .unwrap()
                .contains("Batang tersisa: 7")
        );
        assert!(
            json["view_text"]["en"]
                .as_str()
                .unwrap()
                .contains("Sticks left: 7")
        );
        assert_eq!(json["view_data"]["batang"], 7);
    }
}
