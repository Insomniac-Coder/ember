# A5 (iii): a class method returning a reference into its own field, then a push while it lives.
class Pool:
    dense: Array[int]

    fn at(self, i: int) -> ref mut int:
        return ref mut self.dense[i]

fn main():
    p = Pool(dense=[1, 2])
    r = p.at(0)
    r += 1
    p.dense.push(3)
    println(p.dense)
