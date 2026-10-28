#$ test: run-pass
#$ rules: R15, R6
#$ stdout: 2
#$ stdout: 1
## R15: the lines E2221 gives: the list is the caller's, and a view of it may be held across `yield`
## (AN-77).

struct Link:
    value: int
    next: Option[owns Link] = None

gen fn countdown(head: Link) -> Generator[int] from head:
    cur = ref head
    while true:
        yield cur.value
        if cur.next exists as nx:
            cur = ref nx
        else:
            break

fn main():
    list = Link(value=2, next=Some(Link(value=1)))
    for v in countdown(list):
        println(v)
