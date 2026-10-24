# A generator walking a class field, consumed in place by a loop whose body itself yields.
# Today: E2229 on `replay` (any loop over a field across yield). The proposal allows `replay` when it is
# consumed in place; this probe records what today says, the attack is y15.
class Door:
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:
            yield entry.len()

gen fn lengths(door: Door) -> Generator[int]:
    for n in door.replay():
        yield n

fn main():
    door = Door()
    door.log.push("open".to_string())
    for n in lengths(door):
        door.log.clear()
        println(n)
