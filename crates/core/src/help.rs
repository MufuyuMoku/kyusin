//! Halaman `man <id>` yang dibangkitkan dari manifest dan tutorial
//! (SPEC §5.2, §7.6).

use crate::manifest::{Kind, Manifest, Opponent};
use crate::registry::Cartridge;
use crate::tutorial::{Tutorial, TutorialError};

/// Baris RTP untuk layar info; `None` untuk game non-casino.
pub fn rtp_line(m: &Manifest) -> Option<String> {
    if !m.category.is_casino() {
        return None;
    }
    Some(match m.rtp {
        Some(rtp) => format!("RTP {rtp:.2}% · house edge {:.2}%", 100.0 - rtp),
        None => "antar-pemain, tanpa house edge".into(),
    })
}

fn yes_no(b: bool) -> &'static str {
    if b { "ya" } else { "tidak" }
}

pub fn man_page(cartridge: &Cartridge) -> Result<String, TutorialError> {
    let m = &cartridge.manifest;
    let t = Tutorial::from_toml(cartridge.tutorial_src)?;
    let players = if m.min_players == m.max_players {
        m.min_players.to_string()
    } else {
        format!("{}–{}", m.min_players, m.max_players)
    };
    let opponent = match m.opponent {
        Opponent::Bandar => "bandar",
        Opponent::Bot => "bot",
        Opponent::TidakAda => "tidak ada (solo)",
    };
    let kind = match m.kind {
        Kind::Giliran => "giliran",
        Kind::RealTime => "real-time",
    };

    let mut out = String::new();
    out.push_str(&format!("NAMA\n    {} ({})\n\n", m.name, m.id));
    out.push_str(&format!(
        "RINGKAS\n    Kategori: {}\n    Jenis: {kind} · Pemain: {players} · Lawan: {opponent}\n    Rating lokal: {} · LAN: {} · Agen: {}\n",
        m.category.label(),
        yes_no(m.competitive),
        yes_no(m.lan),
        yes_no(m.agent),
    ));
    if let Some(rtp) = rtp_line(m) {
        out.push_str(&format!("    {rtp}\n"));
    }
    out.push_str(&format!("\nATURAN\n{}\n", indent(&t.man.aturan)));
    out.push_str(&format!("\nKONTROL\n{}\n", indent(&t.man.kontrol)));
    out.push_str("\nPERINTAH\n");
    let width = m.commands.iter().map(|c| c.usage.len()).max().unwrap_or(0);
    for c in &m.commands {
        out.push_str(&format!("    {:<width$}  {}\n", c.usage, c.summary));
    }
    Ok(out)
}

fn indent(text: &str) -> String {
    text.trim()
        .lines()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                format!("    {}", l.trim_end())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
