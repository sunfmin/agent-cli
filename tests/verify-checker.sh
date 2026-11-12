#!/bin/sh
# What ./verify.sh asks of the checker before it checks anything: cargo-flux
# and z3 at the pins Cargo.toml names, and a fixpoint that reports its version,
# of any version, since fixpoint is pinned to nothing (sunfmin/control#741: a
# host's fixpoint moved to liquid-fixpoint's nightly and main went red).
# Every tool is a stand-in on a PATH of its own: nothing is fetched or checked.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail() {
    echo "tests/verify-checker.sh: $*" >&2
    exit 1
}

bin="$tmp/home/.cargo/bin"
mkdir -p "$bin"
flux_rev=$(sed -n 's/^flux-core = .*rev = "\([0-9a-f]*\)".*/\1/p' "$root/Cargo.toml")
z3_pin=$(sed -n 's/^z3 = "\(.*\)"$/\1/p' "$root/Cargo.toml")
[ -n "$flux_rev" ] && [ -n "$z3_pin" ] || fail "Cargo.toml pins no Flux revision or no z3"
grep -q '^fixpoint = ' "$root/Cargo.toml" && fail "Cargo.toml still pins fixpoint"

# cargo flux --version reports the first seven digits of its Flux revision;
# any other cargo flux is a check, which is only written down.
cat >"$bin/cargo" <<EOF
#!/bin/sh
case "\$*" in
"flux --version") echo "cargo-flux $(echo "$flux_rev" | cut -c1-7)" ;;
*) echo "cargo \$*" >>"$tmp/log" ;;
esac
EOF
z3() {
    printf '#!/bin/sh\necho "Z3 version %s - 64 bit"\n' "$1" >"$bin/z3"
}
fixpoint() {
    printf '#!/bin/sh\necho "Liquid-Fixpoint Copyright 2009-25 Regents of the University of California."\necho "fixpoint %s"\n' "$1" >"$bin/fixpoint"
}
verify() {
    : >"$tmp/log"
    HOME="$tmp/home" PATH="/usr/bin:/bin" RUSTUP_TOOLCHAIN=nightly sh "$root/verify.sh" >/dev/null 2>"$tmp/err"
}
z3 "$z3_pin"
chmod +x "$bin/cargo" "$bin/z3"

# fixpoint of a version and commit no one pinned is checked with.
for version in "0.9.6.3.8 (0b924442ce26a8f270bffac18ba6295bc7411646)" "9.9.9 (0000000)"; do
    fixpoint "$version"
    chmod +x "$bin/fixpoint"
    verify || fail "refused fixpoint $version: $(cat "$tmp/err")"
    grep -q '^cargo flux --manifest-path ' "$tmp/log" || fail "checked nothing with fixpoint $version"
done

# No fixpoint, or one that reports no version, is refused, saying so.
printf '#!/bin/sh\nexit 1\n' >"$bin/fixpoint"
verify && fail "checked with a fixpoint that reports no version"
grep -q "fixpoint reports no version" "$tmp/err" || fail "did not say fixpoint reports no version"
[ ! -s "$tmp/log" ] || fail "checked something with no fixpoint"
rm "$bin/fixpoint"
verify && fail "checked with no fixpoint"
fixpoint "0.9.6.3.8 (0b924442ce26a8f270bffac18ba6295bc7411646)"
chmod +x "$bin/fixpoint"

# z3 stays pinned.
z3 0.0.1
verify && fail "checked with z3 0.0.1, where Cargo.toml pins $z3_pin"
grep -q "z3 is 0.0.1" "$tmp/err" || fail "did not name the z3 it refused"
[ ! -s "$tmp/log" ] || fail "checked something with an unpinned z3"
z3 "$z3_pin"

# cargo-flux stays pinned.
printf '#!/bin/sh\necho "cargo-flux 0000000"\n' >"$bin/cargo"
verify && fail "checked with a cargo-flux of another Flux revision"
grep -q "cargo-flux is built from Flux 0000000" "$tmp/err" || fail "did not name the cargo-flux it refused"
exit 0
