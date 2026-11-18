#$ test: compile-fail
#$ rules: ERR-1, STD-9
# `[ERR-1]` — printing an error with neither `Display` nor `Debug` is refused as printing refuses
# (`E2040`, `[STD-9]`).

struct Failure:
    cell: RefCell[int]

fn fail() -> Result[int, Failure]:
    return Err(Failure(RefCell(1)))

fn main():
    match fail():
        Ok(_):
            println("ok")
        Err(e):
            println(e)    #$ error[E2040]: `RefCell[i64]` does not implement `Debug`, which printing a `Failure` needs
