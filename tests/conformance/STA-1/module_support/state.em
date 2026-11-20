#$ test: parse-pass
#$ rules: STA-1
# Module variables another module changes by their names.

pub static mut FRAME: int = 0
pub static mut COUNT: int = 1
pub static mut NAMES: Array[String] = [String("a")]
pub static TABLE: Array[int] = [1, 2, 3]

pub fn bump():
    FRAME += 1
    COUNT += 1
