#pragma once

#include <stdint.h>
#include <stdlib.h>
#include "mshabal.h"

extern mshabal_context global_128;
extern mshabal256_context global_256;

extern mshabal_context_fast global_128_fast;
extern mshabal256_context_fast global_256_fast;

void init_shabal_sse2();

void find_best_deadline_sse2(char* scoops, uint64_t nonce_count, char* gensig,
                             uint64_t* best_deadline, uint64_t* best_offset);
