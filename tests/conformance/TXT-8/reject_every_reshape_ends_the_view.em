#$ test: compile-fail
#$ rules: TXT-8, BRW-1
## `[TXT-8]` - a change that may reshape the text (replacing a `String`, `push_str`, `push`, `+=`) ends
## every view into it; the view read afterwards is `E3021`/`E3022`.

fn main():
    s = String("abc")
    v = s.as_str()
    s.push_str("d")
    println(v)    #$ error[E3060]
    w = s.trim()
    s.push('x')
    println(w)    #$ error[E3060]
    x = s[0..1]
    s = String("zzz")    #$ error[E3021]: `s` cannot be written while it is borrowed
    println(x)
    z = s.as_str()
    s += "q"
    println(z)    #$ error[E3060]
