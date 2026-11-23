#!/usr/bin/env bash
# Throwaway probe (not evidence): does a right click on the Purifier reach its block Part's
# `use_item_on`, and where does the insert stop? The package's `PROBE` switch prints one chat
# line per step (see `src/purifier.rs`); this run only collects them.
set -Eeuo pipefail
CLONE=$(cd "$(dirname "$0")/../../.." && pwd)
PORT=$CLONE/ports/evilcraft
: "${GAME:?}" "${CTL:?}" "${RES:?}" "${E2E_TMP:?}"
SOCK=${SOCK:-$E2E_TMP/ec-probe.sock}
SCRATCH=$(mktemp -d "${E2E_TMP}/ec-probe.XXXXXX")
export HORIZON_HOME=$SCRATCH/game HORIZON_LAUNCHER_HOME=$SCRATCH/launcher \
       XDG_DATA_HOME=$SCRATCH/data XDG_CONFIG_HOME=$SCRATCH/config HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
ctl() { "$CTL" --socket "$SOCK" "$@"; }
cleanup() { local rc=$?; ctl quit >/dev/null 2>&1 || true; sleep 1; echo "KEPT:$SCRATCH"; echo "EXIT:$rc"; }
trap cleanup EXIT
cd "$SCRATCH"
"$CTL" --socket "$SOCK" launch --game "$GAME" --world ProbePurifier --seed 20261105 \
    --creative --offscreen --wait --resources "$RES" >/dev/null
ctl wait-ready 120
ctl wait 40
ctl command "execute in minecraft:the_end run tp @s 200.5 101 0.5" >/dev/null
for _ in $(seq 1 60); do ctl wait 5 >/dev/null; [ "$(ctl block 206 100 -10 | jq -r '.loaded // false')" = true ] && break; done
ctl command "execute in minecraft:the_end run fill 198 100 -14 208 100 14 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill 198 101 -14 208 108 14 minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
ctl wait 10
ctl command "execute in minecraft:the_end run setblock 206 101 -10 evilcraft:purifier" >/dev/null
ctl wait 10
say() { ctl chat "/$1" >/dev/null; }
echo "--- block: $(ctl block 206 101 -10 | jq -c '{name}')"
say "evilcraft purifier 206 101 -10"; ctl wait 12
ctl messages 6 | jq -r '.chat[]' | head -2
# A cursed pickaxe in hand, aimed at the purifier's cell centre, right-clicked standing.
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe[minecraft:enchantments={\"minecraft:vanishing_curse\":1}] 1" >/dev/null
ctl wait 6
ctl select 1 >/dev/null
ctl wait 4
echo "--- held: $(ctl inventory | jq -c '.held')"
ctl command "execute in minecraft:the_end run tp @s 204.5 101 -9.5" >/dev/null
ctl wait 6
ctl look-at 206.5 101.5 -9.5 >/dev/null 2>&1 || true
ctl wait 4
echo "--- crosshair: $(ctl looking-at | jq -c '{block, hit}')"
ctl mouse right click >/dev/null
ctl wait 12
echo "--- chat after the click:"
ctl messages 20 | jq -r '.chat[]' | grep -E "PROBE|Purifier|purifier" | head -12
say "evilcraft purifier 206 101 -10"; ctl wait 12
ctl messages 6 | jq -r '.chat[]' | head -2
echo "--- held after: $(ctl inventory | jq -c '.held')"
echo "--- data: $(ctl command "data get block 206 101 -10 Items" 2>&1 | head -2)"
# The same click with a plain item, and the empty hand.
ctl command "item replace entity @s hotbar.0 with minecraft:cobblestone 1" >/dev/null
ctl wait 6
ctl select 1 >/dev/null
ctl wait 4
ctl mouse right click >/dev/null
ctl wait 12
say "evilcraft purifier 206 101 -10"; ctl wait 12
echo "--- after a plain item: $(ctl messages 4 | jq -r '.chat[]' | grep 'Purifier \[' | head -1)"
ctl command "clear @s" >/dev/null
ctl wait 6
ctl mouse right click >/dev/null
ctl wait 12
say "evilcraft purifier 206 101 -10"; ctl wait 12
echo "--- after an empty click: $(ctl messages 4 | jq -r '.chat[]' | grep 'Purifier \[' | head -1)"
# The crouching extractor pour: tank 2,000 of 3,000, a full extractor, shift held.
say "evilcraft blood 206 101 -10 2000"; ctl wait 12
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.8 with evilcraft:blood_extractor 1" >/dev/null
ctl wait 6
ctl select 9 >/dev/null
ctl wait 4
say "evilcraft held 5000"; ctl wait 12
ctl command "execute in minecraft:the_end run tp @s 204.5 101 -9.5" >/dev/null
ctl wait 6
echo "--- state: $(ctl state | jq -c '{dimension, position: .player.position, yaw, pitch}')"
echo "--- crosshair: $(ctl looking-at | jq -c '{block}')"
echo "--- 205: $(ctl block 205 101 -10 | jq -c '{name}') 206: $(ctl block 206 101 -10 | jq -c '{name}')"
ctl key shift down >/dev/null
ctl wait 4
ctl mouse right click >/dev/null
ctl wait 10
ctl key shift up >/dev/null
ctl wait 10
say "evilcraft purifier 206 101 -10"; ctl wait 12
echo "--- after the crouching pour: $(ctl messages 6 | jq -r '.chat[]' | grep 'Purifier \[' | head -1)"
say "evilcraft held"; ctl wait 12
echo "--- extractor: $(ctl messages 6 | jq -r '.chat[]' | grep 'Blood Extractor held' | head -1)"
echo "--- log warnings:"
ctl logs --lines 300 | grep -iE "probe|purifier|refus|component" | tail -20 || true
