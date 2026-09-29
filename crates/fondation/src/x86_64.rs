//! Primitives brutes x86-64 : ports d'entrée-sortie.
//!
//! La console série 16550 (tâche 0.5) et les Contenants x86 s'appuieront sur
//! ces accès. ARM64 et RISC-V n'ont pas d'espace de ports : leurs modules
//! viendront avec leurs UART (PL011, NS16550 sur bus MMIO).

/// Écrit un octet sur un port x86-64.
#[inline]
pub fn port_ecriture(port: u16, valeur: u8) {
    // SAFETY : écriture sur un port d'entrée-sortie explicite, appelée
    // uniquement par le noyau, qui est seul propriétaire des périphériques
    // à ce stade. Aucun accès concurrent : les tâches n'existent pas encore.
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") valeur, options(nomem, nostack)) }
}

/// Lit un octet sur un port x86-64.
#[inline]
pub fn port_lecture(port: u16) -> u8 {
    // SAFETY : lecture sur un port explicite, mêmes conditions que
    // [`port_ecriture`] : noyau seul propriétaire, pas de concurrence.
    let valeur: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") valeur, in("dx") port, options(nomem, nostack)) }
    valeur
}

/// Attend quelques microsecondes par port vide (usage x86 historique).
#[inline]
pub fn attendre_port() {
    // SAFETY : écriture sur le port 0x80, convention de délai sans effet
    // de bord observable sur les machines visées.
    unsafe { core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack)) }
}
