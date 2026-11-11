#$ test: run-pass
#$ rules: LEX-1
#$ profiles: debug, development
#$ stdout: héllo 世界
# `[LEX-1]` - a source file is UTF-8: names stay ASCII, text and comments may hold any of it.

fn main():
    println("héllo 世界")
