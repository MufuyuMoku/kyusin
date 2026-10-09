//! Craps (SPEC §6.4; M6a, D-069).
//!
//! Satu pertandingan = satu penembak: lemparan berlanjut sampai seven-out
//! (atau pemain berhenti saat tidak ada taruhan kontrak). Dadu tiap
//! lemparan dari seed sesi dan nomor lemparan, jadi provably fair berlaku
//! per penembak seperti per shoe (D-056); `lemparan` di konfigurasi
//! memberi urutan tetap (tes dan tutorial).
//!
//! Taruhan (`bet <tempat> <jumlah>`):
//! - `pass`/`dont-pass` (hanya di come-out; Don't Pass seri pada 12);
//! - `come`/`dont-come` (hanya saat ada titik; pindah ke `come-N`/
//!   `dont-come-N` setelah lemparan pertamanya);
//! - `odds-pass`, `odds-dont-pass`, `odds-come-N`, `odds-dont-come-N`:
//!   paling banyak 3x (4/10), 4x (5/9), 5x (6/8) taruhan dasarnya, lay
//!   paling banyak 6x; dibayar peluang sebenarnya (2:1, 3:2, 6:5; lay 1:2,
//!   2:3, 5:6);
//! - `place-4` … `place-10` (9:5, 7:5, 7:6), `field` (2 → 2:1, 12 → 3:1,
//!   3/4/9/10/11 → 1:1), `hard-4/6/8/10` (7:1 dan 9:1).
//!
//! Place, Hardways, dan Odds Come "off" di lemparan come-out. Taruhan Place
//! dan Hardways yang menang tetap terpasang. `take <tempat>` menarik
//! taruhan selain kontrak (Pass dengan titik, `come-N`). Kelipatan: 10;
//! Place 6/8 dan lay 5/9/6/8 kelipatan 30 supaya bayaran utuh.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::dadu;
use crate::meja::{self, Limits, MejaView, STANDARD, Umum, WagerRtp};

pub const ID: &str = "craps";
pub const POINTS: [u8; 6] = [4, 5, 6, 8, 9, 10];

/// RTP tiap taruhan per taruhan yang diselesaikan (tepat; D-069). Angka
/// manifest = Pass Line. Odds tidak dicantumkan: 100% dan hanya bisa
/// dipasang di belakang Pass/Come/Don't.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "pass",
        percent: 98.5859,
        manifest: true,
    },
    WagerRtp {
        wager: "dont-pass",
        percent: 98.6364,
        manifest: false,
    },
    WagerRtp {
        wager: "place-6",
        percent: 98.4848,
        manifest: false,
    },
    WagerRtp {
        wager: "place-5",
        percent: 96.0,
        manifest: false,
    },
    WagerRtp {
        wager: "place-4",
        percent: 93.3333,
        manifest: false,
    },
    WagerRtp {
        wager: "field",
        percent: 97.2222,
        manifest: false,
    },
    WagerRtp {
        wager: "hard-6",
        percent: 90.9091,
        manifest: false,
    },
    WagerRtp {
        wager: "hard-4",
        percent: 88.8889,
        manifest: false,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("craps/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/craps.toml"),
        create_session::<Craps>,
    )
}

/// Kelipatan Odds paling banyak (3-4-5x) untuk sebuah titik.
pub fn odds_multiple(point: u8) -> i64 {
    match point {
        4 | 10 => 3,
        5 | 9 => 4,
        _ => 5,
    }
}

fn odds_win(point: u8, n: i64) -> i64 {
    match point {
        4 | 10 => 2 * n,
        5 | 9 => n * 3 / 2,
        _ => n * 6 / 5,
    }
}

fn lay_win(point: u8, n: i64) -> i64 {
    match point {
        4 | 10 => n / 2,
        5 | 9 => n * 2 / 3,
        _ => n * 5 / 6,
    }
}

fn place_win(point: u8, n: i64) -> i64 {
    match point {
        4 | 10 => n * 9 / 5,
        5 | 9 => n * 7 / 5,
        _ => n * 7 / 6,
    }
}

