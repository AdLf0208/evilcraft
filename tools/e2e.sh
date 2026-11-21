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
# Slice 2 — the Blood Infuser, per the parity rule:
#  H  the machine places, reads 0 mB at tier 0, and reads all 9 of its own recipe files
#     (the counterfactual for the recipe type: a package that did not register it reads
#     0); an unknown item id is refused by the harness loader;
#  I  a tier-3 recipe waits on a tier-0 machine: 560 ticks, the bone untouched, no
#     product, no blood spent (the tier gate, `RecipeBloodInfuser.matches`);
#  J  the menu lists the seven slots (three machine slots, four upgrade slots) and a
#     Promise of Tenacity III in slot 303 makes the machine tier 3;
#  K  the held bone is infused into one blaze rod for 2,500 mB in 500 ticks (the
#     counterfactual for I: the same machine, the same bone, one tier upgrade later);
#  L  every other carried recipe is crafted the way its file declares, asserting the
#     product, its count, the consumed input and the blood spent;
#  M  the container slot empties a filled extractor into the tank;
#  N  the infuser breaks back into itself (its loot table and the minecraft:mineable tag).
#
# Slice 3 — the tank on the broken machine's own item (`evilcraft:copy_tank_data`):
#  O  a chest holding 6,500 mB is broken in survival: its dropped item carries
#     `minecraft:block_entity_data` tagged `evilcraft:blood_chest` with
#     `fluids=[{Fluid:"evilcraft:blood", Amount:6500}]` and **no** `Items` (the engine
#     spills the cobblestone that was in slot 0 as its own item entity), and placing that
#     item again reads `tank 6500/10000` with empty slots — the blood, and nothing else,
#     makes the round trip;
#  P  the same chest broken **empty** carries no block entity data and places back at
#     0 mB (the payload is the listener's, not something the engine adds), and an item
#     doctored to 20,000 mB (`give … Amount:20000`) places a chest clamped to
#     10,000 mB both in `/evilcraft info` and in the block entity's own `fluids`.
#
# The operator gate on `/evilcraft info|infuser` is unit-tested
# (`the_readout_needs_an_operator_and_names_only_this_chest`) and registered at level 2;
# its in-game leg needs a LAN guest at level 0 (`lan open 0`), because an integrated
# server refuses `op`/`deop` (HANDOVER §9d trap 6), and that leg belongs to the next
# slice's harness (see `notes.md`).
#
# FORMAT (2026-11-18): this port is one flat SDK package, so the code and the content
# travel together. `GAME` must be the game the **launcher** built from this package
# (`ports/tools/launcher-game.sh run evilcraft` supplies it, with `CTL`, `CONTROL_GAME`
# and `RES`); there is no `--dev-mod` any more.
#
# CONTROL=1 (the launcher run sets `CONTROL_GAME`) first launches the same Nightly
# **without** the package, where the ids must be unknown.
#
# SELFTEST=1 flips C's assertion, SELFTEST=infuser flips K's and SELFTEST=tank flips O's
# (each must FAIL), proving the scenarios can fail. Phase I is the infuser's own
# differential: the same machine, the same bone and a full tank do nothing at tier 0 and
# infuse at tier 3. Phase P is the tank's differential: a chest broken with nothing in its
# tank drops an item that carries no payload, which is exactly what a break the listener
# never hears leaves behind.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${EC_DEBUG:-}" ] || set -x

CONTROL=${CONTROL:-0}

CLONE=${CLONE:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/evilcraft
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-}
GAME=${GAME:-}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/evilcraft-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=EvilCraftE2E
CONTROL_WORLD=EvilCraftControl
# Where the arena stands: the End's void, far from the central island.
X0=200
SX=$((X0 + 2))
IX=$((X0 + 6))
CHEST=evilcraft:blood_chest
INFUSER=evilcraft:blood_infuser
EXTRACTOR=evilcraft:blood_extractor
PROMISE=evilcraft:promise_tier_3
TANK_CAPACITY=10000
MB_PER_DAMAGE=5
TICKS_PER_DAMAGE=2

