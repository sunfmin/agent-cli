#!/bin/sh
# Exercises ./verify.sh with a stand-in brew and a stand-in cargo-flux: it
# checks only with Homebrew's sunfmin/tap/flux-toolchain installed and the
# newest the tap has, the tap read fresh, and it runs the tap's cargo-flux,
# ahead of any an older install left in ~/.cargo/bin, once per Flux crate.
# It also holds the tree to what that Flux reads: no `check_overflow`, and no
# pin of cargo-flux, fixpoint or z3 left to read. Nothing is fetched or built.
set -eu
root=$(CDPATH='' cd -- "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail() {
    echo "tests/verify.sh: $1" >&2
    echo "--- log" >&2
    cat "$tmp/log" >&2
    echo "--- stderr" >&2
    cat "$tmp/err" >&2
    exit 1
}
: >"$tmp/log"
: >"$tmp/err"

# The tree, as the tap's Flux reads it.
grep -n 'check_overflow' "$root/Cargo.toml" "$root"/crates/*/Cargo.toml &&
    fail "a Cargo.toml says check_overflow, which the tap's Flux rejects: it is check-overflow"
grep -qE '^\[workspace\.metadata\.checker\]|^(z3|fixpoint) = ' "$root/Cargo.toml" &&
    fail "Cargo.toml still pins fixpoint or z3"
grep -qE 'git clone|xtask|fixpoint --version|z3 --version' "$root/verify.sh" &&
    fail "verify.sh still pins or builds a tool itself"

mkdir -p "$tmp/repo/crates/one" "$tmp/repo/crates/two" "$tmp/repo/crates/plain" \
    "$tmp/home/.cargo/bin" "$tmp/bin" "$tmp/brew/bin" "$tmp/prefix/bin"
cp "$root/verify.sh" "$tmp/repo/verify.sh"
printf '[package]\nname = "one"\n\n[package.metadata.flux]\nenabled = true\n' >"$tmp/repo/crates/one/Cargo.toml"
printf '[package]\nname = "two"\n\n[package.metadata.flux]\nenabled = true\n' >"$tmp/repo/crates/two/Cargo.toml"
printf '[package]\nname = "plain"\n' >"$tmp/repo/crates/plain/Cargo.toml"

# The stand-in brew keeps what is installed in $BREW/installed, and the newest
# the tap has, as last read, in $BREW/newest; `update` reads the tap's newest
# from $BREW/tap, or fails when $BREW/offline exists.
cat >"$tmp/brew/bin/brew" <<'MOCK'
#!/bin/sh
echo "brew $*" >>"$LOG"
f=sunfmin/tap/flux-toolchain
case "$*" in
"update --quiet") [ ! -f "$BREW/offline" ] && cp "$BREW/tap" "$BREW/newest" ;;
"list --versions $f") [ -f "$BREW/installed" ] && echo "flux-toolchain $(cat "$BREW/installed")" ;;
"outdated --quiet $f")
    [ -f "$BREW/installed" ] && [ "$(cat "$BREW/installed")" != "$(cat "$BREW/newest")" ] && echo "$f"
    exit 0
    ;;
"--prefix $f") echo "$PREFIX" ;;
*) exit 1 ;;
esac
MOCK
printf '#!/bin/sh\necho stand-in-toolchain\n' >"$tmp/bin/rustup"
# Cargo runs `cargo flux` as the first cargo-flux on the PATH, as rustup's does.
printf '#!/bin/sh\n[ "$1" = flux ] && exec cargo-flux "$@"\n' >"$tmp/home/.cargo/bin/cargo"
printf '#!/bin/sh\necho "the cargo-flux of an older install ran" >&2\nexit 1\n' >"$tmp/home/.cargo/bin/cargo-flux"
printf '#!/bin/sh\necho "tap cargo-flux $*" >>"$LOG"\n' >"$tmp/prefix/bin/cargo-flux"
chmod +x "$tmp/brew/bin/brew" "$tmp/bin/"* "$tmp/home/.cargo/bin/"* "$tmp/prefix/bin/"*
echo 2026.11.12 >"$tmp/brew/tap"
: >"$tmp/brew/newest"
verify() {
    : >"$tmp/log"
    env -u RUSTUP_TOOLCHAIN -u FLUXFLAGS HOME="$tmp/home" BREW="$tmp/brew" PREFIX="$tmp/prefix" LOG="$tmp/log" \
        PATH="$tmp/home/.cargo/bin:$tmp/brew/bin:$tmp/bin:/usr/bin:/bin" \
        sh "$tmp/repo/verify.sh" "$@" >"$tmp/out" 2>"$tmp/err"
}
asked() { grep -qxF -- "$1" "$tmp/log" || fail "never asked: $1"; }
said() { grep -qF -- "$1" "$tmp/err" || fail "did not say: $1"; }

verify && fail "a run without the toolchain checked something"
said "sunfmin/tap/flux-toolchain is not installed"
said "brew install sunfmin/tap/flux-toolchain"

echo 2026.11.12 >"$tmp/brew/installed"
verify || fail "a run with the newest toolchain failed"
asked "brew update --quiet"
asked "tap cargo-flux flux --manifest-path crates/one/Cargo.toml"
asked "tap cargo-flux flux --manifest-path crates/two/Cargo.toml"
grep -q plain "$tmp/log" && fail "a crate that does not enable Flux was checked"
grep -qxF "verify: checking crates/one" "$tmp/out" || fail "did not say which crate it checks"

verify first || fail "a run of one name failed"
asked "tap cargo-flux flux check --only-check=def:first --manifest-path crates/one/Cargo.toml"

# The tap moves on: the run reads it fresh and refuses the older one.
echo 2026.11.13 >"$tmp/brew/tap"
verify && fail "a run checked with an older toolchain than the tap's newest"
said "sunfmin/tap/flux-toolchain 2026.11.12 is not the newest the tap has"
said "brew upgrade sunfmin/tap/flux-toolchain"
grep -q "tap cargo-flux" "$tmp/log" && fail "the older toolchain checked something"

# A host that cannot reach the tap compares with what it last read, and says so.
echo 2026.11.13 >"$tmp/brew/installed"
touch "$tmp/brew/offline"
echo 2026.11.14 >"$tmp/brew/tap"
verify || fail "a run that could not reach the tap failed with the newest it last read"
said "brew update failed"

# Without brew, the run says where it looked; a host with one in
# /opt/homebrew cannot be without it.
if [ ! -x /opt/homebrew/bin/brew ]; then
    mv "$tmp/brew/bin/brew" "$tmp/brew/brew"
    verify && fail "a run with no brew checked something"
    said "no brew is on the PATH"
fi
echo "tests/verify.sh: ok"
