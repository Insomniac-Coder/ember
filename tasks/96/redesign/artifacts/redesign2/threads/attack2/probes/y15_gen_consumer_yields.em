# Proposed syntax. Rule 2 of A14 (answers report 7.3; kept by the first attack): a generator that walks
# a class field may hold that loan across `yield` when it is consumed by a `for` loop in the same function,
# never stored or passed, and the loop body does not reshape the field. Here the loop body is `yield n`:
# it reshapes nothing itself, but while `lengths` is suspended its own consumer runs, and that code clears
# `door.log`. `replay`'s position then points into the freed buffer.
# Today: E2229 on `replay` (probe x07), so today is safe; the hole is in the proposed rule.
class Door:
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:          # a loan into door.log's buffer, held across yield
            yield entry.len()

gen fn lengths(door: Door) -> Generator[int]:
    for n in door.replay():             # consumed in place; the body's only statement is a yield
        yield n

fn main():
    door = Door()
    door.log.push("open".to_string())
    door.log.push("shut".to_string())
    for n in lengths(door):
        door.log.clear()                # runs while replay is suspended inside lengths
        println(n)
