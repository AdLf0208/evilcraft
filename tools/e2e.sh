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
# Slice 4 — projectile kills give blood (`loot_drops`, upstream's
# `event.getSource().getEntity() instanceof ServerPlayer`):
#  Q  a 20 HP zombie spawned at range is killed with the player's bow: the carried
#     extractor takes its 100..799 mB (before this slice an arrow kill filled nothing,
#     because `mob_killed` heard melee, sweep and spear only);
#  R  the same extractor, emptied, stays empty when the zombie is killed by a *skeleton's*
#     arrow: the arrow's `Owner` is read back as the skeleton's UUID, the zombie dies, and
#     no player credit means no blood (the rule that excludes a mob's kill).
#
# Slice 5 — the Blood Stain (`BlockBloodStain`, `BlockEntityBloodStain`):
#  S  a stain placed from its own item reads `0/5000 mB`; with its tank set to 3,500 and a
#     carried extractor holding 4,000, a **crouching** right-click moves exactly the 1,000 mB
#     the extractor had room for (stain 3,500 -> 2,500, extractor 4,000 -> 5,000 — one
#     transfer, `ItemBloodExtractor.onItemUseFirst`);
#  T  the same click **standing** moves nothing (extractor 4,000, stain 3,500), the same
#     click crouching then does move (the crouch rule's own differential), and the same
#     crouching click on `minecraft:stone` moves nothing (the block rule's differential);
#  U  a stain drained to 0 removes its block (`BlockEntityBloodStain.setAmount`), and the
#     jar's `minecraft:replaceable` entry is measured as inert at this pin (a stone placed
#     onto a stain lands above it).
#
# Slice 6 — a fall death leaves a stain (`BlockBloodStain.bloodStainedBlockEvent`):
#  V  a 20 HP zombie with 1 HP dropped down a one-cell shaft dies of the fall and the cell it
#     died in becomes `evilcraft:blood_stain` holding 20 x 20 = **400 mB** (the max health is
#     the field, so 400 and not the 20 a current-health reading would give), asserted in
#     `/evilcraft stain` and in the block entity's own saved `fluids`;
#  W  the same cell, the same zombie and a diamond sword instead of the fall: the cell stays
#     `minecraft:air` (the death's differential — only a fall death stains);
#  X  a second fall death on the stain V left: **800 mB** in the same cell, and no second
#     stain above it (the counterfactual: a top-up, not a new block).
#
# Slice 7 — the Colossal Blood Chest (`BlockColossalBloodChest`, upstream's hollow cube):
#  Y  a hollow 3x3x3 shell of `evilcraft:reinforced_undead_planks` with exactly one
#     `evilcraft:colossal_blood_chest` in its wall, the middle cell air, forms one machine:
#     `/evilcraft colossal` reads `formed=true`, a 10,000 mB tank and its 46 slots (45 repair
#     slots and the container slot); a damage-250 diamond pickaxe in slot 0 with 1,000 mB
#     repairs one damage point every two ticks for 5 mB (20 points and 100 mB in 40 ticks),
#     and keeps going until the tank is empty (200 points for the 1,000 mB);
#  Z  the counterfactual: one shell block removed reads `formed=false` and the same pickaxe
#     and tank repair nothing over the same 40 ticks; the container slot still empties a
#     filled extractor into the tank while the shell is open (the menu lists all 50 slots),
#     and putting the plank back forms the cube again and the repair resumes;
#  AA the differential: the *same* damage-250 pickaxe, the same 1,000 mB and the same 40
#     ticks repair at the same rate in a plain `evilcraft:blood_chest`;
#  AB the controller breaks into itself in survival with an axe and its own item carries the
#     blood tank (`copy_tank_data` on the third machine), and a plank drops itself.
#
# Slice 8 — the Dark Tank (`BlockDarkTank`, `BlockEntityDarkTank`):
#  AC a fresh `evilcraft:dark_tank` reads `0/16000 mB`; with 14,000 mB in it a crouching click
#     with a full 5,000 mB extractor pours exactly the 2,000 mB room it had (tank 16,000, the
#     extractor keeping 3,000), and `/data get block … fluids` reads the saved 16,000;
#  AD the same click **standing** pours nothing, the same crouching click on `minecraft:stone`
#     pours nothing, and on a fresh tank the same crouching click pours the whole 5,000;
#  AE in survival the tank breaks into its own item carrying 12,000 mB (the machines' clamp
#     would cut that to 10,000) and places back reading `12000/16000 mB`.
#
# Slice 9 — the Purifier's curse action (`ToolBadEnchantPurifyAction`) and the Blook:
#  AF a fresh `evilcraft:purifier` reads `0/3000 mB` with an empty purge slot; a **standing**
#     click with a `/give`n cursed diamond pickaxe puts exactly one of it in the purify slot,
#     and the block entity's own saved `Items[0]` carries `minecraft:enchantments` with
#     `minecraft:vanishing_curse` (the positive control of the curse assertion);
#  AG with the tank full, 140 ticks run the 60-tick action: the purge slot's saved item
#     carries **no** `minecraft:enchantments` any more and the tank fell by exactly one bucket
#     (3,000 -> 2,000), while the item itself stays in the slot (the action strips, never
#     consumes);
#  AH the differential: the same cursed pickaxe in a working `evilcraft:blood_chest` for the
#     same 140 ticks still carries the curse and the chest spends nothing (an undamaged tool
#     is not a repair) — the purifier's result is its own rule;
#  AI the clicks in the other directions: an empty-hand right-click takes the pickaxe back out
#     of the purge slot, a standing click with a filled extractor puts the extractor in the
#     slot (and pours nothing), and once it is back in hand a **crouching** click pours exactly
#     the 1,000 mB room the tank had (extractor 5,000 -> 4,000, the saved `fluids` 3,000).
#
# The Blook (`evilcraft:blook`) is carried as an item and its own infusion recipe: phase L
# crafts it through the Blood Infuser (`minecraft:book` -> `evilcraft:blook` for 500 mB in 500
# ticks at tier 1), which is also what proves the recipe's keys resolve to items this package
# registers.
#
# Slice 10 (0.2.10) — the three review findings:
#  D2 a stack of two chipped anvils (a legal vanilla stack, custom-named) in a Blood Chest:
#     125 mB leaves it untouched, 250 mB turns **both** into anvils for 2 x 125 mB with the
#     custom name kept (0.2.7/0.2.8 replaced the stack with one anvil and dropped the rest);
#  D3 a damaged, enchanted pickaxe is repaired and keeps its enchantment and its one item;
#  E2 a legacy stack of two Blood Extractors — written into the save by the 0.2.9 package
#     (`legacy_leg`: its item had no `max_stack_size`, so one stack holds the amount once) — is
#     resolved before any drain: two single extractors, the first keeping the whole 4,000 mB and
#     the second 0, and a crouching click on a 3,500 mB stain fills only the first (stain 2,500,
#     held 5,000, the other still 0);
#  Y2 the Colossal Blood Chest does the same anvil step as D2 (two anvils for 250 mB, name
#     kept; 125 mB leaves the stack alone).
#
# Slice 11 (0.2.11) — the Vengeance Pickaxe's immunity (`ToolBadEnchantPurifyAction`):
#  AJ the carried `evilcraft:vengeance_pickaxe` is /give-n with a vanilla curse, put in the
#     Purifier's slot with a standing click, and read back with **1 enchantment, 1 curse** and
#     a full 3,000 mB tank after 140 ticks (the clock reset; `canWork` refused it); the same
#     rig with a cursed vanilla diamond pickaxe strips the curse and spends one bucket, which
#     is the item-id differential. An unknown id is refused by `/evilcraft load`.
#
# Slice 12 (0.2.12) — the tank blocks' analogue (comparator) output:
#  AK a comparator facing the Purifier is `powered=true` and a wire in front of it reads 15 at
#     3,000/3,000 mB, **10** at 2,000/3,000 (upstream's ceil(15 x amount/capacity); the dispatch
#     note's 9 is an arithmetic slip) and 9 at 1,800/3,000; re-reads happen without nudging (the
#     port's OutputSignal notification). A comparator facing `minecraft:stone` reads 0.
#
# The previous package (0.2.3) is the third leg: the same bow kill with the same package's
# `/evilcraft held` reads 0 mB, which is the gap slice 4 closes. `OLD_GAME` names that
# game (the launcher's export of the 0.2.3 build).
#
# The operator gate on `/evilcraft info|infuser|stain|colossal|darktank` is unit-tested
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
# CONTROL=1 (the launcher exports `CONTROL_GAME`) runs the whole scenario on the same Nightly
# **without** the package, where the chest item never registers and the run must fail.
#
# SELFTEST=1 flips C's assertion, SELFTEST=infuser flips K's, SELFTEST=tank flips O's,
# SELFTEST=blood flips Q's, SELFTEST=stain flips S's, SELFTEST=fall flips V's,
# SELFTEST=colossal flips Y's `formed` assertion, SELFTEST=darktank flips AC's pour,
# SELFTEST=purifier flips AG's curse-gone assertion, SELFTEST=vengeance flips AJ's immunity,
# SELFTEST=anvil flips D2's anvil count and SELFTEST=extractor flips E2's split,
# SELFTEST=comparator flips AK's 2,000 mB reading, and SELFTEST=curse flips AL's roll-0
# assertion (each must FAIL), proving the scenarios can fail.
# Phase I is the
# infuser's own differential: the same machine, the same bone and a full tank do nothing at
# tier 0 and infuse at tier 3. Phase P is the tank's differential: a chest broken with nothing
# in its tank drops an item that carries no payload, which is exactly what a break the listener
# never hears leaves behind. Phase R is the kill's differential: the same extractor, the same
# mob and a lethal arrow, whose shooter is a skeleton rather than the player. Phase Z is the
# Colossal Blood Chest's counterfactual (one shell block out: no repair) and phase AA its
# differential (the same pickaxe and tank in a plain Blood Chest repair at the same rate).
# Phase AD is the Dark Tank's differential: a standing click and a crouching click on stone
# pour nothing, while the same crouching click on the tank pours its 5,000 mB.
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
# The launcher's export of the previous package (0.2.3): the bow-kill counterfactual.
OLD_GAME=${OLD_GAME:-${GAME:+$(dirname "$GAME")/../evilcraft-0.2.3/horizon-client}}
# The launcher's export of the previous package (0.2.9): it had no `max_stack_size`, so it
# writes the legacy stack of two extractors this package has to resolve. Kept beside the 0.2.10
# export (`cp -al evilcraft evilcraft-0.2.9` before a rebuild), like the 0.2.3 one.
LEGACY_GAME=${LEGACY_GAME:-${GAME:+$(dirname "$GAME")/../evilcraft-0.2.9/horizon-client}}
LEGACY=${LEGACY:-$([ "${CONTROL:-0}" = 1 ] && echo 0 || echo 1)}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/evilcraft-e2e.sock}
SELFTEST=${SELFTEST:-0}
# The previous package's leg runs with the default scenario only (it is a second launch and
# the control/selftest logs are about their own phases).
OLD=${OLD:-$([ "$SELFTEST" = 0 ] && [ "${CONTROL:-0}" = 0 ] && echo 1 || echo 0)}
KEEP=${KEEP:-0}
WORLD=EvilCraftE2E
OLD_WORLD=EvilCraftOld
# Where the arena stands: the End's void, far from the central island.
X0=200
SX=$((X0 + 2))
IX=$((X0 + 6))
CHEST=evilcraft:blood_chest
INFUSER=evilcraft:blood_infuser
STAIN=evilcraft:blood_stain
COLOSSAL=evilcraft:colossal_blood_chest
DARK_TANK=evilcraft:dark_tank
PURIFIER=evilcraft:purifier
PLANKS=evilcraft:reinforced_undead_planks
EXTRACTOR=evilcraft:blood_extractor
PROMISE=evilcraft:promise_tier_3
TANK_CAPACITY=10000
STAIN_TANK=5000
DARK_TANK_CAPACITY=16000
PURIFIER_TANK=3000
# A tool the game itself gives, with one curse on it (`ToolBadEnchantPurifyAction`'s input).
# 26.3's component is the **bare map**: the 1.21.1 shape `{levels:{…}}` is refused by this
# build's `/give` ("Failed to get element minecraft:levels from registry minecraft:enchantment"),
# and SNBT's unquoted form allows no colon, so the enchantment id is quoted.
CURSED_PICKAXE='minecraft:diamond_pickaxe[minecraft:enchantments={"minecraft:vanishing_curse":1}]'
# Slice 11: the carried Vengeance Pickaxe (the item whose curses `ToolBadEnchantPurifyAction`
# refuses to strip). The vanilla curse makes the purifier's own curse path run for it.
VENGEANCE_PICKAXE=evilcraft:vengeance_pickaxe
CURSED_VENGEANCE='evilcraft:vengeance_pickaxe[minecraft:enchantments={"minecraft:vanishing_curse":1}]'
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

