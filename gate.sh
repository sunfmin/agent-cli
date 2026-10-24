#!/usr/bin/env bash
# The definition of "may land": the same for a person and for afkconvoy.
set -euo pipefail
cd "$(dirname "$0")"

# The whole gate runs on rustup's nightly, the one cargo-flux is built for,
# whatever the shell that started it has on its PATH: a Homebrew `cargo`
# ahead of rustup's lacks rustup's components. Flux's attributes expand to
# nothing elsewhere, so a release still builds on stable.
PATH="$HOME/.cargo/bin:$PATH"
RUSTUP_TOOLCHAIN=nightly-2026-09-25
export PATH RUSTUP_TOOLCHAIN

cargo fmt --all --check

# Flux runs first, alone, before anything else is built: run beside the tests
# it shares the cores with every one of them, and how long it took says more of
# them than of it. verify.sh stops a solver query of one function past twenty
# seconds as an error naming it, checks on past an error so one run names every
# slow function, and times each function.
#
# Every function of every Flux crate is checked: none is taken on trust. Flux
# says so itself, in the summary that ends its run on a crate, so an exemption
# counts however it is written. Flux summarises each dependency too: verify.sh
# checks one crate per cargo run, so a crate's summary is the last one before
# that run's `build-finished`, the crate being the last one compiled.
#
# The gate holds back each crate's report of its slowest functions and says it
# once Flux has ended, green or red.
flux_log=$(mktemp)
trap 'rm -f "$flux_log" "$flux_log".*' EXIT
./verify.sh --message-format=json-render-diagnostics >"$flux_log" 2>&1 && flux_status=0 || flux_status=$?
awk -v summaries="$flux_log.summaries" -v timings="$flux_log.timings" '
    /^verify: checking / { crate = substr($0, 18); summary = ""; report = ""; next }
    /^\{/ {
        if (/"reason":"build-finished"/) {
            print crate "\t" summary > summaries
            printf "%s:\n%s", crate, (report == "" ? "Flux reported no timings.\n" : report) > timings
        }
        next
    }
    /Timing Report/ { report = ""; in_report = 1 }
    in_report && !/^(───|Total running time:|Functions checked:|Min:|Max:|Mean:|Std\. Dev\.:|Top 5 Functions|• )/ { in_report = 0 }
    in_report { report = report $0 "\n"; next }
    { print > "/dev/stderr" }
    /^summary\./ { summary = $0 }
' "$flux_log"
slowest() {
    echo "gate: what Flux took longest to check, by crate:" >&2
    cat "$flux_log.timings" >&2 2>/dev/null || echo "Flux reported no timings." >&2
}
[ "$flux_status" -eq 0 ] || {
    slowest
    exit "$flux_status"
}
exempt=$(awk -F'\t' '$2 !~ /; 0 trusted; 0 ignored\./ { print "  " $1 ": " ($2 == "" ? "no summary" : $2) }' "$flux_log.summaries")
[ -z "$exempt" ] || {
    echo "gate: a Flux crate must not exempt anything from Flux (no trusted, no ignored), and Flux reports:" >&2
    echo "$exempt" >&2
    slowest
    exit 1
}
slowest

cargo clippy --workspace --all-targets --quiet -- -D warnings

# nextest runs the test programs side by side, where `cargo test` runs one at a
# time. A workspace with no test yet passes, with nextest's warning.
cargo nextest --version >/dev/null 2>&1 || {
    echo "gate: cargo-nextest is not installed: brew install cargo-nextest (or cargo install --locked cargo-nextest)" >&2
    exit 1
}
cargo nextest run --workspace --cargo-quiet --no-tests=warn --status-level fail --final-status-level fail --show-progress none
