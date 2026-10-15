/* The C twin of bench/probes/soa_retain.em: three columns, the elements kept compacted in place
 * column by column, then the kept lives summed. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
typedef struct { float x, y, z; } vec3;
int main(void) {
    size_t n = 1000000;
    vec3* pos = malloc(n * sizeof *pos);
    vec3* vel = malloc(n * sizeof *vel);
    float* life = malloc(n * sizeof *life);
    if (!pos || !vel || !life) abort();
    for (size_t i = 0; i < n; ++i) {
        pos[i] = (vec3){(float)i, 0.0f, 0.0f};
        vel[i] = (vec3){0.0f, 0.0f, 0.0f};
        life[i] = (float)((i * 7) % 10);
    }
    size_t kept = 0;
    for (size_t i = 0; i < n; ++i) {
        if (life[i] >= 5.0f) {
            pos[kept] = pos[i];
            vel[kept] = vel[i];
            life[kept] = life[i];
            kept += 1;
        }
    }
    double total = 0.0;
    for (size_t i = 0; i < kept; ++i) total += (double)life[i];
    printf("%zu %lld\n", kept, (long long)(total * 10.0));
    free(pos);
    free(vel);
    free(life);
    return 0;
}
