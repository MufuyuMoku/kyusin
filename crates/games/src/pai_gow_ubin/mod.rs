//! Pai Gow ubin (SPEC §6.4; M6a, D-069): 32 ubin domino Cina, bandar selalu
//! bankir.
//!
//! Pemain dan bandar masing-masing menerima empat ubin dan menyusunnya
//! menjadi dua tangan dua ubin. Tangan tinggi dibandingkan dengan tangan
//! tinggi, rendah dengan rendah. Menang keduanya dibayar 1:1 dipotong
//! komisi 5%, satu-satu push, kalah keduanya kehilangan taruhan; seri
//! persis (copy) dimenangkan bandar. Taruhan 20–2.000 kelipatan 20 supaya
//! komisi utuh. Satu pertandingan = satu ronde.
//!
//! Urutan tangan: pasangan Gee Joon > pasangan (Teen, Day, Yun, Gor, Mooy,
//! Chong, Bon, Foo, Ping, Tit, Look, lalu campuran 9, 8, 7, 5) > Wong
//! (Teen/Day + 9) > Gong (Teen/Day + 8) > nilai 9 … 0 (jumlah bulatan mod
//! 10; ubin Gee Joon 3 atau 6, mana yang lebih baik). Nilai sama: ubin
//! tertinggi (urutan di atas, ubin campuran senilai sepangkat, Gee Joon
//! terendah); masih sama, atau sama-sama 0: bandar.
//!
//! House way (bandar, tombol saran, dan aksi netral; ringkasan house way
//! Casino Canberra, D-069) ada di [`house_way`].

use std::cmp::Ordering;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::domino::Tile;
use crate::meja::{self, Limits, MejaView, Umum, WagerRtp};

pub const ID: &str = "pai-gow-ubin";
pub const LIMITS: Limits = Limits {
    min: 20,
    max: 2000,
    step: 20,
};

/// RTP per taruhan, pemain dan bandar memakai house way (enumerasi tepat
/// semua pasangan tangan di GitHub Actions; D-069).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 97.4,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("pai_gow_ubin/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/pai-gow-ubin.toml"),
        create_session::<PaiGowUbin>,
    )
}

const T: fn(u8, u8) -> Tile = Tile::new;

/// Kelompok pangkat ubin dari tertinggi; ubin satu kelompok sepangkat dan
/// berpasangan.
fn groups() -> &'static [Vec<Tile>] {
    static G: OnceLock<Vec<Vec<Tile>>> = OnceLock::new();
    G.get_or_init(|| {
        vec![
            vec![T(6, 6)],
            vec![T(1, 1)],
            vec![T(4, 4)],
            vec![T(3, 1)],
            vec![T(5, 5)],
            vec![T(3, 3)],
            vec![T(2, 2)],
            vec![T(6, 5)],
            vec![T(6, 4)],
            vec![T(6, 1)],
            vec![T(5, 1)],
            vec![T(6, 3), T(5, 4)],
            vec![T(6, 2), T(5, 3)],
            vec![T(5, 2), T(4, 3)],
            vec![T(4, 1), T(3, 2)],
            vec![T(2, 1), T(4, 2)],
        ]
    })
}

/// Pangkat ubin (lebih besar = lebih tinggi); `None` bila bukan ubin Cina.
pub fn rank(t: Tile) -> Option<u8> {
    groups()
        .iter()
        .position(|g| g.contains(&t))
        .map(|i| (groups().len() - i) as u8)
}

fn gee_joon(t: Tile) -> bool {
    t == T(2, 1) || t == T(4, 2)
}

fn teen_or_day(t: Tile) -> bool {
    t == T(6, 6) || t == T(1, 1)
}

/// Satu set 32 ubin: 11 pasangan sipil (dua ubin sama) dan 10 ubin
/// militer.
pub fn set() -> Vec<Tile> {
    groups()
        .iter()
        .flat_map(|g| if g.len() == 1 { vec![g[0], g[0]] } else { g.clone() })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HandKind {
    Points,
    Gong,
    Wong,
    Pair,
    GeeJoon,
}

impl HandKind {
    pub fn key(self) -> &'static str {
        match self {
            HandKind::Points => "points",
            HandKind::Gong => "gong",
            HandKind::Wong => "wong",
            HandKind::Pair => "pair",
            HandKind::GeeJoon => "gee_joon",
        }
    }
}

