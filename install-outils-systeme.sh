#!/usr/bin/env bash
#
# install-outils-systeme.sh
#
# Paquets systeme necessaires au projet "systeme universel inspire de l'eau"
# (cahier d'architecture v0.1, 28/09/2026). A lancer UNE FOIS avec :
#
#   sudo ./install-outils-systeme.sh
#
# Aucun paquet n'est installe en dehors de cette liste. Le script est idempotent :
# le relancer ne reinstalle rien.

set -euo pipefail

# --- Emulation et firmware UEFI -------------------------------------------------
# QEMU : banc d'essai du jalon J0/J1 et simulation de peripheriques (porte F5 de la Forge).
#   x86_64 : qemu-system-x86
#   ARM64  : qemu-system-arm
#   RISC-V : qemu-system-riscv64
# OVMF   : firmware UEFI x86-64 (Code/ Vars) pour le demarrage UEFI de Limine.
PKG_EMULATION=(
  qemu-system-x86
  qemu-system-arm
  qemu-system-riscv
  qemu-utils
  ovmf
  qemu-efi-aarch64
)

# --- Chaine de compilation ------------------------------------------------------
# nasm       : code de demarrage et assembleur du chargeur Limine.
# build-essential, cmake, ninja-build, pkg-config, python3, git, curl, wget, unzip.
# gdb-multiarch : debug noyau et pilotes sur les trois architectures.
PKG_BUILD=(
  build-essential
  nasm
  binutils
  gdb-multiarch
  cmake
  ninja-build
  lld
  pkg-config
  python3
  python3-pip
  python3-venv
  git
  curl
  wget
  unzip
  xz-utils
)

# --- Cross-compilation RISC-V ---------------------------------------------------
# Le noyau Rust n'a pas besoin d'un GCC croise, mais un GDB distant si
# l'on veut un debogueur natif riscv64, et un toolchain de reference pour
# verifier la generation de code. Optionnel.
PKG_CROSS=(
  gcc-riscv64-unknown-elf
  gdb-multiarch
)

# --- Cle USB amphibie : partitionnement, FAT32, ecriture sans montage ---------
# gdisk      : tables GPT de la cle (ESP + SYS + DATA).
# dosfstools : mkfs.vfat pour l'ESP.
# mtools     : ecrire dans l'ESP sans demonter la cle (utile sous WSL, ou
#              quand on ne veut pas toucher au montage de la cle hote).
# parted, util-linux : operations de disque.
PKG_USB=(
  gdisk
  dosfstools
  mtools
  parted
  fdisk
  util-linux
  xorriso
)

# --- Secure Boot et signatures --------------------------------------------------
# sbsigntool : signer les .EFI (chargeur, Molcule).
# openssl    : generer la cle de l'appareil qui signe les pilotes forges (porte F7).
# efibootmgr : enroler une cle sur une machine reelle.
# mokutil    : gestion des cles MOK pour les chaines shim.
PKG_SECURITY=(
  sbsigntool
  openssl
  uuid-runtime
  efibootmgr
  mokutil
)

# --- Inventaire du materiel (etape F1 de la Forge : lire PCI, ACPI, USB) --------
PKG_INVENTORY=(
  pciutils
  usbutils
  acpica-tools
  dmidecode
  read-edid
)

# --- Confort de developpement ---------------------------------------------------
PKG_DEV=(
  ripgrep
  jq
  socat
  picocom
)

ALL_PKGS=(
  "${PKG_EMULATION[@]}"
  "${PKG_BUILD[@]}"
  "${PKG_CROSS[@]}"
  "${PKG_USB[@]}"
  "${PKG_SECURITY[@]}"
  "${PKG_INVENTORY[@]}"
  "${PKG_DEV[@]}"
)

# Deduplication en conservant l'ordre.
declare -A seen=()
UNIQUE=()
for p in "${ALL_PKGS[@]}"; do
  if [[ -z "${seen[$p]:-}" ]]; then
    seen[$p]=1
    UNIQUE+=("$p")
  fi
done

echo "==> Paquets a installer (${#UNIQUE[@]}) :"
printf '    %s\n' "${UNIQUE[@]}"
echo

if [[ $EUID -ne 0 ]]; then
  echo "ERREUR : ce script doit etre lance en root : sudo $0" >&2
  exit 1
fi

echo "==> Mise a jour des listes de paquets"
export DEBIAN_FRONTEND=noninteractive
apt-get update

echo "==> Installation"
apt-get install -y --no-install-recommends "${UNIQUE[@]}"

echo
echo "==> Verification"
fail=0
check() {
  if command -v "$1" >/dev/null 2>&1; then
    printf '  OK    %-28s %s\n' "$1" "$(command -v "$1")"
  else
    printf '  ABSENT %-27s\n' "$1"
    fail=1
  fi
}

for b in qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64 \
         nasm objcopy ld.lld gdb-multiarch gdisk mkfs.vfat mformat sbsign sbverify \
         openssl lspci lsusb acpidump socat; do
  check "$b"
done

echo
# Ubuntu 26.04 ne fournit plus que les variantes _4M, et le metapaquet
# sbsigntool expose des binaires renommes (sbsign, sbverify, sbattach).
for f in /usr/share/OVMF/OVMF_CODE_4M.fd /usr/share/OVMF/OVMF_VARS_4M.fd \
         /usr/share/AAVMF/AAVMF_CODE.fd /usr/share/AAVMF/AAVMF_VARS.fd; do
  if [[ -e "$f" ]]; then
    printf '  OK    firmware  %s\n' "$f"
  else
    printf '  ABSENT firmware  %s\n' "$f"
    fail=1
  fi
done

echo
if [[ $fail -eq 0 ]]; then
  echo "RESULTAT : tout est en place."
else
  echo "RESULTAT : des elements manquent (voir ABSENT ci-dessus)."
fi
exit $fail
