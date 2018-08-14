/* $Id: sph_shabal.h 175 2010-05-07 16:03:20Z tp $ */
/**
 * Shabal interface. Shabal is a family of functions which differ by
 * their output size; this implementation defines Shabal for output
 * sizes 192, 224, 256, 384 and 512 bits.
 *
 * ==========================(LICENSE BEGIN)============================
 *
 * Copyright (c) 2007-2010  Projet RNRT SAPHIR
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 * ===========================(LICENSE END)=============================
 *
 * @file     sph_shabal.h
 * @author   Thomas Pornin <thomas.pornin@cryptolog.com>
 */

#ifndef SPH_SHABAL_H__
#define SPH_SHABAL_H__

#include <stddef.h>
#include "sph_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Output size (in bits) for Shabal-256.
 */
#define SPH_SIZE_shabal256 256

/**
 * This structure is a context for Shabal computations: it contains the
 * intermediate values and some data from the last entered block. Once
 * a Shabal computation has been performed, the context can be reused for
 * another computation.
 *
 * The contents of this structure are private. A running Shabal computation
 * can be cloned by copying the context (e.g. with a simple
 * <code>memcpy()</code>).
 */
typedef struct {
#ifndef DOXYGEN_IGNORE
    unsigned char buf[64]; /* first field, for alignment */
    size_t ptr;
    sph_u32 A[12], B[16], C[16];
    sph_u32 Whigh, Wlow;
#endif
} sph_shabal_context;

/**
 * Type for a Shabal-256 context (identical to the common context).
 */
typedef sph_shabal_context sph_shabal256_context;

/**
 * Initialize a Shabal-256 context. This process performs no memory
 * allocation.
 *
 * @param cc   the Shabal-256 context (pointer to a
 *             <code>sph_shabal256_context</code>)
 */
void sph_shabal256_init(sph_shabal_context* cc);

/**
 * Process some data bytes. It is acceptable that <code>len</code> is zero
 * (in which case this function does nothing).
 *
 * @param cc     the Shabal-256 context
 * @param data   the input data
 * @param len    the input data length (in bytes)
 */
void sph_shabal256(void* cc, const unsigned char* data, size_t len);

/**
 * Terminate the current Shabal-256 computation and output the result into
 * the provided buffer. The destination buffer must be wide enough to
 * accomodate the result (32 bytes). The context is automatically
 * reinitialized.
 *
 * @param cc    the Shabal-256 context
 * @param dst   the destination buffer
 */
void sph_shabal256_close(void* cc, void* dst);

/**
 * Add a few additional bits (0 to 7) to the current computation, then
 * terminate it and output the result in the provided buffer, which must
 * be wide enough to accomodate the result (32 bytes). If bit number i
 * in <code>ub</code> has value 2^i, then the extra bits are those
 * numbered 7 downto 8-n (this is the big-endian convention at the byte
 * level). The context is automatically reinitialized.
 *
 * @param cc    the Shabal-256 context
 * @param ub    the extra bits
 * @param n     the number of extra bits (0 to 7)
 * @param dst   the destination buffer
 */
void sph_shabal256_addbits_and_close(void* cc, unsigned ub, unsigned n, void* dst);

#ifdef __cplusplus
}
#endif
#endif
