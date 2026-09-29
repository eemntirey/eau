# Outils du projet — système universel inspiré de l'eau

État de l'environnement de développement, mesuré le **29 septembre 2026**.

Source : *Cahier d'architecture · système universel inspiré de l'eau · v0.1*, 28 septembre 2026.
Chaque outil ci-dessous est rattaché à la section du cahier qui le rend nécessaire,
pour ne pas installer ce qui ne sert qu'au confort.

---

## 1. Résumé

L'environnement est complet. Tout a été installé puis vérifié en exécution.

| Domaine | Outils | État |
|---|---|---|
| Chaîne Rust | rustup, nightly, `rust-src`, `llvm-tools`, clippy, rustfmt, rust-analyzer | installé et vérifié |
| Cibles bare-metal | `x86_64-unknown-none`, `aarch64-unknown-none`, `riscv64gc-unknown-none-elf` | installé et compilé |
| Cibles auxiliaires | `wasm32-unknown-unknown`, `x86_64-unknown-uefi`, `aarch64-unknown-uefi` | installé |
| Outils cargo | nextest, watch, expand, wasm-tools, binutils, make, fuzz, embed | installé et vérifié |
| Preuves formelles | Verus 0.2026.09.27 | installé, preuve exécutée |
| Emulation | QEMU 10.2.1 x86-64 / aarch64 / riscv64, OVMF 2025.11, AAVMF 2025.11 | installé et démarré |
| Clé USB / Secure Boot | gdisk, dosfstools, mtools, parted, sbsign, openssl | installé et vérifié |
| *Model checking* | Kani | non installé, volontairement différé (§6) |

Seule réserve : le **lien d'un exécutable UEFI en AArch64** est impossible avec les
`lld` disponibles sur cette machine. Le §3 détaille le constat et la solution. Ce n'est
pas bloquant pour J0, qui passe par Limine.

Tout a été obtenu avec `sudo ./install-outils-systeme.sh`, qui est idempotent et se
termine par `RESULTAT : tout est en place.`

---

## 2. Chaîne Rust

| Élément | Version | Rôle |
|---|---|---|
| rustup | 1.29.1 | gestionnaire de toolchains |
| stable (hôte) | 1.98.1 | outils utilisateur, tests sur l'hôte |
| 1.98.1 (dédié) | 1.98.1 | toolchain **exigée par Verus** |
| nightly | 1.99.0-nightly (14cae6813, 2026-07-08) | `no_std`, `-Z build-std`, attributs instables |
| nightly-2026-07-10 | épinglé | reproductibilité, à privilégier pour compiler la Molécule |

Composants nightly : `rust-src`, `llvm-tools`, `clippy`, `rustfmt`, `rust-analyzer`.

`rust-src` n'est pas superflu : c'est lui qui permet de reconstruire `core` et `alloc`
avec les bons drapeaux via `-Z build-std`, ce dont le noyau a besoin dès qu'il
customise la gestion de mémoire ou de la panique.

### Cibles installées

| Cible | Nécessité | Section |
|---|---|---|
| `x86_64-unknown-none` | Molécule x86-64, PC UEFI | §4 trait `Arch` |
| `aarch64-unknown-none` | Molécule ARM64, cartes et humanoïdes | §4, §2 |
| `riscv64gc-unknown-none-elf` | Molécule RISC-V | §4, §2 |
| `wasm32-unknown-unknown` | apps WebAssembly, un binaire pour tous les appareils | §13 |
| `x86_64-unknown-uefi`, `aarch64-unknown-uefi` | module UEFI maison, alternative à Limine (`uefi-rs`) | §10 |

> Le suffixe `-softfloat` de `aarch64-unknown-none` n'a pas été installé : le noyau
> n'a pas besoin de FPU tant qu'il n'exécute pas de code flottant. À ajouter le jour
> où l'interface adaptative (§13) en fait.

### Reproduire l'installation

```bash
rustup toolchain install nightly --component rust-src llvm-tools clippy rustfmt rust-analyzer
rustup +nightly target add \
    x86_64-unknown-none aarch64-unknown-none riscv64gc-unknown-none-elf \
    wasm32-unknown-unknown x86_64-unknown-uefi aarch64-unknown-uefi
```

---

## 3. Vérifié de bout en bout

Un module `no_std` de démonstration a été compilé pour chaque architecture cible avec
reconstruction de la bibliothèque standard, testé sur l'hôte, et passé au vérificateur
de forme.

