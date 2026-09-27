#!/usr/bin/env sh
# Menjalankan pemeriksaan yang sama dengan CI (.github/workflows/ci.yml).
# Pakai: scripts/check.sh   (dari mana saja di dalam repo)
set -eu

cd "$(dirname "$0")/.."

step() {
	printf '\n==> %s\n' "$1"
}

if [ ! -d ui/node_modules ]; then
	step "npm ci (ui)"
	(cd ui && npm ci)
fi

step "svelte-check (ui)"
(cd ui && npm run check)

step "tes antarmuka (ui)"
(cd ui && npm test)

# Konteks Tauri menanam antarmuka yang sudah dibangun.
step "build antarmuka (ui)"
(cd ui && npm run build)

step "lisensi npm (bundle produksi)"
(cd ui && npm run licenses)

step "cargo fmt --check"
cargo fmt --all -- --check

step "cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

step "cargo test (termasuk tes tutorial)"
cargo test --workspace

if command -v cargo-deny >/dev/null 2>&1; then
	step "cargo deny check licenses"
	cargo deny check licenses
else
	printf '\n(cargo-deny tidak terpasang; dilewati. CI tetap menjalankannya.)\n'
fi

printf '\nSemua pemeriksaan lulus.\n'
