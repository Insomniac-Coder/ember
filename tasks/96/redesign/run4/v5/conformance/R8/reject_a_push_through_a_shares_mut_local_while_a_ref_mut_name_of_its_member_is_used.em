#$ test: compile-fail
#$ rules: R8, R9
#$ help: move line 19 below line 20
## R8 (a), (f): a list reached through a local that holds a link (`t`, a `shares mut` owner) gives
## addresses, not positions, and a push may move the members stored inline in it; while `boss`, a
## `ref mut` address, is still used, such a reshape through another name is refused at the reshape
## (E3022). Settled in step 4 (`V5/changes.md` section 8, items 10 and 14). The allowed neighbour is
## `accept_a_push_through_a_shares_mut_local_after_the_ref_mut_names_last_use.em`.

struct Member:
    hp: int

class Team:
    members: Array[Member] = []

fn main():
    t = shares mut Team(members=[Member(hp=10)])
    boss = ref mut t.members[0]
    t.members.push(Member(hp=5))    #$ error[E3022]: `boss` is the only name for `t.members[0]` until its last use on line 20, and line 19 adds to `t.members`, which may move it: `t` is a link to the team, so `boss` is an address, not a position; add after `boss`'s last use: move line 19 below line 20
    boss.hp -= 1
    println(t.members[0].hp)
