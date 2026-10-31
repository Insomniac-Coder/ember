#$ test: run-pass
#$ rules: IFC-4, TYP-17, LEX-19
#$ stdout: [3] [x]
# D-407 (5) — a default method's `where Item: Display` gives the interface's
# associated type that bound in the method's body, which formats an `Item`.
# It was `E1010`, "`Self.Item` cannot be formatted yet".

interface Shower:
    type Item
    fn get() -> Item
    fn shown() -> String where Item: needs Display:
        v = this.get()
        return f"[{v}]"

struct N:
    n: int

extend N implements Shower:
    type Item = int
    fn get() -> int:
        return this.n

struct T:
    s: String

extend T implements Shower:
    type Item = String
    fn get() -> String:
        return this.s.clone()

fn main():
    println(N(n = 3).shown(), T(s = "x".to_string()).shown())
