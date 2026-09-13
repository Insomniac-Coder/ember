#$ test: run-pass
#$ rules: CTL-1, CTL-2, OWN-1, DRP-2
#$ stdout: 7
#$ stdout: 1
#$ stdout: 107

struct Resource:
    value: i32

    fn drop(mut self):
        println(self.value + 100)

fn main():
    values: Array[Resource] = Array[Resource]()
    values.push(Resource(7))
    for value in values:
        println(value.value)
    println(values.len())
