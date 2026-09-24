#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CTL-3, CTL-5
#$ stdout: 7
#$ stdout: 9

class LoopElse:
    first: i32
    second: i32
    items: Array[i32]

    fn init(mut self):
        while false:
            pass
        else:
            self.first = 7
        self.items = Array()
        for _item in self.items:
            pass
        else:
            self.second = 9
        for i in 0..0:
            pass
        else:
            pass

fn main():
    value = LoopElse()
    println(value.first)
    println(value.second)
