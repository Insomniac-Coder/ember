#$ test: run-pass
#$ rules: BRW-15
#$ stdout: 1
## design rules: R12, R5
## R12 (rules.md lines 752-756, 776-779; `changes.md` section 8, item 4): only a call through a stored
## function value whose functions may change their own state is a change of its holder. The function
## stored in `hs` has no state of its own (it changes `game`'s real `score`), so calling it through the
## plain loop name `h` changes nothing `h` reads, and the walk is allowed. Its neighbour with a handler
## that keeps its own copy is refused (reject_click_walks_stateful_handlers_with_a_plain_name).

fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for h in hs:
        h()
    return score

fn main():
    println(game())