# Slice 12 — reading a comparator at this pin. A comparator has NO block entity here (its output
# lives in the level's `comparator_outputs` map), so `data get block … OutputSignal` has nothing to
# read. Its output is measured where the game itself uses it: a redstone wire on its output side
# carries exactly its signal as `power`, and the comparator's own `powered` flag is read too.
# `ctl block` answers `name` and `state` (`name[prop=value,…]`), not a property object, so parse
# the state string.
block_prop() { # block_prop X Y Z NAME
    ctl block "$1" "$2" "$3" | jq -r --arg s "$4" '
        ((.state // "") | [scan("([a-z_]+)=([a-z0-9_]+)")] | map({key: .[0], value: .[1]}) | from_entries | .[$s]) // ""'
}
# Both helpers take the MACHINE's position: the wire is two cells north of it, the comparator
# one. (Powah's versions take the comparator's position instead; keep this convention consistent.)
cmp_signal() { block_prop "$1" "$2" "$(($3 - 2))" power; }   # wire north of the comparator
cmp_powered() { block_prop "$1" "$2" "$(($3 - 1))" powered; } # the comparator itself
# comparator_rig X Y Z: a comparator facing SOUTH at the machine's north face (z-1) reading it,
# and a wire at z-2 carrying the output. One redstone-block nudge wakes it at setup; after that
# the port's `update_neighbors(.., OutputSignal)` is the only thing that re-reads a tank change.
comparator_rig() { # comparator_rig X Y Z
    ctl command "execute in minecraft:the_end run setblock $1 $2 $(($3 - 1)) minecraft:air" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $1 $2 $(($3 - 2)) minecraft:air" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $1 $2 $(($3 - 1)) minecraft:comparator[facing=south]" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $1 $2 $(($3 - 2)) minecraft:redstone_wire" >/dev/null
    ctl wait 3 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $1 $(($2 + 1)) $(($3 - 1)) minecraft:redstone_block" >/dev/null
    ctl wait 3 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $1 $(($2 + 1)) $(($3 - 1)) minecraft:air" >/dev/null
    ctl wait 10 >/dev/null
    [ "$(block_name "$1" "$2" "$(($3 - 1))")" = minecraft:comparator ] ||
        fail "the comparator did not place at $1,$2,$(($3 - 1))"
    [ "$(block_name "$1" "$2" "$(($3 - 2))")" = minecraft:redstone_wire ] ||
        fail "the wire did not stay at $1,$2,$(($3 - 2))"
}

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
    # The floor runs six cells west of X0: `break_survival` stands a player at `x - 3`, and the
    # Colossal Blood Chest's controller sits at `X0 - 2`, so its breaks are made from `X0 - 5` —
    # outside a floor that started at `X0 - 2`, where the player fell into the void and the
    # break sometimes missed (measured 2026-11-23, phase AB).
    ctl command "execute in minecraft:the_end run fill $((X0 - 6)) 100 -14 $((X0 + 8)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 6)) 101 -14 $((X0 + 8)) 108 14 minecraft:air" >/dev/null
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
    # The End spawns endermen, and one standing in the cell makes `ctl place` answer "the block
    # did not change (blocked by an entity)" (measured 2026-11-23 in phase AC; `place_stain`
    # has cleared them for the same reason since slice 5).
    ctl command "execute in minecraft:the_end run kill @e[type=!player,distance=..8]" >/dev/null 2>&1 || true
    ctl wait 6 >/dev/null
    ctl command "item replace entity @s hotbar.0 with $item 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    local aim reply name attempt
    aim=$(ctl look-at-block "$x" $((y - 1)) "$z" up 2>&1) \
        || fail "cannot aim at the floor under $x,$y,$z: $aim"
    # `ctl place` reports the block two ticks after its click, so a click the client dropped (or
    # the server applied a tick late) reads as "the block did not change": the same fault
    # `place_stain` retries around. Three attempts, then the plain right-click a player makes.
    for attempt in 1 2 3; do
        if reply=$(ctl place "$x" "$y" "$z" --item "$item" 2>&1); then
            [ "$(block_name "$x" "$y" "$z")" = "$item" ] && break
        fi
        echo "  ctl place attempt $attempt did not land yet: $(tr -d '\n' <<<"$reply" | cut -c1-200)" >&2
        ctl wait 20 >/dev/null
    done
    name=$(block_name "$x" "$y" "$z")
    if [ "$name" != "$item" ]; then
        echo "  falling back to a plain right-click on the floor below" >&2
        ctl look-at-block "$x" $((y - 1)) "$z" up >/dev/null 2>&1 || true
        ctl wait 4 >/dev/null
        ctl interact "$x" $((y - 1)) "$z" up >/dev/null 2>&1 || true
        ctl wait 20 >/dev/null
        name=$(block_name "$x" "$y" "$z")
    fi
    [ "$name" = "$item" ] || fail "$item did not place at $x,$y,$z (got '$name')"
}

# `ctl container X Y Z ITEM COUNT`: fills the block's first empty slot.
give_to() { # give_to X Y Z ITEM
    ctl container "$1" "$2" "$3" "$4" 1 >/dev/null
    ctl wait 6 >/dev/null
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

# `/evilcraft stain X Y Z`: the line, then the parsed fields in TANK, CAPACITY and TIME.
stain() { # stain X Y Z
    # Wait for the reply itself: the chat is read newest first, and the readout ends in
    # `t=<game time>`, so a new one can never equal the previous one.
    local prev line i
    prev=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Blood Stain \[' || true)
    say "evilcraft stain $1 $2 $3"
    line=$prev
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Blood Stain \[' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no /evilcraft stain reply for $1 $2 $3 (last: '${line}')"
    STAIN_LINE=$line
    eval "$(python3 "$PORT/tools/parse_stain.py" "$STAIN_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the stain line did not parse: $STAIN_LINE"
    echo "  $STAIN_LINE" >&2
}

# `/evilcraft colossal X Y Z`: the line, then the parsed fields in FORMED, TANK,
# CAPACITY, REPAIR, SLOTS, CONTAINER_ITEM, CONTAINER_BLOOD and SLOT<n>_ITEM / SLOT<n>_DAMAGE.
colossal() { # colossal X Y Z
    # Wait for the reply itself: the chat is read newest first and the readout ends in
    # `t=<game time>`, so a new one can never equal the previous one.
    local prev line i
    prev=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Colossal Blood Chest \[' || true)
    say "evilcraft colossal $1 $2 $3"
    line=$prev
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Colossal Blood Chest \[' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no /evilcraft colossal reply for $1 $2 $3 (last: '${line}')"
    COLOSSAL_LINE=$line
    # A readout replaces the previous one: a stack that has left slot 0 must not still read
    # as there because an earlier eval named it (`info()`'s own guard).
    unset "${!SLOT@}" 2>/dev/null || true
    eval "$(python3 "$PORT/tools/parse_colossal.py" "$COLOSSAL_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the colossal line did not parse: $COLOSSAL_LINE"
    echo "  $COLOSSAL_LINE" >&2
}

# `/evilcraft darktank X Y Z`: the line, then the parsed fields in TANK, CAPACITY and TIME.
darktank() { # darktank X Y Z
    # Wait for the reply itself: the chat is read newest first and the readout ends in
    # `t=<game time>`, so a new one can never equal the previous one.
    local prev line i
    prev=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Dark Tank \[' || true)
    say "evilcraft darktank $1 $2 $3"
    line=$prev
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Dark Tank \[' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no /evilcraft darktank reply for $1 $2 $3 (last: '${line}')"
    DARKTANK_LINE=$line
    eval "$(python3 "$PORT/tools/parse_darktank.py" "$DARKTANK_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the dark tank line did not parse: $DARKTANK_LINE"
    echo "  $DARKTANK_LINE" >&2
}

# `/evilcraft purifier X Y Z`: the line, then the parsed fields in TANK, CAPACITY,
# PURGE_ITEM, PURGE_COUNT, ADD_ITEM, ADD_COUNT, ENCH, CURSES, TICK and TIME.
purifier() { # purifier X Y Z
    # Wait for the reply itself: the chat is read newest first and the readout ends in
    # `t=<game time>`, so a new one can never equal the previous one.
    local prev line i
    prev=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Purifier \[' || true)
    say "evilcraft purifier $1 $2 $3"
    line=$prev
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 40 | jq -r '.chat[]' | grep -m1 'Purifier \[' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no /evilcraft purifier reply for $1 $2 $3 (last: '${line}')"
    PURIFIER_LINE=$line
    eval "$(python3 "$PORT/tools/parse_purifier.py" "$PURIFIER_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the purifier line did not parse: $PURIFIER_LINE"
    echo "  $PURIFIER_LINE" >&2
}

# Place a stain from its own item on the arena floor below X Y Z.
# `place_chest` is the chests' helper; a stain is 1px high, and `ctl place` reports the
# block two ticks after its click, so a click the client dropped (or the server applied a
# tick late) reads as "the block did not change". This helper measures that instead of
# assuming it: it re-reads after 20 ticks, and only then falls back to a plain right-click
# on the floor below — which is the same click a player makes — printing what it saw.
place_stain() { # x y z
    local x=$1 y=$2 z=$3 reply name
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 101 $z.5" >/dev/null
    ctl wait 6 >/dev/null
    # A mob standing in the cell blocks a placement ("the block did not change (blocked by an
    # entity)"): the End spawns endermen, so clear the arena around the player first, as
    # `arena` does at the start of the run.
    ctl command "execute in minecraft:the_end run kill @e[type=!player,distance=..8]" >/dev/null 2>&1 || true
    ctl wait 6 >/dev/null
    ctl command "item replace entity @s hotbar.0 with $STAIN 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    ctl look-at-block "$x" 100 "$z" up >/dev/null 2>&1 || true
    if ! reply=$(ctl place "$x" 101 "$z" --item "$STAIN" 2>&1); then
        echo "  ctl place did not land yet: $(tr -d '\n' <<<"$reply" | cut -c1-200)" >&2
        ctl wait 20 >/dev/null
        name=$(block_name "$x" 101 "$z")
        if [ "$name" != "$STAIN" ]; then
            echo "  after 20 ticks the cell is still '$name'" >&2
            echo "  state: $(ctl state | jq -c '{screen, crosshair}')" >&2
            echo "  held: $(ctl inventory | jq -c '.held')" >&2
            echo "  floor: $(ctl block "$x" 100 "$z" | jq -c '{state}')" >&2
            echo "  entities: $(ctl entities 6 | jq -c '[.entities[] | .type]')" >&2
            echo "  falling back to a plain right-click on the floor below" >&2
            ctl look-at-block "$x" 100 "$z" up >/dev/null 2>&1 || true
            ctl wait 4 >/dev/null
            ctl interact "$x" 100 "$z" up >/dev/null 2>&1 || true
            ctl wait 20 >/dev/null
        else
            echo "  the click landed, only the two-tick report was early" >&2
        fi
    fi
    name=$(block_name "$x" 101 "$z")
    [ "$name" = "$STAIN" ] || fail "$STAIN did not place at $x,101,$z (got '$name')"
    echo "  placed $STAIN at $x,101,$z" >&2
}

# Aim at a point inside the block's own shape, and prove the crosshair is on it.
#
# `ctl look-at-block` aims at a *cell's* centre, which for a one-pixel block like a stain is
# above the shape (it stops at y+0.0625), so the ray sails over it and the aim is refused
# ("no face of block ... is visible"). `ctl interact` re-aims the same way, so a stain is
# clicked with a raw point aim and the mouse, exactly as a player aims at the visible splash.
aim_at() { # x y z height expected-state
    local x=$1 y=$2 z=$3 h=$4 want=$5 look hit
    local cx cy cz
    # The cell's centre is `+0.5` in every axis — not string concatenation, which puts a
    # negative coordinate half a block off (`-10.5` is the centre of cell `-11`).
    read -r cx cy cz <<<"$(awk -v x="$x" -v y="$y" -v z="$z" -v h="$h" \
        'BEGIN { printf "%.4f %.4f %.4f", x + 0.5, y + h, z + 0.5 }')"
    look=$(ctl look-at "$cx" "$cy" "$cz" 2>&1) \
        || fail "cannot aim at $x,$y,$z (point $cx,$cy,$cz): $look"
    local i
    for i in $(seq 1 10); do
        ctl wait 2 >/dev/null
        hit=$(ctl looking-at | jq -r '.block.state // ""')
        [ "$hit" = "$want" ] && return 0
    done
    fail "the crosshair is on '$hit', not on $want at $x,$y,$z (aimed at $cx,$cy,$cz)"
}

# Right-click at the current aim, the way a player's mouse does (`ctl interact` would
# re-aim, and cannot aim at a one-pixel block).
click_mouse() {
    ctl mouse right click >/dev/null || fail "the right click did not run"
    ctl wait 8 >/dev/null
}

# Crouch, right-click the block at X Y Z, and stand up again: a player's shift-click.
crouch_click() { # x y z [height] [expected-state]
    local x=$1 y=$2 z=$3 h=${4:-.03} want=${5:-$STAIN}
    aim_at "$x" "$y" "$z" "$h" "$want"
    ctl key shift down >/dev/null
    ctl wait 2 >/dev/null
    click_mouse
    ctl key shift up >/dev/null
    ctl wait 8 >/dev/null
    echo "  crouch + right-click on $x,$y,$z (aim on $want)" >&2
}

# Right-click the block at X Y Z standing up (no crouch).
stand_click() { # x y z [height] [expected-state]
    local x=$1 y=$2 z=$3 h=${4:-.03} want=${5:-$STAIN}
    aim_at "$x" "$y" "$z" "$h" "$want"
    click_mouse
    ctl wait 4 >/dev/null
    echo "  right-click on $x,$y,$z (aim on $want)" >&2
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
break_survival() { # x y z block [tool]
    local x=$1 y=$2 z=$3 block=$4 tool=${5:-minecraft:diamond_pickaxe} held name
    ctl command "execute in minecraft:the_end run tp @s $((x - 3)).5 101 $((z + 1)).5" >/dev/null
    ctl wait 6 >/dev/null
    held=$(ctl inventory | jq -r '.held.item // ""')
    [ "$held" = "$tool" ] || fail "the $tool is not in hand: '$held'"
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

# `/data get block X Y Z Items[SLOT]`: the block entity's own saved stack with its components,
# which is the *server's* view of the item (a mod container saves its stacks under `Items`).
# Sets `SAVED_ITEM`. Waits for a reply newer than the previous one, as every readout does.
saved_item() { # x y z slot
    local slot=$4 prev line i
    prev=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'has the following block data' || true)
    ctl command "data get block $1 101 $3 Items[$slot]" >/dev/null
    line=$prev
    for i in $(seq 1 25); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'has the following block data' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] \
        || fail "no data get reply for Items[$slot] at $1,101,$3 (last: '${line}')"
    SAVED_ITEM=$line
    echo "  $line" >&2
}

# The player's main hand is genuinely empty, and the client knows it: `clear @s` alone leaves
# the client's own held stack stale, and a right-click with a stale stack sends no use packet
# at all (measured 2026-11-23, the probe's empty-hand click after a `clear`). Selecting an empty
# hotbar slot is the player action that syncs it.
empty_hand() {
    ctl command "clear @s" >/dev/null
    ctl wait 8 >/dev/null
    ctl select 2 >/dev/null
    ctl wait 8 >/dev/null
    local held
    held=$(ctl inventory | jq -r '.held.item // ""')
    [ -z "$held" ] || fail "the hand still holds $held after clearing it"
}

# `/evilcraft held`, waiting for the *new* reply: the chat is read newest first, so a fixed
# wait can read the previous readout (the infuser's readout had the same fault). Sets
# `HELD_BLOOD`. The budget is 12 s of ticks: a busy server once answered `/evilcraft held`
# five seconds late and a 5 s budget read the previous line (measured 2026-11-22).
held_blood() { # [what]
    local what=${1:-held} prev line i
    prev=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'Blood Extractor held' || true)
    say "evilcraft held"
    line=$prev
    for i in $(seq 1 60); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 'Blood Extractor held' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] || fail "no /evilcraft held reply for the $what extractor"
    HELD_BLOOD=$(sed -n 's/.*held: \([0-9][0-9]*\)\/[0-9][0-9]* mB.*/\1/p' <<<"$line")
    [ -n "$HELD_BLOOD" ] || fail "the held line did not parse: $line"
    echo "  $line" >&2
}

# Carry a bow (hotbar 1), arrows (hotbar 2) and a Blood Extractor (hotbar 9), bow in hand.
arm_bow() {
    ctl command "clear @s" >/dev/null
    ctl command "item replace entity @s hotbar.0 with minecraft:bow 1" >/dev/null
    ctl command "item replace entity @s hotbar.1 with minecraft:arrow 32" >/dev/null
    ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    [ "$(ctl inventory | jq -r '.held.item')" = "minecraft:bow" ] \
        || fail "the bow is not in hand: $(ctl inventory | jq -r '.held.item')"
    # Resistance 5 (no damage) for the whole kill: the mob has AI and walks in, and a
    # survival player must not die to it mid-phase.
    ctl effect minecraft:resistance 120 4 >/dev/null 2>&1 || true
}

# One zombie at x,101,z, killed with the held bow: aim and fire until it is gone. The mob
# must have AI: `NoAI:1b` freezes it so completely that a projectile's damage never lands
# (measured on this pin: the same arrow kills an AI zombie and leaves a frozen one at 20 HP).
kill_with_bow() { # x z
    local x=$1 z=$2 i id zombies
    ctl command "execute in minecraft:the_end run tp @s $(( ${x%%.*} - 6 )).5 101 $(( ${z%%.*} + 0 )).5" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=minecraft:zombie]" >/dev/null
    for i in $(seq 1 20); do
        zombies=$(ctl entities 20 minecraft:zombie | jq -r '.entities | length')
        [ "$zombies" = 0 ] && break
        ctl wait 4 >/dev/null
    done
    [ "$zombies" = 0 ] || fail "the arena still holds $zombies zombies, so the target is ambiguous"
    ctl summon minecraft:zombie "$x" 101 "$z" >/dev/null || fail "cannot summon a zombie at $x,101,$z"
    ctl wait 8 >/dev/null
    id=$(ctl entities 20 minecraft:zombie | jq -r '.entities[0].id // ""')
    [ -n "$id" ] || fail "the zombie is not among the entities: $(ctl entities 20 | jq -c '.entities[0:3]')"
    # A fresh zombie carries 20 HP: the readout prints it as `20.0`, so compare numerically.
    [ "$(ctl entities 20 minecraft:zombie | jq -r '.entities[0].health == 20')" = true ] \
        || fail "the summoned zombie has $(ctl entities 20 minecraft:zombie | jq -r '.entities[0].health') HP, expected a fresh 20"
    for i in $(seq 1 8); do
        id=$(ctl entities 20 minecraft:zombie | jq -r '.entities[0].id // ""')
        [ -n "$id" ] || break
        ctl look-at-entity "$id" >/dev/null 2>&1 || true
        ctl use-item --hold 1.2 >/dev/null 2>&1 || true
        ctl wait 10 >/dev/null
    done
    [ "$(ctl entities 20 minecraft:zombie | jq -r '.entities | length')" = 0 ] \
        || fail "the zombie survived eight arrows: $(ctl entities 20 | jq -c '.entities[0:3]')"
    echo "  the carried bow killed the zombie at $x,101,$z" >&2
}

# `data get entity` on the newest arrow, waiting for the *new* reply (chat is newest first).
# Sets `ARROW_LINE`.
arrow_line() {
    local prev line i
    prev=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 '^Arrow has the following entity data' || true)
    ctl command "data get entity @e[type=minecraft:arrow,limit=1]" >/dev/null
    line=$prev
    for i in $(seq 1 25); do
        ctl wait 4 >/dev/null
        line=$(ctl messages 20 | jq -r '.chat[]' | grep -m1 '^Arrow has the following entity data' || true)
        [ -n "$line" ] && [ "$line" != "$prev" ] && break
    done
    [ -n "$line" ] && [ "$line" != "$prev" ] || fail "no /data get reply for the arrow"
    ARROW_LINE=$line
    echo "  $ARROW_LINE" >&2
}

# The previous package (0.2.3): the same bow kill must leave the extractor empty. That is the
# gap this slice closes — a build whose `mob_killed` hears melee, sweep and spear only never
# sees an arrow's kill.
old_leg() {
    echo "== the previous package (0.2.3): the same bow kill fills nothing =="
    [ -x "$OLD_GAME" ] || fail "OLD_GAME is not a game: '$OLD_GAME'"
    launch "$OLD_WORLD" "$OLD_GAME"
    arena
    arm_bow
    kill_with_bow 206.5 10.5
    ctl select 9 >/dev/null
    ctl wait 4 >/dev/null
    held_blood "0.2.3 package"
    [ "$HELD_BLOOD" = 0 ] \
        || fail "the 0.2.3 package filled the extractor with $HELD_BLOOD mB from a bow kill, so this leg does not show the gap"
    echo "  0.2.3: the same bow kill left the extractor at $HELD_BLOOD mB"
    ctl quit >/dev/null 2>&1 || true
    sleep 1
}

# The previous package (0.2.9): a save made before the extractor was unstackable (its item
# declared no `max_stack_size`), so it holds one stack of **two** extractors with the amount
# stored once — the state review finding 2 is about. The main launch continues this same
# world, so its first player tick resolves the stack (E2 asserts that).
legacy_leg() {
    echo "== the previous package (0.2.9): a save with one stack of two Blood Extractors =="
    [ -x "$LEGACY_GAME" ] || fail "LEGACY_GAME is not a game: '$LEGACY_GAME'"
    launch "$WORLD" "$LEGACY_GAME"
    # hotbar.0 holds a dirt block so the resolved stack's extra item cannot take the selected
    # slot (the stain E2 places from hotbar.0 must not overwrite the extractor's twin).
    ctl command "clear @s" >/dev/null
    ctl command "item replace entity @s hotbar.0 with minecraft:dirt 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "item replace entity @s hotbar.7 with $EXTRACTOR 2" >/dev/null \
        || fail "0.2.9 refused a stack of two extractors, so this leg cannot build the legacy save"
    ctl wait 6 >/dev/null
    ctl select 8 >/dev/null
    ctl wait 4 >/dev/null
    say "evilcraft held 4000"
    ctl wait 12 >/dev/null
    local slots count
    slots=$(ctl inventory | jq --arg i "$EXTRACTOR" '[.hotbar[], .main[] | select(.item == $i)] | length')
    count=$(ctl inventory | jq --arg i "$EXTRACTOR" '.counts[$i] // 0')
    [ "$slots" = 1 ] || fail "0.2.9 wrote $slots extractor slot(s), expected the one stack"
    [ "$count" = 2 ] || fail "0.2.9 wrote $count extractor(s), expected the stack of 2"
    held_blood "the 0.2.9 stack"
    [ "$HELD_BLOOD" = 4000 ] || fail "the 0.2.9 stack holds $HELD_BLOOD mB, expected 4,000"
    echo "  0.2.9 save: one stack of 2 extractors carrying 4,000 mB once" >&2
    ctl save >/dev/null 2>&1 || true
    ctl quit >/dev/null 2>&1 || true
    sleep 1
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then
    # The control: the whole scenario on the same Nightly **without** the package, where
    # the chest item never registers and the run must fail at phase A's first check.
    [ -n "$CONTROL_GAME" ] || { echo "CONTROL=1 needs CONTROL_GAME" >&2; exit 2; }
    GAME=$CONTROL_GAME
fi
if [ "$OLD" = 1 ]; then
    old_leg
fi
if [ "$LEGACY" = 1 ]; then
    legacy_leg
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

echo "== D2: a stack of broken anvils is repaired whole, one price each (review finding 1) =="
# `AnvilRepairAction.repair` answers `new ItemStack(anvil)`, so upstream — and 0.2.7/0.2.8 —
# replaced a stack of N chipped anvils with **one** anvil and dropped its components. The port
# keeps the stack: N anvils for N x 125 mB, the custom name carried over, and a tank that
# cannot pay for all N leaves the stack alone. Anvils stack to 64 in vanilla, so this is a
# legal stack a player really has.
say "evilcraft blood $SX 101 0 0"
ctl wait 12 >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:chipped_anvil[minecraft:custom_name='\"Two anvils\"'] 2" >/dev/null
ctl wait 8 >/dev/null
open_menu "$SX" 101 0 11
put_in_slot 302 minecraft:chipped_anvil
ctl close >/dev/null 2>&1 || true
ctl wait 10 >/dev/null
info "$SX" 101 0
[ "${SLOT2_ITEM:-}" = "minecraft:chipped_anvil" ] || fail "slot 2 holds ${SLOT2_ITEM:-nothing}, expected the anvil stack"
say "evilcraft blood $SX 101 0 125"
ctl wait 20 >/dev/null
info "$SX" 101 0
saved_item "$SX" 101 0 2
case "$SAVED_ITEM" in
    *"count: 2"*chipped_anvil*)
        echo "  125 mB: the stack of two chipped anvils is untouched" >&2 ;;
    *) fail "125 mB changed the anvil stack: '$SAVED_ITEM'" ;;
esac
[ "$TANK" = 125 ] || fail "the untouched stack left $TANK mB, expected the 125 the tank held"
say "evilcraft blood $SX 101 0 250"
ctl wait 20 >/dev/null
info "$SX" 101 0
saved_item "$SX" 101 0 2
ANVIL_COUNT=2
if [ "$SELFTEST" = anvil ]; then
    echo "(selftest=anvil: asserting the stack was replaced by one anvil)"
    ANVIL_COUNT=1
fi
case "$SAVED_ITEM" in
    *"count: $ANVIL_COUNT"*)
        echo "  two anvils for 250 mB: count $ANVIL_COUNT" >&2 ;;
    *) fail "the repaired stack holds $SAVED_ITEM, expected count $ANVIL_COUNT" ;;
esac
case "$SAVED_ITEM" in
    *'id: "minecraft:anvil"'*) ;;
    *) fail "the stack is not minecraft:anvil: '$SAVED_ITEM'" ;;
