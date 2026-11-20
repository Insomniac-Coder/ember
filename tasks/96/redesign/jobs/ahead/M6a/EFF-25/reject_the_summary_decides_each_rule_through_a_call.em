# Clause EFF-25.2: "It decides: a walk of a collection and a call that may change it (`E3020`,
# `[BRW-18]`); a read-only name and a call that may change what it reads (`E3021`); a call that
# hands a function a part of what it may end or move before its last use of that parameter
# (`E3031`, `[BRW-21]`); a view of an object reached through a link and a call that may end the
# object (`E3060`); a function value stored where it would change its holder (`E3022`)"
#$ test: compile-fail
#$ rules: EFF-25, BRW-18, BRW-1, BRW-21, RC-5, CLO-17
# One case per code, each through a call. The clause's other facts are other milestones': a lock
# this thread holds (`E7018`) and what tasks and threads change (M9), `init` and `E2100`, and
# `@cached` (M11). No text in the spec for these programs: code only.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

class Bus:
    handlers: Array[fn()] = []

static mut NUMS: Array[int] = []
static mut PLAYERS: Array[Player] = []
static mut BUS: Bus = Bus()

fn grow():
    NUMS.push(owns 1)

fn bump_all():
    for ref mut x in NUMS:
        x += 1

fn clear_then(xs: Span[int]):
    NUMS.clear()
    println(xs.len())

fn remove_all():
    PLAYERS.clear()

fn main():
    NUMS.push(owns 1)
    for n in NUMS:
        println(n)
        grow()    #$ error[E3020]
    first = ref NUMS[0]
    bump_all()    #$ error[E3021]
    println(first)
    clear_then(NUMS)    #$ error[E3031]
    PLAYERS.push(owns Player())
    cam = Camera()
    cam.target = weakref PLAYERS[0]
    if cam.target exists as p:
        remove_all()
        println(p.hp)    #$ error[E3060]
    BUS.handlers.push(owns fn(): BUS.handlers.clear())    #$ error[E3022]
