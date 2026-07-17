#!/usr/bin/env bash
# ============================================================
# generate-code-coverage-html.sh
#
# Generates an HTML code-coverage report using cargo-llvm-cov.
# Works on macOS (arm64 / x86-64) and Linux.
#
# Note: --branch coverage requires Rust nightly because it uses
#       -Z coverage-options=branch (not available on stable).
#       This script installs nightly automatically for the
#       coverage run; the main project build stays on 1.97.1.
#
# Prerequisites (run once per environment):
#   ./scripts/setup-rust-1.97.1.sh
#   rustup component add llvm-tools-preview
#   cargo install cargo-llvm-cov --locked
#
# Usage:
#   ./scripts/generate-code-coverage-html.sh              # whole workspace
#   ./scripts/generate-code-coverage-html.sh stats        # single package
#   ./scripts/generate-code-coverage-html.sh ethcore      # single package
#
# Output:
#   target/coverage/<package|all>/html/index.html
# ============================================================

set -euo pipefail

PACKAGE="${1:-}"          # optional first argument: package name
OUTDIR="target/coverage"

# ── macOS: raise FD limit (rayon + RocksDB exhaust default 256) ──────────────
if [[ "$(uname)" == "Darwin" ]]; then
    ulimit -n 65536 2>/dev/null || true
fi

# ── Ensure nightly toolchain is present (required for --branch) ──────────────
if ! rustup toolchain list 2>/dev/null | grep -q "^nightly"; then
    echo "Installing nightly toolchain for branch coverage …"
    rustup toolchain install nightly
fi

# ── Ensure llvm-tools-preview is present on nightly ──────────────────────────
if ! rustup component list --toolchain nightly --installed 2>/dev/null | grep -q "llvm-tools"; then
    echo "Adding llvm-tools-preview to nightly …"
    rustup component add --toolchain nightly llvm-tools-preview
fi

# ── Ensure cargo-llvm-cov is available ───────────────────────────────────────
if ! cargo llvm-cov --version &>/dev/null; then
    echo "Installing cargo-llvm-cov …"
    cargo install cargo-llvm-cov --locked
fi

# ── Fetch test vectors (required for json-tests feature) ─────────────────────
if [[ -f ".gitmodules" ]]; then
    git submodule update --init --recursive
fi

# ── Build coverage command ────────────────────────────────────────────────────
if [[ -n "${PACKAGE}" ]]; then
    REPORT_DIR="${OUTDIR}/${PACKAGE}"
    # cargo-llvm-cov appends "html/" to --output-dir automatically
    HTML_DIR="${REPORT_DIR}/html"
    echo "▶ Running coverage for package: ${PACKAGE}"
    echo "  Output: ${HTML_DIR}/index.html"
    echo ""
    cargo +nightly llvm-cov \
        --package "${PACKAGE}" \
        --branch \
        --html \
        --output-dir "${REPORT_DIR}"
    cargo +nightly llvm-cov \
        --package "${PACKAGE}" \
        --branch \
        --text \
        --output-path "${REPORT_DIR}/coverage-summary.txt"
    echo ""
    cargo +nightly llvm-cov report --package "${PACKAGE}" --branch
else
    REPORT_DIR="${OUTDIR}/all"
    HTML_DIR="${REPORT_DIR}/html"
    echo "▶ Running coverage for entire workspace"
    echo "  Output: ${HTML_DIR}/index.html"
    echo ""
    cargo +nightly llvm-cov \
        --all \
        --branch \
        --html \
        --output-dir "${REPORT_DIR}"
    cargo +nightly llvm-cov \
        --all \
        --branch \
        --text \
        --output-path "${REPORT_DIR}/coverage-summary.txt"
    echo ""
    cargo +nightly llvm-cov report --all --branch
fi

echo ""
echo "✅ Coverage report complete."
echo "   HTML : ${HTML_DIR}/index.html"
echo "   Text : ${REPORT_DIR}/coverage-summary.txt"
