#$ test: run-pass
#$ rules: GRM-27
#$ profiles: debug, release, shipping
#$ stdout: 5 0 4 16
#$ 3 4
#$ 4 10 18
#$ 6 5
#$ 2
# A comprehension is its loop nest: `for` clauses nest left to right, `if`
# clauses filter, and the loop variables are scoped to it.

fn main():
    squares = [x * x for x in range(5)]
    println(len(squares), squares[0], squares[2], squares[4])
    xs = [3, 8, 1, 9, 4]
    small = [x for x in xs if x < 5]
    println(len(small), small[2])
    pairs = [a * b for a in [1, 2, 3] for b in range(4, 7) if a == b - 3]
    println(pairs[0], pairs[1], pairs[2])
    grid = [[1, 2], [3], [4, 5, 6]]
    flat = [v for row in grid for v in row]
    println(len(flat), flat[4])
    names = ["ann", "bo", "cy"]
    println(len([n for n in names if n.len() == 2]))
