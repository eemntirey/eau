# eau · système universel inspiré de l'eau

Dépôt du projet décrit par le *Cahier d'architecture · système universel inspiré de
l'eau · v0.1* (28/09/2026). Le plan d'attaque opérationnel vit dans `PLAN_ATTAQUE.md`,
l'état de l'environnement dans `OUTILS.md`.

## Structure (Vague 0, tâche 0.2)

| Crate | Rôle | Section du cahier |
|---|---|---|
| `crates/fondation` | Le seul endroit où l'`unsafe` est toléré (registres, ports ; plus tard tables de pages, assembleur) | §4 |
| `crates/courants` | Canaux typés à quota (écluses), schémas versionnés vérifiés à l'ouverture | §5 |
| `crates/contenants-sdk` | Contrat des pilotes isolés : classes de risque R0–R3, plages MMIO liées au contrat | §6 |
| `crates/molecule` | Le noyau : vingt appels système, rien d'autre | §4 |

Règles posées dès maintenant (contraintes OUTILS.md §3) :

- `#![deny(unsafe_code)]` à la racine de chaque crate ; `allow(unsafe_code)` local
  uniquement dans `fondation`.
- `#[panic_handler]` conditionné par `#[cfg_attr(not(test), …)]` pour que les tests
  sur l'hôte passent (sinon erreur E0152, double définition avec `std`).
- Toolchain épinglée par `rust-toolchain.toml` : `nightly-2026-07-10`.

## Vérifier

```bash
cargo nextest run   # tests sur l'hôte
cargo clippy --workspace --all-targets
cargo build -Z build-std=core,alloc --target x86_64-unknown-none   # Molécule no_std
```

Prochaine étape : tâche 0.3, intégrer Limine (`BOOTX64.EFI` / `BOOTAA64.EFI`
précompilés) à la place du chargeur UEFI maison — voir PLAN_ATTAQUE.md Vague 0.
