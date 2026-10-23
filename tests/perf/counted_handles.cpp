// `[BEN-6]` — `counted_handles.em`'s twin: the same ring, with an intrusive, non-atomic count.
#include <cstdint>
#include <cstdio>
#include <cstdlib>

struct Cell {
    int64_t count;
    int64_t value;
};

static Cell* make(int64_t value) {
    Cell* cell = static_cast<Cell*>(std::malloc(sizeof(Cell)));
    cell->count = 1;
    cell->value = value;
    return cell;
}

static inline Cell* retain(Cell* cell) {
    cell->count += 1;
    return cell;
}

static inline void release(Cell* cell) {
    if (--cell->count == 0) std::free(cell);
}

int main() {
    Cell* cells[1000];
    for (int64_t i = 0; i < 1000; i++) cells[i] = make(i);
    Cell* ring[1000];
    for (int64_t i = 0; i < 1000; i++) ring[i] = retain(cells[i]);
    // The lengths as the Ember program reads them, from its arrays: not constants.
    volatile int64_t length = 1000;
    const int64_t ring_len = length, cells_len = length;
    uint64_t total = 0;
    for (int64_t round = 0; round < 20000; round++) {
        for (int64_t i = 0; i < ring_len; i++) {
            Cell* next = retain(cells[(i * 7 + round) % cells_len]);
            release(ring[i]);
            ring[i] = next;
        }
        for (int64_t i = 0; i < ring_len; i++) total += static_cast<uint64_t>(ring[i]->value);
    }
    std::printf("%lld\n", static_cast<long long>(total));
    for (int64_t i = 0; i < 1000; i++) release(ring[i]);
    for (int64_t i = 0; i < 1000; i++) release(cells[i]);
    return 0;
}