command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }
[ -n "$CTL" ] || { echo "set CTL to the horizon-ctl of the launcher build" >&2; exit 2; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || { echo "set GAME to a game the launcher built with this package" >&2; exit 2; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "${E2E_TMP:-/tmp/opencode}/evilcraft-home.XXXXXX")
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

launch() { # launch [WORLD] [GAME]
    local world=${1:-$WORLD}
    local game=${2:-$GAME}
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch --game "$game" --world "$world" --seed 20261105 \
        --creative --offscreen --wait --resources "$RES" >/dev/null
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
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 100 -14 $((X0 + 8)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 2)) 101 -14 $((X0 + 8)) 108 14 minecraft:air" >/dev/null
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
    # A readout replaces the previous one: a stack that has left slot 0 must not still read
    # as there because an earlier line named it.
    unset "${!SLOT@}" 2>/dev/null || true
    eval "$(python3 "$PORT/tools/parse_info.py" "$INFO_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the info line did not parse: $INFO_LINE"
    echo "  $INFO_LINE" >&2
}

expect_tank() { # expect_tank AMOUNT
    [ "$TANK" = "$1" ] || fail "the chest holds $TANK mB, expected $1"
}

# Place a block from its item and check the block appeared.
place_chest() { # place_chest X Y Z [ITEM]
    local x=$1 y=$2 z=$3 item=${4:-$CHEST}
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "item replace entity @s hotbar.0 with $item 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    local aim reply
    aim=$(ctl look-at-block "$x" $((y - 1)) "$z" up 2>&1) \
        || fail "cannot aim at the floor under $x,$y,$z: $aim"
    reply=$(ctl place "$x" "$y" "$z" --item "$item" 2>&1) \
        || fail "ctl place at $x,$y,$z failed: $(tr -d '\n' <<<"$reply" | cut -c1-300)"
    ctl wait 6 >/dev/null
    local name
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "$item" ] || fail "$item did not place at $x,$y,$z (got '$name')"
}

# `ctl container X Y Z ITEM COUNT`: fills the block's first empty slot.
give_to() { # give_to X Y Z ITEM
    ctl container "$1" "$2" "$3" "$4" 1 >/dev/null
    ctl wait 6 >/dev/null
}

# The control: the same Nightly, unmodified (the launcher exports it as
# `CONTROL_GAME`), so the only difference is the package. Both ids must be unknown.
control_leg() {
    echo "== control: the same Nightly without this package =="
    launch "$CONTROL_WORLD" "$CONTROL_GAME"
    arena
    ctl command "item replace entity @s hotbar.0 with $CHEST 1" >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e --arg i "$CHEST" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
        || fail "the control game has the chest item, so the package was in it"
    ctl command "item replace entity @s hotbar.0 with $INFUSER 1" >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e --arg i "$INFUSER" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
        || fail "the control game has the infuser item, so the package was in it"
    ctl command "execute in minecraft:the_end run setblock $SX 101 0 $CHEST" >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$SX" 101 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    ctl command "execute in minecraft:the_end run setblock $IX 101 0 $INFUSER" >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    local machine
    machine=$(block_name "$IX" 101 0)
    [ "$machine" = "minecraft:air" ] || fail "the control game holds $machine, so the infuser id was known"
    echo "  without the package: no chest or infuser item, both blocks stayed $name / $machine"
    ctl quit >/dev/null 2>&1 || true
}

