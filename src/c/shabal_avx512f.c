#include "shabal_avx512f.h"
#include <immintrin.h>
#include <string.h>
#include "common.h"
#include "mshabal_512_avx512f.h"
#include "sph_shabal.h"

mshabal512_context global_512;
mshabal512_context_fast global_512_fast;

void init_shabal_avx512f() {
    mshabal_init_avx512f(&global_512, 256);
    global_512_fast.out_size = global_512.out_size;
    for (uint64_t i = 0; i < 704; i++) global_512_fast.state[i] = global_512.state[i];
    global_512_fast.Whigh = global_512.Whigh;
    global_512_fast.Wlow = global_512.Wlow;
}

void find_best_deadline_avx512f(char *scoops, uint64_t nonce_count, char *gensig,
                                uint64_t *best_deadline, uint64_t *best_offset) {
    uint64_t d0 = 0, d1 = 0, d2 = 0, d3 = 0, d4 = 0, d5 = 0, d6 = 0, d7 = 0, d8 = 0, d9 = 0,
             d10 = 0, d11 = 0, d12 = 0, d13 = 0, d14 = 0, d15 = 0;
    char term[32];
    write_term(term);

    // local copy of global fast context
    mshabal512_context_fast x;
    memcpy(&x, &global_512_fast, sizeof(global_512_fast));

    // prepare shabal inputs
    union {
        mshabal_u32 words[16 * MSHABAL512_VECTOR_SIZE];
        __m512i data[16];
    } u1, u2;

    for (uint64_t i = 0; i < 16 * MSHABAL512_VECTOR_SIZE / 2; i += MSHABAL512_VECTOR_SIZE) {
        size_t o = i / 4;
        u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 4] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 5] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 6] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 7] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 8] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 9] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 10] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 11] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 12] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 13] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 14] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 15] = *(mshabal_u32 *)(gensig + o);
        u2.words[i + 0 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 1 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 2 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 3 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 4 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 5 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 6 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 7 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 8 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 9 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 10 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 11 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 12 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 13 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 14 + 128] = *(mshabal_u32 *)(term + o);
        u2.words[i + 15 + 128] = *(mshabal_u32 *)(term + o);
    }

    for (uint64_t i = 0; i < nonce_count;) {
        if (i + 16 <= nonce_count) {
            // load and align data for SIMD

            for (uint64_t j = 0; j < 16 * MSHABAL512_VECTOR_SIZE / 2; j += MSHABAL512_VECTOR_SIZE) {
                size_t o = j / 4;
                u1.words[j + 0 + 128] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
                u1.words[j + 1 + 128] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
                u1.words[j + 2 + 128] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
                u1.words[j + 3 + 128] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
                u1.words[j + 4 + 128] = *(mshabal_u32 *)(&scoops[(i + 4) * 64] + o);
                u1.words[j + 5 + 128] = *(mshabal_u32 *)(&scoops[(i + 5) * 64] + o);
                u1.words[j + 6 + 128] = *(mshabal_u32 *)(&scoops[(i + 6) * 64] + o);
                u1.words[j + 7 + 128] = *(mshabal_u32 *)(&scoops[(i + 7) * 64] + o);
                u1.words[j + 8 + 128] = *(mshabal_u32 *)(&scoops[(i + 8) * 64] + o);
                u1.words[j + 9 + 128] = *(mshabal_u32 *)(&scoops[(i + 9) * 64] + o);
                u1.words[j + 10 + 128] = *(mshabal_u32 *)(&scoops[(i + 10) * 64] + o);
                u1.words[j + 11 + 128] = *(mshabal_u32 *)(&scoops[(i + 11) * 64] + o);
                u1.words[j + 12 + 128] = *(mshabal_u32 *)(&scoops[(i + 12) * 64] + o);
                u1.words[j + 13 + 128] = *(mshabal_u32 *)(&scoops[(i + 13) * 64] + o);
                u1.words[j + 14 + 128] = *(mshabal_u32 *)(&scoops[(i + 14) * 64] + o);
                u1.words[j + 15 + 128] = *(mshabal_u32 *)(&scoops[(i + 15) * 64] + o);
                u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
                u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
                u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
                u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
                u2.words[j + 4] = *(mshabal_u32 *)(&scoops[(i + 4) * 64 + 32] + o);
                u2.words[j + 5] = *(mshabal_u32 *)(&scoops[(i + 5) * 64 + 32] + o);
                u2.words[j + 6] = *(mshabal_u32 *)(&scoops[(i + 6) * 64 + 32] + o);
                u2.words[j + 7] = *(mshabal_u32 *)(&scoops[(i + 7) * 64 + 32] + o);
                u2.words[j + 8] = *(mshabal_u32 *)(&scoops[(i + 8) * 64 + 32] + o);
                u2.words[j + 9] = *(mshabal_u32 *)(&scoops[(i + 9) * 64 + 32] + o);
                u2.words[j + 10] = *(mshabal_u32 *)(&scoops[(i + 10) * 64 + 32] + o);
                u2.words[j + 11] = *(mshabal_u32 *)(&scoops[(i + 11) * 64 + 32] + o);
                u2.words[j + 12] = *(mshabal_u32 *)(&scoops[(i + 12) * 64 + 32] + o);
                u2.words[j + 13] = *(mshabal_u32 *)(&scoops[(i + 13) * 64 + 32] + o);
                u2.words[j + 14] = *(mshabal_u32 *)(&scoops[(i + 14) * 64 + 32] + o);
                u2.words[j + 15] = *(mshabal_u32 *)(&scoops[(i + 15) * 64 + 32] + o);
            }

            mshabal_deadline_fast_avx512f(&x, &u1, &u2, &d0, &d1, &d2, &d3, &d4, &d5, &d6, &d7,
                                           &d8, &d9, &d10, &d11, &d12, &d13, &d14, &d15);

            SET_BEST_DEADLINE(d0, i + 0);
            SET_BEST_DEADLINE(d1, i + 1);
            SET_BEST_DEADLINE(d2, i + 2);
            SET_BEST_DEADLINE(d3, i + 3);
            SET_BEST_DEADLINE(d4, i + 4);
            SET_BEST_DEADLINE(d5, i + 5);
            SET_BEST_DEADLINE(d6, i + 6);
            SET_BEST_DEADLINE(d7, i + 7);
            SET_BEST_DEADLINE(d8, i + 8);
            SET_BEST_DEADLINE(d9, i + 9);
            SET_BEST_DEADLINE(d10, i + 10);
            SET_BEST_DEADLINE(d11, i + 11);
            SET_BEST_DEADLINE(d12, i + 12);
            SET_BEST_DEADLINE(d13, i + 13);
            SET_BEST_DEADLINE(d14, i + 14);
            SET_BEST_DEADLINE(d15, i + 15);
            i += 16;
        } else {
            sph_shabal_deadline_fast(&scoops[i * 64], gensig, &d0);
            SET_BEST_DEADLINE(d0, i);
            i++;
        }
    }
}
