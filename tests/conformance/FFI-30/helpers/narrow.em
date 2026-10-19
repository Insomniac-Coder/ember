# A module of reject_one_declaration_with_two_layouts.em: common.h as written.
import c "../common.h"

pub fn length2(x: f32, y: f32) -> f32:
    v = c.Vec2(x=x, y=y)
    return v.x * v.x + v.y * v.y
