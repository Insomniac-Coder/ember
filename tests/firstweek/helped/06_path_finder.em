# Find the shortest path through a grid maze with breadth-first search and draw it with stars.

const MAZE: str = """
##########
#S.....#.#
#.##.#.#.#
#.#..#...#
#.#.####.#
#...#..#.#
###.#.##.#
#.....#..#
#.###...E#
##########
"""

@derive(Copy)
@derive(Hash)
struct Pos:
    row: int
    col: int

    fn neighbours(self) -> Array[Pos]:
        return [Pos(self.row - 1, self.col), Pos(self.row + 1, self.col),
                Pos(self.row, self.col - 1), Pos(self.row, self.col + 1)]

class Grid:
    cells: Array[Array[String]] = []
    start: Pos = Pos(0, 0)
    goal: Pos = Pos(0, 0)

    fn init(self, text: str):
        for line in text.trim().split("\n"):
            row: Array[String] = []
            for c in line:
                if c == 'S':
                    self.start = Pos(self.cells.len(), row.len())
                elif c == 'E':
                    self.goal = Pos(self.cells.len(), row.len())
                row.push(c.to_string())
            self.cells.push(row)

    fn rows(self) -> int:
        return self.cells.len()

    fn cols(self) -> int:
        return self.cells[0].len()

    fn is_open(self, p: Pos) -> bool:
        if p.row < 0 or p.row >= self.rows() or p.col < 0 or p.col >= self.cols():
            return false
        return self.cells[p.row][p.col] != "#"

fn shortest_path(grid: Grid) -> Option[Array[Pos]]:
    came_from: Map[Pos, Pos] = {}
    queue: Array[Pos] = [grid.start]
    seen: Set[Pos] = {grid.start}
    head = 0
    while head < queue.len():
        current = queue[head]
        head += 1
        if current == grid.goal:
            path = [current]
            while current != grid.start:
                current = came_from[current]
                path.push(current)
            path.reverse()
            return Some(path)
        for next in current.neighbours():
            if grid.is_open(next) and next not in seen:
                seen.add(next)
                came_from[next] = current
                queue.push(next)
    return None

fn main():
    grid = Grid(MAZE)
    println(f"maze is {grid.rows()}x{grid.cols()}, start {grid.start!r}, goal {grid.goal!r}")
    match shortest_path(grid):
        Some(path):
            println(f"shortest path: {path.len() - 1} steps")
            for p in path:
                if grid.cells[p.row][p.col] == ".":
                    grid.cells[p.row][p.col] = "*"
            for row in grid.cells:
                println(row.join(""))
        None:
            println("no path")

    # block the only corridor near the goal and try again
    grid.cells[7][7] = "#"
    grid.cells[8][6] = "#"
    if shortest_path(grid) is None:
        println("after blocking the corridor there is no path any more")
