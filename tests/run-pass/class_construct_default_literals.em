#$ test: run-pass
#$ rules: CLS-1, CLS-3, STR-2
#$ stdout: 3
#$ stdout: true
#$ stdout: 5
#$ stdout: true

class Config:
    retries: i32 = 3
    enabled: bool = true

fn main():
    config = Config()
    println(config.retries)
    println(config.enabled)
    custom = Config(5)
    println(custom.retries)
    println(custom.enabled)
