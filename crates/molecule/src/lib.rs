//! `molecule` — le noyau (cahier §4).
//!
//! Vague 0, tâches 0.2 et 0.4 : échafaudage. Le contenu réel (allocateur de
//! pages, tâches, Courants, clés) arrive en Vague 1. Ce qui est déjà fixé :
//!
//! - `#![deny(unsafe_code)]` à la racine ; l'`unsafe` ne vit que dans
//!   `fondation` (contrainte OUTILS.md §3.1 : `forbid` empêcherait le point
//!   d'entrée).
//! - Le gestionnaire de panique est conditionné pour que les tests sur l'hôte
//!   passent (contrainte OUTILS.md §3.2, erreur E0152).
//! - Un symbole `molecule_demarrage` exporté : c'est l'entrée que le chargeur
//!   Limine appellera une fois le protocole de démarrage branché (tâche 0.4).

#![no_std]
#![deny(unsafe_code)]

use courants::Schema;

/// Schéma du Courant de diagnostic, premier canal du système.
pub const SCHEMA_DIAGNOSTIC: Schema = Schema::nouveau("diagnostic", 1);

/// Nombre cible d'appels système de la Molécule (§4 du cahier : une vingtaine).
pub const NB_APPELS_SYSTEME_CIBLE: usize = 20;

/// Familles d'appels système prévues par le cahier §4.
///
/// Vague 0 : la liste fait foi ; Vague 1 : chacun devient un appel réel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Famille {
    Memoire,
    Taches,
    Canaux,
    Cles,
    Evenements,
    Budgets,
}

/// Compte des appels prévus par famille (4 + 3 + 4 + 4 + 3 + 2 = 20).
pub const APPELS_PAR_FAMILLE: [(Famille, usize); 6] = [
    (Famille::Memoire, 4),
    (Famille::Taches, 3),
    (Famille::Canaux, 4),
    (Famille::Cles, 4),
    (Famille::Evenements, 3),
    (Famille::Budgets, 2),
];

/// Total des appels système prévus : exactement la vingtaine visée.
pub const fn total_appels_systeme() -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < APPELS_PAR_FAMILLE.len() {
        total += APPELS_PAR_FAMILLE[i].1;
        i += 1;
    }
    total
}

/// Message de diagnostic renvoyé sous forme de code : le chargeur (Limine,
/// tâche 0.4) ou les tests l'utilisent comme preuve de vie du noyau.
pub fn molecule_demarrage() -> u32 {
    0x0000_00E4 // « eau » : signature de démarrage
}

/// Gestionnaire de panique du noyau.
///
/// Conditionné par `cfg(not(test))` : en test, `std` fournit le sien
/// (sinon E0152, cf. OUTILS.md §3.2). En mode test il est donc volontairement
/// absent d'un point de vue appel — on tait le dead_code qui en découle.
#[cfg_attr(not(test), panic_handler)]
#[cfg_attr(test, allow(dead_code))]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    // Vague 0 : halt volontaire. La Vague 1 branchera la journalisation série.
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_appels_systeme_font_exactement_vingt() {
        assert_eq!(total_appels_systeme(), NB_APPELS_SYSTEME_CIBLE);
        assert_eq!(total_appels_systeme(), 20);
    }

    #[test]
    fn la_signature_de_demarrage_est_stable() {
        assert_eq!(molecule_demarrage(), 0xE4);
    }

    #[test]
    fn le_schema_diagnostic_est_stable() {
        assert_eq!(SCHEMA_DIAGNOSTIC.nom, "diagnostic");
        assert!(SCHEMA_DIAGNOSTIC.compatible_avec(&SCHEMA_DIAGNOSTIC));
    }
}
