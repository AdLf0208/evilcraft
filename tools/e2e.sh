#!/usr/bin/env bash
# In-game check for EvilCraft slice 1: the blood repair loop.
#
# Upstream (CyclopsMC/EvilCraft 1.21.1-1.2.99, MIT — see ../FINDINGS.md):
# a Blood Extractor fills with `floor(hp x 5) .. floor(hp x 40)` mB when the player
# kills a mob; a Blood Chest's ten repair slots spend 5 mB a damage point, one point
# every two ticks, and 125 mB an anvil step, from its 10-bucket tank.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs the
# console path and bypasses the native `perform_command` hook.
#
# The End, in the void at x=200 (away from the central island, its dragon egg and the
# ambient item entities of a generated level), a stone floor and clear air.
#
# Default run:
#  A  a chest placed from its item reads 0/10,000 mB;
#  B  with an empty tank a damaged pickaxe is NOT repaired over 40 ticks (the control:
#     the tank is the gate, not the tick);
#  C  1,000 mB repairs exactly one damage point per two ticks for 5 mB (damage and
#     tank move by the same 5 mB a point);
#  D  a chipped anvil becomes an anvil for exactly 125 mB, and an empty tank stops
#     the next step;
#  E  killing a mob with the player's melee fills a carried Blood Extractor with the
#     mob's 100..799 mB (a 20 HP zombie);
#  F  the filled extractor in the chest's container slot empties into the tank, and
#     the extractor reads 0.
#
# Default run (EMBEDDED=0): the game loads `content/` as a dev mod.
# EMBEDDED=1: the game is launched with **only the native bundle** (no `--dev-mod`),
# and CONTROL=1 (default with EMBEDDED) first launches the same binary with
# `.horizon-cache` unusable, where the ids must be unknown.
#
# SELFTEST=1 flips C's assertion (it must FAIL), proving the scenario can fail.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${EC_DEBUG:-}" ] || set -x

EMBEDDED=${EMBEDDED:-0}
CONTROL=${CONTROL:-$EMBEDDED}

CLONE=${CLONE:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/evilcraft
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/evilcraft-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=EvilCraftE2E
CONTROL_WORLD=EvilCraftControl
# Where the arena stands: the End's void, far from the central island.
X0=200
SX=$((X0 + 2))
CHEST=evilcraft:blood_chest
EXTRACTOR=evilcraft:blood_extractor
TANK_CAPACITY=10000
MB_PER_DAMAGE=5
TICKS_PER_DAMAGE=2

command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/evilcraft-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --offscreen
                --wait --resources "$RES")
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    else
        args+=(--dev-mod "$PORT/content")
    fi
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.name // .block // empty'; }

# The End's void at X0: a stone floor and clear air above it. The player is
# teleported first, so the chunks the fills touch are loaded.
arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 100 -14 $((X0 + 4)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 101 -14 $((X0 + 4)) 108 14 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 20
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" 101 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# `/evilcraft info X Y Z`: the line, then the parsed fields in TANK, CAPACITY,
# CONTAINER_ITEM, CONTAINER_BLOOD and SLOT<n>_ITEM / SLOT<n>_DAMAGE.
info() { # info X Y Z
    say "evilcraft info $1 $2 $3"
    ctl wait 12 >/dev/null
    INFO_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Blood Chest' || true)
    [ -n "$INFO_LINE" ] || fail "no /evilcraft info reply"
    eval "$(python3 "$PORT/tools/parse_info.py" "$INFO_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the info line did not parse: $INFO_LINE"
    echo "  $INFO_LINE" >&2
}

expect_tank() { # expect_tank AMOUNT
    [ "$TANK" = "$1" ] || fail "the chest holds $TANK mB, expected $1"
}

# Place the chest from its item and check the block appeared.
place_chest() { # place_chest X Y Z
    local x=$1 y=$2 z=$3
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "item replace entity @s hotbar.0 with $CHEST 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    local aim reply
    aim=$(ctl look-at-block "$x" $((y - 1)) "$z" up 2>&1) \
        || fail "cannot aim at the floor under $x,$y,$z: $aim"
    reply=$(ctl place "$x" "$y" "$z" --item "$CHEST" 2>&1) \
        || fail "ctl place at $x,$y,$z failed: $(tr -d '\n' <<<"$reply" | cut -c1-300)"
    ctl wait 6 >/dev/null
    local name
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "$CHEST" ] || fail "the chest did not place at $x,$y,$z (got '$name')"
}

