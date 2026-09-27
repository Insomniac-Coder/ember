#include <stdint.h>

int32_t raw_counter = 41;
int32_t changing_counter = 7;
int32_t read_changing_counter(void) { return changing_counter; }
int32_t bump_changing_counter(void) { return ++changing_counter; }
const int32_t frozen_count = 1;
const int32_t aliased_count = 2;

typedef struct { int32_t left; int32_t right; } ForeignPair;
ForeignPair shared_pair = {3, 4};
const ForeignPair frozen_pair = {5, 6};
int32_t read_pair_sum(void) { return shared_pair.left + shared_pair.right; }
int32_t bump_pair_left(void) { return ++shared_pair.left; }
ForeignPair add_pairs(ForeignPair a, ForeignPair b) {
    ForeignPair result = {a.left + b.left, a.right + b.right};
    return result;
}
int32_t mutate_pair_copy(ForeignPair pair) {
    pair.left += 10;
    return pair.left + pair.right;
}
ForeignPair swap_pair(ForeignPair pair) {
    ForeignPair result = {pair.right, pair.left};
    return result;
}
int32_t foreign_pair_weighted(ForeignPair pair, int32_t weight) {
    return pair.left * weight + pair.right;
}
int32_t pair_and_values(ForeignPair pair, const int32_t* data, uint8_t n) {
    int32_t total = pair.left + pair.right;
    for (uint8_t i = 0; i < n; ++i) total += data[i];
    return total;
}
typedef int32_t (*PairCallback)(ForeignPair);
int32_t score_pair_c(ForeignPair pair) { return pair.left * 4 + pair.right; }
PairCallback get_pair_callback(void) { return score_pair_c; }

typedef struct { int64_t a; int64_t b; int64_t c; } ForeignWide;
ForeignWide transform_wide(ForeignWide value) {
    ForeignWide result = {value.c + 1, value.b + 2, value.a + 3};
    return result;
}

typedef struct { int32_t value; } ForeignInner;
typedef struct { ForeignInner inner; int32_t other; } ForeignOuter;
ForeignOuter nested_pair = {{4}, 6};
const ForeignOuter frozen_nested = {{2}, 3};
int32_t read_nested_total(void) { return nested_pair.inner.value + nested_pair.other; }
int32_t bump_nested_other(void) { return ++nested_pair.other; }
