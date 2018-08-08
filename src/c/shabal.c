#include "shabal.h"
#include <string.h>
#include <immintrin.h>

#define SET_BEST_DEADLINE(d, o) if ((d) < *best_deadline) { *best_deadline = (d); *best_offset = (o); }

// context for 4-dimensional shabal (128bit)
mshabal_context global_128;
mshabal_context_fast global_128_fast;

// context for 8-dimensional shabal (256bit)
mshabal256_context global_256;
mshabal256_context_fast global_256_fast;

void init_shabal_avx2() {
  simd256_mshabal_init(&global_256, 256);
  global_256_fast.out_size = global_256.out_size;
  for (int i = 0; i < 352; i++)
    global_256_fast.state[i] = global_256.state[i];
  global_256_fast.Whigh = global_256.Whigh;
  global_256_fast.Wlow = global_256.Wlow;
}

void init_shabal_avx() {
  simd128_mshabal_init(&global_128, 256);
  global_128_fast.out_size = global_128.out_size;
  for (int i = 0;i<176;i++)
    global_128_fast.state[i] = global_128.state[i];
  global_128_fast.Whigh = global_128.Whigh;
  global_128_fast.Wlow = global_128.Wlow;
}

void init_shabal_sse2() {
  simd128_mshabal_init(&global_128, 256);
  global_128_fast.out_size = global_128.out_size;
  for (int i = 0; i < 176; i++)
    global_128_fast.state[i] = global_128.state[i];
  global_128_fast.Whigh = global_128.Whigh;
  global_128_fast.Wlow = global_128.Wlow;
}

/* //ALL CPUs */
/* void procscoop_sph(const unsigned long long nonce, const unsigned long long n, char const *const data, const size_t acc, const std::string &file_name) { */
/*   char const *cache; */
/*   char sig[32 + 64]; */
/*   cache = data; */
/*   char res[32]; */
/*   memcpy_s(sig, sizeof(sig), signature, sizeof(char) * 32); */

/*   sph_shabal_context x; */
/*   for (unsigned long long v = 0; v < n; v++) { */
/*     memcpy_s(&sig[32], sizeof(sig) - 32, &cache[v * 64], sizeof(char) * 64); */

/*     memcpy(&x, &global_32, sizeof(global_32)); // optimization: sph_shabal256_init(&x); */
/*     sph_shabal256(&x, (const unsigned char*)sig, 64 + 32); */
/*     sph_shabal256_close(&x, res); */

/*     unsigned long long *wertung = (unsigned long long*)res; */

/*     if ((*wertung / baseTarget) <= bests[acc].targetDeadline) { */
/*       if (*wertung < bests[acc].best) { */
/*         Log("\nfound deadline=");	Log_llu(*wertung / baseTarget); Log(" nonce=");	Log_llu(nonce + v); Log(" for account: "); Log_llu(bests[acc].account_id); Log(" file: "); Log((char*)file_name.c_str()); */
/*         EnterCriticalSection(&bestsLock); */
/*         bests[acc].best = *wertung; */
/*         bests[acc].nonce = nonce + v; */
/*         bests[acc].DL = *wertung / baseTarget; */
/*         LeaveCriticalSection(&bestsLock); */
/*         EnterCriticalSection(&sharesLock); */
/*         shares.push_back({ file_name, bests[acc].account_id, bests[acc].best, bests[acc].nonce }); */
/*         LeaveCriticalSection(&sharesLock); */
/*       } */
/*     } */
/*   } */
/* } */

