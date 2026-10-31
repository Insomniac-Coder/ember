#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CTL-3, CTL-5
#$ stdout: 7
#$ stdout: 9

class LoopElse:
    first: i32
    second: i32
    items: Array[i32]

    fn init():
        while false:
            pass
        else:
            this.first = 7
        this.items = Array()
        for _item in this.items:
            pass
        else:
            this.second = 9
        for i in 0..0:
            pass
        else:
            pass

fn main():
    value = LoopElse()
    println(value.first)
    println(value.second)
