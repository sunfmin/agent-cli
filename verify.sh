#!/usr/bin/env bash
# Checks with Flux every crate whose Cargo.toml turns it on
# ([package.metadata.flux]), with the checker the workspace Cargo.toml pins,
# and fails unless each crate's every function is checked: nothing trusted,
# nothing ignored. Needs `cargo-flux`, `fixpoint` and `z3` installed, each the
# one pinned; it refuses to check with any other.
#
#   ./verify.sh          check every such crate
#   ./verify.sh <name>   check only the functions whose path contains <name>
set -euo pipefail
cd "$(dirname "$0")"

# cargo-flux re-invokes `cargo +<its nightly>`, which only rustup's cargo
# understands, and rustup needs some toolchain to start from.
PATH="$HOME/.cargo/bin:$PATH"
: "${RUSTUP_TOOLCHAIN:=stable}"
# A solver query past 20 s fails on its function, by name; -Fcatch-bugs checks
# on past an error, so one run names every failing function. Nothing limits
# the whole run: one cut short would name nothing.
: "${FLUXFLAGS=-Ftimings -Ffixpoint-timeout=20 -Fcatch-bugs}"
logs=$(mktemp -d)
trap 'rm -rf "$logs"' EXIT
FLUXFLAGS="$FLUXFLAGS -Flog-dir=$logs"
export PATH RUSTUP_TOOLCHAIN FLUXFLAGS

pin() { sed -n "/^\[workspace\.metadata\.flux\]/,/^\[/s/^$1 = \"\(.*\)\"$/\1/p" Cargo.toml; }
flux_rev=$(pin rev)
fixpoint_pin=$(pin fixpoint)
z3_pin=$(pin z3)
flux_is=$(cargo flux --version 2>/dev/null | cut -d' ' -f2) || true
fixpoint_is=$(fixpoint --version 2>/dev/null | sed -n 's/^fixpoint //p') || true
z3_is=$(z3 --version 2>/dev/null | sed -n 's/^Z3 version \([^ ]*\).*/\1/p') || true
pinned=yes
# cargo-flux reports the first seven digits of the revision it was built from.
if [ -z "$flux_is" ] || [ "${flux_rev#"$flux_is"}" = "$flux_rev" ]; then
    pinned=
    echo "verify: cargo-flux is built from Flux ${flux_is:-(nothing reported)}, and Cargo.toml pins $flux_rev. Install the pinned one:" >&2
    echo "    git clone https://github.com/flux-rs/flux && cd flux && git checkout $flux_rev && cargo xtask install" >&2
fi
if [ "$fixpoint_is" != "$fixpoint_pin" ]; then
    pinned=
    echo "verify: fixpoint is ${fixpoint_is:-(nothing reported)}, and Cargo.toml pins $fixpoint_pin. Install the pinned one, built at the commit in the brackets:" >&2
    echo "    https://github.com/ucsd-progsys/liquid-fixpoint#how-to-build-and-install" >&2
fi
if [ "$z3_is" != "$z3_pin" ]; then
    pinned=
    echo "verify: z3 is ${z3_is:-(nothing reported)}, and Cargo.toml pins $z3_pin. Install the pinned one:" >&2
    echo "    https://github.com/Z3Prover/z3/releases/tag/z3-$z3_pin" >&2
fi
[ -n "$pinned" ] || exit 1

crates=$(awk 'FNR == 1 { name = "" } /^name = / && name == "" { name = $3; gsub(/"/, "", name) }
    /^\[package\.metadata\.flux\]/ { print name }' crates/*/Cargo.toml)
if [ -n "${1:-}" ]; then
    for crate in $crates; do cargo flux check -p "$crate" --only-check="def:$1"; done
    exit
fi

# -j1 keeps each crate's `Checking` line next to its summary.
args=(check -j1)
for crate in $crates; do args+=(-p "$crate"); done
output=$(mktemp)
trap 'rm -rf "$logs" "$output"' EXIT
if ! cargo flux "${args[@]}" </dev/null >"$output" 2>&1; then
    cat "$output" >&2
    echo "verify: Flux fails; a function named with \"fixpoint query timed out\" took a solver query past 20 s" >&2
    exit 1
fi
for crate in $crates; do
    summary=$(awk -v crate="$crate" '/^ *Checking / { at = $2 } /^summary\./ && at == crate { s = $0 } END { print s }' "$output")
    case "$summary" in
    *"; 0 trusted; 0 ignored."*) echo "verify: $crate: $summary" ;;
    *)
        cat "$output" >&2
        echo "verify: $crate must leave nothing to trust, and Flux reports: ${summary:-no summary}" >&2
        exit 1
        ;;
    esac
done
