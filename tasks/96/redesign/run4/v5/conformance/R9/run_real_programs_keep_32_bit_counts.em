#$ test: run-pass
#$ rules: RC-1, WK-11
#$ stdout: 300 300
#$ assert-c: contains("too many owners of a `Player` (4,294,967,295)")
#$ assert-c: contains("too many `weakref` links to a `Player` (4,294,967,295)")
## design rules: R9, R11
## R9, R11 (verdict 96): every program but the two count-limit tests keeps the 32-bit owner count and
## weak count, whose limit is 4,294,967,295: 300 owners and 300 `weakref` links of one player, more than
## an 8-bit count holds, run to the end, and the emitted C compares each count with the 32-bit maximum,
## its stop texts naming 4,294,967,295 (rules section 2). The 8-bit tests are
## `run_fail_too_many_owners_of_one_player_stop_the_program.em` and
## `R11/run_fail_too_many_weakref_links_to_one_player_stop_the_program.em`.

class Player:
    hp: int = 100

fn main():
    p = Player()
    owners: Array[shares Player] = []
    links: Array[weakref Player] = []
    for _ in 0..300:
        owners.push(shares p)
        links.push(weakref p)
    println(owners.len(), links.len())