/// Satu tangan dua ubin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hand {
    pub tiles: [Tile; 2],
    kind: HandKind,
    points: u8,
}

impl Hand {
    pub fn kind(&self) -> HandKind {
        self.kind
    }

    /// Nilai 0–9 (untuk tangan nilai; pasangan/Wong/Gong tetap dihitung).
    pub fn points(&self) -> u8 {
        self.points
    }

    fn high_rank(&self) -> u8 {
        self.tiles.iter().filter_map(|t| rank(*t)).max().unwrap_or(0)
    }

    /// Kekuatan kasar untuk ambang house way: nilai 0–9, Gong 10, Wong 11,
    /// pasangan 12 ke atas.
    fn level(&self) -> u8 {
        match self.kind {
            HandKind::Points => self.points,
            HandKind::Gong => 10,
            HandKind::Wong => 11,
            HandKind::Pair | HandKind::GeeJoon => 12,
        }
    }
}

pub fn hand(a: Tile, b: Tile) -> Hand {
    let values = |t: Tile| if gee_joon(t) { vec![3u8, 6] } else { vec![t.pips()] };
    let points = values(a)
        .iter()
        .flat_map(|x| values(b).into_iter().map(move |y| (x + y) % 10))
        .max()
        .unwrap_or(0);
    let same_group = rank(a).is_some() && rank(a) == rank(b);
    let other = |t: Tile, u: Tile| if teen_or_day(t) { Some(u) } else { None };
    let partner = other(a, b).or_else(|| other(b, a));
    let kind = if same_group && gee_joon(a) {
        HandKind::GeeJoon
    } else if same_group {
        HandKind::Pair
    } else if partner.is_some_and(|u| u.pips() == 9) {
        HandKind::Wong
    } else if partner.is_some_and(|u| u.pips() == 8) {
        HandKind::Gong
    } else {
        HandKind::Points
    };
    Hand {
        tiles: [a, b],
        kind,
        points,
    }
}

/// Perbandingan dua tangan. `Equal` = copy (dimenangkan bandar).
pub fn compare(a: &Hand, b: &Hand) -> Ordering {
    a.kind.cmp(&b.kind).then_with(|| match a.kind {
        HandKind::GeeJoon => Ordering::Equal,
        HandKind::Pair | HandKind::Wong | HandKind::Gong => a.high_rank().cmp(&b.high_rank()),
        HandKind::Points => a.points.cmp(&b.points).then_with(|| {
            if a.points == 0 {
                Ordering::Equal
            } else {
                a.high_rank().cmp(&b.high_rank())
            }
        }),
    })
}

/// Tiga cara membagi empat ubin, masing-masing (tinggi, rendah).
fn splits(t: [Tile; 4]) -> Vec<(Hand, Hand)> {
    [(1, 2, 3), (2, 1, 3), (3, 1, 2)]
        .iter()
        .map(|&(p, x, y)| {
            let (h1, h2) = (hand(t[0], t[p]), hand(t[x], t[y]));
            if compare(&h1, &h2) == Ordering::Less { (h2, h1) } else { (h1, h2) }
        })
        .collect()
}

fn better_low(a: &(Hand, Hand), b: &(Hand, Hand)) -> Ordering {
    compare(&a.1, &b.1).then_with(|| compare(&a.0, &b.0))
}