# `ctl container X Y Z ITEM COUNT`: fills the block's first empty slot.
give_to() { # give_to X Y Z ITEM
    ctl container "$1" "$2" "$3" "$4" 1 >/dev/null
    ctl wait 6 >/dev/null
}

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. Both ids must then be unknown.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    rm -rf "$SCRATCH/.horizon-cache"
    : > "$SCRATCH/.horizon-cache"
    launch "$CONTROL_WORLD"
    arena
    ctl command "item replace entity @s hotbar.0 with $CHEST 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e --arg i "$CHEST" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
        || fail "the control game has the chest item, so the content did not come from the embedded file"
    ctl command "execute in minecraft:the_end run setblock $SX 101 0 $CHEST" >/dev/null || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$SX" 101 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    [ -f "$SCRATCH/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no chest item, the block stayed $name"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SCRATCH/.horizon-cache"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for byte
# the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/evilcraft-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/evilcraft.hmod" \
        || fail "the embedded content differs from content/evilcraft.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/evilcraft.hmod"
}

# Open the chest's menu and report the slot count of the mod's own slots (300+).
open_menu() { # x y z
    # Stand beside the chest first: a click reaches four blocks, and the kill leg
    # leaves the player at the far end of the arena.
    ctl command "execute in minecraft:the_end run tp @s $(( $1 - 2 )).5 $2 $3.5" >/dev/null
    ctl wait 6 >/dev/null
    ctl look-at-block "$1" "$2" "$3" >/dev/null 2>&1 || true
    ctl interact "$1" "$2" "$3" >/dev/null || fail "the click did not open the chest"
    ctl wait-for screen container --timeout 10 >/dev/null \
        || fail "the chest did not open a menu (screen=$(ctl state | jq -r '.screen // ""'))"
    local slots
    slots=$(ctl screen | jq '[.slots[] | select(.slot >= 300)] | length' 2>/dev/null || echo 0)
    [ "$slots" = 11 ] || fail "the chest menu shows $slots slots, expected 11"
}

