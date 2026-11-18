#$ test: compile-fail
#$ rules: TXT-8, BRW-1
## `[TXT-8]` - a change that may reshape the text (replacing a `String`, `push_str`, `push`, `+=`) ends
## every view into it; the view read afterwards is `E3021`/`E3022`.

fn main():
    s = String("abc")
    v = s.as_str()
    s.push_str("d")    #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(v)
    w = s.trim()
    s.push('x')    #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(w)
    x = s[0..1]
    s = String("zzz")    #$ error[E3021]: `s` cannot be written while it is borrowed
    println(x)
    z = s.as_str()
    s += "q"    #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(z)