/// House way: susunan (tinggi, rendah) untuk empat ubin.
///
/// 1. Dua pasangan: mainkan keduanya.
/// 2. Satu pasangan: pasangan Gee Joon, 4 (Gor, Bon), 5, 6 (Chong, Look), 10
///    (Mooy, Ping), dan 11 (Foo) tidak pernah dipecah. Teen dan Day dipecah
///    bila hasilnya paling sedikit 6 dan 8; 9 campuran bila hasilnya 9 dan 9;
///    8 (Yun, campuran) bila 7 dan 9 atau 8 dan 8; 7 (Tit, campuran) bila 7
///    dan 9. Pasangan tetap dipertahankan bila pemecahannya membuat kedua
///    tangan lebih rendah (tangan rendah hasil pecahan tidak lebih tinggi
///    dari tangan rendah saat pasangan dipertahankan).
/// 3. Tanpa pasangan: High Nine (Teen/Day + 7), lalu Gong, lalu Wong bila
///    tangan rendahnya paling sedikit 4; selain itu pembagian dengan tangan
///    rendah tertinggi (lalu tangan tinggi tertinggi).
pub fn house_way(t: [Tile; 4]) -> ([Tile; 2], [Tile; 2]) {
    let all = splits(t);
    let paired = |s: &(Hand, Hand)| s.0.kind >= HandKind::Pair;
    let pick = |s: &(Hand, Hand)| (s.0.tiles, s.1.tiles);
    if let Some(two) = all.iter().find(|s| paired(s) && s.1.kind >= HandKind::Pair) {
        return pick(two);
    }
    if let Some(kept) = all.iter().find(|s| paired(s)) {
        let pair_tile = kept.0.tiles[0];
        let never = [T(3, 1), T(2, 2), T(4, 1), T(3, 2), T(3, 3), T(5, 1), T(5, 5), T(6, 4), T(6, 5)];
        if kept.0.kind == HandKind::GeeJoon || never.contains(&pair_tile) {
            return pick(kept);
        }
        let need = |s: &(Hand, Hand)| -> bool {
            let (hi, lo) = (s.0.level(), s.1.level());
            match rank(pair_tile) {
                _ if teen_or_day(pair_tile) => lo >= 6 && hi >= 8,
                _ if pips_group(pair_tile) == 9 => lo >= 9 && hi >= 9,
                _ if pips_group(pair_tile) == 8 => (lo >= 7 && hi >= 9) || (lo >= 8 && hi >= 8),
                _ => lo >= 7 && hi >= 9,
            }
        };
        let best = all
            .iter()
            .filter(|s| !paired(s) && need(s) && compare(&s.1, &kept.1) == Ordering::Greater)
            .max_by(|a, b| better_low(a, b));
        return pick(best.unwrap_or(kept));
    }
    let special = |kind: fn(&Hand) -> bool| {
        all.iter()
            .filter(|s| kind(&s.0) && s.1.level() >= 4)
            .max_by(|a, b| better_low(a, b))
    };
    let high_nine = |h: &Hand| h.kind == HandKind::Points && h.points == 9 && h.tiles.iter().any(|t| teen_or_day(*t));
    let gong = |h: &Hand| h.kind == HandKind::Gong;
    let wong = |h: &Hand| h.kind == HandKind::Wong;
    if let Some(s) = special(high_nine).or_else(|| special(gong)).or_else(|| special(wong)) {
        return pick(s);
    }
    pick(all.iter().max_by(|a, b| better_low(a, b)).expect("tiga pembagian"))
}

