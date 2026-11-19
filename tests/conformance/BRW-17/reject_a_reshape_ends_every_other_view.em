# Clause BRW-17.1: "Every other view (a kept view; a view reached through a link; a view into a
# value's own contents: a `String`'s text, an `Array` field's elements, an enum's payload; every
# view a call gives, `[LT-1]`) ends at any change that may reshape what it points into, made through
# any name that may reach it, and a later use is `E3060`."
#$ test: compile-fail
#$ rules: BRW-17, BCK-3, LT-1, LT-46, RC-1
# One function per kind of address. Codes only (shape B7).

struct Enemy:
    hp: int

struct Holder:
    target: ref Enemy

struct Bag:
    items: Array[int]

enum State:
    Walking(path: Array[int])
    Idle

class Team:
    names: Array[String] = []

fn first(xs: Array[Enemy]) -> ref Enemy:
    return ref xs[0]

fn a_kept_view():
    enemies = [Enemy(hp=1)]
    h = Holder(target=ref enemies[0])
    enemies.push(owns Enemy(hp=2))
    println(h.target.hp)    #$ error[E3060]

fn a_view_reached_through_a_link():
    team = shares mut Team()
    team.names.push(owns String("a"))
    n = team.names[0]
    team.names.clear()
    println(n)    #$ error[E3060]

fn a_strings_text():
    s = String("ab")
    w: StrView = s
    s.push_str("c")
    println(w)    #$ error[E3060]

fn an_array_fields_elements():
    bag = Bag(items=[1, 2])
    v = bag.items.as_span()
    bag.items.push(owns 3)
    println(v[0])    #$ error[E3060]

fn an_enums_payload():
    st = State.Walking([1, 2])
    match st:
        State.Walking(path):
            st = State.Idle
            println(path.len())    #$ error[E3060]
        State.Idle:
            pass

fn a_view_a_call_gives():
    enemies = [Enemy(hp=1)]
    top = first(enemies)
    enemies.push(owns Enemy(hp=2))
    println(top.hp)    #$ error[E3060]

fn main():
    a_kept_view()
    a_view_reached_through_a_link()
    a_strings_text()
    an_array_fields_elements()
    an_enums_payload()
    a_view_a_call_gives()
