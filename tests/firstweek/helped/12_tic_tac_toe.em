# Tic-tac-toe between two scripted players on a 3x3 board, rejecting bad moves and detecting a win or a draw.

enum Mark:
    Empty
    X
    O

static LINES: Array[(int, int, int)] = [
    (0, 1, 2), (3, 4, 5), (6, 7, 8),
    (0, 3, 6), (1, 4, 7), (2, 5, 8),
    (0, 4, 8), (2, 4, 6),
]

fn symbol(m: Mark) -> StrView:
    return match m:
        Empty => "."
        X => "X"
        O => "O"

struct Board:
    cells: Array[Mark] = [Mark.Empty for _ in 0..9]

    fn show():
        for row in 0..3:
            println([symbol(this.cells[row * 3 + col]) for col in 0..3].join(" "))

    fn place(pos: int, m: Mark) -> Result[(), String]:
        if pos < 0 or pos >= 9:
            return Err(f"square {pos} is off the board")
        if this.cells[pos] != Mark.Empty:
            return Err(f"square {pos} is taken")
        this.cells[pos] = m
        return Ok(())

    fn winner() -> Option[Mark]:
        for a, b, c in LINES:
            if this.cells[a] != Mark.Empty and this.cells[a] == this.cells[b] == this.cells[c]:
                return Some(this.cells[a])
        return None

    fn full() -> bool:
        return all(c != Mark.Empty for c in this.cells)

fn other(m: Mark) -> Mark:
    return Mark.O if m == Mark.X else Mark.X

fn main():
    board = Board()
    moves = [4, 0, 4, 8, 2, 6, 3, 5, 12, 1, 7]
    player = Mark.X
    for pos in moves:
        match board.place(pos, player):
            Err(msg):
                println(f"{symbol(player)}: {msg}, try again")
                continue
            Ok(_):
                pass
        println(f"{symbol(player)} plays {pos}")
        board.show()
        if board.winner() exists as w:
            println(f"{symbol(w)} wins!")
            return
        if board.full():
            println("draw")
            return
        player = other(player)
    println("out of moves")
