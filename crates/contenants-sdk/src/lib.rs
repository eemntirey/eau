//! `contenants-sdk` — le contrat d'un Contenant (pilote isolé, cahier §6).
//!
//! Deux propriétés du cahier sont encodées ici dès la Vague 0 :
//!
//! - **Classes de risque** R0–R3 et leur règle pour la Forge (§6) :
//!   R0 autonome dès A2, R1 dès A3, R2 seulement à A4 avec validation
//!   humaine initiale, R3 jamais autonome.
//! - **Plages MMIO liées au contrat** : « l'accès aux registres passe par
//!   une clé bornée au contrat : hors plage = erreur, pas de plantage ».
//!   [`Lien::mmio`] refuse toute adresse hors des plages déclarées.
//!
//! Vague 0 : les types et les gardes-fous purs. Le trait complet avec
//! interruptions et DMA arrive avec les premiers pilotes (Vague 2).

#![no_std]
#![deny(unsafe_code)]

/// Classe de risque d'un périphérique (cahier §6).
///
/// Détermine jusqu'où la Forge peut aller seule, selon la Charte (§8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risque {
    /// Lecture d'informations, capteurs.
    R0,
    /// Non critique : clavier, souris, audio, webcam.
    R1,
    /// Données ou disponibilité : stockage, réseau, GPU.
    R2,
    /// Critique ou irréversible : firmware, alimentation, actionneurs.
    R3,
}

impl Risque {
    /// Niveau d'autonomie minimal de la Charte (§8) pour que la Forge
    /// promeuve seule un pilote de cette classe.
    ///
    /// `None` = jamais autonome (R3 : humain et signature, toujours).
    pub fn autonomie_requise(self) -> Option<u8> {
        match self {
            Risque::R0 => Some(2),
            Risque::R1 => Some(3),
            Risque::R2 => Some(4),
            Risque::R3 => None,
        }
    }
}

/// Une plage d'adresses MMIO, bornée et alignée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plage {
    base: u64,
    taille: u64,
}

impl Plage {
    pub const fn nouvelle(base: u64, taille: u64) -> Self {
        Self { base, taille }
    }

    pub const fn base(&self) -> u64 {
        self.base
    }

    pub const fn taille(&self) -> u64 {
        self.taille
    }

    pub const fn contient(&self, adresse: u64) -> bool {
        adresse >= self.base && adresse < self.base + self.taille
    }
}

/// Erreurs du contrat de Contenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Erreur {
    /// Accès hors des plages du contrat : refusé, pas de plantage (§6).
    HorsContrat,
    /// Périphérique non reconnu lors de la sondes.
    NonTrouve,
}

/// Le lien d'un Contenant vers son matériel : des plages, rien d'autre.
///
/// C'est la « clé » du cahier : le pilote ne voit jamais une adresse brute
/// sans passer par ici, et le refus est une erreur, pas une exception.
#[derive(Debug, Clone)]
pub struct Lien {
    plages: [Plage; 4],
    nombre: usize,
}

impl Lien {
    /// Construit un lien avec au plus quatre plages (suffit pour les
    /// standards visés en Vague 2 : NVMe, xHCI, AHCI ont chacune 1–2 BAR).
    pub fn nouveau(plages: &[Plage]) -> Self {
        let mut stockage = [Plage::nouvelle(0, 0); 4];
        let nombre = plages.len().min(4);
        stockage[..nombre].copy_from_slice(&plages[..nombre]);
        Self { plages: stockage, nombre }
    }

    /// Vérifie qu'une adresse est dans le contrat, sans y toucher.
    pub fn autorisee(&self, adresse: u64) -> bool {
        self.plages[..self.nombre].iter().any(|p| p.contient(adresse))
    }

    /// Retourne l'adresse si elle est dans le contrat, sinon l'erreur
    /// `HorsContrat` — jamais d'accès silencieux hors périmètre.
    pub fn mmio(&self, adresse: u64) -> Result<u64, Erreur> {
        if self.autorisee(adresse) {
            Ok(adresse)
        } else {
            Err(Erreur::HorsContrat)
        }
    }

    pub fn plages(&self) -> &[Plage] {
        &self.plages[..self.nombre]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r3_n_est_jamais_autonome() {
        assert_eq!(Risque::R0.autonomie_requise(), Some(2));
        assert_eq!(Risque::R1.autonomie_requise(), Some(3));
        assert_eq!(Risque::R2.autonomie_requise(), Some(4));
        assert_eq!(Risque::R3.autonomie_requise(), None);
    }

    #[test]
    fn mmio_hors_contrat_est_une_erreur_pas_un_plantage() {
        // NVMe : BAR0 4 KiB à 0xFEC00000.
        let lien = Lien::nouveau(&[Plage::nouvelle(0xFEC0_0000, 0x1000)]);
        assert!(lien.autorisee(0xFEC0_0000));
        assert!(lien.autorisee(0xFEC0_0FFF));
        assert_eq!(lien.mmio(0xFEC0_1000), Err(Erreur::HorsContrat));
        assert_eq!(lien.mmio(0), Err(Erreur::HorsContrat));
    }

    #[test]
    fn plusieurs_plages_sont_couvertes() {
        let lien = Lien::nouveau(&[
            Plage::nouvelle(0x1000, 0x100),
            Plage::nouvelle(0x2000, 0x40),
        ]);
        assert!(lien.autorisee(0x10FF));
        assert!(lien.autorisee(0x203F));
        assert!(!lien.autorisee(0x1100));
        assert_eq!(lien.plages().len(), 2);
    }
}
