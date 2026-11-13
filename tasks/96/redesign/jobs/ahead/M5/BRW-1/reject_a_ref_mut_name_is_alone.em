# Clause BRW-1.2: "While a `ref mut` name (a parameter, a local, a loop, pattern or opened name, a kept
# `ref mut` field; one that holds a `MutSpan` included) is still used, its place is read or changed
# only through it: another name that may name the same place (...) is refused, `E3022` to change it
# and `E3021` to read it ("`boss` is the only name for `enemies[0]` until line 9, and line 7 reads
# `enemies[0].hp`; read it through `boss`: `boss.hp`")"
#$ test: compile-fail
#$ rules: BRW-1, BCK-3, LT-46, SPN-3
# The first message is the spec's, with this file's line numbers; the other kinds of `ref mut` name
# have no text in the spec: codes only.

struct Enemy:
    hp: int

struct Grip:
    target: ref mut Enemy

fn a_local_read_by_another_name():
    enemies = [Enemy(hp=5)]
    boss = ref mut enemies[0]
    boss.hp -= 1
    println(enemies[0].hp)    #$ error[E3021]: `boss` is the only name for `enemies[0]` until line 22, and line 21 reads `enemies[0].hp`; read it through `boss`: `boss.hp`
    boss.hp -= 1

fn a_local_changed_by_another_name():
    enemies = [Enemy(hp=5)]
    boss = ref mut enemies[0]
    enemies[0].hp = 1    #$ error[E3022]
    boss.hp -= 1

fn a_name_holding_a_mutspan():
    xs = [1, 2, 3]
    d = ref mut xs.as_mut_span()
    println(xs[1])    #$ error[E3021]
    d[0] = 0

fn a_loop_name():
    enemies = [Enemy(hp=5), Enemy(hp=6)]
    for ref mut e in enemies:
        println(enemies[0].hp)    #$ error[E3021]
        e.hp -= 1

fn a_kept_ref_mut_field():
    enemies = [Enemy(hp=5)]
    g = Grip(target=ref mut enemies[0])
    enemies[0].hp = 2    #$ error[E3022]
    g.target.hp -= 1

fn main():
    a_local_read_by_another_name()
    a_local_changed_by_another_name()
    a_name_holding_a_mutspan()
    a_loop_name()
    a_kept_ref_mut_field()
