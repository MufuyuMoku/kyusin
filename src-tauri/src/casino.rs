//! Chip profil di meja casino (SPEC §6.7; D-056, D-058).
//!
//! Game casino itu murni dan tidak tahu saldo. Host (aplikasi) memeriksa
//! saldo sebelum aksi yang memakai chip dan mencatat setiap ronde yang
//! selesai ke saldo dan ringkasan per game. Kontraknya lewat view game:
//! `bersih` (hasil bersih ronde yang sudah selesai dalam sesi), `ronde`,
//! `fase`, `taruhan_meja` (chip yang sedang dipertaruhkan), `biaya` (biaya
//! aksi seperti `double`/`split`/`insure`), dan `tangan[].taruhan` +
//! `asuransi` untuk jumlah yang dipertaruhkan di ronde itu.
//!
//! Tidak ada yang hanya tinggal di memori (D-059): setelah setiap aksi,
//! perubahan saldo dan keadaan pertandingan ditulis dalam satu transaksi.
//! Taruhan dipotong dari saldo saat dipasang, jadi mematikan aplikasi secara
//! paksa di tengah ronde tidak membatalkan taruhan; saat dibuka lagi ronde
//! itu dilanjutkan dengan kartu dan taruhan yang sama.

use kyusin_core::Localized;
use kyusin_core::i18n::core;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use kyusin_store::{CasinoCheckpoint, Suspended};

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

/// Bagian pertandingan berjalan yang sudah tercatat di profil: hasil bersih
/// dan ronde yang sudah selesai, plus chip yang sedang dipertaruhkan (sudah
/// dipotong dari saldo saat dipasang; D-059).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ledger {
    net: i64,
    rounds: i64,
    stake: i64,
    /// Melawan bandar: ronde masuk ringkasan casino dan biaya aksi diperiksa
    /// terhadap saldo. Meja antar-pemain (buy-in, D-063): tidak keduanya.
    house: bool,
}

impl Ledger {
    /// Mulai dari keadaan view sekarang: pertandingan baru mulai dari nol,
    /// pertandingan yang dilanjutkan sudah mencatat semua itu di titik simpan
    /// terakhirnya. `escrowed = false` untuk pertandingan yang ditunda oleh
    /// versi lama (sebelum D-059), yang taruhannya belum dipotong: titik
    /// simpan pertama memotongnya.
    pub(crate) fn from_view(view: &Value, escrowed: bool, house: bool) -> Self {
        Ledger {
            net: view["bersih"].as_i64().unwrap_or(0),
            rounds: settled_rounds(view),
            stake: if escrowed { stake(view) } else { 0 },
            house,
        }
    }

    pub(crate) fn house(&self) -> bool {
        self.house
    }
}

fn stake(view: &Value) -> i64 {
    view["taruhan_meja"].as_i64().unwrap_or(0)
}

/// Biaya chip sebuah perintah (0 bila tidak memakai chip): `biaya` di view
/// untuk aksi tetap (`double`, `raise`, …), atau angka terakhir perintah
/// (`bet 250`, `bet player 100`) dikali `pengali[kata kerja]` (bawaan 1).
pub(crate) fn cost(view: &Value, command: &str) -> i64 {
    let command = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if let Some(c) = view["biaya"][command.as_str()].as_i64() {
        return c;
    }
    let mut parts = command.split(' ');
    let verb = parts.next().unwrap_or("");
    match parts.next_back().and_then(|t| t.parse::<i64>().ok()) {
        Some(n) => n * view["pengali"][verb].as_i64().unwrap_or(1),
        None => 0,
    }
}

/// Buy-in meja antar-pemain (D-063): paling banyak 2.000, paling sedikit 20
/// big blind (400); bot duduk dengan 2.000.
pub(crate) const BUY_IN: i64 = 2000;
pub(crate) const MIN_BUY_IN: i64 = 400;

/// Buy-in untuk saldo `chips`, atau galat bila tidak cukup.
pub(crate) fn buy_in(chips: i64) -> Result<i64, Localized> {
    if chips < MIN_BUY_IN {
        return Err(core().localized(
            "error.chips",
            &[("need", &MIN_BUY_IN.to_string()), ("have", &chips.max(0).to_string())],
        ));
    }
    Ok(chips.min(BUY_IN))
}

/// Menolak aksi yang biayanya melebihi saldo yang bisa dipakai (taruhan
/// yang sedang berjalan sudah dipotong dari saldo).
pub(crate) fn check(state: &AppState, view: &Value, command: &str) -> Result<(), Localized> {
    let need = cost(view, command);
    if need <= 0 {
        return Ok(());
    }
    let store = state.store.as_ref().map_err(store_error)?;
    let available = store.lock().unwrap().chips(now_ms()).map_err(store_error)?;
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

/// Titik simpan setelah setiap aksi (D-059): taruhan baru dipotong dari
/// saldo, ronde yang baru selesai dikembalikan + hasilnya dan masuk
/// ringkasan, dan keadaan pertandingan disimpan — satu transaksi.
/// Mengembalikan saldo baru.
pub(crate) fn checkpoint(
    state: &AppState,
    game: &str,
    view: &Value,
    ledger: &mut Ledger,
    suspended: &Suspended,
) -> Result<i64, String> {
    let rounds = settled_rounds(view);
    let net = view["bersih"].as_i64().unwrap_or(0);
    let now_stake = stake(view);
    // Meja antar-pemain tidak masuk ringkasan "melawan bandar".
    let finished = if ledger.house {
        rounds - ledger.rounds
    } else {
        0
    };
    // Yang dipertaruhkan di ronde yang baru selesai: `dipertaruhkan`, atau
    // untuk Blackjack tangan + insurance.
    let wagered = if finished > 0 && view["dipertaruhkan"].is_i64() {
        view["dipertaruhkan"].as_i64().unwrap_or(0)
    } else if finished > 0 {
        view["tangan"]
            .as_array()
            .map(|hands| {
                hands
                    .iter()
                    .filter_map(|h| h["taruhan"].as_i64())
                    .sum::<i64>()
            })
            .unwrap_or(0)
            + view["asuransi"].as_i64().unwrap_or(0)
    } else {
        0
    };
    let store = state.store.as_ref().map_err(Clone::clone)?;
    let chips = store
        .lock()
        .unwrap()
        .casino_checkpoint(&CasinoCheckpoint {
            game,
            stake: now_stake - ledger.stake,
            net: net - ledger.net,
            rounds: finished,
            wagered,
            suspended,
            now: now_ms(),
        })
        .map_err(|e| e.to_string())?;
    *ledger = Ledger {
        net,
        rounds,
        stake: now_stake,
        house: ledger.house,
    };
    Ok(chips)
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
        let spots = json!({ "biaya": { "raise": 200 }, "pengali": { "bet": 3 } });
        assert_eq!(cost(&spots, "bet 50"), 150);
        assert_eq!(cost(&spots, "raise"), 200);
        let multi = json!({});
        assert_eq!(cost(&multi, "bet  player  100"), 100);
        assert_eq!(cost(&multi, "deal"), 0);
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