esac
case "$SAVED_ITEM" in
    *custom_name*Two\ anvils*) echo "  the custom name travelled with both anvils" >&2 ;;
    *) fail "the repaired anvils lost their custom name: '$SAVED_ITEM'" ;;
esac
[ "$TANK" = 0 ] || fail "two anvils cost $((250 - TANK)) mB, expected 2 x 125 = 250 (tank $TANK)"

echo "== D3: a damaged enchanted pickaxe is repaired, and keeps its enchantment =="
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe[minecraft:enchantments={\"minecraft:sharpness\":3}] 1" >/dev/null
ctl wait 8 >/dev/null
open_menu "$SX" 101 0 11
put_in_slot 303 minecraft:diamond_pickaxe
ctl close >/dev/null 2>&1 || true
ctl wait 6 >/dev/null
say "evilcraft damage $SX 101 0 3 250"
say "evilcraft blood $SX 101 0 1000"
ctl wait 12 >/dev/null
info "$SX" 101 0
D3_BEFORE=$SLOT3_DAMAGE
[ "${SLOT3_ITEM:-}" = "minecraft:diamond_pickaxe" ] || fail "slot 3 holds ${SLOT3_ITEM:-nothing}"
ctl wait 40 >/dev/null
info "$SX" 101 0
[ "${SLOT3_ITEM:-}" = "minecraft:diamond_pickaxe" ] \
    || fail "the pickaxe left slot 3 ($SLOT3_ITEM)"
[ "$SLOT3_DAMAGE" -lt "$D3_BEFORE" ] \
    || fail "the pickaxe was not repaired (damage $D3_BEFORE -> $SLOT3_DAMAGE)"
saved_item "$SX" 101 0 3
case "$SAVED_ITEM" in
    *"count: 1"*diamond_pickaxe*)
        echo "  one pickaxe, repaired from damage $D3_BEFORE to $SLOT3_DAMAGE" >&2 ;;
    *) fail "the repaired pickaxe's saved stack is '$SAVED_ITEM'" ;;
esac
case "$SAVED_ITEM" in
    *minecraft:enchantments*sharpness*)
        echo "  the enchantment survived the repair" >&2 ;;
    *) fail "the repair dropped the pickaxe's enchantment: '$SAVED_ITEM'" ;;
esac
# Park the repaired pickaxe: a whole item is not repairable, so it cannot spend the tank
# phase F measures between the kill's readout and the container slot's drain.
say "evilcraft damage $SX 101 0 3 0"
ctl wait 8 >/dev/null

echo "== E2: a legacy stack of two extractors is split before any drain (review finding 2) =="
# The extractor is unstackable now (`max_stack_size = 1`), but a save made before that fix —
# written by the 0.2.9 package in `legacy_leg` — holds one stack of N with the amount stored
# **once**. The rule: the first extractor keeps the whole amount, the rest are empty and handed
# back, so both the blood and the item count are conserved. A crouching click then fills one
# extractor and cannot write onto the other.
EX2=$((X0 + 4))
EZ2=4
LEGACY_SLOTS=$(ctl inventory | jq --arg i "$EXTRACTOR" '[.hotbar[], .main[] | select(.item == $i)] | length')
LEGACY_ITEMS=$(ctl inventory | jq --arg i "$EXTRACTOR" '.counts[$i] // 0')
if [ "$SELFTEST" = extractor ]; then
    echo "(selftest=extractor: asserting the legacy stack was not split)"
    LEGACY_SLOTS=1
fi
[ "$LEGACY_SLOTS" = 2 ] \
    || fail "the legacy stack reads as $LEGACY_SLOTS extractor slot(s), expected the split's 2 (inventory: $(ctl inventory | jq -c '.hotbar'))"
[ "$LEGACY_ITEMS" = 2 ] \
    || fail "the inventory holds $LEGACY_ITEMS extractor item(s), expected the 2 the stack had"
echo "  the 0.2.9 stack of two resolved into $LEGACY_SLOTS single extractors ($LEGACY_ITEMS items)" >&2
ctl select 8 >/dev/null
ctl wait 4 >/dev/null
held_blood "the first extractor"
[ "$HELD_BLOOD" = 4000 ] \
    || fail "the first extractor holds $HELD_BLOOD mB, expected the stack's whole 4,000"
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
held_blood "the second extractor"
[ "$HELD_BLOOD" = 0 ] \
    || fail "the second extractor holds $HELD_BLOOD mB, expected the split's 0"
# The stain goes down after the split (place_stain writes hotbar.0, which holds a dirt block,
# so neither extractor is disturbed), then the crouching click with the filled one.
say "evilcraft blood $EX2 101 $EZ2 0"
ctl wait 8 >/dev/null
place_stain "$EX2" 101 "$EZ2"
say "evilcraft blood $EX2 101 $EZ2 3500"
ctl wait 12 >/dev/null
ctl select 8 >/dev/null
ctl wait 4 >/dev/null
crouch_click "$EX2" 101 "$EZ2" .03 "$STAIN"
stain "$EX2" 101 "$EZ2"
[ "$TANK" = 2500 ] || fail "the stain holds $TANK mB after the draw, expected 3500 - 1000 = 2500"
held_blood "the poured extractor"
[ "$HELD_BLOOD" = 5000 ] || fail "the held extractor holds $HELD_BLOOD mB, expected its 5,000"
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
held_blood "the other extractor"
[ "$HELD_BLOOD" = 0 ] \
    || fail "the drain wrote $HELD_BLOOD mB onto the second extractor, expected 0"
echo "  the crouching click drew 1,000 mB into one extractor (4,000 -> 5,000); the other still holds 0 mB"
ctl command "execute in minecraft:the_end run setblock $EX2 101 $EZ2 minecraft:air" >/dev/null

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
# The jar's nine `convenience/` infusions plus `base/blook` (10 as of slice 9), and the
# machine reads every one of them (`recipes_of_type`).
[ "$CARRIED" = 10 ] || fail "the package claims $CARRIED carried recipes, expected 10"
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
# The Blook: `data/evilcraft/recipe/blood_infuser/base/blook.json`, the jar's own recipe (a
# Book and 500 mB for a Blook in 500 ticks at tier 1). It is also the recipe whose keys this
# package has to resolve: its product is an item the package itself registers.
infuse minecraft:book evilcraft:blook 1 500 500

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

echo "== Q: a bow kill fills the carried extractor (every cause of death, the player's) =="
# Phase N left the player in survival, and this phase stays there: a survival kill is the
# real thing (arrows are consumed, 32 are given), and a player who was *switched* from
# survival to creative no longer keeps an inventory write a death hook makes (measured on
# this pin: the same bow kill filled 266 mB in a creative-from-launch player, 598 in
# survival, and 0 after a survival -> creative switch).
# A survival player is the real case (and the only one whose death-hook write lands after
# phase N's switch): ask the game itself, in the player's own selector.
ctl command "execute if entity @s[gamemode=survival] run say EC_SURVIVAL" >/dev/null
ctl wait 6 >/dev/null
ctl messages 8 | jq -r '.chat[]' | grep -q 'EC_SURVIVAL' \
    || fail "the player is not in survival, so the survival kill leg proves nothing"
arm_bow
kill_with_bow 206.5 10.5
ctl select 9 >/dev/null
ctl wait 6 >/dev/null
held_blood "bow kill"
BOW_BLOOD=$HELD_BLOOD
if [ "$SELFTEST" = blood ]; then
    echo "(selftest=blood: asserting the arrow kill filled nothing)"
    BOW_BLOOD=0
fi
[ "$BOW_BLOOD" -ge 100 ] || fail "a bow kill filled $BOW_BLOOD mB, expected at least 100"
[ "$BOW_BLOOD" -le 799 ] || fail "a bow kill filled $BOW_BLOOD mB, expected at most 799"
echo "  a 20 HP zombie killed by an arrow filled the carried extractor with $BOW_BLOOD mB"