void find_best_deadline_sse2(char *scoops, uint64_t nonce_count, char *gensig, uint64_t *best_deadline, uint64_t *best_offset) {
  uint64_t d0, d1, d2, d3;
  char res0[32], res1[32], res2[32], res3[32];
  char end[32];

  end[0] = -128;
  memset(&end[1], 0, 31);

  mshabal_context_fast x1, x2;
  memcpy(&x2, &global_128_fast, sizeof(global_128_fast)); // local copy of global fast context

  //prepare shabal inputs
  union {
    mshabal_u32 words[64];
    __m128i data[16];
  } u1, u2;

  for (int i = 0; i < 64 / 2; i += 4 ) {
    size_t o = i;
    u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
    u2.words[i + 0 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 1 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 2 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 3 + 32] = *(mshabal_u32 *)(end + o);
  }

  for (uint64_t i = 0; i < nonce_count; i += 4) {
    // initialise shabal
    memcpy(&x1, &x2, sizeof(x2)); // optimization: mshabal256_init(&x, 256);

    // load and shuffle data
    // NB: this can be further optimised by preshuffling plot files depending on SIMD length and use avx2 memcpy
    // did not find a away yet to completely avoid memcpys

    for (uint64_t j = 0; j < 64 / 2; j += 4) {
      size_t o = j;
      u1.words[j + 0 + 32] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
      u1.words[j + 1 + 32] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
      u1.words[j + 2 + 32] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
      u1.words[j + 3 + 32] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
      u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
      u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
      u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
      u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
    }

    simd128_mshabal_openclose_fast(&x1, &u1, &u2, res0, res1, res2, res3);

    d0 = *((uint64_t *) res0);
    d1 = *((uint64_t *) res1);
    d2 = *((uint64_t *) res2);
    d3 = *((uint64_t *) res3);

    SET_BEST_DEADLINE(d0, i + 0);
    SET_BEST_DEADLINE(d1, i + 1);
    SET_BEST_DEADLINE(d2, i + 2);
    SET_BEST_DEADLINE(d3, i + 3);
  }
}

void find_best_deadline_avx(char *scoops, uint64_t nonce_count, char *gensig, uint64_t *best_deadline, uint64_t *best_offset) {
  uint64_t d0, d1, d2, d3;
  char res0[32], res1[32], res2[32], res3[32];
  char end[32];

  end[0] = -128;
  memset(&end[1], 0, 31);

  mshabal_context_fast x1, x2;
  memcpy(&x2, &global_128_fast, sizeof(global_128_fast)); // local copy of global fast context

  //prepare shabal inputs
  union {
    mshabal_u32 words[64];
    __m128i data[16];
  } u1, u2;

  for (int i = 0; i < 64 / 2; i += 4) {
    size_t o = i;
    u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
    u2.words[i + 0 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 1 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 2 + 32] = *(mshabal_u32 *)(end + o);
    u2.words[i + 3 + 32] = *(mshabal_u32 *)(end + o);
  }

  for (uint64_t i = 0; i < nonce_count; i += 4) {
    // initialise shabal
    memcpy(&x1, &x2, sizeof(x2)); // optimization: mshabal256_init(&x, 256);

    // load and shuffle data
    // NB: this can be further optimised by preshuffling plot files depending on SIMD length and use avx2 memcpy
    // did not find a away yet to completely avoid memcpys

    for (uint64_t j = 0; j < 64 / 2; j += 4) {
      size_t o = j;
      u1.words[j + 0 + 32] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
      u1.words[j + 1 + 32] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
      u1.words[j + 2 + 32] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
      u1.words[j + 3 + 32] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
      u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
      u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
      u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
      u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
    }

    simd128_mshabal_openclose_fast(&x1, &u1, &u2, res0, res1, res2, res3);

    d0 = *((uint64_t *) res0);
    d1 = *((uint64_t *) res1);
    d2 = *((uint64_t *) res2);
    d3 = *((uint64_t *) res3);

    SET_BEST_DEADLINE(d0, i + 0);
    SET_BEST_DEADLINE(d1, i + 1);
    SET_BEST_DEADLINE(d2, i + 2);
    SET_BEST_DEADLINE(d3, i + 3);
  }
}

