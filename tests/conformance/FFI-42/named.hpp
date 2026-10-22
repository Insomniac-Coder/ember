// The C++ side of ./accept_a_headers_own_name_outranks_an_iterators.em: `Row` is iterable, and the
// header declares a `Row_iter` of its own after it.
#pragma once

struct Row {
    int cells[3];
    int* begin() noexcept { return cells; }
    int* end() noexcept { return cells + 3; }
};

struct Row_iter {
    int at;
};
