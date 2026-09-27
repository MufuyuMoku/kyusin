//! Provably fair: seed gabungan dengan commit-reveal (SPEC §2.3, §5.4).
//!
//! 1. Setiap peserta membuat seed 32 byte dan mengirim `SHA-256(seed)`.
//!    Host mengumumkan semua komitmen, termasuk miliknya.
//! 2. Setelah komitmen lengkap, peserta non-host membuka seed ke host.
//! 3. Seed ronde = `SHA-256(seed_host ‖ seed lain diurutkan menurut id)`.
//! 4. Setelah ronde, semua seed dibuka dan siapa pun bisa memverifikasi.
//!
//! Aturan kegagalan: gagal di tahap komitmen = peserta dikeluarkan dan ronde
//! berjalan; gagal di tahap pembukaan = ronde dibatalkan untuk semua (D-013).
//! Modul ini murni; batas waktu dan jaringan ada di `kyusin-net` (M9).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::Seed;
use crate::hash::{hex, sha256, unhex32};

pub type ParticipantId = String;

pub fn commitment(seed: &Seed) -> [u8; 32] {
    sha256(seed)
}

/// Seed ronde dari seed host dan seed peserta lain (urut menurut id).
pub fn combine(host_seed: &Seed, others: &BTreeMap<ParticipantId, Seed>) -> Seed {
    let mut bytes = host_seed.to_vec();
    for seed in others.values() {
        bytes.extend_from_slice(seed);
    }
    sha256(&bytes)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Menunggu komitmen.
    Committing,
    /// Komitmen lengkap; menunggu seed dibuka.
    Revealing,
    /// Seed ronde siap.
    Ready,
    /// Ronde dibatalkan karena ada yang gagal membuka seed.
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FairError {
    WrongPhase,
    UnknownParticipant(ParticipantId),
    Duplicate(ParticipantId),
    /// Seed yang dibuka tidak cocok dengan komitmennya.
    Mismatch(ParticipantId),
    BadHex(String),
    /// Seed ronde yang dicatat tidak sama dengan hasil hitung ulang.
    RoundSeedMismatch,
    MissingHost,
}

impl std::fmt::Display for FairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FairError {}

/// Satu ronde commit-reveal di sisi host.
#[derive(Debug, Clone)]
pub struct FairRound {
    host: ParticipantId,
    host_seed: Seed,
    /// Peserta non-host yang diharapkan ikut.
    expected: Vec<ParticipantId>,
    commitments: BTreeMap<ParticipantId, [u8; 32]>,
    seeds: BTreeMap<ParticipantId, Seed>,
    excluded: Vec<ParticipantId>,
    failed: Vec<ParticipantId>,
    phase: Phase,
}

impl FairRound {
    pub fn new(
        host: impl Into<ParticipantId>,
        host_seed: Seed,
        expected: Vec<ParticipantId>,
    ) -> Self {
        let host = host.into();
        let mut commitments = BTreeMap::new();
        commitments.insert(host.clone(), commitment(&host_seed));
        FairRound {
            host,
            host_seed,
            expected,
            commitments,
            seeds: BTreeMap::new(),
            excluded: Vec::new(),
            failed: Vec::new(),
            phase: Phase::Committing,
        }
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// Komitmen yang diumumkan (termasuk milik host).
    pub fn commitments(&self) -> &BTreeMap<ParticipantId, [u8; 32]> {
        &self.commitments
    }

    pub fn excluded(&self) -> &[ParticipantId] {
        &self.excluded
    }

    /// Peserta yang dinyatakan gagal membuka seed (ronde dibatalkan).
    pub fn failed(&self) -> &[ParticipantId] {
        &self.failed
    }

    pub fn commit(&mut self, who: &str, hash: [u8; 32]) -> Result<(), FairError> {
        if self.phase != Phase::Committing {
            return Err(FairError::WrongPhase);
        }
        if !self.expected.iter().any(|p| p == who) {
            return Err(FairError::UnknownParticipant(who.into()));
        }
        if self.commitments.contains_key(who) {
            return Err(FairError::Duplicate(who.into()));
        }
        self.commitments.insert(who.into(), hash);
        Ok(())
    }

    /// Batas waktu komitmen lewat (atau semua sudah masuk). Yang belum
    /// berkomitmen dikeluarkan dari ronde; ronde tetap berjalan.
    pub fn close_commits(&mut self) -> Result<(), FairError> {
        if self.phase != Phase::Committing {
            return Err(FairError::WrongPhase);
        }
        self.excluded = self
            .expected
            .iter()
            .filter(|p| !self.commitments.contains_key(*p))
            .cloned()
            .collect();
        self.phase = Phase::Revealing;
        Ok(())
    }

    pub fn reveal(&mut self, who: &str, seed: Seed) -> Result<(), FairError> {
        if self.phase != Phase::Revealing {
            return Err(FairError::WrongPhase);
        }
        let Some(expected) = self.commitments.get(who) else {
            return Err(FairError::UnknownParticipant(who.into()));
        };
        if who == self.host {
            return Err(FairError::UnknownParticipant(who.into()));
        }
        if commitment(&seed) != *expected {
            return Err(FairError::Mismatch(who.into()));
        }
        self.seeds.insert(who.into(), seed);
        Ok(())
    }

    /// Batas waktu pembukaan lewat (atau semua sudah masuk). Bila ada yang
    /// sudah berkomitmen tetapi tidak membuka seed, ronde dibatalkan untuk
    /// semua pemain dan nama peserta itu dicatat.
    pub fn close_reveals(&mut self) -> Result<&Phase, FairError> {
        if self.phase != Phase::Revealing {
            return Err(FairError::WrongPhase);
        }
        self.failed = self
            .commitments
            .keys()
            .filter(|p| **p != self.host && !self.seeds.contains_key(*p))
            .cloned()
            .collect();
        self.phase = if self.failed.is_empty() {
            Phase::Ready
        } else {
            Phase::Cancelled
        };
        Ok(&self.phase)
    }

    /// Seed ronde; hanya ada saat `Ready`.
    pub fn round_seed(&self) -> Option<Seed> {
        (self.phase == Phase::Ready).then(|| combine(&self.host_seed, &self.seeds))
    }

    /// Catatan lengkap untuk dibuka setelah ronde (termasuk seed host).
    pub fn record(&self) -> Option<FairRecord> {
        let round = self.round_seed()?;
        let mut seeds: BTreeMap<ParticipantId, String> = self
            .seeds
            .iter()
            .map(|(k, v)| (k.clone(), hex(v)))
            .collect();
        seeds.insert(self.host.clone(), hex(&self.host_seed));
        Some(FairRecord {
            host: self.host.clone(),
            commitments: self
                .commitments
                .iter()
                .map(|(k, v)| (k.clone(), hex(v)))
                .collect(),
            seeds,
            excluded: self.excluded.clone(),
            round_seed: hex(&round),
        })
    }
}

/// Catatan provably fair yang disimpan di replay dan dibuka ke semua peserta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FairRecord {
    pub host: ParticipantId,
    /// Komitmen per peserta (hex SHA-256), diumumkan sebelum ronde.
    pub commitments: BTreeMap<ParticipantId, String>,
    /// Seed per peserta (hex), dibuka setelah ronde.
    pub seeds: BTreeMap<ParticipantId, String>,
    /// Peserta yang dikeluarkan karena tidak berkomitmen.
    #[serde(default)]
    pub excluded: Vec<ParticipantId>,
    pub round_seed: String,
}

impl FairRecord {
    /// Memeriksa semua komitmen dan menghitung ulang seed ronde.
    pub fn verify(&self) -> Result<Seed, FairError> {
        let host_hex = self.seeds.get(&self.host).ok_or(FairError::MissingHost)?;
        let host_seed = unhex32(host_hex).map_err(FairError::BadHex)?;
        let mut others = BTreeMap::new();
        for (who, seed_hex) in &self.seeds {
            let seed = unhex32(seed_hex).map_err(FairError::BadHex)?;
            let committed = self
                .commitments
                .get(who)
                .ok_or_else(|| FairError::UnknownParticipant(who.clone()))?;
            if hex(&commitment(&seed)) != *committed {
                return Err(FairError::Mismatch(who.clone()));
            }
            if *who != self.host {
                others.insert(who.clone(), seed);
            }
        }
        // Setiap komitmen harus punya seed yang dibuka.
        if let Some(missing) = self
            .commitments
            .keys()
            .find(|k| !self.seeds.contains_key(*k))
        {
            return Err(FairError::UnknownParticipant(missing.clone()));
        }
        let round = combine(&host_seed, &others);
        if hex(&round) != self.round_seed {
            return Err(FairError::RoundSeedMismatch);
        }
        Ok(round)
    }

    pub fn round_seed_bytes(&self) -> Result<Seed, FairError> {
        unhex32(&self.round_seed).map_err(FairError::BadHex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(b: u8) -> Seed {
        [b; 32]
    }

    fn round() -> FairRound {
        FairRound::new("host", s(1), vec!["p1".into(), "p2".into()])
    }

    #[test]
    fn happy_path_and_verify() {
        let mut r = round();
        r.commit("p1", commitment(&s(2))).unwrap();
        r.commit("p2", commitment(&s(3))).unwrap();
        r.close_commits().unwrap();
        r.reveal("p2", s(3)).unwrap();
        r.reveal("p1", s(2)).unwrap();
        assert_eq!(r.close_reveals().unwrap(), &Phase::Ready);
        let rec = r.record().unwrap();
        assert_eq!(rec.verify().unwrap(), r.round_seed().unwrap());
        // Urutan: host lalu peserta lain menurut id.
        let mut bytes = s(1).to_vec();
        bytes.extend_from_slice(&s(2));
        bytes.extend_from_slice(&s(3));
        assert_eq!(r.round_seed().unwrap(), sha256(&bytes));
    }

    #[test]
    fn host_commitment_is_announced_before_reveals() {
        let r = round();
        assert_eq!(r.commitments().get("host"), Some(&commitment(&s(1))));
        assert_eq!(r.round_seed(), None);
        assert_eq!(r.record(), None);
    }

    #[test]
    fn commit_failure_excludes_and_round_continues() {
        let mut r = round();
        r.commit("p1", commitment(&s(2))).unwrap();
        r.close_commits().unwrap();
        assert_eq!(r.excluded(), ["p2".to_string()]);
        r.reveal("p1", s(2)).unwrap();
        assert_eq!(r.close_reveals().unwrap(), &Phase::Ready);
        assert_eq!(r.record().unwrap().excluded, vec!["p2".to_string()]);
        // Peserta yang dikeluarkan tidak bisa membuka seed.
        assert!(matches!(r.reveal("p2", s(3)), Err(FairError::WrongPhase)));
    }

    #[test]
    fn reveal_failure_cancels_round_for_everyone() {
        let mut r = round();
        r.commit("p1", commitment(&s(2))).unwrap();
        r.commit("p2", commitment(&s(3))).unwrap();
        r.close_commits().unwrap();
        r.reveal("p1", s(2)).unwrap();
        assert_eq!(r.close_reveals().unwrap(), &Phase::Cancelled);
        assert_eq!(r.failed(), ["p2".to_string()]);
        assert_eq!(r.round_seed(), None);
    }

    #[test]
    fn wrong_seed_is_rejected() {
        let mut r = round();
        r.commit("p1", commitment(&s(2))).unwrap();
        r.close_commits().unwrap();
        assert_eq!(r.reveal("p1", s(9)), Err(FairError::Mismatch("p1".into())));
    }

    #[test]
    fn unknown_and_duplicate_commits_rejected() {
        let mut r = round();
        assert!(matches!(
            r.commit("x", [0; 32]),
            Err(FairError::UnknownParticipant(_))
        ));
        r.commit("p1", [0; 32]).unwrap();
        assert!(matches!(
            r.commit("p1", [0; 32]),
            Err(FairError::Duplicate(_))
        ));
    }

    #[test]
    fn tampered_record_fails_verify() {
        let mut r = FairRound::new("host", s(1), vec!["p1".into()]);
        r.commit("p1", commitment(&s(2))).unwrap();
        r.close_commits().unwrap();
        r.reveal("p1", s(2)).unwrap();
        r.close_reveals().unwrap();
        let rec = r.record().unwrap();

        let mut bad_seed = rec.clone();
        bad_seed.seeds.insert("host".into(), hex(&s(7)));
        assert_eq!(bad_seed.verify(), Err(FairError::Mismatch("host".into())));

        let mut bad_round = rec.clone();
        bad_round.round_seed = hex(&s(0));
        assert_eq!(bad_round.verify(), Err(FairError::RoundSeedMismatch));

        let mut hidden = rec;
        hidden.seeds.remove("p1");
        assert!(hidden.verify().is_err());
    }
}