# Open a machine's menu and check the slot count of the mod's own slots (300+).
open_menu() { # x y z expected-slots
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
    [ "$slots" = "$4" ] || fail "the menu shows $slots slots, expected $4"
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

# `/evilcraft infuser X Y Z`: the line, then the parsed fields in TANK, CAPACITY,
# TIER, CONT_ITEM, CONT_BLOOD, IN_ITEM, IN_COUNT, OUT_ITEM, OUT_COUNT, PROGRESS,
# RECIPES and CARRIED.
infuser() { # infuser X Y Z
    # Wait for the reply itself, not a fixed number of ticks: a chat command's answer
    # can land a second late, and a fixed wait once read a *stale* readout (the phase-H
    # line came back in phase I on 2026-11-21). Counting the readout lines in the chat
    # window until one more appears is what makes "the newest reply" deterministic.
    local prev line i
    # `messages` lists the newest line first (`info()`'s `grep -m1` relies on it too);
    # the readout ends in `t=<game time>`, so a new one can never equal the last.
    prev=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Blood Infuser \[' || true)
    say "evilcraft infuser $1 $2 $3"
    line=$prev
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Blood Infuser \[' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no /evilcraft infuser reply for $1 $2 $3 (last: '${line}')"
    INF_LINE=$line
    [ -n "$INF_LINE" ] || fail "no /evilcraft infuser reply"
    eval "$(python3 "$PORT/tools/parse_infuser.py" "$INF_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the infuser line did not parse: $INF_LINE"
    echo "  $INF_LINE" >&2
}

# `/evilcraft load X Y Z SLOT ITEM`: put one item in a machine slot (`empty` clears it).
load_slot() { # x y z slot item
    say "evilcraft load $1 $2 $3 $4 $5"
    ctl wait 10 >/dev/null
}

# Take the product out of the result slot the way a player does: click it up, put it in
# an empty inventory slot, and check the inventory gained exactly that many.
take_output() { # item count
    local item=$1 count=$2 before after dst
    before=$(ctl inventory | jq -r --arg i "$item" '.counts[$i] // 0')
    open_menu "$IX" 101 0 7
    dst=$(ctl screen | jq -r '[.slots[] | select(.slot < 300 and .item == null) | .slot] | .[0] // ""')
    [ -n "$dst" ] || fail "no empty inventory slot to put $item in"
    ctl click-slot 302 >/dev/null || fail "the product did not leave the result slot"
    ctl wait 2 >/dev/null
    ctl click-slot "$dst" >/dev/null || fail "the product did not land in slot $dst"
    ctl close >/dev/null 2>&1 || true
    ctl wait 6 >/dev/null
    after=$(ctl inventory | jq -r --arg i "$item" '.counts[$i] // 0')
    [ "$after" -eq $((before + count)) ] \
        || fail "the inventory gained $((after - before)) of $item, expected $count"
    echo "  took $count x $item out of the result slot into inventory slot $dst" >&2
}

# One recipe, the way its file declares it: the input in the infusion slot, a full
# tank, the game's own ticks, then the product, the blood it cost and the input gone.
infuse() { # input output count cost ticks
    local input=$1 output=$2 count=$3 cost=$4 ticks=$5
    load_slot "$IX" 101 0 1 "$input"
    say "evilcraft blood $IX 101 0 $TANK_CAPACITY"
    ctl wait 10 >/dev/null
    ctl wait $((ticks + 60)) >/dev/null
    infuser "$IX" 101 0
    [ "$IN_ITEM" = "empty" ] || fail "$input was not consumed (slot holds $IN_ITEM@$IN_COUNT)"
    [ "$OUT_ITEM" = "$output" ] || fail "$input became $OUT_ITEM, expected $output"
    [ "$OUT_COUNT" = "$count" ] || fail "$input made $OUT_COUNT of $output, expected $count"
    local want=$((TANK_CAPACITY - cost))
    [ "$TANK" = "$want" ] || fail "$input left $TANK mB in the tank, expected $want ($cost spent)"
    echo "  $input -> $output x$count for $cost mB in $ticks ticks" >&2
    take_output "$output" "$count"
}

# Break a machine in survival and keep what the break reported: `BREAK` is `ctl break`'s
# reply (`block`, `drops`, `picked_up`, `items_left_on_ground`), which is the server's own
# count of what the loot tables rolled.
break_survival() { # x y z block
    local x=$1 y=$2 z=$3 block=$4 held name
    ctl command "execute in minecraft:the_end run tp @s $((x - 3)).5 101 $((z + 1)).5" >/dev/null
    ctl wait 6 >/dev/null
    held=$(ctl inventory | jq -r '.held.item // ""')
    [ "$held" = "minecraft:diamond_pickaxe" ] || fail "the pickaxe is not in hand: '$held'"
    BREAK=$(ctl break "$x" "$y" "$z") || fail "$block did not break"
    ctl wait 10 >/dev/null
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "minecraft:air" ] || fail "$block is still $name after mining"
    [ "$(jq -r --arg i "$block" '.drops[$i] // 0' <<<"$BREAK")" -ge 1 ] \
        || fail "the survival break of $block dropped nothing: $(jq -c '.drops' <<<"$BREAK")"
    echo "  break: $(jq -c '{block, drops, picked_up, items_left_on_ground}' <<<"$BREAK")" >&2
}

# `/data get entity @s SelectedItem`, waiting for the *new* reply: the chat is read newest
# first, and a fixed wait once read a stale line (the infuser's readout had the same fault).
# Sets `SELECTED_LINE` and `HELD_COMPONENTS` (the SNBT of the held stack's components).
selected_item_line() { # [what]
    local what=${1:-held} prev line i parsed
    prev=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'has the following entity data' || true)
    ctl command "data get entity @s SelectedItem" >/dev/null
    line=$prev
    for i in $(seq 1 25); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'has the following entity data' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] || fail "no /data get reply for the $what item"
    SELECTED_LINE=$line
    parsed=$(sed -n 's/.*entity data: \(.*\)$/\1/p' <<<"$line")
    [ -n "$parsed" ] || fail "the SelectedItem reply did not parse: $line"
    HELD_COMPONENTS=$parsed
    echo "  $what: $line" >&2
}

