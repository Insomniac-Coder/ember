# Conway's Game of Life on a small wrapping grid, seeded with a glider and a blinker, printed for a few generations.

const WIDTH: int = 12
const HEIGHT: int = 8

struct World:
    cells: Array[Array[bool]]
    generation: int = 0

    fn alive_neighbours(self, row: int, col: int) -> int:
        count = 0
        for dr in -1..=1:
            for dc in -1..=1:
                if dr == 0 and dc == 0:
                    continue
                r = (row + dr + HEIGHT) % HEIGHT
                c = (col + dc + WIDTH) % WIDTH
                if self.cells[r][c]:
                    count += 1
        return count

    fn step(self) -> World:
        next = new_world()
        for r in 0..HEIGHT:
            for c in 0..WIDTH:
                n = self.alive_neighbours(r, c)
                alive = self.cells[r][c]
                next.cells[r][c] = n == 3 or (alive and n == 2)
        next.generation = self.generation + 1
        return next

    fn population(self) -> int:
        return sum(1 for row in self.cells for cell in row if cell)

    fn show(self):
        println(f"generation {self.generation}, population {self.population()}")
        for row in self.cells:
            println("".join(["#" if cell else "." for cell in row]))
        println("")

fn new_world() -> World:
    return World(cells=[[False for _ in 0..WIDTH] for _ in 0..HEIGHT])

fn from_pattern(pattern: Array[str], top: int, left: int) -> World:
    world = new_world()
    for r, line in pattern.iter().enumerate():
        for c, ch in line.chars().enumerate():
            if ch == "#":
                world.cells[top + r][left + c] = True
    return world

fn main():
    glider = [".#.", "..#", "###"]
    world = from_pattern(glider, 1, 1)
    blinker_row = 5
    for c in 7..10:
        world.cells[blinker_row][c] = True

    for _ in 0..5:
        world.show()
        world = world.step()
    world.show()

    # run on quietly and check that the glider keeps the population steady
    start = world.population()
    for _ in 0..40:
        world = world.step()
    if world.population() == start:
        println(f"population still {start} after generation {world.generation}")
    else:
        println(f"population changed from {start} to {world.population()}")
