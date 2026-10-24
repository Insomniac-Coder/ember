/* sizes.c: prints sizeof(Player) for each model of share.c, so the report's byte counts are measured. */
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
typedef struct { int32_t *p; size_t len, cap; } IVec;
typedef struct { int32_t hp; int32_t x; IVec inv; } P0;
typedef struct { uint32_t strong, weak, access, flags; const void *ti; } H1;
typedef struct { H1 h; uint32_t access_inv, access_pad; int32_t hp; int32_t x; IVec inv; } P1;
typedef struct { uint32_t strong, type_index; } H3;
typedef struct { H3 h; int32_t hp; int32_t x; IVec inv; } P3;
typedef struct { uint32_t strong, weak, type_index, flags; } H6;
typedef struct { H6 h; int32_t hp; int32_t x; IVec inv; } P6;
int main(void) { printf("plain %zu today %zu proposed %zu proposed16 %zu\n", sizeof(P0), sizeof(P1), sizeof(P3), sizeof(P6)); return 0; }