# Walk onto the drop, select it in the hotbar and read its components the way the server
# holds them: `minecraft:item`'s `components` in `ctl entities` is the *client's* mirror and
# does not carry a mod component (measured on this pin), so the hand is read with
# `/data get entity @s SelectedItem`.
pick_hold() { # x y z block
    local x=$1 y=$2 z=$3 block=$4 slot
    ctl collect 8 >/dev/null 2>&1 || true
    ctl wait 10 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $x.5 101 $z.5" >/dev/null
    ctl wait 30 >/dev/null
    slot=$(ctl inventory | jq -r --arg i "$block" '[.hotbar[] | select(.item == $i) | .slot] | .[0] // ""')
    [ -n "$slot" ] || fail "the dropped $block was not picked up: $(ctl inventory | jq -c '.hotbar')"
    ctl select $((slot + 1)) >/dev/null
    ctl wait 6 >/dev/null
    selected_item_line "the dropped $block"
}

# Place the held item at a position, then check the block is there.
place_held() { # x y z block
    local x=$1 y=$2 z=$3 block=$4 aim reply name
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 101 $((z + 1)).5" >/dev/null
    ctl wait 6 >/dev/null
    aim=$(ctl look-at-block "$x" $((101 - 1)) "$z" up 2>&1) \
        || fail "cannot aim at the floor under $x,101,$z: $aim"
    reply=$(ctl place "$x" 101 "$z" --item "$block" 2>&1) \
        || fail "ctl place at $x,101,$z failed: $(tr -d '\n' <<<"$reply" | cut -c1-300)"
    ctl wait 8 >/dev/null
    name=$(block_name "$x" 101 "$z")
    [ "$name" = "$block" ] || fail "$block did not place at $x,101,$z (got '$name')"
}

# `/data get block X Y Z fluids[0].Amount`: the block entity's own saved amount, not the
# clamped value a readout would print. Sets `SAVED_MB`. A path reply prints
# `<x>, <y>, <z> has the following block data: <value>` (`commands.data.block.query`).
saved_tank() { # x y z
    local line
    ctl command "data get block $1 101 $3 fluids[0].Amount" >/dev/null
    ctl wait 10 >/dev/null
    line=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'has the following block data' || true)
    [ -n "$line" ] || fail "no data get reply for the tank at $1,101,$3: $(ctl messages 6 | jq -c '.chat')"
    SAVED_MB=$(sed -n 's/.*has the following block data: \(-\{0,1\}[0-9][0-9]*\)$/\1/p' <<<"$line")
    [ -n "$SAVED_MB" ] || fail "the data get reply did not parse: $line"
    echo "  $line" >&2
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then
    [ -n "$CONTROL_GAME" ] || { echo "CONTROL=1 needs CONTROL_GAME" >&2; exit 2; }
    control_leg
fi
launch
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

