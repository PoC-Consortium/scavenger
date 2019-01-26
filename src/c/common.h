#include <stdint.h>

#pragma once

#ifdef _MSC_VER

#include <stdlib.h>
#define bswap_32(x) _byteswap_ulong(x)
#define bswap_64(x) _byteswap_uint64(x)

#elif defined(__APPLE__)

// Mac OS X / Darwin features
#include <libkern/OSByteOrder.h>
#define bswap_32(x) OSSwapInt32(x)
#define bswap_64(x) OSSwapInt64(x)

#elif defined(__sun) || defined(sun)

#include <sys/byteorder.h>
#define bswap_32(x) BSWAP_32(x)
#define bswap_64(x) BSWAP_64(x)

#elif defined(__FreeBSD__)

#include <sys/endian.h>
#define bswap_32(x) bswap32(x)
#define bswap_64(x) bswap64(x)

#elif defined(__OpenBSD__)

#include <sys/types.h>
#define bswap_32(x) swap32(x)
#define bswap_64(x) swap64(x)

#elif defined(__NetBSD__)

#include <machine/bswap.h>
#include <sys/types.h>
#if defined(__BSWAP_RENAME) && !defined(__bswap_32)
#define bswap_32(x) bswap32(x)
#define bswap_64(x) bswap64(x)
#endif

#else

#include <byteswap.h>

#endif

#define HASH_SIZE 32
#define HASH_CAP 4096
#define NUM_SCOOPS 4096
#define SCOOP_SIZE 64
#define NONCE_SIZE (HASH_CAP * SCOOP_SIZE)  // 4096*64

void write_seed(char seed[32], uint64_t numeric_id);

void write_term(char term[32]);

#define SET_BEST_DEADLINE(d, o) \
    if ((d) < *best_deadline) { \
        *best_deadline = (d);   \
        *best_offset = (o);     \
    }
