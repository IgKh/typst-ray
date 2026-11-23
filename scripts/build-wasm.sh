#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Igor Khanin
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Builds the WASM module of every per-Typst-version crate (crates/typstray-<version>)
# into wasm/<version>/
#
set -eu

cd "$(dirname "$0")/.."

opt_flags=""
missing_only=false
for arg in "$@"; do
    case "$arg" in
        --fast) opt_flags="--no-opt" ;;
        --missing-only) missing_only=true ;;
        *) echo "Unknown option: $arg"; exit 1 ;;
    esac
done

for crate in crates/typstray-[0-9]* crates/typstray-dev; do
    version="${crate#crates/typstray-}"
    out="wasm/$version"

    if $missing_only && [ -f "$out/typstray_wasm_bg.wasm" ]; then
        continue
    fi

    echo "Building Typst $version WASM module..."
    wasm-pack build "$crate" --release $opt_flags --target web --out-dir "../../$out" --out-name typstray_wasm
done