echo "== R: a skeleton's arrow kills the same mob and fills nothing (the kill's differential) =="
# The same extractor, emptied: an arrow whose shooter is a mob is a death without player
# credit. Upstream asks `event.getSource().getEntity() instanceof ServerPlayer`, and this
# pin's `loot_drops` names the source's causing entity — so the zombie dies and the
# extractor stays at 0.
say "evilcraft held 0"
ctl wait 12 >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=minecraft:arrow]" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=minecraft:zombie]" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=minecraft:skeleton]" >/dev/null
ctl wait 10 >/dev/null
ctl command "execute in minecraft:the_end run tp @s 200.5 101 10.5" >/dev/null
ctl wait 6 >/dev/null
ctl command "execute in minecraft:the_end run summon minecraft:skeleton 212.5 101 10.5" >/dev/null
ctl wait 8 >/dev/null
ctl command "data get entity @e[type=minecraft:skeleton,limit=1] UUID" >/dev/null
ctl wait 10 >/dev/null
SKEL_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Skeleton has the following entity data: \[I;' || true)
[ -n "$SKEL_LINE" ] || fail "no skeleton UUID reply: $(ctl messages 6 | jq -c '.chat')"
SKEL_UUID=$(sed -n 's/.*entity data: \(.*\)$/\1/p' <<<"$SKEL_LINE")
[ -n "$SKEL_UUID" ] || fail "the skeleton UUID did not parse: $SKEL_LINE"
# Step out of the skeleton's reach first: a skeleton targets players, and this leg is about
# its arrow hitting a *mob*, not about the player surviving a volley. The arrow keeps the
# skeleton's UUID as its `Owner` wherever the player stands.
ctl effect minecraft:resistance 120 4 >/dev/null 2>&1 || true
ctl command "execute in minecraft:the_end run tp @s 200.5 101 -8.5" >/dev/null
ctl wait 8 >/dev/null
# An AI target: a frozen mob never takes a projectile's damage at this pin.
ctl command "execute in minecraft:the_end run summon minecraft:zombie 206.5 101 10.5" >/dev/null
ctl wait 6 >/dev/null
# A 30-block radius: the player stands at z=-8.5 and the target is summoned at z=10.5, which is
# 19.9 blocks away — inside a 20-block query only if the wandering zombie happens to stay put
# (measured 2026-11-23: the count read 0 and the run stopped here).
ZOMBIES=$(ctl entities 30 minecraft:zombie | jq -r '.entities | length')
[ "$ZOMBIES" = 1 ] || fail "the arena holds $ZOMBIES zombies, expected 1: $(ctl entities 30 | jq -c '.entities[0:3]')"
# An AI zombie is the target because a frozen one takes no projectile damage, and an AI zombie
# wanders: its cell is pinned just before the arrow is sent, so the straight flight along z=10.5
# meets it (measured once on 2026-11-23: a free zombie had walked out of the arrow's line).
ctl command "execute in minecraft:the_end run tp @e[type=minecraft:zombie,limit=1] 206.5 101 10.5" >/dev/null
ctl wait 2 >/dev/null
# The arrow is the skeleton's: its own UUID is in the arrow's saved `Owner`, read back
# before it is sent at the zombie, so the leg is about a mob's arrow and not about an
# ownerless one.
ctl command "execute in minecraft:the_end run summon minecraft:arrow 210.5 102 10.5 {NoGravity:1b,Owner:$SKEL_UUID,Motion:[0.0,0.0,0.0],damage:100.0}" >/dev/null \
    || fail "cannot summon the skeleton's arrow"
ctl wait 4 >/dev/null
arrow_line
grep -qF "Owner: $SKEL_UUID" <<<"$ARROW_LINE" \
    || fail "the arrow is not the skeleton's: expected Owner $SKEL_UUID, read '$ARROW_LINE'"
ctl command "execute in minecraft:the_end run tp @e[type=minecraft:zombie,limit=1] 206.5 101 10.5" >/dev/null
ctl wait 2 >/dev/null
ctl command "data merge entity @e[type=minecraft:arrow,limit=1] {Motion:[-2.0,0.0,0.0]}" >/dev/null \
    || fail "the skeleton's arrow does not fly"
for _ in $(seq 1 20); do
    ctl wait 4 >/dev/null
    [ "$(ctl entities 30 minecraft:zombie | jq -r '.entities | length')" = 0 ] && break
done
[ "$(ctl entities 30 minecraft:zombie | jq -r '.entities | length')" = 0 ] \
    || fail "the skeleton's arrow did not kill the zombie: $(ctl entities 30 | jq -c '.entities[0:3]')"
ctl wait 6 >/dev/null
held_blood "skeleton-arrow"
[ "$HELD_BLOOD" = 0 ] \
    || fail "a skeleton's arrow filled the extractor with $HELD_BLOOD mB, so a mob's kill was credited to the player"
echo "  the skeleton's arrow killed the zombie and the extractor stayed at $HELD_BLOOD mB"

echo "== S: a crouching click with the extractor draws a stain's blood into it =="
# The stain: the jar's own block on the arena floor, its tank set with the harness command
# (a fall death fills one — a later slice), and an extractor with room for exactly 1,000 mB.
# The click moves what the extractor has room for, never the whole stain.
STX=$((X0 + 4))
STZ=-10
ctl command "clear @s" >/dev/null
# A `setblock` of the same id keeps the block entity that was there, so clear first.
ctl command "execute in minecraft:the_end run setblock $STX 101 $STZ minecraft:air" >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
place_stain "$STX" 101 "$STZ"
stain "$STX" 101 "$STZ"
[ "$TANK" = 0 ] || fail "a fresh stain holds $TANK mB, expected 0"
[ "$CAPACITY" = "$STAIN_TANK" ] || fail "the stain reports a $CAPACITY mB tank, expected $STAIN_TANK"
say "evilcraft blood $STX 101 $STZ 3500"
ctl wait 10 >/dev/null
stain "$STX" 101 "$STZ"
[ "$TANK" = 3500 ] || fail "the harness command left the stain at $TANK mB, expected 3500"
# The tank is the pin's saved `fluids` list, the same compound the chest and the infuser
# keep: `/data get block` reads the block entity's own data, not the readout's clamp.
ctl command "data get block $STX 101 $STZ fluids" >/dev/null
ctl wait 10 >/dev/null
STAIN_SAVED=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 "has the following block data" || true)
case "$STAIN_SAVED" in
    "$STX, 101, $STZ has the following block data: [{Amount: 3500, Fluid: \"evilcraft:blood\"}]")
        echo "  the stain's saved tank: $STAIN_SAVED" >&2 ;;
    *) fail "the stain's saved data is not 3500 mB of evilcraft:blood: '$STAIN_SAVED'" ;;
esac
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 9 >/dev/null
ctl wait 2 >/dev/null
say "evilcraft held 4000"
ctl wait 10 >/dev/null
held_blood "before the click"
[ "$HELD_BLOOD" = 4000 ] || fail "the extractor holds $HELD_BLOOD mB, expected the 4,000 the command set"
ctl command "execute in minecraft:the_end run tp @s $((STX - 2)).5 101 $((STZ)).5" >/dev/null
ctl wait 6 >/dev/null
crouch_click "$STX" 101 "$STZ"
held_blood "after the crouching click"
STAIN_HELD_AFTER=$HELD_BLOOD
stain "$STX" 101 "$STZ"
STAIN_MOVED=$((STAIN_HELD_AFTER - 4000))
if [ "$SELFTEST" = stain ]; then
    echo "(selftest=stain: asserting the crouching click drew nothing)"
    STAIN_MOVED=0
fi
[ "$STAIN_MOVED" = 1000 ] \
    || fail "the crouching click moved $STAIN_MOVED mB into the extractor, expected exactly the 1000 mB room it had"
[ "$TANK" = 2500 ] || fail "the stain holds $TANK mB after the click, expected 3500 - 1000 = 2500"
[ "$CAPACITY" = "$STAIN_TANK" ] || fail "the stain's tank is $CAPACITY mB, expected $STAIN_TANK"
echo "  extractor 4000 -> $STAIN_HELD_AFTER mB, stain 3500 -> $TANK mB (moved exactly $STAIN_MOVED)"

echo "== T: a standing click moves nothing; a crouching click on stone moves nothing =="
# The same stain, the same extractor, the same click — standing: nothing moves. Then the
# same click crouching: it moves. That pair is the crouch rule's differential.
say "evilcraft blood $STX 101 $STZ 3500"
ctl wait 10 >/dev/null
say "evilcraft held 4000"
ctl wait 10 >/dev/null
stand_click "$STX" 101 "$STZ"
held_blood "after the standing click"
[ "$HELD_BLOOD" = 4000 ] || fail "a standing click moved blood into the extractor ($HELD_BLOOD mB, expected 4000)"
stain "$STX" 101 "$STZ"
[ "$TANK" = 3500 ] || fail "a standing click left the stain at $TANK mB, expected the untouched 3500"
echo "  standing: extractor 4000 mB, stain 3500 mB (nothing moved)"
crouch_click "$STX" 101 "$STZ"
held_blood "after the same click, crouching"
[ "$HELD_BLOOD" = 5000 ] || fail "the crouching click left the extractor at $HELD_BLOOD mB, expected a full 5000"
stain "$STX" 101 "$STZ"
[ "$TANK" = 2500 ] || fail "the crouching click left the stain at $TANK mB, expected 2500"
echo "  crouching: extractor 4000 -> 5000 mB, stain 3500 -> 2500 mB (the same click, sneaking)"
# A vanilla block answers the same crouching click with nothing: the rule is about stains.
ctl command "execute in minecraft:the_end run setblock $STX 101 $((STZ + 2)) minecraft:stone" >/dev/null
ctl wait 6 >/dev/null
say "evilcraft held 4000"
ctl wait 10 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $((STX - 2)).5 101 $((STZ + 2)).5" >/dev/null
ctl wait 6 >/dev/null
crouch_click "$STX" 101 $((STZ + 2)) .99 minecraft:stone
held_blood "after the crouching click on stone"
[ "$HELD_BLOOD" = 4000 ] \
    || fail "a crouching click on minecraft:stone moved blood into the extractor ($HELD_BLOOD mB, expected 4000)"
echo "  the same crouching click on $(block_name "$STX" 101 $((STZ + 2))): extractor still $HELD_BLOOD mB"

echo "== U: a drained stain removes its block, and vanilla may replace it =="
# Upstream's block entity removes its block when a committed change takes the tank to 0.
say "evilcraft blood $STX 101 $STZ 400"
ctl wait 10 >/dev/null
say "evilcraft held 4600"
ctl wait 10 >/dev/null
stain "$STX" 101 "$STZ"
[ "$TANK" = 400 ] || fail "the stain holds $TANK mB, expected 400"
crouch_click "$STX" 101 "$STZ"
held_blood "after draining the stain"
[ "$HELD_BLOOD" = 5000 ] || fail "draining the stain left the extractor at $HELD_BLOOD mB, expected 5000"
STAIN_GONE=$(block_name "$STX" 101 "$STZ")
[ "$STAIN_GONE" = "minecraft:air" ] \
    || fail "a stain drained to nothing is still $STAIN_GONE; upstream removes its block"
echo "  a stain drained to 0 is gone ($STAIN_GONE), and the extractor took its 400 mB ($HELD_BLOOD mB)"
# The jar also adds the stain to minecraft:replaceable, so upstream lets a block replace it.
# This pin carries `replaceable` as a per-state block fact and offers a content block no key
# for it (`BlockProperties` has no `replaceable`), so the tag is inert here: a stone placed
# on a stain lands on top of it. Asserted both ways, so the day the pin reads the tag this
# phase says so instead of passing silently (the gap is recorded in `notes.md`).
ctl command "item replace entity @s hotbar.0 with minecraft:stone 1" >/dev/null
ctl wait 4 >/dev/null
place_stain "$STX" 101 $((STZ - 2))
ctl wait 6 >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:stone 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $((STX - 2)).5 101 $((STZ - 2)).5" >/dev/null
ctl wait 6 >/dev/null
stand_click "$STX" 101 $((STZ - 2))
REPLACED=$(block_name "$STX" 101 $((STZ - 2)))
ABOVE=$(block_name "$STX" 102 $((STZ - 2)))
[ "$REPLACED" = "$STAIN" ] \
    || fail "a stone placed onto a stain replaced it ($REPLACED); this pin now reads minecraft:replaceable — re-read the gap in notes.md"
[ "$ABOVE" = "minecraft:stone" ] \
    || fail "a stone placed onto a stain left $REPLACED in the cell and $ABOVE above it, expected the stain and a stone above"
echo "  a stone placed onto a stain landed above it ($REPLACED stays, $ABOVE above): minecraft:replaceable is inert at this pin"

echo "== V: a mob's fall death leaves a Blood Stain holding its blood =="
# Upstream's `BlockBloodStain.bloodStainedBlockEvent`: a death from fall damage leaves
# `bloodMBPerHP (20) x max health` mB in the cell the mob died in. The zombie carries one HP
# so the **fall** is what kills it, and its max health is still 20: the 400 asserted below is
# the field upstream multiplies, not the current health (which would give 20).
FX=$((X0 + 4))
FZ=-6
ctl command "clear @s" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player,distance=..10]" >/dev/null 2>&1 || true
ctl command "execute in minecraft:the_end run setblock $FX 101 $FZ minecraft:air" >/dev/null
ctl wait 6 >/dev/null
# A stone chimney around the column, its inside three cells wide: the zombie cannot drift out of
# the cell it has to die in, and it is not squeezed into a wall (a 1-wide shaft killed the first
# probe's zombie with `minecraft:in_wall`).
ctl command "execute in minecraft:the_end run fill $((FX - 2)) 102 $((FZ - 2)) $((FX + 2)) 108 $((FZ + 2)) minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill $((FX - 1)) 102 $((FZ - 1)) $((FX + 1)) 108 $((FZ + 1)) minecraft:air" >/dev/null
ctl wait 6 >/dev/null
[ "$(block_name "$FX" 101 "$FZ")" = "minecraft:air" ] || fail "the fall cell is not clear"
[ "$(block_name "$FX" 102 "$FZ")" = "minecraft:air" ] || fail "the fall shaft is not open"
# The summon point is the centre of the target cell: `-6.5` is the centre of cell **-7**, so
# the coordinates are computed, never concatenated.
read -r FALL_X FALL_Z <<<"$(awk -v x="$FX" -v z="$FZ" 'BEGIN { printf "%.4f %.4f", x + 0.5, z + 0.5 }')"
ctl command "execute in minecraft:the_end run tp @s $((FX - 6)).5 101 $FZ.5" >/dev/null
ctl wait 6 >/dev/null

# A one-HP zombie dropped six blocks down the chimney: it dies **of the fall**, in the cell
# below the shaft (the arena floor is at 100, so its feet stop at 101).
drop_a_zombie() {
    local i
    ctl command "execute in minecraft:the_end run summon minecraft:zombie $FALL_X 107 $FALL_Z {Health:1.0f}" >/dev/null \
        || fail "cannot summon the falling zombie"
    for i in $(seq 1 40); do
        ctl wait 4 >/dev/null
        [ "$(ctl entities 20 minecraft:zombie | jq -r '.entities | length')" = 0 ] && return 0
    done
    fail "the zombie did not die of its fall: $(ctl entities 20 minecraft:zombie | jq -c '.entities[0:3]')"
}

drop_a_zombie
FALL_CELL=$(block_name "$FX" 101 "$FZ")
[ "$FALL_CELL" = "$STAIN" ] \
    || fail "a fall death left '$FALL_CELL' at $FX,101,$FZ, expected a $STAIN"
stain "$FX" 101 "$FZ"
FALL_BLOOD=$TANK
if [ "$SELFTEST" = fall ]; then
    echo "(selftest=fall: asserting the fall death left no stain blood)"
    FALL_BLOOD=0
fi
[ "$FALL_BLOOD" = 400 ] \
    || fail "a 20 HP zombie's fall left a stain holding $FALL_BLOOD mB, expected 20 x 20 = 400"
[ "$CAPACITY" = "$STAIN_TANK" ] || fail "the stain reports a $CAPACITY mB tank, expected $STAIN_TANK"
saved_tank "$FX" 101 "$FZ"
[ "$SAVED_MB" = 400 ] || fail "the stain's saved tank is $SAVED_MB mB, expected 400"
echo "  the 20 HP zombie died of its fall and left $STAIN at $FX,101,$FZ holding $FALL_BLOOD mB"

