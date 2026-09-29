//! `fondation` — le seul endroit du projet où l'`unsafe` est toléré.
//!
//! Règle du cahier §4 : l'`unsafe` est confiné à un module fondation (registres,
//! tables de pages, assembleur) ; tout le reste du projet est en
//! `#![deny(unsafe_code)]`. Chaque item `unsafe` porte un commentaire
//! `// SAFETY:` qui dit pourquoi la condition de validité tient.
//!
//! Règle multi-architecture (§4) : chaque architecture a son module, derrière
//! `#[cfg(target_arch)]` — jamais de code générique qui suppose x86.

#![no_std]
// Cette crate est l'exception assumée du cahier §4 : l'`unsafe` y est toléré
// par conception (pas d'attribut deny ici — il serait annulé par les usages
// documentés ci-dessous). Toutes les autres crates du workspace sont en
// `#![deny(unsafe_code)]`.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::{attendre_port, port_ecriture, port_lecture};

#[cfg(test)]
mod tests {
    #[test]
    fn la_crate_se_compile_spartiate() {
        // Vague 0 : la fondation ne promet encore rien de testable sur l'hôte
        // (les accès réels se font sous QEMU, tâche 0.5). Ce test force
        // l'édition et l'édition seule.
        assert!(true);
    }
}