void find_best_deadline_avx2(char *scoops, uint64_t nonce_count, char *gensig, uint64_t *best_deadline, uint64_t *best_offset) {
  uint64_t d0, d1, d2, d3, d4, d5, d6, d7;
  char res0[32], res1[32], res2[32], res3[32], res4[32], res5[32], res6[32], res7[32];
  char end[32];

  end[0] = -128;
  memset(&end[1], 0, 31);

  mshabal256_context_fast x1, x2;
  memcpy(&x2, &global_256_fast, sizeof(global_256_fast)); // local copy of global fast contex

  //prepare shabal inputs
  union {
    mshabal_u32 words[64 * MSHABAL256_FACTOR];
    __m256i data[16];
  } u1, u2;

  for (uint64_t i = 0; i < 64 * MSHABAL256_FACTOR / 2; i += 4 * MSHABAL256_FACTOR) {
    size_t o = i / MSHABAL256_FACTOR;
    u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 4] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 5] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 6] = *(mshabal_u32 *)(gensig + o);
    u1.words[i + 7] = *(mshabal_u32 *)(gensig + o);
    u2.words[i + 0 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 1 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 2 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 3 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 4 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 5 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 6 + 64] = *(mshabal_u32 *)(end + o);
    u2.words[i + 7 + 64] = *(mshabal_u32 *)(end + o);
  }

  for (uint64_t i = 0; i < nonce_count; i += 8) {
    // initialise shabal
    memcpy(&x1, &x2, sizeof(x2)); // optimization: mshabal256_init(&x, 256);

    //Load and shuffle data 
    //NB: this can be further optimised by preshuffling plot files depending on SIMD length and use avx2 memcpy
    //Did not find a away yet to completely avoid memcpys

    for (uint64_t j = 0; j < 64 * MSHABAL256_FACTOR / 2; j += 4 * MSHABAL256_FACTOR) {
      size_t o = j / MSHABAL256_FACTOR;
      u1.words[j + 0 + 64] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
      u1.words[j + 1 + 64] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
      u1.words[j + 2 + 64] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
      u1.words[j + 3 + 64] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
      u1.words[j + 4 + 64] = *(mshabal_u32 *)(&scoops[(i + 4) * 64] + o);
      u1.words[j + 5 + 64] = *(mshabal_u32 *)(&scoops[(i + 5) * 64] + o);
      u1.words[j + 6 + 64] = *(mshabal_u32 *)(&scoops[(i + 6) * 64] + o);
      u1.words[j + 7 + 64] = *(mshabal_u32 *)(&scoops[(i + 7) * 64] + o);
      u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
      u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
      u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
      u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
      u2.words[j + 4] = *(mshabal_u32 *)(&scoops[(i + 4) * 64 + 32] + o);
      u2.words[j + 5] = *(mshabal_u32 *)(&scoops[(i + 5) * 64 + 32] + o);
      u2.words[j + 6] = *(mshabal_u32 *)(&scoops[(i + 6) * 64 + 32] + o);
      u2.words[j + 7] = *(mshabal_u32 *)(&scoops[(i + 7) * 64 + 32] + o);
    }

    simd256_mshabal_openclose_fast(&x1, &u1, &u2,
                                   res0, res1, res2, res3, 
                                   res4, res5, res6, res7);

    d0 = *((uint64_t *) res0);
    d1 = *((uint64_t *) res1);
    d2 = *((uint64_t *) res2);
    d3 = *((uint64_t *) res3);
    d4 = *((uint64_t *) res4);
    d5 = *((uint64_t *) res5);
    d6 = *((uint64_t *) res6);
    d7 = *((uint64_t *) res7);

    SET_BEST_DEADLINE(d0, i + 0);
    SET_BEST_DEADLINE(d1, i + 1);
    SET_BEST_DEADLINE(d2, i + 2);
    SET_BEST_DEADLINE(d3, i + 3);
    SET_BEST_DEADLINE(d4, i + 4);
    SET_BEST_DEADLINE(d5, i + 5);
    SET_BEST_DEADLINE(d6, i + 6);
    SET_BEST_DEADLINE(d7, i + 7);
  }
}
