# A16: a general implementation and a special one for one type (Python: functools.singledispatch).
interface Describe:
    fn describe(self) -> String

extend[T: Debug] T implements Describe:
    fn describe(self) -> String:
        return f"{self:?}"

extend i32 implements Describe:
    fn describe(self) -> String:
        return f"int {self}"

fn main():
    x: i32 = 3
    println(x.describe())
    println("a".describe())
