struct R:
    pub v: Array[i32]

    fn drop():
        println(this.v[0])

fn main():
    a: Array[i32] = Array[i32]()
    a.push(owns 1)
    r = R(a)
    r.drop()
    println(2)

