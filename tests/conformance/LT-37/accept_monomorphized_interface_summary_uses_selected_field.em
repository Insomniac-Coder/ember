#$ test: run-pass
#$ rules: LT-35, LT-36, LT-37, TST-19
#$ stdout: 21

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

interface PairReader:
    fn read(self, pair: Pair) -> i32

struct LeftReader implements PairReader:
    marker: i32

    fn read(self, pair: Pair) -> i32:
        return pair.left[0] + self.marker

fn apply[R: PairReader](reader: R, pair: Pair) -> i32:
    return reader.read(pair)

fn main():
    left: Array[i32] = Array[i32]()
    left.push(20)
    right: Array[i32] = Array[i32]()
    right.push(22)
    pair = Pair(left.as_span(), right.as_span())
    right.push(23)
    println(apply(LeftReader(1), pair))
