#include <stdint.h>

typedef struct { int32_t left; int32_t right; } ForeignPair;

extern int32_t exported_pair_score(ForeignPair pair);
extern ForeignPair exported_pair_mix(ForeignPair left, int32_t scale, ForeignPair right);
extern int32_t api_pair_score(ForeignPair pair);
int32_t call_pair_callback(int32_t (*callback)(ForeignPair)) {
    ForeignPair pair = {6, 7};
    return callback(pair);
}

int32_t call_exported_pair_score(void) {
    ForeignPair pair = {8, 5};
    return exported_pair_score(pair);
}

int32_t call_exported_pair_mix(void) {
    ForeignPair left = {2, 3};
    ForeignPair right = {4, 5};
    ForeignPair result = exported_pair_mix(left, 2, right);
    return result.left * 100 + result.right;
}

int32_t call_named_pair_export(void) {
    ForeignPair pair = {1, 2};
    return api_pair_score(pair);
}
