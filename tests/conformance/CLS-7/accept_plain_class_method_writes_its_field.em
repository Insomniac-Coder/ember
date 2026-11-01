#$ test: run-pass
#$ rules: CLS-7, EXC-16
#$ stdout: [2, 3]
# A plain class method writes its object through the field access rule.
class Bag:
    items: Array[int]

    fn init():
        this.items = [1]

    fn replace():
        this.items = [2]
        this.items.push(owns 3)

fn main():
    b = Bag()
    b.replace()
    println(b.items)
