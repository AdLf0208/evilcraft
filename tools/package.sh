#!/usr/bin/env bash
# Package both artifacts:
#  - the .hmod via the pin's own cargo-horizon-mod (build + check), from content/
#  - the source-only .hnmod (tools/package.py, deterministic bytes, in the port root),
#    carrying the built .hmod at mixins/<id>/data/ (the AE2 embedded-content pattern)
# Run inside a build slot (it compiles the packaging tool):
#   ports/tools/build-slot.sh run --job cg-pack --port ports/evilcraft \
#       --step package -- env CARGO_INCREMENTAL=0 tools/package.sh
set -euo pipefail

PORT=$(cd "$(dirname "$0")/.." && pwd)
HORIZON=${HORIZON:-${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}/ports/_shared/pins/horizon-1d63e0a}
LOGS=$PORT/target/ladder
mkdir -p "$LOGS"
cd "$PORT"

CARGO_TARGET_DIR=$PORT/target/carry-tools
export CARGO_TARGET_DIR
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod build) \
    2>&1 | tee "$LOGS/package-hmod.log"
cp content/target/horizon-mod/evilcraft.hmod content/evilcraft.hmod
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod check evilcraft.hmod) \
    2>&1 | tee -a "$LOGS/package-hmod.log"

# The bundle ships this build's content, never a stale copy: sync it in after the
# build, then let tools/package.py refuse anything that no longer matches (the
# same order `disenchanting_table/tools/package.sh` uses).
cp content/evilcraft.hmod mixins/evilcraft/data/evilcraft-content.hmod

python3 tools/package.py | tee "$LOGS/package-hnmod.log"
# The port's own version, read from the plan: the registry entry, the file and the
# release must all agree (playbook §1.7), so no second copy of the number here.
VERSION=$(python3 -c 'import pathlib,tomllib;print(tomllib.loads(pathlib.Path("mixins/evilcraft/horizon-mixin.toml").read_text())["mod"]["version"])')
cp "target/evilcraft-${VERSION}.hnmod" evilcraft.hnmod

(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod check \
    ../evilcraft.hnmod --kind native-mod) \
    2>&1 | tee "$LOGS/check-native.log"

echo "== artifacts =="
for f in content/evilcraft.hmod evilcraft.hnmod; do
    [ -f "$f" ] || { echo "missing $f"; exit 1; }
    size=$(stat -c %s "$f")
    sha=$(sha256sum "$f" | cut -d' ' -f1)
    entries=$(python3 -c "import zipfile,sys; print(len(zipfile.ZipFile(sys.argv[1]).namelist()))" "$f")
    echo "$f size=$size sha256=$sha entries=$entries"
done
