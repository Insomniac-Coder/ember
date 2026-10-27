#$ test: run-fail
#$ rules: R11
#$ panics: too many `weakref` links to a `Player` (4,294,967,295)
## R11 (verdict 59): every `weakref` link adds one to its target's weak count, and "before each +1 the
## count is compared with its maximum, and at the maximum the program stops ("too many `weakref` links to
## a `Player` (4,294,967,295)")". The loop makes 4,294,967,295 links, which brings the count to its
## maximum; the next `weakref p` stops the program, where a count that rolled over to 0 would let the
## block go while links still test it. Rules section 2, the stop texts. Running this test needs about
## 32 GiB of memory (the links are kept in a list, eight bytes each).

class Player:
    hp: int = 100

fn main():
    p = Player()
    links: Array[weakref Player] = []
    for _ in 0..4294967296:
        links.push(weakref p)     # the last round stops the program before its +1
    println(links.len())          # never reached