/// Jumlah bulatan kelompok ubin (untuk aturan pecah pasangan).
fn pips_group(t: Tile) -> u8 {
    t.pips()
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Urutan ubin tetap: empat pertama untuk pemain, empat berikutnya
    /// untuk bandar (tes dan tutorial).
    #[serde(default)]
    pub ubin: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Set(Tile, Tile),
    HouseWay,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Set,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: [Tile; 4],
    dealer: [Tile; 4],
    bet: i64,
    set: Option<([Tile; 2], [Tile; 2])>,
    dealer_set: Option<([Tile; 2], [Tile; 2])>,
    wins: Option<[bool; 2]>,
    payout: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaiGowUbin {
    tiles: Vec<Tile>,
    phase: Phase,
    round: Option<Round>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Empat ubin pemain.
    pub pemain: Vec<String>,
    /// Empat ubin bandar; `??` sampai ronde selesai.
    pub bandar: Vec<String>,
    pub tinggi: Vec<String>,
    pub rendah: Vec<String>,
    pub bandar_tinggi: Vec<String>,
    pub bandar_rendah: Vec<String>,
    /// Jenis tangan: [tinggi, rendah, bandar tinggi, bandar rendah], misalnya
    /// `pair`, `wong`, `7`.
    pub nama: Vec<String>,
    /// `menang`/`kalah` untuk tangan tinggi dan rendah.
    pub hasil: Vec<String>,
    /// Susunan saran (house way) selama menyusun: dua ubin tangan rendah.
    pub saran: Vec<String>,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

fn name(h: &Hand) -> String {
    match h.kind {
        HandKind::Points => h.points.to_string(),
        k => k.key().into(),
    }
}

impl PaiGowUbin {
    fn settle(&mut self, mine: ([Tile; 2], [Tile; 2])) {
        let r = self.round.as_mut().expect("ronde berjalan");
        let dealer = house_way(r.dealer);
        let win = |a: [Tile; 2], b: [Tile; 2]| compare(&hand(a[0], a[1]), &hand(b[0], b[1])) == Ordering::Greater;
        let wins = [win(mine.0, dealer.0), win(mine.1, dealer.1)];
        r.payout = Some(match wins {
            [true, true] => r.bet * 19 / 20,
            [false, false] => -r.bet,
            _ => 0,
        });
        r.set = Some(mine);
        r.dealer_set = Some(dealer);
        r.wins = Some(wins);
        self.phase = Phase::Over;
        self.ended = Some("ronde_selesai");
    }
}

fn order(a: [Tile; 2], b: [Tile; 2]) -> ([Tile; 2], [Tile; 2]) {
    if compare(&hand(a[0], a[1]), &hand(b[0], b[1])) == Ordering::Less {
        (b, a)
    } else {
        (a, b)
    }
}

impl TurnGame for PaiGowUbin {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let tiles = match config.ubin {
            Some(list) => {
                let t = list
                    .iter()
                    .map(|s| {
                        s.parse::<Tile>()
                            .ok()
                            .filter(|t| rank(*t).is_some())
                            .ok_or_else(|| GameError::Config(format!("ubin `{s}`")))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if t.len() < 8 {
                    return Err(GameError::Config("ubin: paling sedikit 8".into()));
                }
                t
            }
            None => {
                let mut t = set();
                GameRng::from_seed(seed).shuffle(&mut t);
                t
            }
        };
        Ok(PaiGowUbin {
            tiles,
            phase: Phase::Betting,
            round: None,
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.phase == Phase::Over { Vec::new() } else { vec![0] }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 {
            return Vec::new();
        }
        match self.phase {
            Phase::Betting => vec![LIMITS.single(), ActionSpec::fixed("leave")],
            Phase::Set => {
                let options: Vec<String> = self
                    .round
                    .as_ref()
                    .map(|r| r.player.iter().map(Tile::to_string).collect())
                    .unwrap_or_default();
                let tile = || Param {
                    name: "ubin".into(),
                    kind: ParamKind::Choice {
                        options: options.clone(),
                    },
                };
                vec![
                    ActionSpec::template("set", vec![tile(), tile()]),
                    ActionSpec::fixed("houseway"),
                ]
            }
            Phase::Over => Vec::new(),
        }
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        let illegal = GameError::Illegal(self.format_action(&action));
        match (self.phase, action) {
            (Phase::Betting, Action::Bet(n)) if LIMITS.accepts(n) => {
                let p = [self.tiles[0], self.tiles[1], self.tiles[2], self.tiles[3]];
                let d = [self.tiles[4], self.tiles[5], self.tiles[6], self.tiles[7]];
                self.round = Some(Round {
                    player: p,
                    dealer: d,
                    bet: n,
                    set: None,
                    dealer_set: None,
                    wins: None,
                    payout: None,
                });
                self.phase = Phase::Set;
                Ok(())
            }
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Set, Action::Set(a, b)) => {
                let four = self.round.as_ref().expect("ronde").player;
                let mut rest = four.to_vec();
                for t in [a, b] {
                    let Some(i) = rest.iter().position(|x| *x == t) else {
                        return Err(illegal);
                    };
                    rest.remove(i);
                }
                self.settle(order([a, b], [rest[0], rest[1]]));
                Ok(())
            }
            (Phase::Set, Action::HouseWay) => {
                let four = self.round.as_ref().expect("ronde").player;
                self.settle(house_way(four));
                Ok(())
            }
            _ => Err(illegal),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Set => "susun",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, LIMITS);
        let r = self.round.as_ref();
        let over = self.phase == Phase::Over;
        meja.bersih = r.and_then(|r| r.payout).unwrap_or(0);
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = if self.phase == Phase::Set {
            r.map(|r| r.bet).unwrap_or(0)
        } else {
            0
        };
        meja.dipertaruhkan = r.map(|r| r.bet).unwrap_or(0);
        if self.phase == Phase::Set {
            meja.netral = Some("houseway".into());
        }
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        let text = |v: &[Tile]| v.iter().map(Tile::to_string).collect::<Vec<_>>();
        let set = r.and_then(|r| r.set);
        let dset = r.and_then(|r| r.dealer_set);
        let mut nama = Vec::new();
        if let (Some(s), Some(d)) = (set, dset) {
            for h in [s.0, s.1, d.0, d.1] {
                nama.push(name(&hand(h[0], h[1])));
            }
        }
        View {
            meja,
            pemain: r.map(|r| text(&r.player)).unwrap_or_default(),
            bandar: r
                .map(|r| if over { text(&r.dealer) } else { vec!["??".into(); 4] })
                .unwrap_or_default(),
            tinggi: set.map(|s| text(&s.0)).unwrap_or_default(),
            rendah: set.map(|s| text(&s.1)).unwrap_or_default(),
            bandar_tinggi: dset.map(|s| text(&s.0)).unwrap_or_default(),
            bandar_rendah: dset.map(|s| text(&s.1)).unwrap_or_default(),
            nama,
            hasil: r
                .and_then(|r| r.wins)
                .map(|w| w.iter().map(|x| if *x { "menang" } else { "kalah" }.to_string()).collect())
                .unwrap_or_default(),
            saran: if self.phase == Phase::Set {
                r.map(|r| text(&house_way(r.player).1)).unwrap_or_default()
            } else {
                Vec::new()
            },
            taruhan: r.map(|r| r.bet).unwrap_or(0),
            bayar: r.and_then(|r| r.payout),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            out.push_str(&c.text(lang, "tiles", &[("you", &view.pemain.join(" ")), ("dealer", &view.bandar.join(" "))]));
            out.push('\n');
        }
        if view.nama.len() == 4 {
            let kind = |n: &String| {
                if n.parse::<u8>().is_ok() {
                    c.text(lang, "kind.points", &[("n", n)])
                } else {
                    c.text(lang, &format!("kind.{n}"), &[])
                }
            };
            out.push_str(&c.text(
                lang,
                "hands",
                &[
                    ("hi", &format!("{} ({})", view.tinggi.join(" "), kind(&view.nama[0]))),
                    ("lo", &format!("{} ({})", view.rendah.join(" "), kind(&view.nama[1]))),
                    ("dhi", &format!("{} ({})", view.bandar_tinggi.join(" "), kind(&view.nama[2]))),
                    ("dlo", &format!("{} ({})", view.bandar_rendah.join(" "), kind(&view.nama[3]))),
                ],
            ));
            out.push('\n');
        }
        out.push_str(&match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
            ("taruhan", _) => c.text(lang, "place_bet", &[]),
            ("susun", _) => c.text(lang, "set", &[]),
            (_, Some(reason)) => c.text(
                lang,
                &format!("over.{reason}"),
                &[("net", &format!("{:+}", view.bayar.unwrap_or(0)))],
            ),
            _ => String::new(),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        (self.phase == Phase::Over).then(|| {
            meja::result(
                catalog(),
                "summary",
                self.round.as_ref().and_then(|r| r.payout).unwrap_or(0),
                u32::from(self.round.is_some()),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["set", a, b] => Ok(Action::Set(a.parse().map_err(|_| err())?, b.parse().map_err(|_| err())?)),
            ["houseway"] => Ok(Action::HouseWay),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Set(a, b) => format!("set {a} {b}"),
            Action::HouseWay => "houseway".into(),
            Action::Leave => "leave".into(),
        }
    }
}