# Put ITEM (whatever its components) into the chest's own slot NUMBER (300+n):
# a menu click moves the cursor stack, so pick the item up from the inventory first.
put_in_slot() { # slot item
    local src
    src=$(ctl screen | jq -r --arg i "$2" \
        '[.slots[] | select(.slot < 300 and .item == $i) | .slot] | .[0] // ""' 2>/dev/null || true)
    if [ -z "$src" ]; then
        echo "screen with no $2:" >&2
        ctl screen >&2
        fail "$2 is not in the inventory"
    fi
    local out
    out=$(ctl click-slot "$src" 2>&1) || {
        echo "screen: $(ctl screen | jq -c '[.slots[] | select(.item != null) | {slot, item, count}]')" >&2
        fail "cannot pick up $2 from slot $src: $(tr -d '\n' <<<"$out" | cut -c1-200)"
    }
    ctl wait 2 >/dev/null
    out=$(ctl click-slot "$1" 2>&1) || {
        echo "screen: $(ctl screen | jq -c '[.slots[] | select(.item != null) | {slot, item, count}]')" >&2
        fail "cannot place $2 into slot $1: $(tr -d '\n' <<<"$out" | cut -c1-200)"
    }
    ctl wait 2 >/dev/null
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi
arena

ctl command "item replace entity @s hotbar.0 with $CHEST 1" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e --arg i "$CHEST" '[.hotbar[], .main[] | select(.item == $i)] | length >= 1' >/dev/null \
    || fail "the chest item did not register: $(ctl inventory | jq -c '{hotbar,main}')"

echo "== A: a chest placed from its item is empty =="
place_chest "$SX" 101 0
info "$SX" 101 0
expect_tank 0
[ "$CAPACITY" = "$TANK_CAPACITY" ] || fail "the chest reports a $CAPACITY mB tank, expected $TANK_CAPACITY"
echo "  block $(block_name "$SX" 101 0), $TANK/$CAPACITY mB"

echo "== B: an empty tank repairs nothing (the control) =="
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 2 >/dev/null
give_to "$SX" 101 0 minecraft:diamond_pickaxe
say "evilcraft damage $SX 101 0 0 250"
ctl wait 12 >/dev/null
info "$SX" 101 0
[ "${SLOT0_ITEM:-}" = "minecraft:diamond_pickaxe" ] || fail "slot 0 holds ${SLOT0_ITEM:-nothing}, expected the pickaxe"
[ "$SLOT0_DAMAGE" = 250 ] || fail "the harness did not damage the pickaxe (reads $SLOT0_DAMAGE)"
ctl wait 40 >/dev/null
info "$SX" 101 0
[ "$SLOT0_DAMAGE" = 250 ] || fail "an empty tank repaired the pickaxe to damage $SLOT0_DAMAGE"
expect_tank 0
echo "  after 40 ticks: damage still 250, tank 0/10000"

echo "== C: 1,000 mB repairs one damage a point every two ticks for 5 mB =="
say "evilcraft blood $SX 101 0 1000"
ctl wait 12 >/dev/null
info "$SX" 101 0
TANK_BEFORE=$TANK
DAMAGE_BEFORE=$SLOT0_DAMAGE
TICKS_START=$TIME
ctl wait 40 >/dev/null
info "$SX" 101 0
TICKS=$((TIME - TICKS_START))
[ "$TICKS" -gt 0 ] || fail "no ticks passed between the two reads"
CURED=$((DAMAGE_BEFORE - SLOT0_DAMAGE))
SPENT=$((TANK_BEFORE - TANK))
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the pickaxe was not repaired)"
    CURED=0
fi
[ "$CURED" -gt 0 ] || fail "the pickaxe was not repaired in $TICKS ticks (damage $SLOT0_DAMAGE)"
[ "$SPENT" -eq $((MB_PER_DAMAGE * CURED)) ] \
    || fail "$CURED damage points cost $SPENT mB, expected $((MB_PER_DAMAGE * CURED)) (5 mB a point)"
[ "$CURED" -le $((TICKS / TICKS_PER_DAMAGE + 1)) ] \
    || fail "$CURED damage points in $TICKS ticks is faster than one a $TICKS_PER_DAMAGE ticks"
echo "  $TICKS ticks: damage $DAMAGE_BEFORE -> $SLOT0_DAMAGE ($CURED points), tank $TANK_BEFORE -> $TANK (-$SPENT mB)"

echo "== D: a chipped anvil becomes an anvil for 125 mB =="
# Park the repaired pickaxe first: a whole item is not repairable, so it cannot eat
# the anvil's blood between two reads.
say "evilcraft damage $SX 101 0 0 0"
say "evilcraft blood $SX 101 0 0"
ctl wait 12 >/dev/null
give_to "$SX" 101 0 minecraft:chipped_anvil
ctl wait 8 >/dev/null
info "$SX" 101 0
[ "${SLOT1_ITEM:-}" = "minecraft:chipped_anvil" ] || fail "slot 1 holds ${SLOT1_ITEM:-nothing}, expected the chipped anvil"
[ "$TANK" = 0 ] || fail "an empty tank changed the anvil (tank $TANK)"
echo "  an empty tank leaves the chipped anvil alone"
say "evilcraft blood $SX 101 0 125"
ctl wait 20 >/dev/null
info "$SX" 101 0
[ "${SLOT1_ITEM:-}" = "minecraft:anvil" ] || fail "the chipped anvil became ${SLOT1_ITEM:-nothing}, expected minecraft:anvil"
[ "$TANK" = 0 ] || fail "the anvil step left $TANK mB, expected 125 - 125 = 0"
echo "  chipped_anvil -> anvil, tank 125 -> 0 (one 125 mB step)"