| Vérification | Résultat |
|---|---|
| `x86_64-unknown-none`, release, `-Z build-std=core,alloc` | compile |
| `aarch64-unknown-none`, idem | compile |
| `riscv64gc-unknown-none-elf`, idem | compile |
| `wasm32-unknown-unknown` | compile |
| tests unitaires sur l'hôte (`cargo nextest run`) | 1 passé |
| clippy sur cible `no_std` | aucun avertissement |
| désassemblage `llvm-objdump` de la fonction exportée | correct |
| preuve Verus (invariant de boucle bornée) | 3 vérifiés, 0 erreur |
| exécution du binaire produit par Verus | correct |

### Démarrage UEFI réel, x86-64

Chaîne complète reconstruite et exécutée, qui est le cœur du jalon J0 :

```
Rust no_std  ->  x86_64-unknown-uefi  ->  rust-lld  ->  BOOTX64.EFI
             ->  ESP FAT32 (mtools)   ->  OVMF       ->  QEMU
             ->  console serie COM1
```

Sortie réellement obtenue sur le port série, via `-serial file:` :

```
BdsDxe: loading Boot0002 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,...)
BdsDxe: starting Boot0002 "UEFI QEMU HARDDISK QM00001 " from PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,...)
[UEFI] molecule_eau : demarrage UEFI
[UEFI] toolchain Rust no_std -> OVMF -> QEMU : OK
[UEFI] arret du systeme
```

Commandes reproductibles, dans cet ordre :

```bash
# 1. Compiler l'application UEFI
cargo +nightly build --release -Z build-std=core --target x86_64-unknown-uefi

# 2. Lier en PE/COFF : c'est la seule formulation acceptee ici
LLD=$(rustc +nightly --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld
AR=$(rustc +nightly --print target-libdir --target x86_64-unknown-uefi)
$LLD -flavor gnu -m i386pep --subsystem=efi_application -shared -e efi_main \
     -L "$AR" -o BOOTX64.EFI target/x86_64-unknown-uefi/release/libuefi_app.a

# 3. ESP sans montage et sans root
dd if=/dev/zero of=esp.img bs=1M count=64
mformat -i esp.img -F -v ESP::
mmd -i esp.img ::/EFI ::/EFI/BOOT
mcopy -i esp.img BOOTX64.EFI ::/EFI/BOOT/BOOTX64.EFI

# 4. Demarrer, port series capture dans un fichier
cp /usr/share/OVMF/OVMF_VARS_4M.fd VARS.fd
qemu-system-x86_64 -machine q35 -m 512 \
  -drive if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
  -drive if=pflash,format=raw,unit=1,file=VARS.fd \
  -drive format=raw,file=esp.img \
  -device isa-debug-exit,iobase=0x501,iosize=0x01 \
  -serial file:serie.log -display none -no-reboot
```

`isa-debug-exit` est ce qui rend la fin d'exécution observable : l'application écrit
son code de sortie sur le port `0x501` et QEMU se termine. La variable `VARS.fd` doit
être **copiée** à chaque essai ; OVMF y inscrit les entrées de démarrage, et le
réutiliser tel quel finit par mentionné un `Boot0001` fantôme.

### AArch64 : le firmware est bon, le linker ne l'est pas

`AAVMF` 2025.11 démarre correctement sur `qemu-system-aarch64` et balaie les
périphériques :

```
UEFI firmware (version 2025.11-3ubuntu7.2 built at 14:10:51 on Aug 26 2026)
BdsDxe: failed to load Boot0002 "UEFI Non-Block Boot Device" ... Not Found
BdsDxe: No bootable option or device was found.
```

L'échec est normal : l'ESP ne contient que `BOOTX64.EFI`, en code x86-64, que le
firmware ARM ne peut pas exécuter. Le vrai obstacle est ailleurs.

**Aucun linker disponible ne produit de PE/COFF en AArch64.** Le pilote GNU de `lld`
annonce `supported targets: elf` ; le pilote PE n'accepte que les émulations x86.
Ni le `rust-lld` 22.1.8 embarqué dans le toolchain nightly, ni le paquet `lld` 21.1.8
d'Ubuntu, ne connaissent d'émulation AArch64 en mode PE :

| Commande tentée | Résultat |
|---|---|
| `-m aarch64pep` | `unknown emulation` |
| `-m aarch64` | `unknown emulation` |
| `-m aarch64linux` | lié, mais produit un **ELF**, illisible par le firmware |
| `-m aarch64windows`, `arm64windows`, `aarch64efi` | `unknown emulation` |

