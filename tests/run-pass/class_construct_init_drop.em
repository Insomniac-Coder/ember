#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6, OWN-2
#$ assert-c: contains(em_Holder_init)
#$ assert-c: contains(drop_adapter)
#$ stdout: 1

class Holder:
    values: Array[i32]

    fn init(mut self, owned values: Array[i32]):
        self.values = values

    fn drop(mut self):
        println(self.values.len())

fn main():
    values: Array[i32] = Array()
    values.push(1)
    holder = Holder(values)
