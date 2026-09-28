//! Catur (SPEC §6.1). Generator langkah: cozy-chess (MIT).
//!
//! Aturan FIDE. Putih = kursi 0. Notasi kanonik SAN; notasi koordinat dan
//! `0-0` diterima sebagai alias lewat `canonical` (D-042). Remis otomatis:
//! pat, ulangan tiga kali, aturan 50 langkah, bahan tidak cukup. `resign`
//! untuk menyerah; `timeout` (hanya bila ada jam) diajukan host saat waktu
//! pemain habis. Jam sendiri dihitung host, bukan game (tanpa jam dinding).

pub mod pgn;
pub mod san;

use std::sync::OnceLock;

use cozy_chess::{Board, Color, Piece, Square};
use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize, Serializer};

use san::{Castle, Legal};

pub const ID: &str = "catur";
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("catur/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/catur.toml"),
        create_session::<Catur>,
    )
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Posisi awal (FEN); bawaan posisi awal baku.
    #[serde(default)]
    pub fen: Option<String>,
    /// Jam opsional; waktunya dihitung host.
    #[serde(default)]
    pub jam: Option<Jam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Jam {
    pub menit: u32,
    pub tambahan_detik: u32,
}

#[derive(Debug, Clone)]
pub enum Action {
    Play(Box<Legal>),
    Resign,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastMove {
    pub dari: String,
    pub ke: String,
    pub san: String,
    /// Huruf kecil bidak promosi (`q`, `r`, `b`, `n`).
    pub promosi: Option<String>,
    /// `pendek` atau `panjang`.
    pub rokade: Option<String>,
    pub en_passant: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveView {
    pub dari: String,
    pub ke: String,
    pub promosi: Option<String>,
    pub san: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    /// 8 baris dari baris 8 ke baris 1; huruf FEN (besar = putih), `.` kosong.
    pub papan: Vec<String>,
    pub giliran: Option<PlayerId>,
    pub kamu: PlayerId,
    /// Langkah sah pemain yang sedang melangkah.
    pub langkah: Vec<MoveView>,
    pub terakhir: Option<LastMove>,
    pub skak: bool,
    pub raja_skak: Option<String>,
    pub riwayat: Vec<String>,
    pub selesai: bool,
    pub pemenang: Option<Vec<PlayerId>>,
    /// `skakmat`, `pat`, `ulangan`, `50_langkah`, `bahan`, `menyerah`, `waktu`.
    pub alasan: Option<String>,
    pub jam: Option<Jam>,
    pub fen: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Ending {
    alasan: &'static str,
    winners: Vec<PlayerId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Catur {
    #[serde(serialize_with = "fen_of")]
    board: Board,
    pub(crate) start_fen: String,
    pub(crate) history: Vec<String>,
    /// Hash posisi sejak awal, untuk ulangan tiga kali.
    positions: Vec<u64>,
    last: Option<LastMove>,
    pub(crate) jam: Option<Jam>,
    ending: Option<Ending>,
    #[serde(skip)]
    legal: Vec<Legal>,
}

fn fen_of<S: Serializer>(b: &Board, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&b.to_string())
}

pub fn seat_of(c: Color) -> PlayerId {
    match c {
        Color::White => 0,
        Color::Black => 1,
    }
}

fn color_of(seat: PlayerId) -> Color {
    if seat == 0 {
        Color::White
    } else {
        Color::Black
    }
}

/// Tidak ada pihak yang bisa mat: tanpa pion/benteng/menteri, dan paling
/// banyak satu bidak ringan, atau semua gajah di warna petak yang sama.
fn insufficient(b: &Board) -> bool {
    let heavy = b.pieces(Piece::Pawn) | b.pieces(Piece::Rook) | b.pieces(Piece::Queen);
    if !heavy.is_empty() {
        return false;
    }
    let knights = b.pieces(Piece::Knight).len();
    let bishops = b.pieces(Piece::Bishop);
    if knights + bishops.len() <= 1 {
        return true;
    }
    if knights > 0 {
        return false;
    }
    let shade = |sq: Square| (sq.file() as usize + sq.rank() as usize) % 2;
    let mut shades = bishops.into_iter().map(shade);
    let first = shades.next();
    shades.all(|s| Some(s) == first)
}

/// `color` tidak mungkin mat lagi: tinggal raja, atau raja + satu bidak
/// ringan melawan raja sendirian.
fn cannot_mate(b: &Board, color: Color) -> bool {
    let mine = b.colors(color);
    let others = b.colors(!color);
    let minors = (b.pieces(Piece::Knight) | b.pieces(Piece::Bishop)) & mine;
    let only_king = mine.len() == 1;
    only_king || (mine.len() == 2 && minors.len() == 1 && others.len() == 1)
}

impl Catur {
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn legal(&self) -> &[Legal] {
        &self.legal
    }

    fn mover(&self) -> PlayerId {
        seat_of(self.board.side_to_move())
    }

    fn evaluate(&mut self) {
        let mover = self.mover();
        let other = 1 - mover;
        self.ending = if self.legal.is_empty() {
            Some(if self.board.checkers().is_empty() {
                Ending {
                    alasan: "pat",
                    winners: vec![],
                }
            } else {
                Ending {
                    alasan: "skakmat",
                    winners: vec![other],
                }
            })
        } else if self.board.halfmove_clock() >= 100 {
            Some(Ending {
                alasan: "50_langkah",
                winners: vec![],
            })
        } else if self
            .positions
            .iter()
            .filter(|h| **h == self.board.hash())
            .count()
            >= 3
        {
            Some(Ending {
                alasan: "ulangan",
                winners: vec![],
            })
        } else if insufficient(&self.board) {
            Some(Ending {
                alasan: "bahan",
                winners: vec![],
            })
        } else {
            None
        };
    }
}

fn name(lang: Lang, seat: PlayerId) -> String {
    catalog().text(lang, if seat == 0 { "white" } else { "black" }, &[])
}

fn outcome(lang: Lang, alasan: &str, winners: &[PlayerId]) -> String {
    let c = catalog();
    let key = format!("end.{alasan}");
    match winners.first() {
        Some(&w) => c.text(
            lang,
            &key,
            &[("winner", &name(lang, w)), ("loser", &name(lang, 1 - w))],
        ),
        None => c.text(lang, &key, &[]),
    }
}

impl TurnGame for Catur {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, _seed: Seed) -> Result<Self, GameError> {
        let start_fen = config.fen.clone().unwrap_or_else(|| START_FEN.to_string());
        let board = Board::from_fen(&start_fen, false)
            .map_err(|e| GameError::Config(format!("FEN `{start_fen}`: {e:?}")))?;
        let mut g = Catur {
            positions: vec![board.hash()],
            legal: san::legal_moves(&board),
            board,
            start_fen,
            history: Vec::new(),
            last: None,
            jam: config.jam,
            ending: None,
        };
        g.evaluate();
        Ok(g)
    }

    fn seats(&self) -> u8 {
        2
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.ending.is_some() {
            Vec::new()
        } else {
            vec![self.mover()]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if self.ending.is_some() || player != self.mover() {
            return Vec::new();
        }
        let mut out: Vec<ActionSpec> = self
            .legal
            .iter()
            .map(|m| ActionSpec::fixed(&m.san))
            .collect();
        out.push(ActionSpec::fixed("resign"));
        if self.jam.is_some() {
            out.push(ActionSpec::fixed("timeout"));
        }
        out
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.ending.is_some() {
            return Err(GameError::Over);
        }
        if player != self.mover() {
            return Err(GameError::NotPending(player));
        }
        match action {
            Action::Play(m) => {
                if !self.legal.iter().any(|l| l.raw == m.raw) {
                    return Err(GameError::Illegal(m.san.clone()));
                }
                self.board.play(m.raw);
                self.history.push(m.san.clone());
                self.positions.push(self.board.hash());
                self.last = Some(LastMove {
                    dari: m.from.to_string(),
                    ke: m.to.to_string(),
                    san: m.san.clone(),
                    promosi: m.promotion.map(|p| san::promotion_char(p).to_string()),
                    rokade: m.castle.map(|c| match c {
                        Castle::King => "pendek".to_string(),
                        Castle::Queen => "panjang".to_string(),
                    }),
                    en_passant: m.en_passant,
                });
                self.legal = san::legal_moves(&self.board);
                self.evaluate();
            }
            Action::Resign => {
                self.ending = Some(Ending {
                    alasan: "menyerah",
                    winners: vec![1 - player],
                });
            }
            Action::Timeout => {
                if self.jam.is_none() {
                    return Err(GameError::Illegal("timeout".into()));
                }
                let opponent = color_of(1 - player);
                let winners = if cannot_mate(&self.board, opponent) {
                    vec![]
                } else {
                    vec![1 - player]
                };
                self.ending = Some(Ending {
                    alasan: "waktu",
                    winners,
                });
            }
        }
        Ok(())
    }

    fn view_for(&self, player: PlayerId) -> View {
        let mut papan = Vec::new();
        for rank in (0..8).rev() {
            let mut row = String::new();
            for file in 0..8 {
                let sq = Square::new(cozy_chess::File::index(file), cozy_chess::Rank::index(rank));
                row.push(match (self.board.piece_on(sq), self.board.color_on(sq)) {
                    (Some(p), Some(Color::White)) => san::piece_letter(p),
                    (Some(p), Some(Color::Black)) => san::piece_letter(p).to_ascii_lowercase(),
                    _ => '.',
                });
            }
            papan.push(row);
        }
        let skak = !self.board.checkers().is_empty();
        let over = self.ending.is_some();
        View {
            papan,
            giliran: (!over).then(|| self.mover()),
            kamu: player,
            langkah: if over {
                Vec::new()
            } else {
                self.legal
                    .iter()
                    .map(|m| MoveView {
                        dari: m.from.to_string(),
                        ke: m.to.to_string(),
                        promosi: m.promotion.map(|p| san::promotion_char(p).to_string()),
                        san: m.san.clone(),
                    })
                    .collect()
            },
            terakhir: self.last.clone(),
            skak,
            raja_skak: skak.then(|| self.board.king(self.board.side_to_move()).to_string()),
            riwayat: self.history.clone(),
            selesai: over,
            pemenang: self.ending.as_ref().map(|e| e.winners.clone()),
            alasan: self.ending.as_ref().map(|e| e.alasan.to_string()),
            jam: self.jam.clone(),
            fen: self.board.to_string(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::from("    a b c d e f g h\n");
        for (i, row) in view.papan.iter().enumerate() {
            let rank = 8 - i;
            let cells: Vec<String> = row.chars().map(|ch| ch.to_string()).collect();
            out.push_str(&format!(" {rank}  {}  {rank}\n", cells.join(" ")));
        }
        out.push_str("    a b c d e f g h\n\n");
        if let Some(last) = &view.terakhir {
            out.push_str(&c.text(lang, "last", &[("san", &last.san)]));
            out.push('\n');
        }
        let status = match (&view.alasan, view.giliran) {
            (Some(alasan), _) => outcome(lang, alasan, view.pemenang.as_deref().unwrap_or(&[])),
            (None, Some(seat)) if seat == view.kamu => {
                c.text(lang, "your_turn", &[("color", &name(lang, seat))])
            }
            (None, Some(seat)) => c.text(lang, "their_turn", &[("color", &name(lang, seat))]),
            (None, None) => String::new(),
        };
        out.push_str(&status);
        if view.skak && view.alasan.is_none() {
            out.push(' ');
            out.push_str(&c.text(lang, "check", &[]));
        }
        out
    }

    fn is_over(&self) -> bool {
        self.ending.is_some()
    }

    fn result(&self) -> Option<GameResult> {
        let e = self.ending.as_ref()?;
        // Skor = poin × 2 (menang 2, remis 1, kalah 0).
        let scores = match e.winners.as_slice() {
            [w] => (0..2).map(|s| if s == *w { 2 } else { 0 }).collect(),
            _ => vec![1, 1],
        };
        Some(GameResult {
            winners: e.winners.clone(),
            scores,
            summary: Localized::build(|lang| outcome(lang, e.alasan, &e.winners)),
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        match command.trim() {
            "resign" => Ok(Action::Resign),
            "timeout" if self.jam.is_some() => Ok(Action::Timeout),
            other => san::find(&self.legal, other)
                .map(|i| Action::Play(Box::new(self.legal[i].clone())))
                .ok_or_else(|| GameError::Parse(other.into())),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Play(m) => m.san.clone(),
            Action::Resign => "resign".into(),
            Action::Timeout => "timeout".into(),
        }
    }

    fn canonical(&self, command: &str) -> Option<String> {
        san::find(&self.legal, command).map(|i| self.legal[i].san.clone())
    }
}