fn number(spot: &str) -> Option<u8> {
    spot.rsplit('-').next()?.parse().ok()
}

/// Semua nama tempat taruhan.
pub fn spots() -> &'static [&'static str] {
    static S: OnceLock<&'static [&'static str]> = OnceLock::new();
    S.get_or_init(|| {
        let mut v: Vec<String> = [
            "pass",
            "dont-pass",
            "come",
            "dont-come",
            "odds-pass",
            "odds-dont-pass",
            "field",
        ]
        .map(String::from)
        .to_vec();
        for p in POINTS {
            for k in ["come", "dont-come", "odds-come", "odds-dont-come", "place"] {
                v.push(format!("{k}-{p}"));
            }
        }
        for h in [4, 6, 8, 10] {
            v.push(format!("hard-{h}"));
        }
        crate::papan_taruhan::leak(v)
    })
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Urutan lemparan tetap (tes dan tutorial).
    #[serde(default)]
    pub lemparan: Option<Vec<[u8; 2]>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Take(&'static str),
    Roll,
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Craps {
    seed: Seed,
    fixed: Vec<[u8; 2]>,
    rolls: u32,
    point: Option<u8>,
    bets: BTreeMap<String, i64>,
    dice: Option<[u8; 2]>,
    payouts: BTreeMap<String, i64>,
    resolved: i64,
    net: i64,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Titik yang sedang berlaku (`null` = lemparan come-out).
    pub titik: Option<u8>,
    /// Dadu lemparan terakhir.
    pub dadu: Option<[u8; 2]>,
    pub lemparan: u32,
    /// Semua taruhan yang sedang terpasang.
    pub taruhan: BTreeMap<String, i64>,
    /// Hasil bersih tiap taruhan pada lemparan terakhir (yang menang,
    /// kalah, atau seri).
    pub bayar: BTreeMap<String, i64>,
    /// Batas taruhan tiap tempat yang sedang terbuka: [min, maks, langkah].
    pub batas: BTreeMap<String, [i64; 3]>,
    /// Taruhan yang boleh ditarik.
    pub bisa_ditarik: Vec<String>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl Craps {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn get(&self, spot: &str) -> i64 {
        self.bets.get(spot).copied().unwrap_or(0)
    }

    /// Batas taruhan yang masih boleh ditambahkan di `spot` sekarang.
    fn limits(&self, spot: &str) -> Option<Limits> {
        let now = self.get(spot);
        let cap = |max: i64, step: i64| {
            let room = max - now;
            let max = room - room % step;
            (max >= step).then_some(Limits {
                min: step,
                max,
                step,
            })
        };
        let standard = || cap(STANDARD.max, STANDARD.step);
        match spot {
            "pass" | "dont-pass" if self.point.is_none() => standard(),
            "come" | "dont-come" if self.point.is_some() => standard(),
            "field" => standard(),
            "odds-pass" => {
                let p = self.point?;
                (self.get("pass") > 0).then_some(())?;
                cap(odds_multiple(p) * self.get("pass"), 10)
            }
            "odds-dont-pass" => {
                let p = self.point?;
                (self.get("dont-pass") > 0).then_some(())?;
                cap(
                    6 * self.get("dont-pass"),
                    if p == 4 || p == 10 { 10 } else { 30 },
                )
            }
            _ => {
                let n = number(spot)?;
                if spot.starts_with("odds-come-") {
                    let base = self.get(&format!("come-{n}"));
                    (base > 0).then_some(())?;
                    cap(odds_multiple(n) * base, 10)
                } else if spot.starts_with("odds-dont-come-") {
                    let base = self.get(&format!("dont-come-{n}"));
                    (base > 0).then_some(())?;
                    cap(6 * base, if n == 4 || n == 10 { 10 } else { 30 })
                } else if spot.starts_with("place-") {
                    cap(STANDARD.max, if n == 6 || n == 8 { 30 } else { 10 })
                } else if spot.starts_with("hard-") {
                    standard()
                } else {
                    None
                }
            }
        }
    }

    /// Taruhan yang boleh ditarik (bukan kontrak).
    fn removable(&self, spot: &str) -> bool {
        self.get(spot) > 0
            && !(spot == "pass" && self.point.is_some())
            && !(spot.starts_with("come-"))
    }

    fn contract(&self) -> bool {
        self.bets.keys().any(|s| !self.removable(s))
    }

    fn next_dice(&mut self) -> [u8; 2] {
        let i = self.rolls as usize;
        match self.fixed.get(i) {
            Some(d) => *d,
            None => {
                let mut rng =
                    GameRng::from_seed(derive(&self.seed, &format!("lempar:{}", self.rolls)));
                [dadu::roll(&mut rng), dadu::roll(&mut rng)]
            }
        }
    }

    fn roll(&mut self) {
        let d = self.next_dice();
        self.rolls += 1;
        let s = d[0] + d[1];
        let hard = d[0] == d[1];
        let comeout = self.point.is_none();
        let mut pay: BTreeMap<String, i64> = BTreeMap::new();
        let mut resolved = 0i64;
        let mut remove: Vec<String> = Vec::new();
        let mut moves: Vec<(String, String)> = Vec::new();
        let mut settle =
            |spot: &str, stake: i64, net: i64, stays: bool, pay: &mut BTreeMap<String, i64>| {
                pay.insert(spot.to_string(), net);
                resolved += stake;
                if !stays {
                    remove.push(spot.to_string());
                }
            };
        for (spot, &n) in &self.bets {
            let sp = spot.as_str();
            match sp {
                "field" => {
                    let net = match s {
                        2 => 2 * n,
                        12 => 3 * n,
                        3 | 4 | 9 | 10 | 11 => n,
                        _ => -n,
                    };
                    settle(sp, n, net, false, &mut pay);
                }
                "pass" => match (self.point, s) {
                    (None, 7 | 11) => settle(sp, n, n, false, &mut pay),
                    (None, 2 | 3 | 12) => settle(sp, n, -n, false, &mut pay),
                    (Some(p), _) if s == p => settle(sp, n, n, false, &mut pay),
                    (Some(_), 7) => settle(sp, n, -n, false, &mut pay),
                    _ => {}
                },
                "dont-pass" => match (self.point, s) {
                    (None, 2 | 3) => settle(sp, n, n, false, &mut pay),
                    (None, 12) => settle(sp, n, 0, false, &mut pay),
                    (None, 7 | 11) => settle(sp, n, -n, false, &mut pay),
                    (Some(_), 7) => settle(sp, n, n, false, &mut pay),
                    (Some(p), _) if s == p => settle(sp, n, -n, false, &mut pay),
                    _ => {}
                },
                "odds-pass" => match self.point {
                    Some(p) if s == p => settle(sp, n, odds_win(p, n), false, &mut pay),
                    Some(_) if s == 7 => settle(sp, n, -n, false, &mut pay),
                    _ => {}
                },
                "odds-dont-pass" => match self.point {
                    Some(p) if s == 7 => settle(sp, n, lay_win(p, n), false, &mut pay),
                    Some(p) if s == p => settle(sp, n, -n, false, &mut pay),
                    _ => {}
                },
                "come" => match s {
                    7 | 11 => settle(sp, n, n, false, &mut pay),
                    2 | 3 | 12 => settle(sp, n, -n, false, &mut pay),
                    _ => moves.push((sp.into(), format!("come-{s}"))),
                },
                "dont-come" => match s {
                    2 | 3 => settle(sp, n, n, false, &mut pay),
                    12 => settle(sp, n, 0, false, &mut pay),
                    7 | 11 => settle(sp, n, -n, false, &mut pay),
                    _ => moves.push((sp.into(), format!("dont-come-{s}"))),
                },
                _ => {
                    let Some(k) = number(sp) else { continue };
                    if sp.starts_with("come-") {
                        if s == k {
                            settle(sp, n, n, false, &mut pay);
                        } else if s == 7 {
                            settle(sp, n, -n, false, &mut pay);
                        }
                    } else if sp.starts_with("dont-come-") {
                        if s == 7 {
                            settle(sp, n, n, false, &mut pay);
                        } else if s == k {
                            settle(sp, n, -n, false, &mut pay);
                        }
                    } else if sp.starts_with("odds-come-") {
                        // Off di come-out: dikembalikan bila come-N selesai.
                        if s == k {
                            let net = if comeout { 0 } else { odds_win(k, n) };
                            settle(sp, n, net, false, &mut pay);
                        } else if s == 7 {
                            settle(sp, n, if comeout { 0 } else { -n }, false, &mut pay);
                        }
                    } else if sp.starts_with("odds-dont-come-") {
                        if s == 7 {
                            settle(sp, n, lay_win(k, n), false, &mut pay);
                        } else if s == k {
                            settle(sp, n, -n, false, &mut pay);
                        }
                    } else if sp.starts_with("place-") && !comeout {
                        if s == k {
                            settle(sp, n, place_win(k, n), true, &mut pay);
                        } else if s == 7 {
                            settle(sp, n, -n, false, &mut pay);
                        }
                    } else if sp.starts_with("hard-") && !comeout {
                        if s == k && hard {
                            let pays = if k == 6 || k == 8 { 9 } else { 7 };
                            settle(sp, n, pays * n, true, &mut pay);
                        } else if s == 7 || s == k {
                            settle(sp, n, -n, false, &mut pay);
                        }
                    }
                }
            }
        }
        for spot in remove {
            self.bets.remove(&spot);
        }
        for (from, to) in moves {
            if let Some(n) = self.bets.remove(&from) {
                *self.bets.entry(to).or_insert(0) += n;
            }
        }
        self.net += pay.values().sum::<i64>();
        self.resolved = resolved;
        self.payouts = pay;
        self.dice = Some(d);
        match self.point {
            None if POINTS.contains(&s) => self.point = Some(s),
            Some(p) if s == p => self.point = None,
            Some(_) if s == 7 => {
                self.point = None;
                // Seven-out: semua taruhan sudah selesai.
                self.bets.clear();
                self.ended = Some("seven_out");
            }
            _ => {}
        }
    }

    fn open_limits(&self) -> BTreeMap<String, Limits> {
        spots()
            .iter()
            .filter_map(|s| self.limits(s).map(|l| (s.to_string(), l)))
            .collect()
    }
}

impl TurnGame for Craps {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let fixed = config.lemparan.unwrap_or_default();
        if fixed.iter().flatten().any(|x| !(1..=6).contains(x)) {
            return Err(GameError::Config("lemparan".into()));
        }
        Ok(Craps {
            seed,
            fixed,
            rolls: 0,
            point: None,
            bets: BTreeMap::new(),
            dice: None,
            payouts: BTreeMap::new(),
            resolved: 0,
            net: 0,
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.over() { Vec::new() } else { vec![0] }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 || self.over() {
            return Vec::new();
        }
        let mut out: Vec<ActionSpec> = self
            .open_limits()
            .iter()
            .filter_map(|(s, l)| {
                // `Limits::spot` menghitung sisa ruang dari isi tempat;
                // batas di sini sudah sisa ruang, jadi isi dianggap 0.
                l.spot(spots().iter().find(|x| **x == s.as_str())?, 0)
            })
            .collect();
        let takeable: Vec<String> = self
            .bets
            .keys()
            .filter(|s| self.removable(s))
            .cloned()
            .collect();
        if !takeable.is_empty() {
            out.push(ActionSpec::template(
                "take",
                vec![Param {
                    name: "tempat".into(),
                    kind: ParamKind::Choice { options: takeable },
                }],
            ));
        }
        if !self.bets.is_empty() {
            out.push(ActionSpec::fixed("roll"));
        }
        if !self.contract() {
            out.push(ActionSpec::fixed("leave"));
        }
        out
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.over() {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        let text = self.format_action(&action);
        match action {
            Action::Bet(spot, n) => {
                let l = self
                    .limits(spot)
                    .ok_or_else(|| GameError::Illegal(text.clone()))?;
                if !l.accepts(n) {
                    return Err(GameError::Illegal(text));
                }
                *self.bets.entry(spot.to_string()).or_insert(0) += n;
                Ok(())
            }
            Action::Take(spot) if self.removable(spot) => {
                self.bets.remove(spot);
                // Odds ikut ditarik bila taruhan dasarnya ditarik.
                if spot == "dont-pass" {
                    self.bets.remove("odds-dont-pass");
                }
                if let Some(n) = spot.strip_prefix("dont-come-") {
                    self.bets.remove(&format!("odds-dont-come-{n}"));
                }
                if spot == "pass" {
                    self.bets.remove("odds-pass");
                }
                Ok(())
            }
            Action::Take(_) => Err(GameError::Illegal(text)),
            Action::Roll if !self.bets.is_empty() => {
                self.roll();
                Ok(())
            }
            Action::Roll => Err(GameError::Illegal(text)),
            Action::Leave if !self.contract() => {
                self.bets.clear();
                self.ended = Some("berhenti");
                Ok(())
            }
            Action::Leave => Err(GameError::Illegal(text)),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let mut meja = Umum::new(if self.over() { "selesai" } else { "taruhan" }, STANDARD);
        meja.bersih = self.net;
        meja.ronde = self.rolls;
        meja.taruhan_meja = self.bets.values().sum();
        meja.dipertaruhkan = self.resolved;
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        if !self.over() && !self.bets.is_empty() {
            meja.netral = Some("roll".into());
        }
        View {
            meja,
            titik: self.point,
            dadu: self.dice,
            lemparan: self.rolls,
            taruhan: self.bets.clone(),
            bayar: self.payouts.clone(),
            batas: self
                .open_limits()
                .into_iter()
                .map(|(s, l)| (s, [l.min, l.max, l.step]))
                .collect(),
            bisa_ditarik: self
                .bets
                .keys()
                .filter(|s| self.removable(s))
                .cloned()
                .collect(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if let Some(d) = view.dadu {
            out.push_str(&c.text(
                lang,
                "roll",
                &[
                    ("a", &d[0].to_string()),
                    ("b", &d[1].to_string()),
                    ("sum", &(d[0] + d[1]).to_string()),
                ],
            ));
            out.push('\n');
            for (spot, pay) in &view.bayar {
                out.push_str(&format!("  {spot} → {pay:+}\n"));
            }
        }
        out.push_str(&match view.titik {
            Some(p) => c.text(lang, "point", &[("p", &p.to_string())]),
            None => c.text(lang, "comeout", &[]),
        });
        out.push('\n');
        let bets: Vec<String> = view
            .taruhan
            .iter()
            .map(|(s, n)| format!("{s} {n}"))
            .collect();
        out.push_str(&c.text(
            lang,
            "bets",
            &[(
                "bets",
                &if bets.is_empty() {
                    "-".into()
                } else {
                    bets.join(" · ")
                },
            )],
        ));
        out.push('\n');
        out.push_str(&match view.meja.alasan.as_deref() {
            Some(r) => c.text(
                lang,
                &format!("over.{r}"),
                &[("net", &format!("{:+}", view.meja.bersih))],
            ),
            None => c.text(lang, "prompt", &[]),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.over()
    }

    fn result(&self) -> Option<GameResult> {
        self.over()
            .then(|| meja::result(catalog(), "summary", self.net, self.rolls))
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        let spot = |s: &str| spots().iter().copied().find(|x| *x == s);
        match parts.as_slice() {
            ["bet", s, n] => Ok(Action::Bet(
                spot(s).ok_or_else(err)?,
                meja::amount(n).ok_or_else(err)?,
            )),
            ["take", s] => Ok(Action::Take(spot(s).ok_or_else(err)?)),
            ["roll"] => Ok(Action::Roll),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(s, n) => format!("bet {s} {n}"),
            Action::Take(s) => format!("take {s}"),
            Action::Roll => "roll".into(),
            Action::Leave => "leave".into(),
        }
    }
}