echo "== X: a second fall death on the same cell tops the stain up to 800 mB =="
# The same shaft, the same cell: the first zombie left a stain there, and the second death
# lands on it — upstream adds to the block entity it finds rather than placing a second block.
drop_a_zombie
[ "$(block_name "$FX" 101 "$FZ")" = "$STAIN" ] \
    || fail "the stain at $FX,101,$FZ is gone after a second fall: '$(block_name "$FX" 101 "$FZ")'"
[ "$(block_name "$FX" 102 "$FZ")" = "minecraft:air" ] \
    || fail "a second stain was placed above the first ($(block_name "$FX" 102 "$FZ") at $FX,102,$FZ)"
stain "$FX" 101 "$FZ"
TOPPED=$TANK
[ "$TOPPED" = 800 ] || fail "the second fall left $TOPPED mB, expected the first 400 plus 400"
saved_tank "$FX" 101 "$FZ"
[ "$SAVED_MB" = 800 ] || fail "the saved tank is $SAVED_MB mB after two falls, expected 800"
echo "  a second fall death on the same cell: 400 -> $TOPPED mB in one stain, no second block"

echo "== W: the same mob killed with a sword leaves the cell empty (the death's differential) =="
# The same cell, the same mob, one difference: the killing blow. A sword kill is not a fall,
# so no stain is placed — the blood in V and X can only come from the fall's damage type.
ctl command "execute in minecraft:the_end run fill $((FX - 2)) 102 $((FZ - 2)) $((FX + 2)) 108 $((FZ + 2)) minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run setblock $FX 101 $FZ minecraft:air" >/dev/null
ctl wait 6 >/dev/null
[ "$(block_name "$FX" 101 "$FZ")" = "minecraft:air" ] || fail "the test cell did not clear"
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_sword 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $((FX - 2)).5 101 $FZ.5" >/dev/null
ctl wait 6 >/dev/null
ctl command "execute in minecraft:the_end run summon minecraft:zombie $FALL_X 101 $FALL_Z {NoAI:1b}" >/dev/null \
    || fail "cannot summon the sword target"
ctl wait 8 >/dev/null
SWORD_ZID=$(ctl entities 12 minecraft:zombie | jq -r '.entities[0].id // ""')
[ -n "$SWORD_ZID" ] || fail "the sword target is not among the entities: $(ctl entities 12 | jq -c '.entities[0:3]')"
ctl look-at-entity "$SWORD_ZID" >/dev/null || fail "cannot face the sword target"
for _ in $(seq 1 8); do
    ctl attack "$SWORD_ZID" >/dev/null 2>&1 || true
    ctl wait 12 >/dev/null
    [ "$(ctl entities 12 minecraft:zombie | jq -r '.entities | length')" = 0 ] && break
done
[ "$(ctl entities 12 minecraft:zombie | jq -r '.entities | length')" = 0 ] \
    || fail "the sword target survived eight blows: $(ctl entities 12 | jq -c '.entities[0:3]')"
SWORD_CELL=$(block_name "$FX" 101 "$FZ")
[ "$SWORD_CELL" = "minecraft:air" ] \
    || fail "a sword kill left '$SWORD_CELL' where the fall left a stain; only a fall death may stain"
echo "  the same cell after a sword kill is $SWORD_CELL (no stain)"

echo "== Y: a hollow 3x3x3 shell forms one Colossal Blood Chest, which repairs one point a two ticks =="
# Upstream's multiblock is not 27 controllers: `HollowCubeDetector` + `ExactSizeValidator(2,2,2)`
# + `ExactBlockCountValidator(1)` ask for a hollow 3x3x3 cube whose wall is reinforced undead
# planks and **exactly one** colossal blood chest, the middle cell air. The player is still in
# survival after N; the cube is built with the game's own `/fill` and `/setblock`.
COLX=$((X0 - 2))
COLY=101
COLZ=-10
COL_C=$((COLX + 1))     # the cube's middle (air)
COL_MY=$((COLY + 1))
COL_MZ=$((COLZ + 1))
COL_FAR=$((COLX + 2))   # the corner the counterfactual removes
COL_FARY=$((COLY + 2))
COL_FARZ=$((COLZ + 2))
# `break_survival` stands three cells west of what it breaks, which is outside the arena's
# own floor (x 198..208): extend it so the player does not fall into the void.
ctl command "execute in minecraft:the_end run fill $((COLX - 4)) 100 -14 $((X0 + 8)) 100 14 minecraft:stone" >/dev/null
ctl wait 6 >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player,distance=..8]" >/dev/null 2>&1 || true
ctl command "execute in minecraft:the_end run fill $COLX $COLY $COLZ $COL_FAR $COL_FARY $COL_FARZ $PLANKS" >/dev/null
ctl wait 6 >/dev/null
ctl command "execute in minecraft:the_end run setblock $COL_C $COL_MY $COL_MZ minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run setblock $COLX $COLY $COLZ $COLOSSAL" >/dev/null
ctl wait 12 >/dev/null
[ "$(block_name "$COLX" "$COLY" "$COLZ")" = "$COLOSSAL" ] \
    || fail "the controller did not place at $COLX,$COLY,$COLZ ('$(block_name "$COLX" "$COLY" "$COLZ")')"
[ "$(block_name "$COL_FAR" "$COL_FARY" "$COL_FARZ")" = "$PLANKS" ] || fail "the shell's far corner is not a plank"
[ "$(block_name "$COL_C" "$COL_MY" "$COL_MZ")" = "minecraft:air" ] || fail "the cube's middle is not hollow"
colossal "$COLX" "$COLY" "$COLZ"
COLOSSAL_FORMED=$FORMED
if [ "$SELFTEST" = colossal ]; then
    echo "(selftest=colossal: asserting the 3x3x3 shell is not formed)"
    COLOSSAL_FORMED=false
fi
[ "$COLOSSAL_FORMED" = true ] \
    || fail "the hollow 3x3x3 shell at $COLX,$COLY,$COLZ is not formed (formed=$FORMED)"
[ "$CAPACITY" = "$TANK_CAPACITY" ] || fail "the colossal chest reports a $CAPACITY mB tank, expected $TANK_CAPACITY"
[ "$REPAIR" = 45 ] || fail "the machine reports $REPAIR repair slots, expected 45"
[ "$SLOTS" = 46 ] || fail "the machine reports $SLOTS slots, expected 46 (45 repair + the container)"
echo "  a hollow 3x3x3 of $PLANKS around one controller: formed=true, $REPAIR repair slots, $SLOTS slots"
# The brief's number: 1,000 mB and a damage-250 diamond pickaxe in the first repair slot.
say "evilcraft blood $COLX $COLY $COLZ 1000"
ctl wait 10 >/dev/null
say "evilcraft load $COLX $COLY $COLZ 0 minecraft:diamond_pickaxe"
ctl wait 10 >/dev/null
say "evilcraft damage $COLX $COLY $COLZ 0 250"
ctl wait 12 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "${SLOT0_ITEM:-}" = "minecraft:diamond_pickaxe" ] || fail "slot 0 holds ${SLOT0_ITEM:-nothing}, expected the pickaxe"
# The machine repairs from the moment it is formed, so the readout that follows the harness
# command already shows the pickaxe a little past 250 and the tank a little under 1,000; the
# *rate* is measured between two readouts below.
[ "$SLOT0_DAMAGE" -le 250 ] || fail "the harness did not damage the pickaxe (reads $SLOT0_DAMAGE)"
[ "$SLOT0_DAMAGE" -ge 200 ] || fail "the pickaxe is at damage $SLOT0_DAMAGE, so the harness set nothing near 250"
[ "$TANK" -le 1000 ] || fail "the harness set the tank to $TANK mB, more than the 1,000 it asked for"
[ "$TANK" -ge 900 ] || fail "the tank is at $TANK mB right after being filled to 1,000"
COL_DAMAGE_BEFORE=$SLOT0_DAMAGE
COL_TANK_BEFORE=$TANK
COL_TICKS_START=$TIME
ctl wait 40 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
COL_TICKS=$((TIME - COL_TICKS_START))
COL_CURED=$((COL_DAMAGE_BEFORE - SLOT0_DAMAGE))
COL_SPENT=$((COL_TANK_BEFORE - TANK))
[ "$COL_TICKS" -gt 0 ] || fail "no ticks passed between the two colossal reads"
[ "$COL_CURED" -gt 0 ] || fail "the formed machine repaired nothing in $COL_TICKS ticks (damage $SLOT0_DAMAGE)"
[ "$COL_CURED" -le $((COL_TICKS / TICKS_PER_DAMAGE + 1)) ] \
    || fail "$COL_CURED damage points in $COL_TICKS ticks is faster than one a $TICKS_PER_DAMAGE ticks"
[ "$COL_CURED" -ge $((COL_TICKS / TICKS_PER_DAMAGE - 1)) ] \
    || fail "$COL_CURED damage points in $COL_TICKS ticks is slower than one a $TICKS_PER_DAMAGE ticks"
[ "$COL_SPENT" -eq $((MB_PER_DAMAGE * COL_CURED)) ] \
    || fail "$COL_CURED damage points cost $COL_SPENT mB, expected $((MB_PER_DAMAGE * COL_CURED)) (5 mB a point)"
echo "  $COL_TICKS ticks: damage $COL_DAMAGE_BEFORE -> $SLOT0_DAMAGE ($COL_CURED points), tank $COL_TANK_BEFORE -> $TANK (-$COL_SPENT mB)"
# And it keeps repairing until the tank is empty: 1,000 mB is 200 points, 400 ticks.
say "evilcraft damage $COLX $COLY $COLZ 0 250"
say "evilcraft blood $COLX $COLY $COLZ 1000"
ctl wait 12 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
LONG_START=$SLOT0_DAMAGE
LONG_TANK=$TANK
ctl wait 460 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$TANK" = 0 ] || fail "460 ticks left $TANK mB in the tank, expected the 1,000 mB to be spent"
# Every point costs exactly 5 mB, so the tank read the fill at pays exactly `tank / 5` more
# points: the damage fell by that many, whatever the readout's own latency had already spent.
LONG_EXPECTED=$((LONG_START - LONG_TANK / MB_PER_DAMAGE))
[ "${SLOT0_DAMAGE:-}" = "$LONG_EXPECTED" ] \
    || fail "the tank's $LONG_TANK mB repaired the pickaxe from $LONG_START to ${SLOT0_DAMAGE:-nothing}, expected $LONG_EXPECTED"
ctl wait 40 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$TANK" = 0 ] || fail "an empty tank gained $TANK mB"
[ "${SLOT0_DAMAGE:-}" = "$LONG_EXPECTED" ] \
    || fail "an empty tank repaired the pickaxe further (damage ${SLOT0_DAMAGE:-nothing})"
echo "  the tank emptied: damage $LONG_START -> ${SLOT0_DAMAGE:-?} for $LONG_TANK mB ($((LONG_TANK / MB_PER_DAMAGE)) points at $MB_PER_DAMAGE mB), and 40 more ticks repair nothing"
COLOSSAL_RATE=$COL_CURED

echo "== Z: open the shell and the machine stops (the counterfactual) =="
# The same tank and the same pickaxe with one plank of the shell gone: `canWork()` asks
# `getSize()` for the exact size, and `BulkRepairItemTickAction.canTick` asks `canWork` first.
ctl command "execute in minecraft:the_end run setblock $COL_FAR $COL_FARY $COL_FARZ minecraft:air" >/dev/null
ctl wait 10 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$FORMED" = false ] \
    || fail "one shell block removed still reads formed=$FORMED, so the counterfactual proves nothing"
say "evilcraft damage $COLX $COLY $COLZ 0 250"
say "evilcraft blood $COLX $COLY $COLZ 1000"
ctl wait 12 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
OPEN_DAMAGE=$SLOT0_DAMAGE
OPEN_TANK=$TANK
OPEN_TICKS_START=$TIME
ctl wait 40 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
OPEN_TICKS=$((TIME - OPEN_TICKS_START))
[ "$OPEN_TICKS" -gt 0 ] || fail "no ticks passed between the two open-shell reads"
[ "$SLOT0_DAMAGE" = "$OPEN_DAMAGE" ] \
    || fail "an open shell repaired the pickaxe ($OPEN_DAMAGE -> $SLOT0_DAMAGE over $OPEN_TICKS ticks)"
[ "$TANK" = "$OPEN_TANK" ] || fail "an open shell spent blood ($OPEN_TANK -> $TANK)"
echo "  one shell block out: formed=false, damage $SLOT0_DAMAGE and tank $TANK unchanged over $OPEN_TICKS ticks"
# The container slot does not ask `canWork` (upstream's `EmptyInTankTickAction.canTick`), so a
# filled extractor still empties into the tank while the cube is open — and the menu lists all
# fifty slots, which is the parity row for the machine's GUI.
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 9 >/dev/null
ctl wait 2 >/dev/null
say "evilcraft held 1000"
ctl wait 12 >/dev/null
say "evilcraft blood $COLX $COLY $COLZ 0"
ctl wait 10 >/dev/null
open_menu "$COLX" "$COLY" "$COLZ" 50
put_in_slot 345 "$EXTRACTOR"
ctl close >/dev/null 2>&1 || true
ctl wait 20 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$TANK" = 1000 ] || fail "the open shell's container slot moved $TANK mB into the tank, expected 1,000"
[ "${CONTAINER_ITEM:-}" = "$EXTRACTOR" ] || fail "the container slot holds ${CONTAINER_ITEM:-nothing}, expected the extractor"
[ "${CONTAINER_BLOOD:-}" = 0 ] || fail "the extractor still holds ${CONTAINER_BLOOD} mB"
echo "  the menu lists 50 slots, and an extractor of 1,000 mB empties into the open machine's tank ($TANK mB, container $CONTAINER_BLOOD mB)"
# The plank back: the cube forms again and the repair resumes from the same tank.
ctl command "execute in minecraft:the_end run setblock $COL_FAR $COL_FARY $COL_FARZ $PLANKS" >/dev/null
ctl wait 10 >/dev/null
say "evilcraft damage $COLX $COLY $COLZ 0 250"
ctl wait 12 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$FORMED" = true ] || fail "putting the plank back did not form the cube again (formed=$FORMED)"
CLOSED_DAMAGE=$SLOT0_DAMAGE
CLOSED_TANK=$TANK
ctl wait 40 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
CLOSED_CURED=$((CLOSED_DAMAGE - SLOT0_DAMAGE))
[ "$CLOSED_CURED" -gt 0 ] || fail "the repair did not resume once the shell was whole (damage $SLOT0_DAMAGE)"
[ $((CLOSED_TANK - TANK)) -eq $((MB_PER_DAMAGE * CLOSED_CURED)) ] \
    || fail "the resumed repair spent $((CLOSED_TANK - TANK)) mB for $CLOSED_CURED points"
echo "  the plank back: formed=true, damage $CLOSED_DAMAGE -> $SLOT0_DAMAGE in 40 ticks again"

echo "== Y2: the Colossal Blood Chest repairs a whole anvil stack, one price each =="
# The same rule as D2, on the machine the review names: a stack of two chipped anvils (a legal
# vanilla stack) becomes two anvils for 2 x 125 mB, the custom name carried, and a tank that
# cannot pay for both leaves the stack untouched.
load_slot "$COLX" "$COLY" "$COLZ" 0 minecraft:diamond_pickaxe
load_slot "$COLX" "$COLY" "$COLZ" 45 empty
say "evilcraft blood $COLX $COLY $COLZ 125"
ctl wait 10 >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:chipped_anvil[minecraft:custom_name='\"Colossal pair\"'] 2" >/dev/null
ctl wait 8 >/dev/null
open_menu "$COLX" "$COLY" "$COLZ" 50
put_in_slot 301 minecraft:chipped_anvil
ctl close >/dev/null 2>&1 || true
ctl wait 20 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "${SLOT1_ITEM:-}" = "minecraft:chipped_anvil" ] || fail "slot 1 holds ${SLOT1_ITEM:-nothing}, expected the anvil stack"
saved_item "$COLX" "$COLY" "$COLZ" 1
case "$SAVED_ITEM" in
    *"count: 2"*chipped_anvil*) echo "  125 mB: the colossal's stack of two is untouched" >&2 ;;
    *) fail "125 mB changed the colossal's anvil stack: '$SAVED_ITEM'" ;;
