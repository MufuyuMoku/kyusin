//! Halaman `man <id>` yang dibangkitkan dari manifest dan tutorial, dalam dua
//! bahasa (SPEC §5.2, §7.6, D-027).

use crate::i18n::{Lang, Localized, tr, trf};
use crate::manifest::{Kind, Manifest, Opponent};
use crate::registry::Cartridge;
use crate::tutorial::{Tutorial, TutorialError};

/// Baris RTP untuk layar info; `None` untuk game non-casino.
pub fn rtp_line(m: &Manifest) -> Option<Localized> {
    if !m.category.is_casino() {
        return None;
    }
    Some(Localized::build(|lang| match m.rtp {
        Some(rtp) => trf(
            lang,
            "rtp.house",
            &[
                ("rtp", &format!("{rtp:.2}")),
                ("edge", &format!("{:.2}", 100.0 - rtp)),
            ],
        ),
        None => tr(lang, "rtp.pvp"),
    }))
}

fn yes_no(lang: Lang, b: bool) -> String {
    tr(lang, if b { "yes" } else { "no" })
}

pub fn man_page(cartridge: &Cartridge) -> Result<Localized, TutorialError> {
    let t = Tutorial::from_toml(cartridge.tutorial_src)?;
    Ok(Localized::build(|lang| {
        render(&cartridge.manifest, &t, lang)
    }))
}

fn render(m: &Manifest, t: &Tutorial, lang: Lang) -> String {
    let players = if m.min_players == m.max_players {
        m.min_players.to_string()
    } else {
        format!("{}–{}", m.min_players, m.max_players)
    };
    let opponent = tr(
        lang,
        match m.opponent {
            Opponent::Bandar => "opponent.bandar",
            Opponent::Bot => "opponent.bot",
            Opponent::TidakAda => "opponent.tidak-ada",
        },
    );
    let kind = tr(
        lang,
        match m.kind {
            Kind::Giliran => "kind.giliran",
            Kind::RealTime => "kind.real-time",
        },
    );
    let h = |key: &str| tr(lang, key);

    let mut out = String::new();
    out.push_str(&format!(
        "{}\n    {} ({})\n\n",
        h("man.name"),
        m.name.get(lang),
        m.id
    ));
    out.push_str(&format!(
        "{}\n    {}: {}\n    {}: {kind} · {}: {players} · {}: {opponent}\n    {}: {} · {}: {} · {}: {}\n",
        h("man.summary"),
        h("man.category"),
        m.category.label(lang),
        h("man.kind"),
        h("man.players"),
        h("man.opponent"),
        h("man.rating"),
        yes_no(lang, m.competitive),
        h("man.lan"),
        yes_no(lang, m.lan),
        h("man.agent"),
        yes_no(lang, m.agent),
    ));
    if let Some(rtp) = rtp_line(m) {
        out.push_str(&format!("    {}\n", rtp.get(lang)));
    }
    out.push_str(&format!(
        "\n{}\n{}\n",
        h("man.rules"),
        indent(t.man.aturan.get(lang))
    ));
    out.push_str(&format!(
        "\n{}\n{}\n",
        h("man.controls"),
        indent(t.man.kontrol.get(lang))
    ));
    out.push_str(&format!("\n{}\n", h("man.commands")));
    let width = m.commands.iter().map(|c| c.usage.len()).max().unwrap_or(0);
    for c in &m.commands {
        out.push_str(&format!(
            "    {:<width$}  {}\n",
            c.usage,
            c.summary.get(lang)
        ));
    }
    out
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
