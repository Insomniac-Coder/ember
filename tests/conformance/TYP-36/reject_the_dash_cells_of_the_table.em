#$ test: compile-fail
#$ rules: TYP-36
## `[TYP-36]` - the table's `—` cells: a bound that needs an interface the type does not implement is `E2040`.

class K:
    n: int

type Pct = u8 in 0 ..= 100

[T: needs Clone]
fn need_Clone(x: T):
    pass

[T: needs Copy]
fn need_Copy(x: T):
    pass

[T: needs Eq]
fn need_Eq(x: T):
    pass

[T: needs Ord]
fn need_Ord(x: T):
    pass

[T: needs Hash]
fn need_Hash(x: T):
    pass

[T: needs Default]
fn need_Default(x: T):
    pass

[T: needs Display]
fn need_Display(x: T):
    pass

[T: needs Debug]
fn need_Debug(x: T):
    pass

fn cell12(v: f64):
    need_Hash(v)  #$ error[E2040]

fn cell38(v: void):
    need_Display(v)  #$ error[E2040]

fn cell49(v: String):
    need_Copy(v)  #$ error[E2040]

fn cell81(v: Result[int, int]):
    need_Default(v)  #$ error[E2040]

fn cell85(v: Array[int]):
    need_Copy(v)  #$ error[E2040]

fn cell93(v: Map[int, int]):
    need_Copy(v)  #$ error[E2040]

fn cell95(v: Map[int, int]):
    need_Ord(v)  #$ error[E2040]

fn cell96(v: Map[int, int]):
    need_Hash(v)  #$ error[E2040]

fn cell100(v: Set[int]):
    need_Copy(v)  #$ error[E2040]

fn cell102(v: Set[int]):
    need_Ord(v)  #$ error[E2040]

fn cell103(v: Set[int]):
    need_Hash(v)  #$ error[E2040]

fn cell107(v: K):
    need_Ord(v)  #$ error[E2040]

fn cell109(v: K):
    need_Default(v)  #$ error[E2040]

fn cell110(v: K):
    need_Display(v)  #$ error[E2040]

fn cell117(v: Pct):
    need_Default(v)  #$ error[E2040]

fn main():
    println(1)