echo "== E: a player kill fills a carried Blood Extractor =="
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_sword 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $X0.5 101 6.5" >/dev/null
ctl wait 6 >/dev/null
# One block east, on the floor, frozen: a wandering mob walks off the arena and
# dies to the void, which is not a player kill and would prove nothing.
ctl summon minecraft:zombie $((X0 + 1)) 101 6 >/dev/null || fail "cannot summon a zombie"
ctl wait 8 >/dev/null
ctl command "execute in minecraft:the_end run data merge entity @e[type=minecraft:zombie,limit=1] {NoAI:1b}" >/dev/null
ctl wait 4 >/dev/null
ZOMBIES=$(ctl entities 12 minecraft:zombie | jq -r '.entities | length')
[ "$ZOMBIES" = 1 ] || fail "the arena holds $ZOMBIES zombies after the summon, expected 1: $(ctl entities 12 | jq -c '.entities[0:3]')"
ZID=$(ctl entities 12 minecraft:zombie | jq -r '.entities[0].id // ""' 2>/dev/null || true)
[ -n "$ZID" ] || fail "the zombie is not among the entities: $(ctl entities 12 | jq -c '.entities[0:3]')"
ctl look-at-entity "$ZID" >/dev/null || fail "cannot face the zombie"
for _ in $(seq 1 8); do
    ctl attack "$ZID" >/dev/null 2>&1 || true
    ctl wait 12 >/dev/null
    if [ "$(ctl entities 12 minecraft:zombie | jq -r '.entities | length')" = 0 ]; then break; fi
done
[ "$(ctl entities 12 minecraft:zombie | jq -r '.entities | length')" = 0 ] \
    || fail "the zombie survived eight blows: $(ctl entities 12 | jq -c '.entities[0:3]')"
ctl select 9 >/dev/null
ctl wait 6 >/dev/null
say "evilcraft held"
ctl wait 12 >/dev/null
HELD_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Blood Extractor held' || true)
[ -n "$HELD_LINE" ] || fail "no /evilcraft held reply after the kill"
HELD_BLOOD=$(sed -n 's/.*held: \([0-9][0-9]*\)\/[0-9][0-9]* mB.*/\1/p' <<<"$HELD_LINE")
[ -n "$HELD_BLOOD" ] || fail "the held line did not parse: $HELD_LINE"
[ "$HELD_BLOOD" -ge 100 ] || fail "a 20 HP zombie gave $HELD_BLOOD mB, expected at least 100"
[ "$HELD_BLOOD" -le 799 ] || fail "a 20 HP zombie gave $HELD_BLOOD mB, expected at most 799"
echo "  $HELD_LINE"

echo "== F: the extractor in the container slot empties into the tank =="
say "evilcraft blood $SX 101 0 0"
ctl wait 12 >/dev/null
info "$SX" 101 0
expect_tank 0
open_menu "$SX" 101 0
put_in_slot 310 "$EXTRACTOR"
ctl close >/dev/null 2>&1 || true
ctl wait 10 >/dev/null
info "$SX" 101 0
[ "${CONTAINER_ITEM:-}" = "$EXTRACTOR" ] || fail "the container slot holds ${CONTAINER_ITEM:-nothing}, expected the extractor"
[ "$TANK" = "$HELD_BLOOD" ] || fail "the tank holds $TANK mB, expected the extractor's $HELD_BLOOD"
[ "${CONTAINER_BLOOD:-}" = 0 ] || fail "the extractor still holds ${CONTAINER_BLOOD} mB after emptying"
echo "  extractor $HELD_BLOOD mB -> tank $TANK mB, extractor $CONTAINER_BLOOD mB"

echo "PASS: an empty chest reads 0/10,000 mB, an empty tank repairs nothing, 1,000 mB repairs one damage point every two ticks for exactly 5 mB a point, a chipped anvil becomes an anvil for 125 mB, a player kill fills the carried Blood Extractor with a 20 HP zombie's 100..799 mB, and the extractor in the container slot empties into the tank"
