# A5 (iii) misuse: the same reference kept across the push.
class Pool:
    dense: Array[int]

    fn at(self, i: int) -> ref mut int:
        return ref mut self.dense[i]

fn main():
    p = Pool(dense=[1, 2])
    r = p.at(0)
    p.dense.push(3)
    r += 1
    println(p.dense)