echo "== A2: the diagnostic answers only for a Blood Chest =="
# A vanilla chest with saved data (a custom name) is a block entity that carries data.
# The old readout answered it like a Blood Chest and printed a tank and slots; it must
# now say the block is not one, so the diagnostic cannot read another mod's container.
ctl command "execute in minecraft:the_end run setblock $X0 101 4 minecraft:chest" >/dev/null
ctl wait 4 >/dev/null
ctl command "data merge block $X0 101 4 {CustomName:'\"other\"'}" >/dev/null
ctl wait 4 >/dev/null
say "evilcraft info $X0 101 4"
ctl wait 12 >/dev/null
FOREIGN=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Block Chest\|Blood Chest' || true)
case "$FOREIGN" in
    *"is not a Block Chest"*) echo "  a vanilla chest answers: $FOREIGN" ;;
    *) fail "the diagnostic answered a vanilla chest: '${FOREIGN}'" ;;
esac

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
open_menu "$SX" 101 0 11
put_in_slot 310 "$EXTRACTOR"
ctl close >/dev/null 2>&1 || true
ctl wait 10 >/dev/null
info "$SX" 101 0
[ "${CONTAINER_ITEM:-}" = "$EXTRACTOR" ] || fail "the container slot holds ${CONTAINER_ITEM:-nothing}, expected the extractor"
[ "$TANK" = "$HELD_BLOOD" ] || fail "the tank holds $TANK mB, expected the extractor's $HELD_BLOOD"
[ "${CONTAINER_BLOOD:-}" = 0 ] || fail "the extractor still holds ${CONTAINER_BLOOD} mB after emptying"
echo "  extractor $HELD_BLOOD mB -> tank $TANK mB, extractor $CONTAINER_BLOOD mB"

echo "== H: the Blood Infuser's own recipe files load, and an unknown id is refused =="
place_chest "$IX" 101 0 "$INFUSER"
infuser "$IX" 101 0
[ "$TANK" = 0 ] || fail "a fresh infuser holds $TANK mB, expected 0"
[ "$TIER" = 0 ] || fail "a fresh infuser is tier $TIER, expected 0"
[ "$CARRIED" = 9 ] || fail "the package claims $CARRIED carried recipes, expected 9"
[ "$RECIPES" = "$CARRIED" ] || fail "the infuser reads $RECIPES of its $CARRIED recipe files, so the recipe type did not load"
[ "$OUT_ITEM" = "empty" ] || fail "a fresh infuser holds $OUT_ITEM in its result slot"
echo "  block $(block_name "$IX" 101 0), $RECIPES/$CARRIED recipe files in force"
# An unknown id is refused rather than written as a nameless stack.
say "evilcraft load $IX 101 0 1 evilcraft:no_such_item"
ctl wait 12 >/dev/null
UNKNOWN=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'not an item this game has' || true)
[ -n "$UNKNOWN" ] || fail "an unknown item id was accepted: $(ctl messages 12 | jq -r '.chat[]' | tail -3)"
echo "  $UNKNOWN"

echo "== I: a tier-3 recipe waits for a tier-3 machine (the tier gate) =="
say "evilcraft blood $IX 101 0 $TANK_CAPACITY"
say "evilcraft load $IX 101 0 1 minecraft:bone"
ctl wait 560 >/dev/null
infuser "$IX" 101 0
[ "$TIER" = 0 ] || fail "the machine is tier $TIER before a promise"
[ "$IN_ITEM" = "minecraft:bone" ] || fail "the bone left the infusion slot ($IN_ITEM)"
[ "$OUT_ITEM" = "empty" ] || fail "a tier-0 machine infused the bone into $OUT_ITEM"
[ "$PROGRESS" = 0 ] || fail "a tier-0 machine sits at progress $PROGRESS"
[ "$TANK" = "$TANK_CAPACITY" ] || fail "a tier-0 machine spent $((TANK_CAPACITY - TANK)) mB"
echo "  560 ticks at tier 0: the bone is untouched, no product, $TANK mB left"

echo "== J: the menu's seven slots, and the promise that tiers the machine =="
ctl command "item replace entity @s hotbar.7 with $PROMISE 1" >/dev/null
ctl wait 4 >/dev/null
open_menu "$IX" 101 0 7
put_in_slot 303 "$PROMISE"
ctl close >/dev/null 2>&1 || true
ctl wait 12 >/dev/null
infuser "$IX" 101 0
[ "$TIER" = 3 ] || fail "the promise in the upgrade slot left the machine at tier $TIER"
echo "  the menu lists 7 slots; a Promise of Tenacity III in slot 303 makes the machine tier 3"

