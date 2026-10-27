#include "stamps.h"

double span_length(span_t s) {
    return s.end - s.start;
}

span_t span_shift(span_t s, when_t by) {
    span_t shifted = { s.start + by, s.end + by };
    return shifted;
}

int stamp_tag(stamp_t s) {
    return s.tag + (int)s.at;
}

stamp_t stamp_make(when_t at, tag_t tag) {
    stamp_t made = { at, tag };
    return made;
}

when_t lone_value(lone_t l) {
    return l.only;
}

lone_t lone_twice(lone_t l) {
    lone_t twice = { l.only * 2.0 };
    return twice;
}
