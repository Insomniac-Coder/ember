#$ test: run-pass
#$ rules: CELL-8, CELL-5, THR-1
#$ stdout: 2
## `[CELL-8]` — `RefCell[T]` is `!Sync`, which only keeps it out of what threads
## share: a class that is not `@sync` holds one and mutates through it.

class Log:
    lines: RefCell[Array[str]]

    fn init(mut self):
        self.lines = RefCell(Array[str]())

    fn add(self, line: str):
        with lines = self.lines.borrow_mut():
            lines.push(line)

fn main():
    log = Log()
    log.add("one")
    log.add("two")
    with lines = log.lines.borrow():
        println(lines.len())