echo "== K: the bone is infused into a blaze rod for 2,500 mB in 500 ticks =="
TANK_BEFORE=$TANK
ctl wait 560 >/dev/null
infuser "$IX" 101 0
PRODUCED=1
if [ "$SELFTEST" = infuser ]; then
    echo "(selftest=infuser: asserting the bone was not infused)"
    PRODUCED=0
fi
[ "$PRODUCED" = 1 ] || fail "the bone was not infused into a blaze rod (out $OUT_ITEM, in $IN_ITEM, tank $TANK)"
[ "$OUT_ITEM" = "minecraft:blaze_rod" ] || fail "the bone became $OUT_ITEM, expected a blaze rod"
[ "$OUT_COUNT" = 1 ] || fail "the infusion made $OUT_COUNT blaze rods, expected 1"
[ "$IN_ITEM" = "empty" ] || fail "the bone survived the infusion ($IN_ITEM@$IN_COUNT)"
[ "$TANK" = "$((TANK_BEFORE - 2500))" ] \
    || fail "the infusion left $TANK mB, expected $((TANK_BEFORE - 2500)) (2,500 mB spent)"
echo "  500 ticks: tank $TANK_BEFORE -> $TANK mB, one blaze rod for one bone"
take_output minecraft:blaze_rod 1

echo "== L: every other carried recipe is crafted the way its file declares =="
infuse minecraft:wheat_seeds minecraft:nether_wart 1 5000 1000
infuse minecraft:sand minecraft:soul_sand 1 2500 1000
infuse minecraft:dirt minecraft:grass_block 1 350 300
infuse minecraft:cobblestone minecraft:netherrack 1 500 500
infuse minecraft:rotten_flesh minecraft:spider_eye 8 375 500
infuse minecraft:brown_mushroom minecraft:red_mushroom 1 2000 150
infuse minecraft:skeleton_skull minecraft:zombie_head 1 1000 500
infuse minecraft:zombie_head minecraft:creeper_head 1 5000 1000

echo "== M: the container slot empties a filled extractor into the tank =="
say "evilcraft blood $IX 101 0 0"
ctl wait 12 >/dev/null
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 9 >/dev/null
ctl wait 2 >/dev/null
say "evilcraft held 1000"
ctl wait 12 >/dev/null
HELD_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Blood Extractor held' || true)
HELD=$(sed -n 's/.*held: \([0-9][0-9]*\)\/[0-9][0-9]* mB.*/\1/p' <<<"$HELD_LINE")
[ "$HELD" = 1000 ] || fail "the held extractor holds $HELD mB, expected 1,000"
open_menu "$IX" 101 0 7
put_in_slot 300 "$EXTRACTOR"
ctl close >/dev/null 2>&1 || true
ctl wait 20 >/dev/null
infuser "$IX" 101 0
[ "$TANK" = 1000 ] || fail "the container slot moved $TANK mB into the tank, expected 1,000"
[ "$CONT_ITEM" = "$EXTRACTOR" ] || fail "the container slot holds $CONT_ITEM, expected the extractor"
[ "$CONT_BLOOD" = 0 ] || fail "the extractor still holds $CONT_BLOOD mB"
echo "  extractor 1,000 mB -> tank $TANK mB, extractor $CONT_BLOOD mB"

echo "== N: both machines break into themselves (loot tables and the mineable tag) =="
# In creative a broken block drops nothing (vanilla drops nothing for a creative
# player), so this phase drops to survival: the block's own loot table and its
# `minecraft:mineable/pickaxe` membership (the jar's tag) are what have to answer.
# Fresh blocks, so the machines above keep their promise, extractor and blood.
# A creative player's break drops nothing, so this phase is survival: wait for the
# game's own confirmation before believing a break.
ctl command "gamemode survival @s" >/dev/null
for _ in $(seq 1 10); do
    ctl wait 4 >/dev/null
    ctl messages 6 | jq -r '.chat[]' | grep -qi 'survival mode' && break
