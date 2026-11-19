#$ test: compile-fail
#$ rules: OWN-9

fn from_a_view(s: String) -> Array[String]:
    return [s]            #$ error[E3013]

fn main():
    a = String("a")
    xs = [a]
    println(a, xs.len())  #$ error[E3040]
    println(from_a_view(String("b")).len())
