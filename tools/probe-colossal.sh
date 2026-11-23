#!/usr/bin/env bash
# Throwaway probe (not evidence): does a click on the Colossal Blood Chest open its menu, and
# what does the game log say about the package's menu type?
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
"$CTL" --socket "$SOCK" launch --game "$GAME" --world ProbeColossal --seed 20261105 \
    --creative --offscreen --wait --resources "$RES" >/dev/null
ctl wait-ready 120
ctl wait 40
ctl command "execute in minecraft:the_end run tp @s 198.5 101 -6.5" >/dev/null
for _ in $(seq 1 60); do ctl wait 5 >/dev/null; [ "$(ctl block 198 100 -10 | jq -r '.loaded // false')" = true ] && break; done
ctl command "execute in minecraft:the_end run fill 194 100 -14 208 100 14 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill 198 101 -10 200 103 -8 evilcraft:reinforced_undead_planks" >/dev/null
ctl command "execute in minecraft:the_end run setblock 199 102 -9 minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run setblock 198 101 -10 evilcraft:colossal_blood_chest" >/dev/null
ctl wait 10
echo "--- blocks:"; ctl block 198 101 -10 | jq -c '{name, loaded}'; ctl block 200 103 -8 | jq -c '{name}'
say() { ctl chat "/$1" >/dev/null; }
say "evilcraft colossal 198 101 -10"; ctl wait 12
ctl messages 6 | jq -r '.chat[]' | head -3
# Stand in the controller's own row (cell z=-10), aim at the cell centre and click.
ctl command "execute in minecraft:the_end run tp @s 196.5 101 -9.5" >/dev/null
ctl wait 8
ctl look-at-block 198 101 -10 2>&1 || true
ctl wait 4
echo "--- crosshair: $(ctl looking-at | jq -c '{block, hit}')"
ctl interact 198 101 -10 2>&1 || true
ctl wait 10
echo "--- state: $(ctl state | jq -c '{screen}')"
echo "--- screen: $(ctl screen 2>&1 | head -c 400)"
ctl close >/dev/null 2>&1 || true
# The plain chest, the control: the same click opens an 11-slot menu.
ctl command "execute in minecraft:the_end run setblock 198 101 -12 evilcraft:blood_chest" >/dev/null
ctl wait 8
ctl command "execute in minecraft:the_end run tp @s 196.5 101 -11.5" >/dev/null
ctl wait 6
ctl look-at-block 198 101 -12 2>&1 || true
ctl interact 198 101 -12 2>&1 || true
ctl wait 10
echo "--- chest state: $(ctl state | jq -c '{screen}')"
ctl close >/dev/null 2>&1 || true
# Is the colossal chest a container at all? A stack in a slot spills as its own item entity
# when the block is removed, and only the engine's container table knows the slots.
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_axe 1" >/dev/null
ctl wait 4
ctl select 1 >/dev/null
ctl wait 2
say "evilcraft load 198 101 -10 0 minecraft:cobblestone"
ctl wait 12
say "evilcraft colossal 198 101 -10"
ctl wait 12
ctl messages 6 | jq -r '.chat[]' | head -2
ctl command "execute in minecraft:the_end run tp @s 195.5 101 -9.5" >/dev/null
ctl wait 6
echo "--- break: $(ctl break 198 101 -10 | jq -c '{block, drops, items_left_on_ground}')"
ctl wait 10
echo "--- after break: $(ctl block 198 101 -10 | jq -c '{name}')"
echo "--- log warnings:"
ctl logs --lines 400 | grep -iE "menu|colossal|content|mod " | tail -40 || true
