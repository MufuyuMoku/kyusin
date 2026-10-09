//! Mesin papan taruhan bersama untuk casino dadu dan roda (SPEC §6.4;
//! M6a, D-069): Roulette, Sic Bo, Chuck-a-luck, Big Six, Fan-Tan.
//!
//! Satu pertandingan = satu putaran/lemparan, jadi commit-reveal SPEC §5.4
//! berlaku per ronde seperti Three Card Poker (D-061). Pemain menyusun
//! taruhan (`bet <tempat> <jumlah>`, `clear`), lalu memutar/melempar
//! dengan kata kerja game (`spin`, `roll`, `open`). Hasil diambil dari RNG
//! seed ronde, atau dari `hasil` di konfigurasi (tes dan tutorial). Game
//! ini tidak tahu saldo; host membaca kontrak chip [`Umum`] (D-058, D-059).
//!
//! Setiap game cukup menyebut tempat taruhannya, batas per tempat, cara
//! mengundi hasil, bayaran tiap tempat, dan sebaran peluang hasil (untuk
//! menghitung RTP tepat, [`exact_rtp`]).

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::OnceLock;

use kyusin_core::game::GameResult;
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, GameError, GameRng, PlayerId, Seed, TurnGame};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::meja::{self, Limits, MejaView, STANDARD, Umum};

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("papan_i18n.toml")).expect("papan_i18n.toml tidak sah")
    })
}

/// Aturan satu game papan taruhan.
pub trait BoardRules: Clone + Debug + PartialEq + Eq + Send + Sync + 'static {
    /// Hasil satu putaran/lemparan (juga bentuknya di view dan konfigurasi).
    type Outcome: Clone + Debug + PartialEq + Eq + Serialize + DeserializeOwned + Send + Sync;

    /// Semua tempat taruhan, sebagai kata di perintah `bet`.
    fn spots() -> &'static [&'static str];

    /// Batas taruhan satu tempat.
    fn limits(_spot: &str) -> Limits {
        STANDARD
    }

    /// Kata kerja yang menjalankan ronde (`spin`, `roll`, `open`).
    fn verb() -> &'static str;

    /// Mengundi hasil dari RNG seed ronde.
    fn draw(rng: &mut GameRng) -> Self::Outcome;

    /// Hasil bersih taruhan `stake` di `spot` (positif menang, 0 seri,
    /// negatif kalah).
    fn settle(spot: &str, stake: i64, outcome: &Self::Outcome) -> i64;

    /// Sebaran peluang semua hasil (jumlahnya 1), untuk RTP tepat.
    fn outcomes() -> Vec<(Self::Outcome, f64)>;

    /// Hasil yang sah (untuk konfigurasi tetap).
    fn valid(outcome: &Self::Outcome) -> bool;

    /// Bidang tambahan di view untuk hasil itu (misalnya angka Fan-Tan).
    fn extra(_outcome: &Self::Outcome) -> serde_json::Map<String, serde_json::Value> {
        serde_json::Map::new()
    }

    /// Hasil dalam teks.
    fn describe(outcome: &Self::Outcome, lang: Lang) -> String;
}

/// RTP tepat (persen) taruhan paling kecil di `spot`.
pub fn exact_rtp<R: BoardRules>(spot: &str) -> f64 {
    let stake = R::limits(spot).min;
    let back: f64 = R::outcomes()
        .iter()
        .map(|(o, p)| p * (stake + R::settle(spot, stake, o)) as f64)
        .sum();
    100.0 * back / stake as f64
}

/// Daftar nama tempat yang dibangun saat program berjalan, disimpan untuk
/// selamanya (dipakai `spots()` yang mengembalikan `&'static`).
pub fn leak(names: Vec<String>) -> &'static [&'static str] {
    let v: Vec<&'static str> = names
        .into_iter()
        .map(|s| &*Box::leak(s.into_boxed_str()))
        .collect();
    Box::leak(v.into_boxed_slice())
}

/// Konfigurasi: hasil tetap (tes dan tutorial).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, bound = "O: DeserializeOwned")]
pub struct Config<O> {
    #[serde(default = "none")]
    pub hasil: Option<O>,
}

fn none<O>() -> Option<O> {
    None
}

impl<O> Default for Config<O> {
    fn default() -> Self {
        Config { hasil: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Go,
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(bound = "")]
pub struct Board<R: BoardRules> {
    seed: Seed,
    fixed: Option<R::Outcome>,
    bets: BTreeMap<String, i64>,
    outcome: Option<R::Outcome>,
    last_bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
    net: i64,
    ended: Option<&'static str>,
    #[serde(skip)]
    _r: PhantomData<R>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "O: Serialize + DeserializeOwned")]
pub struct View<O> {
    #[serde(flatten)]
    pub meja: Umum,
    /// Taruhan yang sedang disusun.
    pub taruhan: BTreeMap<String, i64>,
    /// Hasil putaran/lemparan, setelah ronde dijalankan.
    pub hasil: Option<O>,
    pub taruhan_terakhir: BTreeMap<String, i64>,
    /// Hasil bersih tiap tempat taruhan.
    pub bayar: BTreeMap<String, i64>,
    #[serde(flatten)]
    pub lain: serde_json::Map<String, serde_json::Value>,
}

impl<O> MejaView for View<O> {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl<R: BoardRules> Board<R> {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn spot(name: &str) -> Option<&'static str> {
        R::spots().iter().copied().find(|s| *s == name)
    }

