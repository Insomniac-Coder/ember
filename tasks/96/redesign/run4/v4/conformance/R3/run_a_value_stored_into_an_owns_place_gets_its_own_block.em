#$ test: run-pass
#$ rules: R3
#$ stdout: 3
## R3 and `V4/rules.md` section 2 ("an allocation where `owns` puts a value in its own block"):
## `Option[owns Tree]` holds each child alone in its own heap block, so a type may contain itself
## (AN-41's NEW). Verified by its output and a clean AddressSanitizer and leak run (each block freed
## once, with its owner).

struct Tree:
    value: int
    left: Option[owns Tree] = None
    right: Option[owns Tree] = None

fn main():
    t = Tree(value=1, left=Some(Tree(value=2)))
    if t.left exists as l:
        println(t.value + l.value)