esac
[ "$TANK" = 125 ] || fail "the untouched stack left $TANK mB, expected the 125 the tank held"
say "evilcraft blood $COLX $COLY $COLZ 250"
ctl wait 20 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
saved_item "$COLX" "$COLY" "$COLZ" 1
case "$SAVED_ITEM" in
    *"count: 2"*) echo "  the colossal converted both anvils (count 2)" >&2 ;;
    *) fail "the colossal's repaired stack holds '$SAVED_ITEM', expected count 2" ;;
esac
case "$SAVED_ITEM" in
    *'id: "minecraft:anvil"'*) ;;
    *) fail "the colossal's stack is not minecraft:anvil: '$SAVED_ITEM'" ;;
esac
case "$SAVED_ITEM" in
    *custom_name*Colossal\ pair*) echo "  the custom name travelled with both anvils" >&2 ;;
    *) fail "the colossal's repaired anvils lost their custom name: '$SAVED_ITEM'" ;;
esac
[ "$TANK" = 0 ] || fail "the colossal spent $((250 - TANK)) mB on two anvils, expected 250 (tank $TANK)"
load_slot "$COLX" "$COLY" "$COLZ" 1 empty

echo "== AA: the same pickaxe in a plain Blood Chest repairs at the same rate (the differential) =="
# The Colossal Blood Chest's own numbers are the Blood Chest's: `baseMBPerDamage` (5) and
# `ticksPerDamage` (2). The same damage-250 pickaxe, the same 1,000 mB and the same 40 ticks
# in a plain chest must cure the same number of points for the same blood.
DCHX=$((X0 - 2))
DCHZ=$((COLZ - 2))
ctl command "execute in minecraft:the_end run setblock $DCHX 101 $DCHZ $CHEST" >/dev/null
ctl wait 10 >/dev/null
say "evilcraft blood $DCHX 101 $DCHZ 1000"
ctl wait 10 >/dev/null
say "evilcraft load $DCHX 101 $DCHZ 0 minecraft:diamond_pickaxe"
ctl wait 10 >/dev/null
say "evilcraft damage $DCHX 101 $DCHZ 0 250"
ctl wait 12 >/dev/null
info "$DCHX" 101 "$DCHZ"
[ "${SLOT0_ITEM:-}" = "minecraft:diamond_pickaxe" ] || fail "the plain chest's slot 0 holds ${SLOT0_ITEM:-nothing}"
PLAIN_DAMAGE=$SLOT0_DAMAGE
PLAIN_TANK=$TANK
PLAIN_TICKS_START=$TIME
ctl wait 40 >/dev/null
info "$DCHX" 101 "$DCHZ"
PLAIN_TICKS=$((TIME - PLAIN_TICKS_START))
PLAIN_CURED=$((PLAIN_DAMAGE - SLOT0_DAMAGE))
PLAIN_SPENT=$((PLAIN_TANK - TANK))
[ "$PLAIN_TICKS" -gt 0 ] || fail "no ticks passed between the two plain-chest reads"
[ "$PLAIN_SPENT" -eq $((MB_PER_DAMAGE * PLAIN_CURED)) ] \
    || fail "the plain chest spent $PLAIN_SPENT mB for $PLAIN_CURED points"
# The same rule as the colossal chest's: one point a two ticks and 5 mB a point, measured over
# this leg's own window (the readout's latency is in both numbers, so the bound scales).
[ "$PLAIN_CURED" -le $((PLAIN_TICKS / TICKS_PER_DAMAGE + 1)) ] \
    || fail "the plain chest cured $PLAIN_CURED points in $PLAIN_TICKS ticks, faster than one a $TICKS_PER_DAMAGE"
[ "$PLAIN_CURED" -ge $((PLAIN_TICKS / TICKS_PER_DAMAGE - 1)) ] \
    || fail "the plain chest cured $PLAIN_CURED points in $PLAIN_TICKS ticks, slower than one a $TICKS_PER_DAMAGE"
echo "  plain chest: $PLAIN_TICKS ticks, damage $PLAIN_DAMAGE -> $SLOT0_DAMAGE ($PLAIN_CURED points), tank -$PLAIN_SPENT mB — one point a $TICKS_PER_DAMAGE ticks for $MB_PER_DAMAGE mB, as the colossal chest's $COLOSSAL_RATE"

echo "== AB: the controller breaks into itself carrying its tank, and a plank drops itself =="
# `copy_tank_data` runs on the colossal chest's loot table too: the three machines' drops all
# carry their tank through `block_drops` + `minecraft:block_entity_data`. A plank's table is the
# plain one, and both blocks are in the jar's `minecraft:mineable/axe` tag (an axe, not a
# pickaxe, is their tool).
ctl command "gamemode survival @s" >/dev/null
for _ in $(seq 1 10); do
    ctl wait 4 >/dev/null
    ctl messages 6 | jq -r '.chat[]' | grep -qi 'survival mode' && break
done
ctl command "clear @s" >/dev/null 2>&1 || true
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_axe 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
# No damaged stack in the machine before the tank is set: a formed machine would spend some of
# the 4,000 mB repairing it while the break is prepared, and the drop must carry exactly what
# the tank held.
say "evilcraft load $COLX $COLY $COLZ 0 empty"
ctl wait 10 >/dev/null
say "evilcraft blood $COLX $COLY $COLZ 4000"
ctl wait 12 >/dev/null
colossal "$COLX" "$COLY" "$COLZ"
[ "$TANK" = 4000 ] || fail "the harness left the tank at $TANK mB, expected 4,000"
[ -z "${SLOT0_ITEM:-}" ] \
    || fail "the machine still holds ${SLOT0_ITEM} in slot 0, so it would spend the tank repairing it before the break"
# A plank of the shell the player can see: the west wall. The far corner is the counterfactual's
# cell, but it sits behind the cube's own wall from outside, and `ctl break` refuses what it
# cannot see (measured 2026-11-23: "did not break").
break_survival "$COLX" "$COLY" "$((COLZ + 2))" "$PLANKS" minecraft:diamond_axe
echo "  a plank of the shell drops itself with an axe"
break_survival "$COLX" "$COLY" "$COLZ" "$COLOSSAL" minecraft:diamond_axe
# The player cannot stand in the cube's middle after the controller is gone (its cell is a
# plank), so the drop is collected rather than walked onto, and the hand is read the way the
# server holds it (`SelectedItem`), as `pick_hold` does.
ctl collect 8 >/dev/null 2>&1 || true
ctl wait 12 >/dev/null
AB_SLOT=$(ctl inventory | jq -r --arg i "$COLOSSAL" '[.hotbar[] | select(.item == $i) | .slot] | .[0] // ""')
[ -n "$AB_SLOT" ] || fail "the dropped controller was not picked up: $(ctl inventory | jq -c '.hotbar')"
ctl select $((AB_SLOT + 1)) >/dev/null
ctl wait 6 >/dev/null
selected_item_line "the dropped $COLOSSAL"
eval "$(python3 "$PORT/tools/parse_components.py" "$HELD_COMPONENTS" "$COLOSSAL")"
AB_CARRIED=${carried:-none}
[ "${ok:-0}" = 1 ] || fail "the dropped colossal chest carries no block entity data: '$HELD_COMPONENTS'"
[ "$AB_CARRIED" = 4000 ] || fail "the dropped colossal chest carries $AB_CARRIED mB, expected 4000"
[ "$fluid" = "evilcraft:blood" ] || fail "the dropped colossal chest carries '$fluid', expected evilcraft:blood"
echo "  the controller broke into itself carrying $AB_CARRIED mB of $fluid"

echo "== AC: a Dark Tank holds sixteen buckets, and a filled extractor's crouching click fills it =="
# `BlockEntityDarkTank.BASE_CAPACITY` = 16,000 mB; upstream's `BlockDarkTank.useItemOn` hands a
# click on the tank to `FluidUtil.interactWithFluidHandler`, which fills it from the held fluid
# container. This port's container is the extractor, and its gesture is the crouching click.
DTX=$((X0 + 4))
DTY=101
DTZ=6
place_chest "$DTX" "$DTY" "$DTZ" "$DARK_TANK"
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 0 ] || fail "a fresh dark tank reads $TANK mB, expected 0"
[ "$CAPACITY" = "$DARK_TANK_CAPACITY" ] \
    || fail "the dark tank reports a $CAPACITY mB tank, expected $DARK_TANK_CAPACITY"
echo "  a fresh $DARK_TANK at $DTX,101,$DTZ reads $TANK/$CAPACITY mB"
# The clamp: 14,000 mB in the tank and a full 5,000 mB extractor leave 2,000 mB behind.
say "evilcraft blood $DTX $DTY $DTZ 14000"
ctl wait 10 >/dev/null
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 14000 ] || fail "the harness left the dark tank at $TANK mB, expected 14,000"
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 9 >/dev/null
ctl wait 2 >/dev/null
say "evilcraft held 5000"
ctl wait 10 >/dev/null
held_blood "before the fill"
[ "$HELD_BLOOD" = 5000 ] || fail "the extractor holds $HELD_BLOOD mB, expected the 5,000 the command set"
ctl command "execute in minecraft:the_end run tp @s $((DTX - 2)).5 101 $DTZ.5" >/dev/null
ctl wait 6 >/dev/null
crouch_click "$DTX" "$DTY" "$DTZ" .5 "$DARK_TANK"
darktank "$DTX" "$DTY" "$DTZ"
DT_MOVED=$((TANK - 14000))
held_blood "after the crouching click"
DT_HELD=$HELD_BLOOD
if [ "$SELFTEST" = darktank ]; then
    echo "(selftest=darktank: asserting the crouching click poured nothing)"
    DT_MOVED=0
fi
[ "$DT_MOVED" = 2000 ] \
    || fail "the crouching click poured $DT_MOVED mB into the tank, expected the 2,000 mB room it had"
[ "$TANK" = "$DARK_TANK_CAPACITY" ] \
    || fail "the dark tank holds $TANK mB after the click, expected $DARK_TANK_CAPACITY"
[ "$DT_HELD" = 3000 ] \
    || fail "the extractor holds $DT_HELD mB after the click, expected the 3,000 that did not fit"
echo "  extractor 5000 -> $DT_HELD mB, dark tank 14000 -> $TANK mB (the 2,000 mB room, the rest kept)"
# The tank is the pin's saved `fluids` list, the same compound the chests keep:
# `/data get block` reads the block entity's own data, not the readout's clamp.
ctl command "data get block $DTX $DTY $DTZ fluids" >/dev/null
ctl wait 10 >/dev/null
DT_SAVED=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 "has the following block data" || true)
case "$DT_SAVED" in
    "$DTX, $DTY, $DTZ has the following block data: [{Amount: 16000, Fluid: \"evilcraft:blood\"}]")
        echo "  the dark tank's saved tank: $DT_SAVED" >&2 ;;
    *) fail "the dark tank's saved data is not 16000 mB of evilcraft:blood: '$DT_SAVED'" ;;
esac

echo "== AD: a standing click pours nothing, and neither does a crouching click on stone =="
# The same tank, emptied, and the same full extractor: standing, the click pours nothing; the
# same crouching click on a vanilla block pours nothing; the same crouching click on the tank
# pours the whole 5,000. That triple is the gesture's and the block's differential.
say "evilcraft blood $DTX $DTY $DTZ 0"
say "evilcraft held 5000"
ctl wait 12 >/dev/null
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 0 ] || fail "the harness left the dark tank at $TANK mB, expected 0"
stand_click "$DTX" "$DTY" "$DTZ" .5 "$DARK_TANK"
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 0 ] || fail "a standing click poured $TANK mB into the tank, expected nothing"
held_blood "after the standing click"
[ "$HELD_BLOOD" = 5000 ] || fail "a standing click emptied the extractor to $HELD_BLOOD mB"
echo "  standing: extractor $HELD_BLOOD mB, dark tank $TANK mB (nothing moved)"
ctl command "execute in minecraft:the_end run setblock $DTX 101 $((DTZ + 2)) minecraft:stone" >/dev/null
ctl wait 6 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $((DTX - 2)).5 101 $((DTZ + 2)).5" >/dev/null
ctl wait 6 >/dev/null
crouch_click "$DTX" 101 $((DTZ + 2)) .99 minecraft:stone
held_blood "after the crouching click on stone"
[ "$HELD_BLOOD" = 5000 ] \
    || fail "a crouching click on minecraft:stone emptied the extractor to $HELD_BLOOD mB"
echo "  the same crouching click on $(block_name "$DTX" 101 $((DTZ + 2))): extractor still $HELD_BLOOD mB"
ctl command "execute in minecraft:the_end run tp @s $((DTX - 2)).5 101 $DTZ.5" >/dev/null
ctl wait 6 >/dev/null
crouch_click "$DTX" "$DTY" "$DTZ" .5 "$DARK_TANK"
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 5000 ] || fail "the crouching click on the tank left it at $TANK mB, expected 5000"
held_blood "after the crouching click on the tank"
[ "$HELD_BLOOD" = 0 ] || fail "the crouching click on the tank left $HELD_BLOOD mB in the extractor"
echo "  crouching on the tank: extractor 5000 -> $HELD_BLOOD mB, dark tank 0 -> $TANK mB"

echo "== AE: the broken tank drops itself carrying its blood, and places back full =="
# `copy_tank_data` runs on the dark tank's loot table too; 12,000 mB is more than the ten-bucket
# machines could carry, so this also proves the tank's own sixteen-bucket capacity travels.
ctl command "clear @s" >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
say "evilcraft blood $DTX $DTY $DTZ 12000"
ctl wait 12 >/dev/null
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 12000 ] || fail "the harness left the dark tank at $TANK mB, expected 12,000"
break_survival "$DTX" "$DTY" "$DTZ" "$DARK_TANK" minecraft:diamond_pickaxe
pick_hold "$DTX" "$DTY" "$DTZ" "$DARK_TANK"
eval "$(python3 "$PORT/tools/parse_components.py" "$HELD_COMPONENTS" "$DARK_TANK")"
DT_CARRIED=${carried:-none}
[ "${ok:-0}" = 1 ] || fail "the dropped dark tank carries no block entity data: '$HELD_COMPONENTS'"
[ "$DT_CARRIED" = 12000 ] || fail "the dropped dark tank carries $DT_CARRIED mB, expected 12000"
[ "$fluid" = "evilcraft:blood" ] || fail "the dropped dark tank carries '$fluid', expected evilcraft:blood"
place_held "$DTX" "$DTY" "$DTZ" "$DARK_TANK"
darktank "$DTX" "$DTY" "$DTZ"
[ "$TANK" = 12000 ] || fail "the placed dark tank reads $TANK mB, expected the carried 12,000"
[ "$CAPACITY" = "$DARK_TANK_CAPACITY" ] || fail "the placed dark tank reports a $CAPACITY mB tank"
echo "  the dark tank broke into itself carrying $DT_CARRIED mB of $fluid, and places back at $TANK/$CAPACITY mB"

echo "== AF: a cursed tool goes into the Purifier's purge slot by a click =="
# `BlockPurifier.useItemOn` puts the held item into `SLOT_PURIFY` one at a time, and
# `ToolBadEnchantPurifyAction.canWork` reads `DataComponents.ENCHANTMENTS` on it. The pickaxe
# is the game's own: a `/give` with one `minecraft:vanishing_curse` on it.
PUX=$((X0 + 6))
PUY=101
PUZ=-10
ctl command "execute in minecraft:the_end run setblock $PUX 101 $PUZ minecraft:air" >/dev/null 2>&1 || true
ctl command "clear @s" >/dev/null
ctl wait 4 >/dev/null
place_chest "$PUX" "$PUY" "$PUZ" "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$TANK" = 0 ] || fail "a fresh purifier reads $TANK mB, expected 0"
[ "$CAPACITY" = "$PURIFIER_TANK" ] \
    || fail "the purifier reports a $CAPACITY mB tank, expected $PURIFIER_TANK"
