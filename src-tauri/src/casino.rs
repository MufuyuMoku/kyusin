//! Chip profil di meja casino (SPEC §6.7; D-056, D-058).
//!
//! Game casino itu murni dan tidak tahu saldo. Host (aplikasi) memeriksa
//! saldo sebelum aksi yang memakai chip dan mencatat setiap ronde yang
//! selesai ke saldo dan ringkasan per game. Kontraknya lewat view game:
//! `bersih` (hasil bersih ronde yang sudah selesai dalam sesi), `ronde`,
//! `fase`, `taruhan_meja` (chip yang sedang dipertaruhkan), `biaya` (biaya
//! aksi seperti `double`/`split`/`insure`), dan `tangan[].taruhan` +
//! `asuransi` untuk jumlah yang dipertaruhkan di ronde itu.

use kyusin_core::Localized;
use kyusin_core::i18n::core;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::AppState;
use crate::profile::now_ms;

fn store_error(e: impl ToString) -> Localized {
    core().localized("error.store", &[("detail", &e.to_string())])
}

/// Ronde yang sudah selesai menurut view.
fn settled_rounds(view: &Value) -> i64 {
    let started = view["ronde"].as_i64().unwrap_or(0);
    match view["fase"].as_str() {
        Some("taruhan") | Some("selesai") => started,
        _ => (started - 1).max(0),
    }
}

/// Bagian hasil yang sudah dicatat ke profil untuk pertandingan berjalan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ledger {
    net: i64,
    rounds: i64,
}

impl Ledger {
    /// Mulai dari keadaan view sekarang: pertandingan baru mulai dari nol,
    /// pertandingan yang dilanjutkan sudah mencatat ronde sebelum ditunda.
    pub(crate) fn from_view(view: &Value) -> Self {
        Ledger {
            net: view["bersih"].as_i64().unwrap_or(0),
            rounds: settled_rounds(view),
        }
    }
}

/// Biaya chip sebuah perintah (0 bila tidak memakai chip).
pub(crate) fn cost(view: &Value, command: &str) -> i64 {
    let command = command.trim();
    if let Some(n) = command.strip_prefix("bet ") {
        return n.trim().parse().unwrap_or(0);
    }
    view["biaya"][command].as_i64().unwrap_or(0)
}

/// Menolak aksi yang biayanya melebihi chip yang tersedia (saldo dikurangi
/// yang sedang dipertaruhkan di meja).
pub(crate) fn check(state: &AppState, view: &Value, command: &str) -> Result<(), Localized> {
    let need = cost(view, command);
    if need <= 0 {
        return Ok(());
    }
    let store = state.store.as_ref().map_err(store_error)?;
    let chips = store.lock().unwrap().chips(now_ms()).map_err(store_error)?;
    let available = chips - view["taruhan_meja"].as_i64().unwrap_or(0);
    if need > available {
        return Err(core().localized(
            "error.chips",
            &[
                ("need", &need.to_string()),
                ("have", &available.max(0).to_string()),
            ],
        ));
    }
    Ok(())
}

/// Mencatat ronde yang baru selesai ke saldo dan ringkasan casino.
pub(crate) fn settle(state: &AppState, game: &str, view: &Value, ledger: &mut Ledger) {
    let rounds = settled_rounds(view);
    if rounds <= ledger.rounds {
        return;
    }
    let net = view["bersih"].as_i64().unwrap_or(0);
    // Yang dipertaruhkan di ronde yang baru selesai (tangan + insurance).
    let wagered = view["tangan"]
        .as_array()
        .map(|hands| {
            hands
                .iter()
                .filter_map(|h| h["taruhan"].as_i64())
                .sum::<i64>()
        })
        .unwrap_or(0)
        + view["asuransi"].as_i64().unwrap_or(0);
    if let Ok(store) = state.store.as_ref() {
        let recorded = store.lock().unwrap().record_casino(
            game,
            rounds - ledger.rounds,
            wagered,
            net - ledger.net,
            now_ms(),
        );
        if recorded.is_ok() {
            *ledger = Ledger { net, rounds };
        }
    }
}

pub(crate) fn chips(state: &AppState) -> Option<i64> {
    state
        .store
        .as_ref()
        .ok()
        .and_then(|s| s.lock().unwrap().chips(now_ms()).ok())
}

#[derive(Serialize)]
pub(crate) struct AllowanceDto {
    before: i64,
    after: i64,
    granted: bool,
}

/// Tunjangan harian (SPEC §6.7). `date` adalah tanggal lokal pemain
/// (`YYYY-MM-DD`), dikirim UI karena hanya UI yang tahu zona waktu sistem.
#[tauri::command]
pub(crate) fn chips_daily(
    date: String,
    state: State<'_, AppState>,
) -> Result<AllowanceDto, Localized> {
    let valid = date.len() == 10
        && date.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if !valid {
        return Err(core().localized("error.date", &[("date", &date)]));
    }
    let store = state.store.as_ref().map_err(store_error)?;
    let a = store
        .lock()
        .unwrap()
        .daily_allowance(&date, now_ms())
        .map_err(store_error)?;
    Ok(AllowanceDto {
        before: a.before,
        after: a.after,
        granted: a.granted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn costs_come_from_the_bet_or_the_view() {
        let v = json!({ "biaya": { "double": 100, "insure": 50 } });
        assert_eq!(cost(&v, "bet 250"), 250);
        assert_eq!(cost(&v, "double"), 100);
        assert_eq!(cost(&v, "insure"), 50);
        assert_eq!(cost(&v, "hit"), 0);
        assert_eq!(cost(&v, "split"), 0);
    }

    #[test]
    fn settled_rounds_follow_the_phase() {
        assert_eq!(settled_rounds(&json!({ "ronde": 0, "fase": "taruhan" })), 0);
        assert_eq!(settled_rounds(&json!({ "ronde": 3, "fase": "giliran" })), 2);
        assert_eq!(
            settled_rounds(&json!({ "ronde": 3, "fase": "asuransi" })),
            2
        );
        assert_eq!(settled_rounds(&json!({ "ronde": 3, "fase": "taruhan" })), 3);
        assert_eq!(settled_rounds(&json!({ "ronde": 4, "fase": "selesai" })), 4);
    }
}