done
ctl messages 6 | jq -r '.chat[]' | grep -qi 'survival mode' \
    || fail "the game did not put the player in survival: $(ctl messages 4 | jq -c '.chat')"
IX2=$((IX + 2))
CX2=$((X0 + 4))
place_chest "$IX2" 101 0 "$INFUSER"
place_chest "$CX2" 101 2 "$CHEST"
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
break_into() { # x y z block
    local x=$1 y=$2 z=$3 block=$4 name held before after drops picked broken
    ctl command "execute in minecraft:the_end run tp @s $((x - 3)).5 101 $((z + 1)).5" >/dev/null
    ctl wait 6 >/dev/null
    held=$(ctl inventory | jq -r '.held.item // ""')
    [ "$held" = "minecraft:diamond_pickaxe" ] || fail "the pickaxe is not in hand: '$held'"
    before=$(ctl inventory | jq -r --arg i "$block" '.counts[$i] // 0')
    broken=$(ctl break "$x" "$y" "$z") || fail "$block did not break"
    ctl wait 12 >/dev/null
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "minecraft:air" ] || fail "$block is still $name after mining"
    # The break's own reply is the server's count of what the loot table rolled; the drop
    # may still be on the ground or already picked up, and either proves it.
    drops=$(jq -r --arg i "$block" '.drops[$i] // 0' <<<"$broken")
    after=$(ctl inventory | jq -r --arg i "$block" '.counts[$i] // 0')
    picked=$((after - before))
    [ "$((drops + picked))" -ge 1 ] \
        || fail "a survival break of $block dropped nothing: $(jq -c '.drops' <<<"$broken"), inventory $picked"
    echo "  $block: mined with a pickaxe, $drops dropped, $picked picked up" >&2
}
break_into "$IX2" 101 0 "$INFUSER"
break_into "$CX2" 101 2 "$CHEST"

echo "== O: a broken machine's item carries its blood tank =="
# Still in survival with the pickaxe, so the chest really drops. A fresh chest, 6,500 mB
# in it, and one stack of cobblestone in slot 0: the tank must travel on the item and the
# stack must NOT (the engine spills a mod container's stacks itself, so a payload that
# carried them would put every one of them in the world twice).
OX=$((X0 + 2))
ctl command "clear @s" >/dev/null
ctl wait 4 >/dev/null
place_chest "$OX" 101 -4 "$CHEST"
give_to "$OX" 101 -4 minecraft:cobblestone
say "evilcraft blood $OX 101 -4 6500"
ctl wait 12 >/dev/null
info "$OX" 101 -4
expect_tank 6500
[ "${SLOT0_ITEM:-}" = "minecraft:cobblestone" ] || fail "slot 0 holds ${SLOT0_ITEM:-nothing}, expected the cobblestone"
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
break_survival "$OX" 101 -4 "$CHEST"
pick_hold "$OX" 101 -4 "$CHEST"
eval "$(python3 "$PORT/tools/parse_components.py" "$HELD_COMPONENTS" "$CHEST")"
CARRIED=${carried:-none}
if [ "$SELFTEST" = tank ]; then
    echo "(selftest=tank: asserting the broken chest carried no tank)"
    CARRIED=none
fi
[ "${ok:-0}" = 1 ] || fail "the dropped chest carries no block entity data: '$HELD_COMPONENTS'"
[ "$CARRIED" = 6500 ] || fail "the dropped chest carries $CARRIED mB, expected 6500"
[ "$fluid" = "evilcraft:blood" ] || fail "the dropped chest carries '$fluid', expected evilcraft:blood"
[ "$items" = 0 ] || fail "the payload carries the chest's own stacks ($items Items key(s)) as well as the tank, so they would double-carry"
echo "  the picked-up chest carries $CARRIED mB of $fluid in $carrier, $items stack(s)" >&2
# The cobblestone left the chest as its own item entity, which is why the payload is the
# tank alone.
SPILLED=$(jq -r '.items_left_on_ground // 0' <<<"$BREAK")
[ "$SPILLED" -ge 1 ] || fail "the engine did not spill the chest's cobblestone, so the empty payload slot proves nothing"
echo "  the cobblestone the chest held spilled on its own ($SPILLED item(s) left on the ground)" >&2
place_held "$OX" 101 -8 "$CHEST"
info "$OX" 101 -8
expect_tank 6500
[ "$CAPACITY" = "$TANK_CAPACITY" ] || fail "the replaced chest reports a $CAPACITY mB tank"
[ -z "${SLOT0_ITEM:-}" ] || fail "the replaced chest came back holding ${SLOT0_ITEM}, so its stacks were carried twice"
echo "  placed again: tank $TANK/$CAPACITY mB, slots empty"

