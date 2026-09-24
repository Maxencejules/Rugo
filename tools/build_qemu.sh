#!/usr/bin/env bash
# Build a pinned QEMU (x86_64 system emulator only) into a prefix.
#
# The SMP and native-NVMe lanes boot `-cpu qemu64,+x2apic` under TCG. QEMU
# before 9.0 cannot emulate x2APIC under TCG (CPUID_EXT_X2APIC joined
# TCG_EXT_FEATURES in v9.0.0) and silently drops the flag, so those lanes
# fail on distro QEMU 8.2 (Ubuntu 24.04). CI and the Docker image use this
# build instead.
#
# Build deps (Debian/Ubuntu): ninja-build pkg-config libglib2.0-dev
# libpixman-1-dev libslirp-dev flex bison python3-venv. libslirp backs
# `-netdev user`, which QEMU no longer bundles.
#
# Usage: bash tools/build_qemu.sh <install-prefix>
set -euo pipefail

QEMU_VERSION="11.0.4"
# sha256 of qemu-11.0.4.tar.xz; the tarball's GPG signature verified against
# the QEMU stable release key CEACC9E15534EBABB82D3FA03353C9CEF108B584.
QEMU_SHA256="b099a10abe4a98ac1a53a4f52cb2367651ad9fcd1c0531beb14921f1524ca9ae"

PREFIX="${1:?usage: bash tools/build_qemu.sh <install-prefix>}"
QEMU_BIN="$PREFIX/bin/qemu-system-x86_64"

if [ -x "$QEMU_BIN" ] && "$QEMU_BIN" --version | grep -q "version $QEMU_VERSION"; then
    echo "build-qemu: $QEMU_VERSION already installed at $PREFIX"
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

curl -fsSL "https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz" -o "$WORK/qemu.tar.xz"
echo "${QEMU_SHA256}  $WORK/qemu.tar.xz" | sha256sum -c -
tar -C "$WORK" -xJf "$WORK/qemu.tar.xz"

cd "$WORK/qemu-${QEMU_VERSION}"
./configure \
    --prefix="$PREFIX" \
    --target-list=x86_64-softmmu \
    --enable-slirp \
    --disable-docs \
    --disable-werror \
    --disable-gtk \
    --disable-sdl \
    --disable-opengl
make -j"$(nproc)"
make install

"$QEMU_BIN" --version | head -1
