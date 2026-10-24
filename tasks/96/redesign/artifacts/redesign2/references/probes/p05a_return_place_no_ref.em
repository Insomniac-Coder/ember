# A5: the return type already says `ref mut int`; the body returns the place without repeating `ref mut`.
struct Pool:
    dense: Array[int]

    fn at(mut self, i: int) -> ref mut int:
        return self.dense[i]

fn main():
    p = Pool(dense=[1, 2])
    r = p.at(0)
    r += 1
    println(p.dense)