echo "== P: an empty chest carries nothing, and a doctored payload is clamped =="
# The same block, the same break, a tank with nothing in it: the item carries no payload
# at all. That is what a break whose block entity the listener could not read leaves
# behind, and it is where the tank in O can only have come from the listener.
PX=$((X0 + 6))
ctl command "clear @s" >/dev/null
ctl wait 4 >/dev/null
place_chest "$PX" 101 -4 "$CHEST"
info "$PX" 101 -4
expect_tank 0
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
break_survival "$PX" 101 -4 "$CHEST"
pick_hold "$PX" 101 -4 "$CHEST"
if python3 "$PORT/tools/parse_components.py" "$HELD_COMPONENTS" "$CHEST" | grep -q '^ok=1$'; then
    fail "an empty chest's item carries a block entity data payload: $HELD_COMPONENTS"
fi
echo "  an empty chest's item carries no block entity data: '$HELD_COMPONENTS'" >&2
place_held "$PX" 101 -8 "$CHEST"
info "$PX" 101 -8
expect_tank 0
echo "  placed again: tank $TANK/$CAPACITY mB"
# A hand-written payload past the capacity: the item asks for 20,000 mB and the machine
# can hold 10,000, so the placement clamps it — in the readout and in the saved data.
ctl command "clear @s" >/dev/null
ctl command "give @s $CHEST[minecraft:block_entity_data={id:\"$CHEST\",fluids:[{Fluid:\"evilcraft:blood\",Amount:20000}]}]" >/dev/null
ctl wait 8 >/dev/null
DOCTORED=$(ctl inventory | jq -r --arg i "$CHEST" '[.hotbar[] | select(.item == $i) | .slot] | .[0] // ""')
[ -n "$DOCTORED" ] || fail "the doctored chest is not in the hotbar: $(ctl inventory | jq -c '.hotbar')"
ctl select $((DOCTORED + 1)) >/dev/null
ctl wait 4 >/dev/null
selected_item_line "the doctored chest"
eval "$(python3 "$PORT/tools/parse_components.py" "$HELD_COMPONENTS" "$CHEST")"
[ "${carried:-none}" = 20000 ] || fail "the doctored item asks for ${carried:-none} mB, not 20000"
echo "  the doctored item asks for $carried mB of $fluid" >&2
place_held "$PX" 101 -12 "$CHEST"
info "$PX" 101 -12
expect_tank "$TANK_CAPACITY"
saved_tank "$PX" 101 -12
[ "$SAVED_MB" = "$TANK_CAPACITY" ] \
    || fail "the placed chest's own tank holds $SAVED_MB mB, expected the clamped $TANK_CAPACITY"
echo "  20,000 mB asked by the item, $TANK mB in the machine (saved $SAVED_MB)" >&2

echo "PASS: an empty chest reads 0/10,000 mB, an empty tank repairs nothing, 1,000 mB repairs one damage point every two ticks for exactly 5 mB a point, a chipped anvil becomes an anvil for 125 mB, a player kill fills the carried Blood Extractor with a 20 HP zombie's 100..799 mB, the extractor in a container slot empties into the tank, the Blood Infuser reads all 9 of its carried recipe files and refuses an unknown id, holds a tier-3 recipe at tier 0 and infuses it once a Promise of Tenacity III tiers the machine, crafts every carried recipe for its own blood cost, empties a filled extractor into its tank, breaks back into itself, and a chest holding 6,500 mB is broken into an item that carries that tank (and none of its stacks), places back with 6,500 mB, while an empty chest carries nothing and a 20,000 mB payload is clamped to 10,000"