**Ce n'est pas bloquant pour J0**, et la raison est dans le cahier lui-même : le §10
prescrit Limine, dont les chargeurs `BOOTX64.EFI` et `BOOTAA64.EFI` sont **précompilés
et distribués avec le projet**. En ARM64, on n'a donc pas besoin de produire un `.efi`
nous-mêmes, seulement de compiler la Molécule et de la laisser charger. La voie
maison `uefi-rs` mentionnée en §10 resterait bloquée en ARM64 tant qu'un linker
PE/COFF AArch64 n'est pas fourni.

La crate `uefi` a été essayée comme contournement et n'a pas abouti non plus : la
version 0.35 a changé d'API (plus de module `uefi::text`, plus de `#[uefi::main]`,
elle ré-exporte `entry`), et son build passe par le même linker. Si la voie UEFI
maison est retenue, verrouiller d'abord une version de `uefi` compatible avec
l'API attendue, avant d'investir.
### Cinq contraintes rencontrées, à connaître avant d'écrire le noyau

1. **`#![forbid(unsafe_code)]` interdit le point d'entrée.**
   En édition 2024, `#[no_mangle]` est lui-même un attribut `unsafe`. Un `forbid`
   à la racine du crate empêche donc de déclarer `_start`, et `forbid` ne peut pas
   être relâché par un `allow` local. Le cahier (§4) demande pourtant les deux :
   `forbid` sur le noyau, `unsafe` confiné à un module fondation.
   **Forme qui fonctionne :** `#![deny(unsafe_code)]` à la racine, et
   `#![allow(unsafe_code)]` à l'intérieur du seul module `fondation`.

2. **`cargo test` et `no_std` se marchent dessus.**
   Le `#[panic_handler]` du crate entre en conflit avec celui de `std` (erreur E0152,
   double définition). Il faut le conditionner : `#[cfg(not(test))] #[panic_handler]`.
   Sans cela, aucun test unitaire du code pur de la Molécule n'est possible.

3. **`cargo clippy` seul échoue sur du `no_std`.**
   Message *« unwinding panics are not supported without std »*. Il faut passer par la
   cible : `cargo clippy -Z build-std=core,alloc --target x86_64-unknown-none`.
   C'est aussi l'invariant à réutiliser tel quel comme **porte F4** de la Forge (§7),
   qui exige « zéro violation ».

4. **Verus n'accepte qu'une seule clause `invariant` par boucle.**
   Deux clauses `invariant ...,` successives font échouer l'analyse syntaxique avec
   *« expected curly braces »*. Il faut les combiner : `invariant A && B,`.
   Écrit tel quel, le premier jet d'une propriété de boucle est refusé.

5. **`cargo size` ne fonctionne pas sur une bibliothèque statique.**
   Sur un crate `crate-type = ["staticlib"]`, la commande répond *« Could not
   determine the wanted artifact »*. Or c'est exactement la forme d'une Molécule
   destined à être liée par Limine. Pour tenir l'objectif « moins de 15 000 lignes »
   et l'invariant de taille du §4/§11, mesurer sur le binaire ou l'image finale
   produits après linkage, avec `llvm-size` :

   ```bash
   LLVM=~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin
   $LLVM/llvm-size  molecule/EFI/BOOT/BOOTX64.EFI
   $LLVM/llvm-objdump -d --disassemble-symbols=_start molecule/EFI/BOOT/BOOTX64.EFI
   ```

   Réserve associée : le raccourci `rust-objdump` de rustup cherche ses outils dans le
   toolchain du répertoire courant et non celui du suffixe `+nightly` ; appeler
   directement les binaires `llvm-*` du toolchain nightly est plus fiable.

---

## 4. Outils cargo

Installés dans `~/.cargo/bin`, donc sans `sudo`.

| Outil | Version | Rôle dans le projet | Section |
|---|---|---|---|
| `cargo-nextest` | 0.9.146 | exécuteur de tests, dès que la suite de la Molécule grossit | §4 |
| `cargo-watch` | 8.5.3 | boucle de développement sur la Forge | §7 |
| `cargo-expand` | 1.0.126 | inspection des macros du SDK de pilotes et de `#[courant]` | §5, §6 |
| `wasm-tools` | 1.259.0 | validation et optimisation des apps WebAssembly | §13 |
| `cargo-binutils` | 0.4.0 | `cargo objdump`, `cargo nm`, `cargo size` — taille de la Molécule | §4, §11 |
| `cargo-make` | 0.37.24 | tâches multi-cibles, une commande par architecture | §15 |
| `cargo-fuzz` | 0.13.2 | fuzzing des pilotes forgés — porte F5 | §7 |
| `cargo-embed` + `probe-rs` | 0.32.0 | flash et débogage sur matériel réel via ST-Link | §15, jalon J2 |

