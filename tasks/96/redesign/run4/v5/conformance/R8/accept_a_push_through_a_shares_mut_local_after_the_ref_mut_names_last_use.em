#$ test: run-pass
#$ rules: R8, R9
#$ stdout: 9 2
## R8 (a), (f): the way out E3022's message gives: the push through the link `t` comes after the last use
## of `boss`, the `ref mut` address of a member stored inline. The nearest program to
## `reject_a_push_through_a_shares_mut_local_while_a_ref_mut_name_of_its_member_is_used.em` that is
## allowed.

struct Member:
    hp: int

class Team:
    members: Array[Member] = []

fn main():
    t = shares mut Team(members=[Member(hp=10)])
    boss = ref mut t.members[0]
    boss.hp -= 1
    t.members.push(Member(hp=5))
    println(t.members[0].hp, t.members.len())