[ "$PURGE_ITEM" = empty ] || fail "a fresh purifier's purge slot holds $PURGE_ITEM, expected empty"
echo "  a fresh $PURIFIER at $PUX,101,$PUZ reads $TANK/$CAPACITY mB, purge $PURGE_ITEM"
ctl command "item replace entity @s hotbar.0 with $CURSED_PICKAXE 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
[ "$(ctl inventory | jq -r '.held.item')" = "minecraft:diamond_pickaxe" ] \
    || fail "the cursed pickaxe is not in hand: '$(ctl inventory | jq -r '.held.item')'"
ctl command "execute in minecraft:the_end run tp @s $((PUX - 2)).5 101 $PUZ.5" >/dev/null
ctl wait 6 >/dev/null
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = "minecraft:diamond_pickaxe" ] \
    || fail "the click put $PURGE_ITEM in the purge slot, expected the pickaxe"
[ "$PURGE_COUNT" = 1 ] || fail "the purge slot holds $PURGE_COUNT pickaxes, expected one"
[ "$ENCH" = 1 ] || fail "the purge slot's pickaxe carries $ENCH enchantment(s), expected 1"
[ "$CURSES" = 1 ] || fail "the purge slot's pickaxe carries $CURSES curse(s), expected 1"
[ "$TANK" = 0 ] || fail "the click moved $TANK mB into the tank, expected nothing (it was empty)"
saved_item "$PUX" "$PUY" "$PUZ" 0
case "$SAVED_ITEM" in
    *minecraft:enchantments*minecraft:vanishing_curse*)
        echo "  the saved pickaxe carries the curse" >&2 ;;
    *) fail "the purge slot's saved item does not carry the curse: '$SAVED_ITEM'" ;;
esac

echo "== AG: sixty ticks of the action strip the curse for exactly one bucket =="
# `ToolBadEnchantPurifyAction.PURIFY_DURATION` = 60 ticks for a level-1 curse, and the
# completed action takes `setBuckets(getBucketsFloored() - 1, rest)`: one bucket, 1,000 mB.
say "evilcraft blood $PUX $PUY $PUZ $PURIFIER_TANK"
ctl wait 12 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
[ "$TANK" = "$PURIFIER_TANK" ] \
    || fail "the harness left the purifier at $TANK mB, expected $PURIFIER_TANK"
ctl wait 140 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
PURIFIED_TANK=$TANK
PURIFIED_ENCH=$ENCH
PURIFIED_CURSES=$CURSES
PURIFIED_ITEM=$PURGE_ITEM
saved_item "$PUX" "$PUY" "$PUZ" 0
if [ "$SELFTEST" = purifier ]; then
    echo "(selftest=purifier: asserting the curse is still on the tool)"
    PURIFIED_ENCH=1
    PURIFIED_CURSES=1
fi
[ "$PURIFIED_CURSES" = 0 ] \
    || fail "the pickaxe still carries $PURIFIED_CURSES curse(s) after 140 ticks, expected none"
[ "$PURIFIED_ENCH" = 0 ] \
    || fail "the pickaxe still carries $PURIFIED_ENCH enchantment(s) after 140 ticks, expected none"
case "$SAVED_ITEM" in
    *minecraft:enchantments*)
        [ "$SELFTEST" = purifier ] \
            || fail "the purge slot's saved item still carries minecraft:enchantments: '$SAVED_ITEM'" ;;
    *)
        [ "$SELFTEST" != purifier ] \
            || fail "the saved item carries no minecraft:enchantments to keep: '$SAVED_ITEM'" ;;
esac
[ "$PURIFIED_TANK" = $((PURIFIER_TANK - 1000)) ] \
    || fail "the tank is $PURIFIED_TANK mB after one curse, expected $((PURIFIER_TANK - 1000))"
[ "$PURIFIED_ITEM" = "minecraft:diamond_pickaxe" ] \
    || fail "the pickaxe left the purge slot ($PURIFIED_ITEM), the action strips and keeps nothing"
echo "  the curse is gone from the saved pickaxe and the tank fell $PURIFIER_TANK -> $PURIFIED_TANK mB (one bucket)"
# The empty hand takes it back out (`BlockPurifier.useWithoutItem`'s `isEmpty()` branch). AF's
# insert left the hand empty, so this is the same click a player makes on a loaded machine.
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = empty ] || fail "the empty-hand click left $PURGE_ITEM in the purge slot"
[ "$(ctl inventory | jq -r '.counts["minecraft:diamond_pickaxe"] // 0')" -ge 1 ] \
    || fail "the purified pickaxe did not come back to the hand: $(ctl inventory | jq -c '.counts')"
echo "  the empty hand took the purified pickaxe back out of the purge slot"

echo "== AH: the same cursed pickaxe in a Blood Chest keeps its curse (the differential) =="
# The purifier's effect is that machine's own rule (`ToolBadEnchantPurifyAction`): the same
# item, the same 140 ticks and a working Blood Chest touch no enchantment at all —
# `RepairItemTickAction` writes damage, and an undamaged tool is not even taken.
DCHX=$((X0 + 6))
# Two cells north of the purifier, and **not** the arena's edge: `open_menu` stands the player
# at `$z.5`, which for a negative z is the cell *south* of the block, and beyond the arena's
# floor there is nothing to stand on (measured 2026-11-23: the player fell and the click missed).
DCHZ=$((PUZ - 2))
ctl command "clear @s" >/dev/null
ctl command "execute in minecraft:the_end run setblock $DCHX 101 $DCHZ $CHEST" >/dev/null
ctl wait 10 >/dev/null
say "evilcraft blood $DCHX 101 $DCHZ 1000"
ctl wait 10 >/dev/null
ctl command "item replace entity @s hotbar.0 with $CURSED_PICKAXE 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
open_menu "$DCHX" 101 "$DCHZ" 11
put_in_slot 300 minecraft:diamond_pickaxe
ctl close >/dev/null 2>&1 || true
ctl wait 12 >/dev/null
info "$DCHX" 101 "$DCHZ"
[ "${SLOT0_ITEM:-}" = "minecraft:diamond_pickaxe" ] \
    || fail "the chest's slot 0 holds ${SLOT0_ITEM:-nothing}, expected the cursed pickaxe"
DCH_TANK=$TANK
ctl wait 140 >/dev/null
info "$DCHX" 101 "$DCHZ"
saved_item "$DCHX" 101 "$DCHZ" 0
case "$SAVED_ITEM" in
    *minecraft:enchantments*minecraft:vanishing_curse*)
        echo "  a Blood Chest for 140 ticks: the pickaxe still carries the curse" >&2 ;;
    *) fail "the Blood Chest stripped the curse too ('$SAVED_ITEM'), so the purifier's result proves nothing" ;;
esac
[ "$TANK" = "$DCH_TANK" ] \
    || fail "the chest spent $((DCH_TANK - TANK)) mB on an undamaged tool"

echo "== AI: a standing extractor click loads the slot, the empty hand takes it back, the crouch pours =="
# The blood route: a crouching click with a filled extractor (upstream's `FluidUtil` branch,
# with this port's container and gesture). A **standing** click puts the extractor in the slot
# instead and pours nothing, which is the gesture's differential; the empty hand then takes it
# back with its blood intact, and the crouching click pours exactly the room the tank has.
ctl command "execute in minecraft:the_end run tp @s $((PUX - 2)).5 101 $PUZ.5" >/dev/null
ctl wait 6 >/dev/null
empty_hand
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 9 >/dev/null
ctl wait 4 >/dev/null
say "evilcraft held 5000"
ctl wait 10 >/dev/null
held_blood "before the pour"
[ "$HELD_BLOOD" = 5000 ] || fail "the extractor holds $HELD_BLOOD mB, expected the 5,000 the command set"
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$TANK" = "$PURIFIED_TANK" ] \
    || fail "a standing click with the extractor poured $TANK mB, expected the tank to stay at $PURIFIED_TANK"
[ "$PURGE_ITEM" = "$EXTRACTOR" ] \
    || fail "the standing click put $PURGE_ITEM in the purge slot, expected the extractor"
echo "  standing: the extractor went into the purge slot and the tank stayed at $TANK mB"
# The insert emptied the hand, so the same click takes it straight back out.
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = empty ] || fail "the extractor did not come back out ($PURGE_ITEM)"
held_blood "the extractor back in hand"
[ "$HELD_BLOOD" = 5000 ] || fail "the extractor holds $HELD_BLOOD mB after the round trip, expected 5,000"
# A fresh extractor from the command, as phase AC pours with: a click that follows the machine's
# own write to the player's hand carries the *client's* older stack there (measured 2026-11-23:
# the same crouching click poured 0 mB straight after the take-back), and this phase measures the
# pour, not the client's inventory prediction.
ctl command "item replace entity @s hotbar.8 with $EXTRACTOR 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 9 >/dev/null
ctl wait 4 >/dev/null
say "evilcraft held 5000"
ctl wait 10 >/dev/null
held_blood "before the crouching pour"
[ "$HELD_BLOOD" = 5000 ] || fail "the extractor holds $HELD_BLOOD mB before the pour, expected 5,000"
crouch_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
POURED=$((TANK - PURIFIED_TANK))
held_blood "after the crouching pour"
[ "$POURED" = 1000 ] \
    || fail "the crouching click poured $POURED mB, expected the 1,000 mB room the $PURIFIED_TANK mB tank had"
[ "$TANK" = "$PURIFIER_TANK" ] || fail "the tank is $TANK mB, expected $PURIFIER_TANK"
[ "$HELD_BLOOD" = 4000 ] \
    || fail "the extractor holds $HELD_BLOOD mB after the pour, expected the 4,000 that did not fit"
saved_tank "$PUX" "$PUY" "$PUZ"
[ "$SAVED_MB" = "$PURIFIER_TANK" ] || fail "the purifier's saved tank is $SAVED_MB mB"
echo "  extractor 5000 -> $HELD_BLOOD mB, purifier tank $PURIFIED_TANK -> $TANK mB (the 1,000 mB room, the rest kept)"

echo "== AJ: a Vengeance Pickaxe's curse is immune to the Purifier's action =="
# `ToolBadEnchantPurifyAction.canWork` and `.work` both guard every curse with
# `!(purifyItem.getItem() instanceof ItemVengeancePickaxe)`: the pickaxe's own curse survives a
# full tank and 140 ticks. The same rig with a cursed vanilla pickaxe strips the curse and spends
# one bucket (phase AG's rule), which makes this an item-id rule, not a change to curses.
ctl command "execute in minecraft:the_end run tp @s $((PUX - 2)).5 101 $PUZ.5" >/dev/null
empty_hand
# An unknown id is refused rather than written as a nameless stack (the item exists in the
# package; this id does not).
say "evilcraft load $PUX $PUY $PUZ 0 evilcraft:no_such_item"
ctl wait 12 >/dev/null
UNKNOWN=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'not an item this game has' || true)
[ -n "$UNKNOWN" ] || fail "an unknown item id was accepted in the purifier: $(ctl messages 12 | jq -r '.chat[]' | tail -3)"
echo "  $UNKNOWN"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = empty ] || fail "the refused unknown id wrote $PURGE_ITEM into the purge slot"

ctl command "item replace entity @s hotbar.0 with $CURSED_VENGEANCE 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
HELD_ID=$(ctl inventory | jq -r '.held.item')
[ "$HELD_ID" = "$VENGEANCE_PICKAXE" ] \
    || fail "the Vengeance Pickaxe is not in hand: '$HELD_ID' (is the item registered?)"
echo "  the carried $VENGEANCE_PICKAXE reads back in hand"
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = "$VENGEANCE_PICKAXE" ] \
    || fail "the click put $PURGE_ITEM in the purge slot, expected the Vengeance Pickaxe"
[ "$PURGE_COUNT" = 1 ] || fail "the purge slot holds $PURGE_COUNT pickaxes, expected one"
[ "$ENCH" = 1 ] || fail "the Vengeance Pickaxe carries $ENCH enchantment(s), expected 1"
[ "$CURSES" = 1 ] || fail "the Vengeance Pickaxe carries $CURSES curse(s), expected 1"
[ "$TANK" = "$PURIFIER_TANK" ] || fail "the purifier tank is $TANK mB, expected $PURIFIER_TANK"
saved_item "$PUX" "$PUY" "$PUZ" 0
case "$SAVED_ITEM" in
    *minecraft:enchantments*minecraft:vanishing_curse*)
        echo "  the saved Vengeance Pickaxe carries the curse" >&2 ;;
    *) fail "the purge slot's pickaxe has no curse in its saved item: '$SAVED_ITEM'" ;;
esac

# The tank is already full from AI (a re-set to the same value is a no-op at this pin).
# After 140 ticks the *readout* is the assertion: it reads the saved stack and the saved
# tank on the server, so `ench 1, curses 1` and an unchanged 3,000 mB prove the refusal.
# (A second `/data get block … Items[0]` would wait for a *changing* reply, which a
# deliberately unchanged stack never produces — the stale-line trap from slice 3.)
ctl wait 140 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
IMMUNE_TANK=$TANK
IMMUNE_ENCH=$ENCH
IMMUNE_CURSES=$CURSES
IMMUNE_TICK=$TICK
if [ "$SELFTEST" = vengeance ]; then
    echo "(selftest=vengeance: asserting the pickaxe's curse was stripped and blood spent)"
    IMMUNE_ENCH=0
    IMMUNE_CURSES=0
    IMMUNE_TANK=$((PURIFIER_TANK - 1000))
fi
[ "$IMMUNE_CURSES" = 1 ] \
    || fail "after 140 ticks the Vengeance Pickaxe carries $IMMUNE_CURSES curse(s), expected its curse kept"
[ "$IMMUNE_ENCH" = 1 ] \
    || fail "after 140 ticks the Vengeance Pickaxe carries $IMMUNE_ENCH enchantment(s), expected one"
[ "$IMMUNE_TANK" = "$PURIFIER_TANK" ] \
    || fail "the purifier spent $((PURIFIER_TANK - IMMUNE_TANK)) mB on the immune pickaxe, expected none"
[ "$IMMUNE_TICK" = 0 ] \
    || fail "the immune pickaxe left the action clock at $IMMUNE_TICK, expected 0 (canWork refused)"
echo "  the Vengeance Pickaxe kept its curse and the tank stayed $IMMUNE_TANK/$PURIFIER_TANK mB after 140 ticks"

# The differential: the same machine, tank, curse and 140 ticks on a vanilla pickaxe.
# First take the immune pickaxe back out with an empty-hand click (`useWithoutItem`).
empty_hand
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = empty ] \
    || fail "the empty hand did not take the Vengeance Pickaxe back out ($PURGE_ITEM)"
empty_hand
ctl command "item replace entity @s hotbar.0 with $CURSED_PICKAXE 1" >/dev/null
ctl wait 6 >/dev/null
ctl select 1 >/dev/null
ctl wait 2 >/dev/null
[ "$(ctl inventory | jq -r '.held.item')" = minecraft:diamond_pickaxe ] \
    || fail "the cursed vanilla pickaxe is not in hand"
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = minecraft:diamond_pickaxe ] \
    || fail "the differential click put $PURGE_ITEM in the purge slot, expected the vanilla pickaxe"
ctl wait 140 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
[ "$CURSES" = 0 ] || fail "the vanilla pickaxe kept $CURSES curse(s), expected the action to strip it"
[ "$TANK" = $((PURIFIER_TANK - 1000)) ] \
    || fail "the vanilla pickaxe's purify left the tank at $TANK mB, expected $((PURIFIER_TANK - 1000))"
