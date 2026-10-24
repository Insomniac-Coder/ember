# A14: Ember has no async (reserved word); its generators are the same state-machine frames.
# A Python programmer's generator that walks its own local list:

gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:
        yield s.clone()

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:
            yield entry.len()

fn main():
    for s in lines():
        println(s)
    d = Door()
    for n in d.replay():
        println(n)
