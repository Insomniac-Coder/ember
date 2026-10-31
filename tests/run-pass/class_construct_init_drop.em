#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6, OWN-2
#$ assert-c: contains(em_6Holder4init)
#$ assert-c: contains(drop_adapter)
#$ stdout: 1

class Holder:
    values: Array[i32]

    fn init(owns values: Array[i32]):
        this.values = values

    fn drop():
        println(this.values.len())

fn main():
    values: Array[i32] = Array()
    values.push(1)
    holder = Holder(values)
