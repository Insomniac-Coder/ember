#$ test: compile-fail
#$ rules: TYP-34
#$ help: remove `@view`
## design rules: R7
## R7: `from static` on a struct's field does not make the struct a view, so a struct whose only view
## field says `from static` holds no view, and `@view` on it is refused (E2030, "`@view` on a type that is
## not a view"). The allowed neighbour is `accept_a_struct_whose_view_field_is_from_static_is_not_a_view.em`.

@view    #$ error[E2030]: `@view` on `Tag`, which holds no view (its field `label` says `from static`, text written in the program); remove `@view`
struct Tag:
    label: StrView from static

fn main():
    t = Tag(label="positive")
    println(t.label)
