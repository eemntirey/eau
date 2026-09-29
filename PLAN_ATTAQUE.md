# Plan d'attaque · système universel inspiré de l'eau

Complément opérationnel du cahier d'architecture v0.1 (28/09/2026) et d'OUTILS.md (29/09/2026).
Principe : **une seule vague active à la fois**, chaque tâche a un critère de sortie vérifiable,
et toute tâche qui n'affecte pas la Molécule, la Charte ou la clé amphibie est reportée.

---

## 0. État des lieux (29/09/2026)

Le plan part de ce qui est déjà acquis, pas de zéro :

| Acquis | Preuve |
|---|---|
| Environnement complet : Rust nightly, 6 cibles, QEMU 10.2.1, OVMF/AAVMF, Verus | OUTILS.md §1–§6 |
| Chaîne UEFI x86-64 de bout en bout : Rust `no_std` → PE/COFF → ESP (mtools) → OVMF → QEMU → console série | OUTILS.md §3 |
| Cinq contraintes techniques documentées (`forbid(unsafe_code)` vs point d'entrée, panics en test, clippy `no_std`, invariant Verus unique, `llvm-size` sur le binaire final) | OUTILS.md §3 |
| Préuves Verus exécutées (3 propriétés vérifiées) | OUTILS.md §6 |
| Verrou ARM64 : aucun `lld` disponible ne produit de PE/COFF AArch64 → **Limine obligatoire** pour le multi-arch | OUTILS.md §3 |

Conséquence : **le premier objectif réel est J0 via Limine** ; la voie maison `uefi-rs`
reste bloquée en ARM64 tant qu'un linker PE/COFF AArch64 n'est pas fourni (risque R1).

---

## 1. Décisions à trancher avant J0

| # | Décision | Recommandation | Bloque |
|---|---|---|---|
| D1 | Licence | Apache-2.0 + notice MPL (conditionne la réutilisation d'OSTD, cf. §16 du cahier) | J2+ |
| D2 | Modèle de la Forge | local d'abord (`systeme.toml` : `modele = "local"`) | J4 |
| D3 | Premier matériel réel | PC x86-64 récent avec IOMMU activable (vérifier DMAR/IR dans le firmware) | J2 |
| D4 | Autonomie par défaut | A1 (proposer), clé de signature humaine via openssl | J5 |
| D5 | Écrire ou réutiliser | réutiliser OSTD / rust-osdev / virtio-drivers si compatibles édition 2024 ; sinon isoler derrière `fondation` | J1–J2 |
| D6 | Ordonnanceur J1 | round-robin à budgets ; EEVDF reporté à J3 | J1 |

Le plan suppose D1=Apache-2.0, D2=local, D5=réutiliser. Chaque décision non tranchée
retarde sa vague.

---

## 2. Les cinq vagues

### VAGUE 0 — Fondations (J0) · ~2 semaines
**Objectif : même image qui affiche un message sur deux architectures via Limine.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 0.1 | Trancher D1–D6 et les consigner | tableau §1 rempli dans ce fichier |
| 0.2 | Dépôt Git : crates `molecule` (noyau), `fondation` (unsafe confiné), `courants` (types de canaux), `contenants-sdk` (SDK pilote) | `cargo nextest run` vert sur l'hôte |
| 0.3 | Intégrer Limine : chargeurs précompilés à la place du chargeur maison | menu Limine affiché sous OVMF |
| 0.4 | Molécule en `staticlib` + liaison ELF Limine (protocole de démarrage, framebuffer au boot) | image ELF 64 bits chargée par Limine |
| 0.5 | Console série 16550 x86-64 ; `#![deny(unsafe_code)]` racine + `allow` dans `fondation` uniquement | « Molécule v0 » sur COM1 sous QEMU |
| 0.6 | CI minimale : clippy cible no_std (cf. OUTILS.md §3.3), `llvm-size` sur l'ELF final, nextest | `cargo make ci` vert |
| 0.7 | ARM64 : QEMU virt, AAVMF → Limine → noyau | même message sur PL011 |
| 0.8 | Jalon J0 atteint : même image, deux architectures | critère §15 du cahier |

**Livrable : dépôt public, README, deux logs de boot sérialisés et datés.**

### VAGUE 1 — Molécule (J1) · ~4 à 6 semaines
**Objectif : mémoire, tâches, Courants, clés ; deux services qui dialoguent.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 1.1 | Allocateur de pages (buddy simplifié, pages 4 KiB) | tests unitaires + preuve Verus |
| 1.2 | Appels mémoire : `mem_creer`, `mem_mapper`, `mem_liberer`, `mem_partager` | tests sous QEMU ; Kani sur les invariants d'ordre |
| 1.3 | Tâches : changement de contexte en assembleur (dans `fondation`) par architecture ; round-robin à budgets (D6) | trois tâches s'exécutent en alternance |
| 1.4 | Courants : file circulaire à quota, schéma versionné vérifié à l'ouverture | émetteur bloqué au quota, repris après lecture (test) |
| 1.5 | Clés (capacités) : `deriver`, `restreindre`, `revoquer` | révoquer une clé tue le canal correspondant |
| 1.6 | Kani sur mémoire et clés (installation : OUTILS.md §6) | zéro résultat indéterminé |
| 1.7 | Deux services qui dialoguent par un Courant (binaires bruts chargés par le noyau, pas d'ELF encore) | échanges visibles sur la console série |
| 1.8 | Benchmarks reproductibles : durée boot, aller-retour Courant | chiffres consignés dans BENCH.md |

**Livrable : Molécule < 15 000 lignes (mesuré avec `llvm-size`), deux services qui dialoguent.**

### VAGUE 2 — Contenants standards (J2) · ~6 à 8 semaines
**Objectif : un vrai PC démarre avec écran, clavier, stockage en lecture seule.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 2.1 | GOP (framebuffer) via Limine | pixel affiché sur QEMU puis vrai PC |
| 2.2 | virtio (blk, net) via crate `virtio-drivers` (D5) | lecture de blocs sous QEMU |
| 2.3 | NVMe | lecture de blocs sur le PC réel |
| 2.4 | xHCI + HID (clavier) | touches affichées à l'écran |
| 2.5 | ACPI de base : RSDP, MADT (cœurs), entrées MMIO | nombre de cœurs correct |
| 2.6 | IOMMU (DMAR) : fenêtres DMA par clé | périphérique confiné sous QEMU (machine q35) |
| 2.7 | Profil matériel : construction du graphe, cache sur clé DATA | second boot mesuré plus rapide |
| 2.8 | Jalon J2 : PC x86-64 réel | critère §15 du cahier |

**Livrable : logs de démarrage datés ; phases 3+4 du boot < 2 s (mesuré).**

### VAGUE 3 — Bassin et clé amphibie (J3) · ~4 semaines
**Objectif : trois PC démarrent avec la même clé.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 3.1 | Magasin immuable adressé par contenu (hash), organisé par architecture | — |
| 3.2 | Mises à jour A/B atomiques, N échecs → retour à A | bascule prouvée sous QEMU (test de coupure) |
| 3.3 | Instantanés de configuration et des pilotes forgés | retour arrière en une opération |
| 3.4 | Partition DATA : profils matériels, pilotes forgés signés, journal en ajout seul | signature vérifiée au démarrage |
| 3.5 | Clé amphibie réelle : GPT (ESP + SYS + DATA) via gdisk/mtools (OUTILS.md §5) | clé USB démarrable x86-64 |
| 3.6 | Jalon J3 : trois PC x86-64, une seule clé | critère §15 du cahier |

**Livrable : une clé physique qui boote trois PC, journal signé des démarrages.**

### VAGUE 4 — Forge v0 puis Charte (J4–J5) · ~8 semaines
**Objectif : un pilote R1 forgé, validé par les portes, chargé puis promu sur vrai matériel.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 4.1 | Étapes F1–F3 : lecture PCI/ACPI/USB, contrat de Contenant, génération de code Rust via SDK | contrat complet sans devinette |
| 4.2 | Porte F4 : analyse statique (pas d'unsafe, plages MMIO dans le contrat, boucles bornées, pas d'allocation en IRQ) — réutiliser la commande clippy de OUTILS.md §3.3 | zéro violation |
| 4.3 | Porte F5 : simulation virtio + fuzzing (`cargo-fuzz`, installé) | zéro échec sur N cas |
| 4.4 | Porte F6 : mode ombre sur matériel réel, IOMMU actif | aucun écart pendant la durée fixée |
| 4.5 | Porte F7 : signature par la clé de l'appareil, journal signé | promotion conforme à la Charte |
| 4.6 | Charte A0–A2 : niveaux d'autonomie appliqués par le noyau (Vague 1 a posé les clés) | chargement refusé sans confirmation humaine en A1 |
| 4.7 | Jalon J4 (banc virtio) puis J5 (vrai matériel) | critères §15 du cahier |

**Livrable : un pilote R1 forgé de bout en bout ; journal de la Forge signé.**

### VAGUE 5 — Ouverture (J6–J7) · sans date
**Objectif : ARM64 et RISC-V réels, mode hôte, robots.**

| # | Tâche | Critère de sortie |
|---|---|---|
| 5.1 | ARM64 réel (carte UEFI ou chargeur dédié) | mêmes services qu'en x86-64 |
| 5.2 | RISC-V : trancher edk2-riscv64 vs chemin Limine (écart signalé dans OUTILS.md §5) | boot natif RISC-V |
| 5.3 | Mode hôte : runtime + pont d'affichage/entrée/réseau vers Linux et Windows | mêmes services en natif et en hôte |
| 5.4 | Pont ROS 2 pour humanoïdes ; classe R3 verrouillée (jamais autonome) | démo contrôlée |
| 5.5 | WebAssembly : apps `wasm32` (cible installée) | même binaire sur tous les appareils |

**Livrable : les trois architectures natif + un mode hôte démontré.**

---

## 3. Risques suivis

| # | Risque | Gravité | Parade | Vague |
|---|---|---|---|---|
| R1 | Pas de linker PE/COFF AArch64 → voie UEFI maison ARM64 bloquée | Moyenne | Limine fournit BOOTAA64.EFI précompilé ; voie maison reportée | 0 |
| R2 | Édition 2024 vs crates anciennes (rust-osdev, OSTD) | Moyenne | vérifier avant adoption ; sinon confiner dans `fondation` | 1–2 |
| R3 | La génération de pilotes échoue une fois sur deux (AutoDriver : 56,4 %) | Élevée | portes F4–F7 indépendantes du modèle ; essais bornés (D2) | 4 |
| R4 | Charge pour une personne seule | Élevée | vagues courtes, réutilisation (D5), QEMU avant matériel, publier tôt | toutes |
| R5 | Secure Boot exige une action humaine | Moyenne | documenter l'enrôlement ; prévoir un mode démonstration | 3 |
| R6 | Firmwares fermés (iPhone, Huawei, Apple Silicon) | Moyenne | mode hôte uniquement ; ne pas promettre le natif partout | 5 |

---

## 4. Cadence hebdomadaire

| Moment | Rituel |
|---|---|
| Lundi | choisir les tâches de la semaine (2 à 4 maximum) |
| Mercredi | lancer les benchmarks, consigner les chiffres dans BENCH.md |
| Dimanche | revue : ce qui marche, ce qui coince, mise à jour de ce plan et du journal d'OUTILS.md |

Règle transversale : chaque tâche finie = un critère de sortie vérifié + un log daté.
Un critère non vérifiable est réécrit jusqu'à ce qu'il le soit.

---

## 5. Les 72 prochaines heures

| Quand | Quoi | Résultat attendu |
|---|---|---|
| H+0 | trancher D1–D6 ; init dépôt Git, crates vides, CI clippy | dépôt vert |
| H+2 | Limine x86-64 sous OVMF ; console série 16550 | « Molécule v0 » sur COM1 |
| H+4 | ARM64 : QEMU virt + AAVMF → Limine → noyau ; PL011 | « Molécule v0 » sur PL011 |
| H+6 | `cargo make ci` complet (clippy + size + nextest) ; README | **J0 atteint, tag `v0.0.1`** |

---

## 6. Tableau de bord des métriques (§11 du cahier)

| Métrique | Objectif de conception | Actuel | Vague |
|---|---|---|---|
| Interface visible après le firmware | < 5 s | non mesuré | 2 |
| Nombre d'appels système | ~20 | 0 (à définir en Vague 1) | 1 |
| Aller-retour d'un Courant sur un cœur | quelques µs | non mesuré | 1 |
| Latence entrée → affichage | < 20 ms | non mesuré | 2 |
| Taille de la Molécule | < 15 000 lignes | 0 (échafaudage) | 1 |

Mise à jour à chaque revue du dimanche. Sans mesure, « rapide » reste un slogan (règle du §11 du cahier).

---

## 7. Références croisées

- Cahier d'architecture v0.1 : §4 Molécule · §5 Courants · §6 Contenants · §7 Forge · §8 Charte · §10 clé USB · §11 performance · §12 sécurité · §14 nouveauté · §15 jalons · §16 décisions
- OUTILS.md : §3 contraintes et chaîne UEFI prouvée · §5 paquets système · §6 Verus et Kani · §7 points de vigilance