echo "  the same curse on a vanilla pickaxe was stripped for one bucket (the item is the difference)"

echo "== AK: a comparator reads the Purifier's tank fraction (the analogue output) =="
# `BlockPurifier.getAnalogOutputSignal` is ceil(15 x amount / capacity) with CyclopsCore's
# COMPARATOR_MULTIPLIER = 15: a full 3,000 mB tank reads 15, 2,000 reads 10 and 1,800 reads 9.
# (The dispatch note's "2,000 mB reads 9" is an arithmetic slip; the artefact's 2,000/3,000 is
# ceil(10.0) = 10, and the 9 is exercised at 1,800 mB below.) The hook cannot read a block entity,
# so the port publishes the fraction from the machine's tick and notifies the comparator when it
# changes — the wire is read again without nudging the comparator.
[ "$(block_name "$PUX" "$PUY" "$PUZ")" = "$PURIFIER" ] \
    || fail "the purifier from phase AJ is gone; the comparator rig needs it"
# An uncursed item in the purge slot would make the action spend blood mid-phase.
empty_hand
stand_click "$PUX" "$PUY" "$PUZ" .5 "$PURIFIER"
purifier "$PUX" "$PUY" "$PUZ"
[ "$PURGE_ITEM" = empty ] \
    || fail "the purge slot still holds $PURGE_ITEM before the comparator phase"
say "evilcraft blood $PUX $PUY $PUZ $PURIFIER_TANK"
ctl wait 12 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
[ "$TANK" = "$PURIFIER_TANK" ] || fail "the purifier tank is $TANK mB, expected $PURIFIER_TANK"
comparator_rig "$PUX" "$PUY" "$PUZ"
FULL_POWERED=$(cmp_powered "$PUX" "$PUY" "$PUZ")
FULL_SIGNAL=$(cmp_signal "$PUX" "$PUY" "$PUZ")
echo "  purifier $PURIFIER_TANK/$PURIFIER_TANK mB: comparator powered=$FULL_POWERED, wire reads $FULL_SIGNAL"
[ "$FULL_POWERED" = true ] || fail "a comparator beside a full purifier must be powered, got '$FULL_POWERED'"
[ "$FULL_SIGNAL" = 15 ] || fail "a full purifier must read comparator 15, got '$FULL_SIGNAL'"

# A tank changing without a block-state change: the port's OutputSignal notification is the only
# thing that makes the comparator read again (no nudge between the three readings).
say "evilcraft blood $PUX $PUY $PUZ 2000"
ctl wait 16 >/dev/null
purifier "$PUX" "$PUY" "$PUZ"
[ "$TANK" = 2000 ] || fail "the purifier tank is $TANK mB after setting 2,000"
TWO_SIGNAL=$(cmp_signal "$PUX" "$PUY" "$PUZ")
echo "  purifier 2000/$PURIFIER_TANK mB: wire reads $TWO_SIGNAL (ceil(15 x 2000 / 3000) = 10)"
[ "$TWO_SIGNAL" = 10 ] \
    || fail "2,000 of 3,000 mB is ceil(10.0) = comparator 10, got '$TWO_SIGNAL'"
if [ "$SELFTEST" = comparator ]; then
    echo "  SELFTEST: asserting the 2,000 mB reading is NOT 10 (this must fail)"
    [ "$TWO_SIGNAL" != 10 ] || fail "SELFTEST flipped: the comparator read $TWO_SIGNAL at 2,000 mB, as it should"
fi

say "evilcraft blood $PUX $PUY $PUZ 1800"
ctl wait 16 >/dev/null
NINE_SIGNAL=$(cmp_signal "$PUX" "$PUY" "$PUZ")
echo "  purifier 1800/$PURIFIER_TANK mB: wire reads $NINE_SIGNAL (ceil(15 x 1800 / 3000) = 9)"
[ "$NINE_SIGNAL" = 9 ] \
    || fail "1,800 of 3,000 mB is ceil(9.0) = comparator 9, got '$NINE_SIGNAL'"

echo "== AK: a comparator facing vanilla stone reads 0 (the vanilla differential) =="
STONE_X=200; STONE_Y=101; STONE_Z=10
ctl command "execute in minecraft:the_end run setblock $STONE_X $STONE_Y $STONE_Z minecraft:stone" >/dev/null
ctl wait 5 >/dev/null
comparator_rig "$STONE_X" "$STONE_Y" "$STONE_Z"
STONE_SIGNAL=$(cmp_signal "$STONE_X" "$STONE_Y" "$STONE_Z")
STONE_POWERED=$(cmp_powered "$STONE_X" "$STONE_Y" "$STONE_Z")
echo "  stone: comparator powered=$STONE_POWERED, wire reads $STONE_SIGNAL"
[ "$STONE_SIGNAL" = 0 ] || fail "a comparator facing stone must read 0, got '$STONE_SIGNAL'"
[ "$STONE_POWERED" = false ] || fail "a comparator facing stone must be unpowered, got '$STONE_POWERED'"

echo "== AL: the Blood Chest's 1-in-10,000 curse roll on a repaired damage point =="
# The artefact (`DamageableItemRepairAction.repair`, line 41) gates the curse on four
# conditions: !isBulk (the ordinary Blood Chest, not the Colossal's bulk action),
# addRandomBadEnchants=true, chance>=0 and random.nextInt(chance) == 0. A win picks a
# random `#minecraft:curse` tag entry (binding or vanishing curse) at level 1. The
# harness arms the draw for one chest position, so roll 0 and roll 9,999 can both be
# reached in game; the default (no armed draw) still uses the level's random stream.
CURSE_X=$((X0 + 6)); CURSE_Y=101; CURSE_Z=-12
ctl command "execute in minecraft:the_end run setblock $CURSE_X $CURSE_Y $CURSE_Z $CHEST" >/dev/null
ctl wait 10 >/dev/null
[ "$(block_name "$CURSE_X" "$CURSE_Y" "$CURSE_Z")" = "$CHEST" ] \
    || fail "the curse-roll chest did not place at $CURSE_X,$CURSE_Y,$CURSE_Z"
# Damage first with an **empty** tank: a funded chest repairs during the setup read
# (measured: a 2,000 mB chest took a damage-250 pickaxe to 242 in the 12 ticks the
# readout waits), so the starting damage would not be the value the scenario set.
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
give_to "$CURSE_X" "$CURSE_Y" "$CURSE_Z" minecraft:diamond_pickaxe
say "evilcraft damage $CURSE_X $CURSE_Y $CURSE_Z 0 250"
ctl wait 12 >/dev/null
info "$CURSE_X" "$CURSE_Y" "$CURSE_Z"
[ "${SLOT0_ITEM:-}" = minecraft:diamond_pickaxe ] \
    || fail "the curse chest's slot 0 holds ${SLOT0_ITEM:-nothing}, expected a clean pickaxe"
[ "$SLOT0_DAMAGE" = 250 ] || fail "the curse chest's pickaxe was not damaged (reads $SLOT0_DAMAGE)"
saved_item "$CURSE_X" "$CURSE_Y" "$CURSE_Z" 0
eval "$(python3 "$PORT/tools/parse_enchants.py" "$SAVED_ITEM")"
[ "$ok" = 0 ] || fail "the fresh pickaxe already carries enchantments: $SAVED_ITEM"
echo "  a clean damage-250 pickaxe in an unfunded chest carries no enchantments"

# Roll 0, curse index 1 = the artefact's `minecraft:vanishing_curse` tag entry.
say "evilcraft blood $CURSE_X $CURSE_Y $CURSE_Z 2000"
say "evilcraft curse $CURSE_X $CURSE_Y $CURSE_Z 0 1"
ctl wait 8 >/dev/null
info "$CURSE_X" "$CURSE_Y" "$CURSE_Z"
CURSE_DAMAGE_BEFORE=$SLOT0_DAMAGE
CURSE_TANK_BEFORE=$TANK
ctl wait 40 >/dev/null
info "$CURSE_X" "$CURSE_Y" "$CURSE_Z"
CURSE_DAMAGE_AFTER=$SLOT0_DAMAGE
saved_item "$CURSE_X" "$CURSE_Y" "$CURSE_Z" 0
eval "$(python3 "$PORT/tools/parse_enchants.py" "$SAVED_ITEM")"
echo "  roll 0: damage $CURSE_DAMAGE_BEFORE -> $CURSE_DAMAGE_AFTER, tank $CURSE_TANK_BEFORE -> $TANK, $SAVED_ITEM"
[ "$ok" = 1 ] || fail "roll 0 added no enchantment component: '$SAVED_ITEM'"
[ "$CURSE_DAMAGE_AFTER" -lt "$CURSE_DAMAGE_BEFORE" ] \
    || fail "roll 0 did not repair the pickaxe ($CURSE_DAMAGE_BEFORE -> $CURSE_DAMAGE_AFTER)"
[ "$TANK" -lt "$CURSE_TANK_BEFORE" ] \
    || fail "roll 0 repaired without spending blood ($CURSE_TANK_BEFORE -> $TANK)"
[ "$ench" = "minecraft:vanishing_curse@1" ] \
    || fail "roll 0 added '$ench', expected the artefact's minecraft:vanishing_curse@1"
if [ "$SELFTEST" = curse ]; then
    echo "  SELFTEST: asserting roll 0 added NO curse (this must fail)"
    [ "$ench" != "minecraft:vanishing_curse@1" ] \
        || fail "SELFTEST flipped: roll 0 added $ench, as it should"
fi

# The differential: a second, otherwise identical chest at roll 9,999 (inside
# nextInt's domain, but not the winning roll) repairs just as much and adds nothing.
NO_CURSE_X=$((X0 + 8)); NO_CURSE_Z=-12
ctl command "execute in minecraft:the_end run setblock $NO_CURSE_X $CURSE_Y $NO_CURSE_Z $CHEST" >/dev/null
ctl wait 10 >/dev/null
ctl command "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1" >/dev/null
ctl wait 4 >/dev/null
give_to "$NO_CURSE_X" "$CURSE_Y" "$NO_CURSE_Z" minecraft:diamond_pickaxe
say "evilcraft damage $NO_CURSE_X $CURSE_Y $NO_CURSE_Z 0 250"
ctl wait 12 >/dev/null
say "evilcraft blood $NO_CURSE_X $CURSE_Y $NO_CURSE_Z 2000"
say "evilcraft curse $NO_CURSE_X $CURSE_Y $NO_CURSE_Z 9999 1"
ctl wait 8 >/dev/null
info "$NO_CURSE_X" "$CURSE_Y" "$NO_CURSE_Z"
NO_DAMAGE_BEFORE=$SLOT0_DAMAGE
NO_TANK_BEFORE=$TANK
ctl wait 40 >/dev/null
info "$NO_CURSE_X" "$CURSE_Y" "$NO_CURSE_Z"
NO_DAMAGE_AFTER=$SLOT0_DAMAGE
saved_item "$NO_CURSE_X" "$CURSE_Y" "$NO_CURSE_Z" 0
eval "$(python3 "$PORT/tools/parse_enchants.py" "$SAVED_ITEM")"
echo "  roll 9999: damage $NO_DAMAGE_BEFORE -> $NO_DAMAGE_AFTER, tank $NO_TANK_BEFORE -> $TANK, $SAVED_ITEM"
[ "$ok" = 0 ] || fail "roll 9999 added an enchantment: '$SAVED_ITEM'"
[ "$NO_DAMAGE_AFTER" -lt "$NO_DAMAGE_BEFORE" ] \
    || fail "roll 9999 did not repair the pickaxe ($NO_DAMAGE_BEFORE -> $NO_DAMAGE_AFTER)"
[ "$TANK" -lt "$NO_TANK_BEFORE" ] \
    || fail "roll 9999 repaired without spending blood ($NO_TANK_BEFORE -> $TANK)"
echo "  the same tool, chest, tank and 40 ticks: roll 0 curses, roll 9,999 stays clean (the gate is roll == 0)"

echo "PASS: an empty chest reads 0/10,000 mB, an empty tank repairs nothing, 1,000 mB repairs one damage point every two ticks for exactly 5 mB a point, a chipped anvil becomes an anvil for 125 mB, a stack of two chipped anvils becomes two anvils for 250 mB with its custom name kept (125 mB leaves the stack alone) and the same stack in the Colossal Blood Chest does too, a damaged enchanted pickaxe keeps its enchantment through the repair, a legacy stack of two Blood Extractors is split into two single extractors (the first keeping the whole 4,000 mB, the second 0) and a crouching click on a 3,500 mB stain fills only the first (held 5,000, the other still 0), a player kill fills the carried Blood Extractor with a 20 HP zombie's 100..799 mB, a bow kill fills it too while a skeleton's arrow leaves it empty, the extractor in a container slot empties into the tank, the Blood Infuser reads all 10 of its carried recipe files (the Blook's included) and refuses an unknown id, holds a tier-3 recipe at tier 0 and infuses it once a Promise of Tenacity III tiers the machine, crafts every carried recipe for its own blood cost, empties a filled extractor into its tank, breaks back into itself, a chest holding 6,500 mB is broken into an item that carries that tank (and none of its stacks) and places back with 6,500 mB while an empty chest carries nothing and a 20,000 mB payload is clamped to 10,000, a crouching click with a 4,000 mB extractor draws exactly the 1,000 mB it had room for out of a 3,500 mB blood stain (standing moves nothing, stone moves nothing), a drained stain removes its block, a stone placed onto a stain lands above it (minecraft:replaceable is inert at this pin), a 20 HP zombie that dies of a fall leaves a blood stain holding 20 x 20 = 400 mB which a second fall on the same cell tops up to 800 mB with no second block, the same zombie killed with a sword leaves the cell empty, a hollow 3x3x3 shell of reinforced undead planks around one colossal blood chest reads formed=true with 45 repair slots and repairs a damage-250 pickaxe one point every two ticks for 5 mB (1,000 mB = 200 points until the tank is empty) while a single shell block out reads formed=false and repairs nothing over the same ticks, the open machine still empties a 1,000 mB extractor from its 50-slot menu into its tank and repairs again once the plank is back, a plain Blood Chest repairs the same pickaxe at the same rate, the controller breaks into an item carrying its 4,000 mB with an axe, a dark tank reads 0/16000 mB and a crouching click with a full extractor pours exactly the 2,000 mB room a 14,000 mB tank had (the extractor keeping 3,000, the saved fluids 16,000) while a standing click and a crouching click on stone pour nothing and the same crouching click on a fresh tank pours its 5,000, and a tank broken with 12,000 mB drops itself carrying that blood and places back at 12,000/16000 mB, a fresh purifier reads 0/3000 mB and a standing click puts one cursed diamond pickaxe in its purge slot (the block entity's own saved Items[0] carrying minecraft:vanishing_curse), 140 ticks strip that curse for exactly one bucket (the saved item carries no minecraft:enchantments any more and the tank fell 3000 -> 2000) while the same cursed pickaxe in a working blood chest for the same ticks still carries it and spends nothing, the empty hand takes the purified pickaxe back out, a standing click puts a filled extractor in the purge slot instead of pouring and the same click crouching pours exactly the 1,000 mB room the tank had (the extractor keeping 4,000, the saved fluids 3000), the Blook is crafted from a book through the Blood Infuser for 500 mB in 500 ticks, and a cursed Vengeance Pickaxe keeps its curse and its Purifier tank of blood after 140 ticks while the same curse on a vanilla pickaxe is stripped for one bucket, a comparator beside a full Purifier is powered and reads 15 on the wire in front of it, a tank of 2,000/3,000 mB re-reads as 10 without a nudge (upstream's ceil(15 x amount/capacity), the artefact's answer where the brief guessed 9) and 1,800/3,000 mB reads 9, while a comparator facing vanilla stone reads 0, and the Blood Chest's non-bulk repair adds minecraft:vanishing_curse@1 at forced roll 0 while the same tool, chest, tank and ticks at roll 9,999 stay clean and both repair and spend blood"