### Deux notes sur cette installation

**`cargo-embed` ne s'installe plus par cargo.** Depuis la version 1.0.1, le crate
`cargo-embed` est un simple message d'erreur qui renvoie vers le programme
d'installation de la suite probe-rs. La commande qui fonctionne est :

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```

Elle installe dans `~/.cargo/bin`, sans `sudo`. Elle ajoute aussi une ligne `PATH`
au profil du shell ; c'est superflu ici, `~/.cargo/bin` étant déjà dans le `PATH`.
À noter pour la suite : la commande `cargo embed` a désormais un léger changement
d'API, et `probe-rs` remplace progressivement `cargo-flash`.

**`cargo-miri` est déjà présent** dans `~/.cargo/bin` (installé avec le toolchain).
Il n'était pas demandé, mais il sert directement l'exigence du §4 « mémoire sûre,
`unsafe` confiné » : `cargo miri test` exécute les tests du code pur sous
interpréteur et détecte les comportements indéfinis avant même Kani. À utiliser sur
le code `no_std` de la Molécule, pas sur les pilotes.

### Commande de réinstallation

```bash
cargo install --locked cargo-nextest cargo-watch cargo-expand wasm-tools
cargo +nightly install --locked cargo-binutils cargo-make cargo-fuzz
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```

### Pourquoi `cargo-make` plutôt qu'un script shell

Le cahier prévoit à terme trois cibles noyau plus l'hôte et le mode WebAssembly.
Un `Makefile` ou `justfile` par cibles rend la duplication inutile et donne une
entrée unique du type `cargo make run-x86` qui construit, copie l'image et lance
QEMU. Le choix du fichier est à trancher avec la décision « écrire ou réutiliser »
du §16.

---

## 5. Paquets système

Installés par **`install-outils-systeme.sh`**, idempotent, qui affiche la liste
complète avant d'agir puis vérifie chaque binaire et chaque firmware à la fin.

```bash
cd /home/eemntirey/projet1
sudo ./install-outils-systeme.sh
```

Dernière exécution : `RESULTAT : tout est en place.`

| Groupe | Paquets | Justification |
|---|---|---|
| Émulation | `qemu-system-x86`, `qemu-system-arm`, `qemu-system-riscv`, `qemu-utils`, `ovmf`, `qemu-efi-aarch64` | jalons J0/J1 sous QEMU ; porte F5 (simulation de périphériques) ; OVMF est le firmware UEFI de test. |
| Compilation | `build-essential`, `nasm`, `binutils`, `lld`, `cmake`, `ninja-build`, `pkg-config`, `python3`, `git`, `curl`, `wget`, `unzip`, `xz-utils` | `nasm` pour le code de démarrage et l'assembleur de Limine ; le reste pour construire Limine et les crates à liaison C. |
| Débogage | `gdb-multiarch` | débogage du noyau sur les trois architectures ; un `gdb` x86-64 seul ne lit ni les DWARF ARM64 ni RISC-V. |
| Cross RISC-V | `gcc-riscv64-unknown-elf` | facultatif : le noyau Rust n'en a pas besoin, mais donne un débogueur et une référence de génération de code pour J6. |
| Clé USB | `gdisk`, `dosfstools`, `mtools`, `parted`, `fdisk`, `util-linux`, `xorriso` | clé amphibie : GPT, ESP en FAT32, partitions SYS A/B et DATA. `mtools` écrit dans l'ESP **sans montage**, ce qui compte sous WSL et quand on veut ne pas toucher au montage de la clé hôte. |
| Secure Boot | `sbsigntool`, `openssl`, `uuid-runtime`, `efibootmgr`, `mokutil` | signer les `.EFI` et la Molécule ; `openssl` pour la clé de l'appareil qui signe les pilotes forgés (porte F7) ; `efibootmgr` et `mokutil` pour enrôler une clé sur une machine réelle. |
| Inventaire | `pciutils`, `usbutils`, `acpica-tools`, `dmidecode`, `read-edid` | étape F1 de la Forge : lire PCI, ACPI et USB pour construire le Profil matériel. |
| Confort | `ripgrep`, `jq`, `socat`, `picocom` | `socat` ouvre la console série QEMU sur un port TCP, ce qui permet de la capturer dans un fichier de journal — utile pour mesurer le temps de la phase 2 du §10 ; `picocom` pour la console série physique. |

### Trois noms de paquets qui ont changé

Le script a été écrit puis corrigé après vérification directe dans le dépôt :

| Écrit au départ | Retenu | Raison |
|---|---|---|
| `qemu-system-riscv64` | `qemu-system-riscv` | `qemu-system-riscv64` est un paquet **virtuel** sans version installable. `-hwe` existe aussi, mais tire QEMU 10.2.1+ds-1ubuntu4.3 contre le 10.2.1+ds-1ubuntu3.2 déjà présent : il aurait installé une seconde version de QEMU à côté. |
| `edid-debian` | `read-edid` | `edid-debian` n'existe pas sur cette version. |
| `sbsigntool` | `sbsign` | `sbsigntool` est un **métapaquet** qui ne pose aucun binaire portant ce nom. Les exécutables sont `sbsign`, `sbverify`, `sbattach`, `sbsiglist`, `sbvarsign`. C'est `sbsign` qui signe les `.EFI`. |

Et les firmware OVMF n'existent qu'en variantes `_4M` sur Ubuntu 26.04 :
`OVMF_CODE_4M.fd` et `OVMF_VARS_4M.fd`. Un chemin `OVMF_CODE.fd` sans le suffixe
donne un fichier introuvable alors que tout est installé.

### Versions installées

| Élément | Version |
|---|---|
| QEMU x86-64, aarch64, riscv64 | 10.2.1 (même build sur les trois) |
| OVMF, AAVMF | 2025.11-3ubuntu7.2 |
| `ld.lld` | 21.1.8 |
| gdb | 17.1 |
| cmake | 4.2.3 |

Fichiers de firmware en place :

```
/usr/share/OVMF/OVMF_CODE_4M.fd          # x86-64
/usr/share/OVMF/OVMF_VARS_4M.fd
/usr/share/AAVMF/AAVMF_CODE.fd           # ARM64
/usr/share/AAVMF/AAVMF_VARS.fd
```

> **Pas de firmware UEFI RISC-V empaqueté par Ubuntu.** Deux voies : construire
> `edk2-riscv64` depuis les sources, ou s'appuyer sur le support RISC-V de Limine qui
> embarque son propre chemin de démarrage. À trancher au jalon J6. L'écart est réel :
> le §10 suppose UEFI sur les trois architectures, et le §15 place les autres
> architectures en J6, bien après J0.

---

## 6. Vérification formelle

Le cahier (§4) nomme deux outils : Kani pour le *model checking* ciblé, Verus pour
les preuves sur l'allocateur de pages et le contrôle des clés.

### Verus — installé et vérifié

| Élément | Valeur |
|---|---|
| Version | 0.2026.09.27.3cf1832 (release officielle du 27/09/2026) |
| Emplacement | `~/verus` (hors du dépôt, car ~460 Mo de binaires) |
| Toolchain exigée | `1.98.1-x86_64-unknown-linux-gnu`, installée pour cette raison |
| Vérification | 3 propriétés prouvées, 0 erreur ; binaire compilé et exécuté |

Utilisation :

```bash
~/verus/verus --crate-type=lib src/allocateur.rs     # preuve seule
~/verus/verus --compile        src/allocateur.rs     # prouve puis compile
```

La syntaxe a un piège : **une seule clause `invariant` par boucle**, combinée avec
`&&` (voir §3, point 4). Les bornes de boucle que la Forge doit contrôler en porte F4
sont exactement le genre de propriété que cet outil traite, ce qui en fait le premier
choix pour l'allocateur de pages et le contrôle des clés.

### Kani — non installé, volontairement

```bash
cargo +nightly install --locked kani-verifier
```

Sa compilation mobilise plusieurs gigaoctets et plusieurs dizaines de minutes, sans
apport avant que la Molécule et ses tests existent. Kani est pertinent au moment où
les appels système de la §4 (mémoire, tâches, canaux, clés) deviennent du code
vérifiable, c'est-à-dire au jalon J1. La commande est notée ici pour ne pas la
retrouver de mémoire.

---

## 7. Points de vigilance

- **Cibles sur le bon nightly.** Si la Molécule est épinglée sur `nightly-2026-07-10`
  (déjà installé), les cibles doivent être ajoutées à *ce* nightly aussi, sinon la
  commande échoue. Vérifier avec
  `rustup +nightly-2026-07-10 target list --installed`.
- **Le toolchain de Verus n'est pas celui du noyau.** Verus exige 1.98.1 exactement.
  Le projet tourne sur le nightly. Les deux doivent coexister ; ne pas faire de 1.98.1
  le toolchain par défaut, cela masquerait les usages nightly d'`unsafe`.
- **Édition 2024 et `unsafe`.** Le toolchain est en édition 2024 par défaut et le
  cahier date de septembre 2026. Les crates embarquées qui réutilisent du code plus
  ancien en édition 2021 sont à vérifier avant d'être reprises, en particulier
  `rust-osdev` et OSTD, cités au §15.
- **Le tri du §15 est à respecter.** Ne pas installer d'outillage de pilote
  (`linux-headers`, chaînes de cross du noyau) : le cahier demande d'évaluer la
  réutilisation d'OSTD et des crates `rust-osdev`, qui rendront ces outils
  partiellement inutiles. Installer le noyau Linux en entier serait du travail
  contrary au conseil de tri.
- **Secure Boot demande une action humaine.** Les §8 et §10 sont explicites : les
  options « désactiver » et « enrôler notre clé » supposent l'utilisateur. Aucun
  contournement automatique n'a été mis en place, volontairement.
- **Le `Zone.Identifier` joint n'est pas un outil.** C'est un marqueur de
  téléchargement Windows (`ZoneId=3`) transporté avec le PDF. Sans intérêt, ignoré.

---

## 8. Vérifier l'environnement

```bash
cd /home/eemntirey/projet1

