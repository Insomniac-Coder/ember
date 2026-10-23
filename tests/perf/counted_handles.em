#$ perf: counted
## `[BEN-6]` — counted-object code against a C++ intrusive count, non-atomic: a ring of 1,000 class
## handles, each turn storing another handle into each slot (the new object's count up, the old
## one's down), then a pass reading every object through the ring.

class Cell:
    value: int

    fn init(mut self, value: int):
        self.value = value

@overflow(wrap)
fn main():
    cells: Array[Cell] = []
    for i in 0..1000:
        cells.push(Cell(i))
    ring: Array[Cell] = []
    for i in 0..1000:
        ring.push(cells[i])
    total = 0
    for round in 0..20000:
        for i in 0..ring.len():
            ring[i] = cells[(i * 7 + round) % cells.len()]
        for i in 0..ring.len():
            total = total + ring[i].value
    println(total)
