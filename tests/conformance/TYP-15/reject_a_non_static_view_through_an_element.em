#$ test: compile-fail
#$ rules: TYP-15, LT-3
# `[TYP-15]` (D-352, D-353, D-198) — an `Array`'s elements take only `static`
# views, however the store reaches them: an element's field, a `mut`
# parameter or a `ref mut` aimed at an element, `mem.replace` or `mem.swap`
# of one, `extend` from a local array, or a generic function that pushes its argument. Each was
# accepted before and read freed memory.

struct P:
    a: str

fn put(mut x: str, v: str):
    x = v

fn store[T](mut xs: Array[T], owned x: T):
    xs.push(x)

fn main():
    names = ["ann", "bob"]
    ps = [P("x")]
    s = String.from("zed")
    ps[0].a = s.as_str() #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    put(names[0], s.as_str()) #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    old = mem.replace(names[1], s.as_str()) #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    local = s.as_str()
    mem.swap(names[0], local) #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    r = ref mut names[1]
    r = s.as_str() #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    parts = [s.as_str()] #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    inline: [str; 1] = [s.as_str()]
    names.extend(inline) #$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
    store(names, s.as_str()) #$ error[E3063]: `str` is a view, so it may not be passed to `store`, which stores it where only a `static` view may go
    println(names, ps[0].a, old, parts)
