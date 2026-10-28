#$ test: run-fail
#$ rules: R9
#$ panics: too many owners of a `Player` (4,294,967,295)
## R9 (verdict 59): "A count is checked before it rolls over: before each +1 of an owner count ... it is
## compared with its maximum, and at the maximum the program stops ("too many owners of a `Player`
## (4,294,967,295)")". `p` is one owner and the loop adds 4,294,967,294 more, which brings the count to
## its maximum; the next `shares p` stops the program, where a count that rolled over would let one
## owner's end free the player the others still use (safety validation (b), hole 5). Rules section 2,
## the stop texts. With `mem.forget` gone (verdict 58), the owners are kept in a list, so running this
## test needs about 32 GiB of memory (4,294,967,295 eight-byte owners).

class Player:
    hp: int = 100

fn main():
    p = Player()
    owners: Array[shares Player] = []
    for _ in 0..4294967295:
        owners.push(shares p)     # the last round stops the program before its +1
    println(owners.len())         # never reached
