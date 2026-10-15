/* The C twin of bench/probes/soa_integrate.em: three columns (position, velocity, life) in one
 * 64-byte-aligned block, the position column moved by the velocity column 200 times. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#if defined(_WIN32)
#include <malloc.h>
#define BLOCK_ALLOC(size) _aligned_malloc((size), 64)
#define BLOCK_FREE(p) _aligned_free(p)
#else
#define BLOCK_ALLOC(size) aligned_alloc(64, (size))
#define BLOCK_FREE(p) free(p)
#endif
typedef struct { float x, y, z; } vec3;
static void integrate(vec3* pos, const vec3* vel, size_t n, float dt) {
    for (size_t i = 0; i < n; ++i) {
        pos[i].x += vel[i].x * dt;
        pos[i].y += vel[i].y * dt;
        pos[i].z += vel[i].z * dt;
    }
}
int main(void) {
    size_t n = 100000;
    size_t columns = ((n * sizeof(vec3) + 63) / 64 * 64);
    unsigned char* block = BLOCK_ALLOC(columns * 2 + (n * sizeof(float) + 63) / 64 * 64);
    if (!block) abort();
    vec3* pos = (vec3*)block;
    vec3* vel = (vec3*)(block + columns);
    float* life = (float*)(block + columns * 2);
    for (size_t i = 0; i < n; ++i) {
        pos[i] = (vec3){0.0f, 0.0f, 0.0f};
        vel[i] = (vec3){(float)(i % 100), 1.0f, 0.5f};
        life[i] = 1.0f;
    }
    for (int step = 0; step < 200; ++step) integrate(pos, vel, n, 0.01f);
    double total = 0.0;
    for (size_t i = 0; i < n; ++i) total += (double)pos[i].x;
    printf("%lld\n", (long long)(total * 100.0));
    BLOCK_FREE(block);
    return 0;
}
