# A small matrix type with addition, multiplication, transpose and identity, tried on a few examples.

struct Matrix:
    rows: int
    cols: int
    data: Array[float]

    fn get(self, r: int, c: int) -> float:
        return self.data[r * self.cols + c]

    fn set(mut self, r: int, c: int, v: float):
        self.data[r * self.cols + c] = v

    fn transpose(self) -> Matrix:
        result = zeros(self.cols, self.rows)
        for r in 0..self.rows:
            for c in 0..self.cols:
                result.set(c, r, self.get(r, c))
        return result

    fn show(self):
        for r in 0..self.rows:
            row = [f"{self.get(r, c):7.2f}" for c in 0..self.cols]
            println(f"[{row.join(" ")} ]")

fn zeros(rows: int, cols: int) -> Matrix:
    return Matrix(rows=rows, cols=cols, data=[0.0 for _ in 0..rows * cols])

fn identity(n: int) -> Matrix:
    m = zeros(n, n)
    for i in 0..n:
        m.set(i, i, 1.0)
    return m

fn from_rows(rows: Array[Array[float]]) -> Matrix:
    m = zeros(rows.len(), rows[0].len())
    for r, row in rows.iter().enumerate():
        for c, v in row.iter().enumerate():
            m.set(r, c, v)
    return m

fn add(a: Matrix, b: Matrix) -> Result[Matrix, String]:
    if a.rows != b.rows or a.cols != b.cols:
        return Err(f"cannot add {a.rows}x{a.cols} and {b.rows}x{b.cols}")
    return Ok(Matrix(rows=a.rows, cols=a.cols, data=[x + y for x, y in zip(a.data, b.data)]))

fn multiply(a: Matrix, b: Matrix) -> Result[Matrix, String]:
    if a.cols != b.rows:
        return Err(f"cannot multiply {a.rows}x{a.cols} by {b.rows}x{b.cols}")
    result = zeros(a.rows, b.cols)
    for i in 0..a.rows:
        for j in 0..b.cols:
            total = 0.0
            for k in 0..a.cols:
                total += a.get(i, k) * b.get(k, j)
            result.set(i, j, total)
    return Ok(result)

fn main():
    a = from_rows([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
    b = from_rows([[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]])
    println("A =")
    a.show()
    println("B =")
    b.show()
    println("A x B =")
    multiply(a, b).unwrap().show()
    println("transpose(A) =")
    a.transpose().show()
    println("A + transpose(B) =")
    add(a, b.transpose()).unwrap().show()
    println("A + B =")
    match add(a, b):
        Ok(m) => m.show()
        Err(e) => println(f"error: {e}")
    same = multiply(b, identity(2)).unwrap()
    println(f"B x I == B: {same == b}")
