#$ test: compile-fail
#$ rules: R15
## R15: a walk with no position, held by views of the generator's own local, is E2221 (AN-77's
## `countdown`).

struct Link:
    value: int
    next: Option[owns Link] = None

gen fn countdown() -> Generator[int]:
    head = Link(value=2, next=Some(Link(value=1)))
    cur = ref head
    while true:
        yield cur.value    #$ error[E2221]: `cur` (line 12) views `head`, a local of this generator's own frame, which may move while it is suspended at the `yield` on line 14; a view of what the caller keeps may be held: take the list as a parameter, `gen fn countdown(head: Link) -> Generator[int] from head:` instead of line 10, and remove line 11
        if cur.next exists as nx:
            cur = ref nx
        else:
            break

fn main():
    for v in countdown():
        println(v)