# Script de verification des paquets systeme (idempotent)
sudo ./install-outils-systeme.sh          # doit finir par : RESULTAT : tout est en place.

# Chaine Rust
rustup +nightly target list --installed
cargo nextest --version
~/verus/verus --version
ld.lld --version
```

État au 29/09/2026 : toutes les vérifications passent. Le jalon J0 du §15
(« chaîne Rust multi-cibles, démarrage Limine en UEFI sous QEMU x86-64 et ARM64,
console série ») est **atteignable**. La chaîne UEFI complète a déjà été exécutée
sans Limine (§3) ; il reste à brancher Limine à la place du chargeur maison.

---

## 9. Journal

- **28/09/2026** — lecture du cahier d'architecture v0.1 (19 pages).
- **29/09/2026** — nettoyage d'une installation partielle des cibles bare-metal dans
  le toolchain nightly, leftovers d'une session précédente qui bloquaient
  `rustup target add` par conflit de fichiers. Les six cibles ensuite installées
  normalement, une par une.
- **29/09/2026** — compilation `no_std` validée sur les trois architectures cibles,
  plus wasm32 ; les cinq contraintes de la §3 ont été identifiées en corrigeant le
  code au fur et à mesure, pas lues dans une documentation.
- **29/09/2026** — outils cargo installés (huit outils). `cargo-embed` a échoué via
  `cargo install` puis installé via la suite probe-rs, cf. §4.
- **29/09/2026** — Verus installé dans `~/verus`, toolchain 1.98.1 ajoutée, preuve de
  boucle bornée vérifiée et binaire exécuté.
- **29/09/2026** — `install-outils-systeme.sh` exécuté. Trois noms de paquets se sont
  révélés inexacts à l'usage (`qemu-system-riscv64` virtuel, `edid-debian` absent,
  `sbsigntool` métapaquet sans binaire homonyme) ; corrigés puis vérifiés, script
  idempotent revu pour ne plus les rejouer.
- **29/09/2026** — **démarrage UEFI x86-64 de bout en bout réussi** : Rust `no_std`
  lié en PE/COFF, ESP FAT32 bâti avec `mtools` sans montage ni root, boot sous OVMF,
  sortie sur console série, arrêt piloté par `isa-debug-exit`. C'est la démonstration
  que l'environnement tient la promesse du jalon J0.
- **29/09/2026** — ARM64 : AAVMF démarre correctement, mais aucun `lld` disponible ne
  produit de PE/COFF AArch64. Contournement non nécessaire, Limine fournissant
  `BOOTAA64.EFI` précompilé. Constat détaillé en §3.
- **29/09/2026** — `lld` ajouté au script pour reproductibilité, même s'il ne résout
  pas le point ARM64.
