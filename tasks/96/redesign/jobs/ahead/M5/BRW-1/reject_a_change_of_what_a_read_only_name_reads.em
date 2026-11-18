# Clause BRW-1.9: "While a read-only view (a plain name for a part, a `ref` name, a plain parameter, a
# `Span` parameter, a plain name holding a `MutSpan`, a plain loop, pattern or opened name, a view a
# call gave) is still used, what it points into is not changed through any other name (`E3021`, at
# the change) unless the compiler proves the two differ (...). The refusal gives the ways out that
# compile in that program: "`enemies[0].hp = 1` (line 7) changes the enemy `boss` reads (`boss =
# enemies[0]`, line 5), and `boss` is used on line 9; change it after `boss`'s last use, or write
# `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`"."
#$ test: compile-fail
#$ rules: BRW-1, BRW-12, BCK-3, LT-1, SPN-3
# The first message is the spec's, with this file's line numbers; the other kinds of read-only view
# have no text in the spec: codes only. (A parameter's promise inside its callee is `[BRW-21]`'s:
# Q9.)

struct Enemy:
    hp: int

fn first(xs: Array[Enemy]) -> ref Enemy:
    return ref xs[0]

fn a_part_name():
    enemies = [Enemy(hp=5)]
    boss = enemies[0]
    println(boss.hp)
    enemies[0].hp = 1    #$ error[E3021]: `enemies[0].hp = 1` (line 24) changes the enemy `boss` reads (`boss = enemies[0]`, line 22), and `boss` is used on line 25; change it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`
    println(boss.hp)

fn a_ref_name():
    x = 1
    r = ref x
    x = 5    #$ error[E3021]
    println(r)

fn a_span():
    xs = [1, 2, 3]
    v = xs.as_span()
    xs[0] = 9    #$ error[E3021]
    println(v[0])

fn a_plain_name_holding_a_mutspan():
    xs = [1, 2, 3]
    m = xs.as_mut_span()
    xs[0] = 9    #$ error[E3021]
    println(m[0])

fn a_view_a_call_gave():
    enemies = [Enemy(hp=5)]
    top = first(enemies)
    enemies[0].hp = 1    #$ error[E3021]
    println(top.hp)

fn main():
    a_part_name()
    a_ref_name()
    a_span()
    a_plain_name_holding_a_mutspan()
    a_view_a_call_gave()