    fn place(&mut self, spot: &'static str, n: i64) -> bool {
        let l = R::limits(spot);
        let now = self.bets.get(spot).copied().unwrap_or(0);
        if n < l.min || n % l.step != 0 || now + n > l.max {
            return false;
        }
        self.bets.insert(spot.to_string(), now + n);
        true
    }

    fn go(&mut self) {
        let outcome = self
            .fixed
            .clone()
            .unwrap_or_else(|| R::draw(&mut GameRng::from_seed(self.seed)));
        let bets = std::mem::take(&mut self.bets);
        let payouts: BTreeMap<String, i64> = bets
            .iter()
            .map(|(spot, &b)| (spot.clone(), R::settle(spot, b, &outcome)))
            .collect();
        self.net += payouts.values().sum::<i64>();
        self.last_bets = bets;
        self.payouts = payouts;
        self.outcome = Some(outcome);
        self.ended = Some("ronde_selesai");
    }
}

impl<R: BoardRules> TurnGame for Board<R> {
    type Config = Config<R::Outcome>;
    type Action = Action;
    type View = View<R::Outcome>;

    fn new(config: Self::Config, seed: Seed) -> Result<Self, GameError> {
        if let Some(o) = &config.hasil
            && !R::valid(o)
        {
            return Err(GameError::Config(format!("hasil {o:?}")));
        }
        Ok(Board {
            seed,
            fixed: config.hasil,
            bets: BTreeMap::new(),
            outcome: None,
            last_bets: BTreeMap::new(),
            payouts: BTreeMap::new(),
            net: 0,
            ended: None,
            _r: PhantomData,
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
        let mut out: Vec<ActionSpec> = R::spots()
            .iter()
            .filter_map(|s| R::limits(s).spot(s, self.bets.get(*s).copied().unwrap_or(0)))
            .collect();
        if !self.bets.is_empty() {
            out.push(ActionSpec::fixed("clear"));
            out.push(ActionSpec::fixed(R::verb()));
        }
        out.push(ActionSpec::fixed("leave"));
        out
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.over() {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        match action {
            Action::Bet(spot, n) => {
                if self.place(spot, n) {
                    Ok(())
                } else {
                    Err(GameError::Illegal(format!("bet {spot} {n}")))
                }
            }
            Action::Clear => {
                self.bets.clear();
                Ok(())
            }
            Action::Go if !self.bets.is_empty() => {
                self.go();
                Ok(())
            }
            Action::Go => Err(GameError::Illegal(R::verb().into())),
            Action::Leave => {
                self.bets.clear();
                self.ended = Some("berhenti");
                Ok(())
            }
        }
    }

    fn view_for(&self, _player: PlayerId) -> Self::View {
        let mut meja = Umum::new(if self.over() { "selesai" } else { "taruhan" }, STANDARD);
        let min = R::spots()
            .iter()
            .map(|s| R::limits(s))
            .next()
            .unwrap_or(STANDARD);
        meja.min_taruhan = min.min;
        meja.maks_taruhan = min.max;
        meja.langkah_taruhan = min.step;
        meja.bersih = self.net;
        meja.ronde = u32::from(self.outcome.is_some());
        meja.taruhan_meja = self.bets.values().sum();
        meja.dipertaruhkan = self.last_bets.values().sum();
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        if !self.over() && !self.bets.is_empty() {
            meja.netral = Some(R::verb().into());
        }
        View {
            meja,
            taruhan: self.bets.clone(),
            hasil: self.outcome.clone(),
            taruhan_terakhir: self.last_bets.clone(),
            bayar: self.payouts.clone(),
            lain: self.outcome.as_ref().map(R::extra).unwrap_or_default(),
        }
    }

    fn to_text(view: &Self::View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if let Some(o) = &view.hasil {
            out.push_str(&c.text(lang, "result", &[("result", &R::describe(o, lang))]));
            out.push('\n');
            for (spot, pay) in &view.bayar {
                out.push_str(&format!(
                    "  {spot} {} → {pay:+}\n",
                    view.taruhan_terakhir.get(spot).copied().unwrap_or(0)
                ));
            }
        }
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
            Some(reason) => c.text(
                lang,
                &format!("over.{reason}"),
                &[("net", &format!("{:+}", view.meja.bersih))],
            ),
            None => c.text(lang, "place_bet", &[("verb", R::verb())]),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.over()
    }

    fn result(&self) -> Option<GameResult> {
        self.over().then(|| {
            meja::result(
                catalog(),
                "summary",
                self.net,
                u32::from(self.outcome.is_some()),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", spot, n] => {
                let spot = Self::spot(spot).ok_or_else(err)?;
                Ok(Action::Bet(spot, meja::amount(n).ok_or_else(err)?))
            }
            ["clear"] => Ok(Action::Clear),
            [verb] if *verb == R::verb() => Ok(Action::Go),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(spot, n) => format!("bet {spot} {n}"),
            Action::Clear => "clear".into(),
            Action::Go => R::verb().into(),
            Action::Leave => "leave".into(),
        }
    }
}
