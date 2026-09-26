#$ test: compile-fail
#$ rules: FFI-10, FFI-8, TYP-15
#$ error[E0900]: foreign static access requires a Copy C-compatible value without borrowed fields

@derive(Copy)
struct HasView:
    text: str

unsafe extern "C":
    static foreign_view: HasView

fn main():
    pass
