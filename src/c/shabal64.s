# Implementation of Shabal: amd64
#
# This code should run on any x86-compatible system running in 64-bit
# mode. It uses SSE2 opcodes (which are part of the amd64 architecture,
# aka '64-bit x86').
#
# This assembly file uses the GNU binutils syntax and follows the
# ELF call conventions. It was tested with GNU as 2.20.1 on a Linux
# system (Intel Core2 Q6600).
#
# -----------------------------------------------------------------------
# (c) 2010 SAPHIR project. This software is provided 'as-is', without
# any epxress or implied warranty. In no event will the authors be held
# liable for any damages arising from the use of this software.
#
# Permission is granted to anyone to use this software for any purpose,
# including commercial applications, and to alter it and redistribute it
# freely, subject to no restriction.
#
# Technical remarks and questions can be addressed to:
# <thomas.pornin@cryptolog.com>
# -----------------------------------------------------------------------

	.file   "shabal_amd64.s"
	.text

# Context structure layout:
#  offset    contents
#     0      buffer for final block (64 bytes)
#    64      'ptr' field (8 bytes)
#    72      A[] (48 bytes)
#   120      B[] (64 bytes)
#   184      C[] (64 bytes)
#   248      W (8 bytes)
#   256      out_size (4 bytes)
# Total size is 264 bytes, with a 8-byte alignment.
#
# The shabal_inner() function expects a pointer to A[0] and accesses
# the A[], B[], C[] and W fields. The pointer to A[0] MUST be 16-byte
# aligned. For optimal performance, it is best if B[0] is 64-byte
# aligned, i.e. address of A[0] is equal to 16 modulo 64 (this ensures
# that A[], B[] and C[] are each on a single L1 cache line, and those
# three cache lines are distinct from each other).
# From the shabal_inner() point of view:
#     0      A[] (48 bytes)
#    48      B[] (64 bytes)
#   112      C[] (64 bytes)
#   176      W (8 bytes)
# No specific alignment for the input data is required, but a 16-byte
# alignment is better.

# shabal_inner(stt, buf, num)
#    stt   pointer to context state (address of A[0], assumed 16-byte aligned)
#    buf   pointer to data (no specific alignment required)
#    num   number of 64-byte blocks in data (1 or more)
#
	.type   shabal_inner, @function
shabal_inner:
	pushq   %rbp

	# Conventions:
	#    %rbp   pointer to state words
	#    %rsi   pointer to current input data block (already set)
	#    %rdi   remaining data length (in blocks)
	movq    %rdi, %rbp
	movq    %rdx, %rdi

	# Reserve a 256-byte stack area for Bx[] (64-byte aligned).
	movq    %rsp, %rax
	subq    $8, %rsp
	andq    $0xFFFFFFFFFFFFFFC0, %rsp
	subq    $256, %rsp
	movq    %rax, 256(%rsp)        # Saved stack pointer.

	.align  8
Lm0:
	# Read the B words and M words, add, rotate, store in Bx[].
	# %rsp points to Bx[0]. The M words are stored in %xmm4:7;
	# since we try to be zero-copy, they may be unaligned, hence
	# the movdqu. We keep those xmm register values for quite some
	# time.

	movdqu  0(%rsi), %xmm4
	movdqu  16(%rsi), %xmm5
	movdqu  32(%rsi), %xmm6
	movdqu  48(%rsi), %xmm7

	movdqa  48(%rbp), %xmm0
	movdqa  64(%rbp), %xmm1
	paddd   %xmm4, %xmm0
	paddd   %xmm5, %xmm1
	movdqa  %xmm0, %xmm2
	movdqa  %xmm1, %xmm3
	psrld   $15, %xmm0
	psrld   $15, %xmm1
	pslld   $17, %xmm2
	pslld   $17, %xmm3
	por     %xmm2, %xmm0
	por     %xmm3, %xmm1
	movdqa  %xmm0, 0(%rsp)
	movdqa  %xmm1, 16(%rsp)

	movdqa  80(%rbp), %xmm0
	movdqa  96(%rbp), %xmm1
	paddd   %xmm6, %xmm0
	paddd   %xmm7, %xmm1
	movdqa  %xmm0, %xmm2
	movdqa  %xmm1, %xmm3
	psrld   $15, %xmm0
	psrld   $15, %xmm1
	pslld   $17, %xmm2
	pslld   $17, %xmm3
	por     %xmm2, %xmm0
	por     %xmm3, %xmm1
	movdqa  %xmm0, 32(%rsp)
	movdqa  %xmm1, 48(%rsp)

	# Xor W into A[0]/A[1]
	movq    176(%rbp), %rax
	xorq    %rax, 0(%rbp)

	# Conventions for the 48 rounds:
	#    %eax      previous A word, accumulator
	#    %rsp      points to Bx[0] (already set)
	#    %rbp      points to state words (already set)
	#    %xmm4:7   data block (already set)

	movl    44(%rbp), %eax

	# AUTO BEGIN

	# u = 0.
	roll    $15, %eax
	movl    24(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm4, %ecx
	xorl    0(%rbp), %eax
	notl    %edx
	xorl    52(%rsp), %ecx
	andl    36(%rsp), %edx
	xorl    144(%rbp), %eax
	xorl    %ecx, %edx
	movl    0(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 0(%rbp)
	xorl    %eax, %ecx

	# u = 1.
	roll    $15, %eax
	movl    %ecx, 64(%rsp) # delayed
	pshufd  $1, %xmm4, %xmm0
	movl    28(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    4(%rbp), %eax
	notl    %edx
	xorl    56(%rsp), %ecx
	andl    40(%rsp), %edx
	xorl    140(%rbp), %eax
	xorl    %ecx, %edx
	movl    4(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 4(%rbp)
	xorl    %eax, %ecx

	# u = 2.
	roll    $15, %eax
	movl    %ecx, 68(%rsp) # delayed
	pshufd  $2, %xmm4, %xmm0
	movl    32(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    8(%rbp), %eax
	notl    %edx
	xorl    60(%rsp), %ecx
	andl    44(%rsp), %edx
	xorl    136(%rbp), %eax
	xorl    %ecx, %edx
	movl    8(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 8(%rbp)
	xorl    %eax, %ecx

	# u = 3.
	roll    $15, %eax
	movl    %ecx, 72(%rsp) # delayed
	pshufd  $3, %xmm4, %xmm0
	movl    36(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    12(%rbp), %eax
	notl    %edx
	xorl    64(%rsp), %ecx
	andl    48(%rsp), %edx
	xorl    132(%rbp), %eax
	xorl    %ecx, %edx
	movl    12(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 12(%rbp)
	xorl    %eax, %ecx

	# u = 4.
	roll    $15, %eax
	movl    %ecx, 76(%rsp) # delayed
	movl    40(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm5, %ecx
	xorl    16(%rbp), %eax
	notl    %edx
	xorl    68(%rsp), %ecx
	andl    52(%rsp), %edx
	xorl    128(%rbp), %eax
	xorl    %ecx, %edx
	movl    16(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 16(%rbp)
	xorl    %eax, %ecx

	# u = 5.
	roll    $15, %eax
	movl    %ecx, 80(%rsp) # delayed
	pshufd  $1, %xmm5, %xmm0
	movl    44(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    20(%rbp), %eax
	notl    %edx
	xorl    72(%rsp), %ecx
	andl    56(%rsp), %edx
	xorl    124(%rbp), %eax
	xorl    %ecx, %edx
	movl    20(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 20(%rbp)
	xorl    %eax, %ecx

	# u = 6.
	roll    $15, %eax
	movl    %ecx, 84(%rsp) # delayed
	pshufd  $2, %xmm5, %xmm0
	movl    48(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    24(%rbp), %eax
	notl    %edx
	xorl    76(%rsp), %ecx
	andl    60(%rsp), %edx
	xorl    120(%rbp), %eax
	xorl    %ecx, %edx
	movl    24(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 24(%rbp)
	xorl    %eax, %ecx

	# u = 7.
	roll    $15, %eax
	movl    %ecx, 88(%rsp) # delayed
	pshufd  $3, %xmm5, %xmm0
	movl    52(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    28(%rbp), %eax
	notl    %edx
	xorl    80(%rsp), %ecx
	andl    64(%rsp), %edx
	xorl    116(%rbp), %eax
	xorl    %ecx, %edx
	movl    28(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 28(%rbp)
	xorl    %eax, %ecx

	# u = 8.
	roll    $15, %eax
	movl    %ecx, 92(%rsp) # delayed
	movl    56(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm6, %ecx
	xorl    32(%rbp), %eax
	notl    %edx
	xorl    84(%rsp), %ecx
	andl    68(%rsp), %edx
	xorl    112(%rbp), %eax
	xorl    %ecx, %edx
	movl    32(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 32(%rbp)
	xorl    %eax, %ecx

	# u = 9.
	roll    $15, %eax
	movl    %ecx, 96(%rsp) # delayed
	pshufd  $1, %xmm6, %xmm0
	movl    60(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    36(%rbp), %eax
	notl    %edx
	xorl    88(%rsp), %ecx
	andl    72(%rsp), %edx
	xorl    172(%rbp), %eax
	xorl    %ecx, %edx
	movl    36(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 36(%rbp)
	xorl    %eax, %ecx

	# u = 10.
	roll    $15, %eax
	movl    %ecx, 100(%rsp) # delayed
	pshufd  $2, %xmm6, %xmm0
	movl    64(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    40(%rbp), %eax
	notl    %edx
	xorl    92(%rsp), %ecx
	andl    76(%rsp), %edx
	xorl    168(%rbp), %eax
	xorl    %ecx, %edx
	movl    40(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 40(%rbp)
	xorl    %eax, %ecx

	# u = 11.
	roll    $15, %eax
	movl    %ecx, 104(%rsp) # delayed
	pshufd  $3, %xmm6, %xmm0
	movl    68(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    44(%rbp), %eax
	notl    %edx
	xorl    96(%rsp), %ecx
	andl    80(%rsp), %edx
	xorl    164(%rbp), %eax
	xorl    %ecx, %edx
	movl    44(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 44(%rbp)
	xorl    %eax, %ecx

	# u = 12.
	roll    $15, %eax
	movl    %ecx, 108(%rsp) # delayed
	movl    72(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm7, %ecx
	xorl    0(%rbp), %eax
	notl    %edx
	xorl    100(%rsp), %ecx
	andl    84(%rsp), %edx
	xorl    160(%rbp), %eax
	xorl    %ecx, %edx
	movl    48(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 0(%rbp)
	xorl    %eax, %ecx

	# u = 13.
	roll    $15, %eax
	movl    %ecx, 112(%rsp) # delayed
	pshufd  $1, %xmm7, %xmm0
	movl    76(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    4(%rbp), %eax
	notl    %edx
	xorl    104(%rsp), %ecx
	andl    88(%rsp), %edx
	xorl    156(%rbp), %eax
	xorl    %ecx, %edx
	movl    52(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 4(%rbp)
	xorl    %eax, %ecx

	# u = 14.
	roll    $15, %eax
	movl    %ecx, 116(%rsp) # delayed
	pshufd  $2, %xmm7, %xmm0
	movl    80(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    8(%rbp), %eax
	notl    %edx
	xorl    108(%rsp), %ecx
	andl    92(%rsp), %edx
	xorl    152(%rbp), %eax
	xorl    %ecx, %edx
	movl    56(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 8(%rbp)
	xorl    %eax, %ecx

	# u = 15.
	roll    $15, %eax
	movl    %ecx, 120(%rsp) # delayed
	pshufd  $3, %xmm7, %xmm0
	movl    84(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    12(%rbp), %eax
	notl    %edx
	xorl    112(%rsp), %ecx
	andl    96(%rsp), %edx
	xorl    148(%rbp), %eax
	xorl    %ecx, %edx
	movl    60(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 12(%rbp)
	xorl    %eax, %ecx

	# u = 16.
	roll    $15, %eax
	movl    %ecx, 124(%rsp) # delayed
	movl    88(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm4, %ecx
	xorl    16(%rbp), %eax
	notl    %edx
	xorl    116(%rsp), %ecx
	andl    100(%rsp), %edx
	xorl    144(%rbp), %eax
	xorl    %ecx, %edx
	movl    64(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 16(%rbp)
	xorl    %eax, %ecx

	# u = 17.
	roll    $15, %eax
	movl    %ecx, 128(%rsp) # delayed
	pshufd  $1, %xmm4, %xmm0
	movl    92(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    20(%rbp), %eax
	notl    %edx
	xorl    120(%rsp), %ecx
	andl    104(%rsp), %edx
	xorl    140(%rbp), %eax
	xorl    %ecx, %edx
	movl    68(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 20(%rbp)
	xorl    %eax, %ecx

	# u = 18.
	roll    $15, %eax
	movl    %ecx, 132(%rsp) # delayed
	pshufd  $2, %xmm4, %xmm0
	movl    96(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    24(%rbp), %eax
	notl    %edx
	xorl    124(%rsp), %ecx
	andl    108(%rsp), %edx
	xorl    136(%rbp), %eax
	xorl    %ecx, %edx
	movl    72(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 24(%rbp)
	xorl    %eax, %ecx

	# u = 19.
	roll    $15, %eax
	movl    %ecx, 136(%rsp) # delayed
	pshufd  $3, %xmm4, %xmm0
	movl    100(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    28(%rbp), %eax
	notl    %edx
	xorl    128(%rsp), %ecx
	andl    112(%rsp), %edx
	xorl    132(%rbp), %eax
	xorl    %ecx, %edx
	movl    76(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 28(%rbp)
	xorl    %eax, %ecx

	# u = 20.
	roll    $15, %eax
	movl    %ecx, 140(%rsp) # delayed
	movl    104(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm5, %ecx
	xorl    32(%rbp), %eax
	notl    %edx
	xorl    132(%rsp), %ecx
	andl    116(%rsp), %edx
	xorl    128(%rbp), %eax
	xorl    %ecx, %edx
	movl    80(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 32(%rbp)
	xorl    %eax, %ecx

	# u = 21.
	roll    $15, %eax
	movl    %ecx, 144(%rsp) # delayed
	pshufd  $1, %xmm5, %xmm0
	movl    108(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    36(%rbp), %eax
	notl    %edx
	xorl    136(%rsp), %ecx
	andl    120(%rsp), %edx
	xorl    124(%rbp), %eax
	xorl    %ecx, %edx
	movl    84(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 36(%rbp)
	xorl    %eax, %ecx

	# u = 22.
	roll    $15, %eax
	movl    %ecx, 148(%rsp) # delayed
	pshufd  $2, %xmm5, %xmm0
	movl    112(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    40(%rbp), %eax
	notl    %edx
	xorl    140(%rsp), %ecx
	andl    124(%rsp), %edx
	xorl    120(%rbp), %eax
	xorl    %ecx, %edx
	movl    88(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 40(%rbp)
	xorl    %eax, %ecx

	# u = 23.
	roll    $15, %eax
	movl    %ecx, 152(%rsp) # delayed
	pshufd  $3, %xmm5, %xmm0
	movl    116(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    44(%rbp), %eax
	notl    %edx
	xorl    144(%rsp), %ecx
	andl    128(%rsp), %edx
	xorl    116(%rbp), %eax
	xorl    %ecx, %edx
	movl    92(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 44(%rbp)
	xorl    %eax, %ecx

	# u = 24.
	roll    $15, %eax
	movl    %ecx, 156(%rsp) # delayed
	movl    120(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm6, %ecx
	xorl    0(%rbp), %eax
	notl    %edx
	xorl    148(%rsp), %ecx
	andl    132(%rsp), %edx
	xorl    112(%rbp), %eax
	xorl    %ecx, %edx
	movl    96(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 0(%rbp)
	xorl    %eax, %ecx

	# u = 25.
	roll    $15, %eax
	movl    %ecx, 160(%rsp) # delayed
	pshufd  $1, %xmm6, %xmm0
	movl    124(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    4(%rbp), %eax
	notl    %edx
	xorl    152(%rsp), %ecx
	andl    136(%rsp), %edx
	xorl    172(%rbp), %eax
	xorl    %ecx, %edx
	movl    100(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 4(%rbp)
	xorl    %eax, %ecx

	# u = 26.
	roll    $15, %eax
	movl    %ecx, 164(%rsp) # delayed
	pshufd  $2, %xmm6, %xmm0
	movl    128(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    8(%rbp), %eax
	notl    %edx
	xorl    156(%rsp), %ecx
	andl    140(%rsp), %edx
	xorl    168(%rbp), %eax
	xorl    %ecx, %edx
	movl    104(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 8(%rbp)
	xorl    %eax, %ecx

	# u = 27.
	roll    $15, %eax
	movl    %ecx, 168(%rsp) # delayed
	pshufd  $3, %xmm6, %xmm0
	movl    132(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    12(%rbp), %eax
	notl    %edx
	xorl    160(%rsp), %ecx
	andl    144(%rsp), %edx
	xorl    164(%rbp), %eax
	xorl    %ecx, %edx
	movl    108(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 12(%rbp)
	xorl    %eax, %ecx

	# u = 28.
	roll    $15, %eax
	movl    %ecx, 172(%rsp) # delayed
	movl    136(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm7, %ecx
	xorl    16(%rbp), %eax
	notl    %edx
	xorl    164(%rsp), %ecx
	andl    148(%rsp), %edx
	xorl    160(%rbp), %eax
	xorl    %ecx, %edx
	movl    112(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 16(%rbp)
	xorl    %eax, %ecx

	# u = 29.
	roll    $15, %eax
	movl    %ecx, 176(%rsp) # delayed
	pshufd  $1, %xmm7, %xmm0
	movl    140(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    20(%rbp), %eax
	notl    %edx
	xorl    168(%rsp), %ecx
	andl    152(%rsp), %edx
	xorl    156(%rbp), %eax
	xorl    %ecx, %edx
	movl    116(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 20(%rbp)
	xorl    %eax, %ecx

	# u = 30.
	roll    $15, %eax
	movl    %ecx, 180(%rsp) # delayed
	pshufd  $2, %xmm7, %xmm0
	movl    144(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    24(%rbp), %eax
	notl    %edx
	xorl    172(%rsp), %ecx
	andl    156(%rsp), %edx
	xorl    152(%rbp), %eax
	xorl    %ecx, %edx
	movl    120(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 24(%rbp)
	xorl    %eax, %ecx

	# u = 31.
	roll    $15, %eax
	movl    %ecx, 184(%rsp) # delayed
	pshufd  $3, %xmm7, %xmm0
	movl    148(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    28(%rbp), %eax
	notl    %edx
	xorl    176(%rsp), %ecx
	andl    160(%rsp), %edx
	xorl    148(%rbp), %eax
	xorl    %ecx, %edx
	movl    124(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 28(%rbp)
	xorl    %eax, %ecx

	# u = 32.
	roll    $15, %eax
	movl    %ecx, 188(%rsp) # delayed
	movl    152(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm4, %ecx
	xorl    32(%rbp), %eax
	notl    %edx
	xorl    180(%rsp), %ecx
	andl    164(%rsp), %edx
	xorl    144(%rbp), %eax
	xorl    %ecx, %edx
	movl    128(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 32(%rbp)
	xorl    %eax, %ecx

	# u = 33.
	roll    $15, %eax
	movl    %ecx, 192(%rsp) # delayed
	pshufd  $1, %xmm4, %xmm0
	movl    156(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    36(%rbp), %eax
	notl    %edx
	xorl    184(%rsp), %ecx
	andl    168(%rsp), %edx
	xorl    140(%rbp), %eax
	xorl    %ecx, %edx
	movl    132(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 36(%rbp)
	xorl    %eax, %ecx

	# u = 34.
	roll    $15, %eax
	movl    %ecx, 196(%rsp) # delayed
	pshufd  $2, %xmm4, %xmm0
	movl    160(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    40(%rbp), %eax
	notl    %edx
	xorl    188(%rsp), %ecx
	andl    172(%rsp), %edx
	xorl    136(%rbp), %eax
	xorl    %ecx, %edx
	movl    136(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 40(%rbp)
	xorl    %eax, %ecx

	# u = 35.
	roll    $15, %eax
	movl    %ecx, 200(%rsp) # delayed
	pshufd  $3, %xmm4, %xmm0
	movl    164(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    44(%rbp), %eax
	notl    %edx
	xorl    192(%rsp), %ecx
	andl    176(%rsp), %edx
	xorl    132(%rbp), %eax
	xorl    %ecx, %edx
	movl    140(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 44(%rbp)
	xorl    %eax, %ecx

	# u = 36.
	roll    $15, %eax
	movl    %ecx, 204(%rsp) # delayed
	movl    168(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm5, %ecx
	xorl    0(%rbp), %eax
	notl    %edx
	xorl    196(%rsp), %ecx
	andl    180(%rsp), %edx
	xorl    128(%rbp), %eax
	xorl    %ecx, %edx
	movl    144(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 0(%rbp)
	xorl    %eax, %ecx

	# u = 37.
	roll    $15, %eax
	movl    %ecx, 208(%rsp) # delayed
	pshufd  $1, %xmm5, %xmm0
	movl    172(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    4(%rbp), %eax
	notl    %edx
	xorl    200(%rsp), %ecx
	andl    184(%rsp), %edx
	xorl    124(%rbp), %eax
	xorl    %ecx, %edx
	movl    148(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 4(%rbp)
	xorl    %eax, %ecx

	# u = 38.
	roll    $15, %eax
	movl    %ecx, 212(%rsp) # delayed
	pshufd  $2, %xmm5, %xmm0
	movl    176(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    8(%rbp), %eax
	notl    %edx
	xorl    204(%rsp), %ecx
	andl    188(%rsp), %edx
	xorl    120(%rbp), %eax
	xorl    %ecx, %edx
	movl    152(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 8(%rbp)
	xorl    %eax, %ecx

	# u = 39.
	roll    $15, %eax
	movl    %ecx, 216(%rsp) # delayed
	pshufd  $3, %xmm5, %xmm0
	movl    180(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    12(%rbp), %eax
	notl    %edx
	xorl    208(%rsp), %ecx
	andl    192(%rsp), %edx
	xorl    116(%rbp), %eax
	xorl    %ecx, %edx
	movl    156(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 12(%rbp)
	xorl    %eax, %ecx

	# u = 40.
	roll    $15, %eax
	movl    %ecx, 220(%rsp) # delayed
	movl    184(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm6, %ecx
	xorl    16(%rbp), %eax
	notl    %edx
	xorl    212(%rsp), %ecx
	andl    196(%rsp), %edx
	xorl    112(%rbp), %eax
	xorl    %ecx, %edx
	movl    160(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 16(%rbp)
	xorl    %eax, %ecx

	# u = 41.
	roll    $15, %eax
	movl    %ecx, 224(%rsp) # delayed
	pshufd  $1, %xmm6, %xmm0
	movl    188(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    20(%rbp), %eax
	notl    %edx
	xorl    216(%rsp), %ecx
	andl    200(%rsp), %edx
	xorl    172(%rbp), %eax
	xorl    %ecx, %edx
	movl    164(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 20(%rbp)
	xorl    %eax, %ecx

	# u = 42.
	roll    $15, %eax
	movl    %ecx, 228(%rsp) # delayed
	pshufd  $2, %xmm6, %xmm0
	movl    192(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    24(%rbp), %eax
	notl    %edx
	xorl    220(%rsp), %ecx
	andl    204(%rsp), %edx
	xorl    168(%rbp), %eax
	xorl    %ecx, %edx
	movl    168(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 24(%rbp)
	xorl    %eax, %ecx

	# u = 43.
	roll    $15, %eax
	movl    %ecx, 232(%rsp) # delayed
	pshufd  $3, %xmm6, %xmm0
	movl    196(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    28(%rbp), %eax
	notl    %edx
	xorl    224(%rsp), %ecx
	andl    208(%rsp), %edx
	xorl    164(%rbp), %eax
	xorl    %ecx, %edx
	movl    172(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 28(%rbp)
	xorl    %eax, %ecx

	# u = 44.
	roll    $15, %eax
	movl    %ecx, 236(%rsp) # delayed
	movl    200(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm7, %ecx
	xorl    32(%rbp), %eax
	notl    %edx
	xorl    228(%rsp), %ecx
	andl    212(%rsp), %edx
	xorl    160(%rbp), %eax
	xorl    %ecx, %edx
	movl    176(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 32(%rbp)
	xorl    %eax, %ecx

	# u = 45.
	roll    $15, %eax
	movl    %ecx, 240(%rsp) # delayed
	pshufd  $1, %xmm7, %xmm0
	movl    204(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    36(%rbp), %eax
	notl    %edx
	xorl    232(%rsp), %ecx
	andl    216(%rsp), %edx
	xorl    156(%rbp), %eax
	xorl    %ecx, %edx
	movl    180(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 36(%rbp)
	xorl    %eax, %ecx

	# u = 46.
	roll    $15, %eax
	movl    %ecx, 244(%rsp) # delayed
	pshufd  $2, %xmm7, %xmm0
	movl    208(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    40(%rbp), %eax
	notl    %edx
	xorl    236(%rsp), %ecx
	andl    220(%rsp), %edx
	xorl    152(%rbp), %eax
	xorl    %ecx, %edx
	movl    184(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 40(%rbp)
	xorl    %eax, %ecx

	# u = 47.
	roll    $15, %eax
	movl    %ecx, 248(%rsp) # delayed
	pshufd  $3, %xmm7, %xmm0
	movl    212(%rsp), %edx
	leal    (%rax, %rax, 4), %eax
	movd    %xmm0, %ecx
	xorl    44(%rbp), %eax
	notl    %edx
	xorl    240(%rsp), %ecx
	andl    224(%rsp), %edx
	xorl    148(%rbp), %eax
	xorl    %ecx, %edx
	movl    188(%rsp), %ecx
	leal    (%rax, %rax, 2), %eax
	notl    %ecx
	xorl    %edx, %eax
	roll    $1, %ecx
	movl    %eax, 44(%rbp)
	xorl    %eax, %ecx
	movl    %ecx, 252(%rsp)

	# AUTO END

	# Subtract M words from C words, and store in B.

	movdqa  112(%rbp), %xmm0       # Load C[0:3].
	movdqa  128(%rbp), %xmm1       # Load C[4:7].
	movdqa  144(%rbp), %xmm2       # Load C[8:11].
	movdqa  160(%rbp), %xmm3       # Load C[12:15].
	psubd   %xmm4, %xmm0           # Subtract M[0:3] from C[0:3].
	psubd   %xmm5, %xmm1           # Subtract M[4:7] from C[4:7].
	psubd   %xmm6, %xmm2           # Subtract M[8:11] from C[8:11].
	psubd   %xmm7, %xmm3           # Subtract M[12:15] from C[12:15].
	movdqa  %xmm0, 48(%rbp)        # Store new B[0:3].
	movdqa  %xmm1, 64(%rbp)        # Store new B[4:7].
	movdqa  %xmm2, 80(%rbp)        # Store new B[8:11].
	movdqa  %xmm3, 96(%rbp)        # Store new B[12:15].

	# Add C words to A words.

	movdqa  112(%rbp), %xmm0       # Load C[0:3].
	movdqu  124(%rbp), %xmm1       # Load C[3:6].
	movdqu  140(%rbp), %xmm2       # Load C[7:10].
	pslldq  $4, %xmm0              # Compute 0|C[0:2].
	movl    172(%rbp), %eax        # Load C[15].
	movdqu  156(%rbp), %xmm3       # Load C[11:14].
	movdqa  0(%rbp), %xmm4         # Load A[0:3].
	movd    %eax, %xmm7            # Put C[15] in xmm register.
	movdqa  16(%rbp), %xmm5        # Load A[4:7].
	movdqa  32(%rbp), %xmm6        # Load A[8:11].
	por     %xmm7, %xmm0           # Compute C[15]|C[0:2].

	paddd   %xmm1, %xmm5           # Add C[3:6] to A[4:7].
	paddd   %xmm3, %xmm1           # Add C[11:14] to C[3:6].
	paddd   %xmm2, %xmm5           # Add C[7:10] to A[4:7].
	paddd   %xmm2, %xmm6           # Add C[7:10] to A[8:11].
	paddd   %xmm1, %xmm4           # Add C[3:6]+C[11:14] to A[0:3].
	paddd   %xmm0, %xmm5           # Add C[15]|C[0:2] to A[4:7].
	paddd   %xmm1, %xmm6           # Add C[3:6]+C[11:14] to A[8:11].
	paddd   %xmm0, %xmm4           # Add C[15]|C[0:2] to A[0:3].

	movdqa  %xmm5, 16(%rbp)        # Store A[4:7].
	movdqa  %xmm6, 32(%rbp)        # Store A[8:11].
	movdqa  %xmm4, 0(%rbp)         # Store A[0:3].

	# Store new C words (from Bx[48:63]).

	movdqa  192(%rsp), %xmm0       # Load B[0:3] (from Bx[]).
	movdqa  208(%rsp), %xmm1       # Load B[4:7] (from Bx[]).
	movdqa  224(%rsp), %xmm2       # Load B[8:11] (from Bx[]).
	movdqa  240(%rsp), %xmm3       # Load B[12:15] (from Bx[]).
	movdqa  %xmm0, 112(%rbp)       # Store new C[0:3].
	movdqa  %xmm1, 128(%rbp)       # Store new C[4:7].
	movdqa  %xmm2, 144(%rbp)       # Store new C[8:11].
	movdqa  %xmm3, 160(%rbp)       # Store new C[12:15].

	# Increment W.
	incq    176(%rbp)

	# Update data pointer and decrement block counter; if not zero, loop.
	addq    $64, %rsi
	decq    %rdi
	jnz     Lm0

	# Remove the local variables, restore the registers and exit.
	movq    256(%rsp), %rax
	movq    %rax, %rsp
	popq    %rbp
	ret
	.size   shabal_inner, .-shabal_inner

# shabal_init(sc, out_size)
#    sc         pointer to context structure
#    out_size   hash output size (in bits)
#
	.globl  shabal_init
	.type   shabal_init, @function
shabal_init:
	movl    %esi, %eax

	# Obtain address of 'iv' in a position-independent way.
	call    Li1
Li1:
	popq    %rsi
	addq    $(iv - Li1) - 176, %rsi

	# Compute start address of the relevant precomputed IV.
	movl    %eax, 256(%rdi)        # Store out_size field.
	shrl    $5, %eax
	imull   $176, %eax, %eax
	addq    %rax, %rsi

	# Copy state.
	addq    $72, %rdi
	movl    $22, %ecx
	rep movsq

	# Set W.
	movl    $1, %eax
	stosq

	# Set ptr field.
	subq    $192, %rdi
	xorl    %eax, %eax
	stosq

	ret
	.size   shabal_init, .-shabal_init

# Precomputed IV for all 16 supported sizes.
	.align  8
	.type   iv, @object
iv:
	# output size = 32 bits
	.long   0x30991624, 0x806813D1, 0xBCA662CB, 0xD12B9E3E
	.long   0x015C94E0, 0x90864C9D, 0x8EE87628, 0xC9B16670
	.long   0x6300436B, 0x00B74372, 0x6410F22C, 0x727CCCA8
	.long   0x405762CA, 0xADAFA2DD, 0x094A20D4, 0x0299DB0E
	.long   0x8E697024, 0x80301429, 0x9FD43B12, 0x43429851
	.long   0xB6CE9CE7, 0xFD7E01E8, 0x05C08832, 0x1B0B9BF9
	.long   0x203139BE, 0xDF941A78, 0x9028DC03, 0xD5079E88
	.long   0xDC16B134, 0x1D902C0F, 0x7FABDE1C, 0x3BD00C56
	.long   0x2C6130DA, 0xDDACF5F4, 0x5FC89982, 0x9992F1D5
	.long   0xA26EC215, 0x82DFFD2C, 0xCA1F5AC1, 0x3005D535
	.long   0x4F845186, 0x624E8186, 0x7E8EAA93, 0x0CC5813E

	# output size = 64 bits
	.long   0xAFBA1D74, 0x43D93628, 0xED6741E5, 0x24E6B5AD
	.long   0x7CF8195B, 0x1348594B, 0x0B1D67BB, 0x8ECE0945
	.long   0x27A57DFA, 0xA3C28A8C, 0x88108C3E, 0x4E48C0AF
	.long   0xC581116E, 0x16ED2D7F, 0xF5A2D28F, 0xC56D559D
	.long   0xBAA08E92, 0x64D7C8FC, 0x3E690AC9, 0x38B69953
	.long   0x04DCCD05, 0xD1C9359A, 0x6A8545FA, 0x515BB34E
	.long   0x81FAC868, 0xEB40C1E5, 0x6D95629C, 0x45C0F632
	.long   0x02105ED2, 0xBCA69AFF, 0x7402E45C, 0x3B3AE370
	.long   0x08A04FA4, 0x5A6CD21D, 0x462EB4C0, 0x8CC6966C
	.long   0x500569B9, 0x5B580756, 0x8A23BEA0, 0x0C0D95F0
	.long   0xA8A2CC5E, 0x9DF90247, 0xE33F5B0F, 0x4CA34C8E

	# output size = 96 bits
	.long   0x9D63B7AE, 0x30D32AD2, 0xFD2D64F8, 0x1A4F88ED
	.long   0x9F8C1E0E, 0x5B40AC3A, 0xBC49378A, 0x78C4CEB7
	.long   0x4D460412, 0x368F606F, 0x203C7132, 0x07775329
	.long   0x2A7DD153, 0x9E434DA3, 0x6986024A, 0x160620C6
	.long   0x6A0DDA63, 0x0C300A9D, 0x928D0C82, 0xA2AEA46C
	.long   0x5B026685, 0x9D381B86, 0x852DF223, 0x336AFB47
	.long   0xFCE8B147, 0xF7D77805, 0x24FD4088, 0xDCA077F4
	.long   0x48B9AF9E, 0xABDF6C90, 0x0DCD5ED5, 0xEECB6206
	.long   0x6D12A365, 0x98F87C66, 0xCDC87E8D, 0xB5B4A748
	.long   0x704AC162, 0x9F1CBD4B, 0xE61902FF, 0xB1BB72EA
	.long   0x2970E70A, 0xCEF5B6A6, 0x14DF55CB, 0x20943F97

	# output size = 128 bits
	.long   0x78B31FB9, 0xB6752098, 0x7E0A5E4F, 0xB9909576
	.long   0xD29FEA11, 0x5001C1B0, 0x173061EB, 0x6259160F
	.long   0xBA854A36, 0x6B335E13, 0xB6DBCF5C, 0x1D2D8037
	.long   0x1DFBAFBB, 0xDAA1D17F, 0x11432D8F, 0x22FE6988
	.long   0x438A5BB8, 0x017AA3EA, 0xDC20F53C, 0xA32C1D59
	.long   0xF4EB3A99, 0x9F6B309D, 0x475AF4D1, 0x04AE709E
	.long   0x6FE7FEB6, 0x88F92195, 0x80DD47EF, 0xEC247FDE
	.long   0xD1E7C920, 0xAE103B8D, 0x98C6AD6B, 0x66B44C42
	.long   0x26ACADFF, 0x17CE770A, 0xEC52610A, 0x8EA29C91
	.long   0x9ADA9EB5, 0x68EAB2A2, 0x0D7CFF0C, 0xF34510D5
	.long   0x0CA1C747, 0xAFF5CB59, 0xEDBD3DC8, 0x35E3FEBA

	# output size = 160 bits
	.long   0x23043D95, 0xADF7E880, 0xF5A289A7, 0x08CE1221
	.long   0xD8033158, 0x655D4342, 0xF8F100BC, 0xAEB91981
	.long   0x2A2F6A37, 0xAF532EE8, 0x3637E85F, 0x26FF1DB0
	.long   0x9C04E33B, 0x001EA45F, 0xBFF4AD80, 0xBE0EFADB
	.long   0x3F4E1699, 0x4AC015FF, 0xCE944991, 0x657C86A4
	.long   0x16237082, 0x69E66332, 0x081B6170, 0x616FBCEF
	.long   0xAA62AB9A, 0xD722D78F, 0x74CA0624, 0xA6CA8341
	.long   0x9FFD9230, 0x29A59196, 0x533E5573, 0xD4DF9963
	.long   0x6171440E, 0x0ECC85CF, 0xCB2B959C, 0x89654A9F
	.long   0x4CA1C101, 0xB55C326C, 0xF6D7614C, 0x380EED0C
	.long   0x51AC69B0, 0x6199E925, 0xCAAB12F6, 0x1B3AB589

	# output size = 192 bits
	.long   0xFD749ED4, 0xB798E530, 0x33904B6F, 0x46BDA85E
	.long   0x076934B4, 0x454B4058, 0x77F74527, 0xFB4CF465
	.long   0x62931DA9, 0xE778C8DB, 0x22B3998E, 0xAC15CFB9
	.long   0x58BCBAC4, 0xEC47A08E, 0xAEE933B2, 0xDFCBC824
	.long   0xA7944804, 0xBF65BDB0, 0x5A9D4502, 0x59979AF7
	.long   0xC5CEA54E, 0x4B6B8150, 0x16E71909, 0x7D632319
	.long   0x930573A0, 0xF34C63D1, 0xCAF914B4, 0xFDD6612C
	.long   0x61550878, 0x89EF2B75, 0xA1660C46, 0x7EF3855B
	.long   0x7297B58C, 0x1BC67793, 0x7FB1C723, 0xB66FC640
	.long   0x1A48B71C, 0xF0976D17, 0x088CE80A, 0xA454EDF3
	.long   0x1C096BF4, 0xAC76224B, 0x5215781C, 0xCD5D2669

	# output size = 224 bits
	.long   0xA5201467, 0xA9B8D94A, 0xD4CED997, 0x68379D7B
	.long   0xA7FC73BA, 0xF1A2546B, 0x606782BF, 0xE0BCFD0F
	.long   0x2F25374E, 0x069A149F, 0x5E2DFF25, 0xFAECF061
	.long   0xEC9905D8, 0xF21850CF, 0xC0A746C8, 0x21DAD498
	.long   0x35156EEB, 0x088C97F2, 0x26303E40, 0x8A2D4FB5
	.long   0xFEEE44B6, 0x8A1E9573, 0x7B81111A, 0xCBC139F0
	.long   0xA3513861, 0x1D2C362E, 0x918C580E, 0xB58E1B9C
	.long   0xE4B573A1, 0x4C1A0880, 0x1E907C51, 0x04807EFD
	.long   0x3AD8CDE5, 0x16B21302, 0x02512C53, 0x2204CB18
	.long   0x99405F2D, 0xE5B648A1, 0x70AB1D43, 0xA10C25C2
	.long   0x16F1AC05, 0x38BBEB56, 0x9B01DC60, 0xB1096D83

	# output size = 256 bits
	.long   0x52F84552, 0xE54B7999, 0x2D8EE3EC, 0xB9645191
	.long   0xE0078B86, 0xBB7C44C9, 0xD2B5C1CA, 0xB0D2EB8C
	.long   0x14CE5A45, 0x22AF50DC, 0xEFFDBC6B, 0xEB21B74A
	.long   0xB555C6EE, 0x3E710596, 0xA72A652F, 0x9301515F
	.long   0xDA28C1FA, 0x696FD868, 0x9CB6BF72, 0x0AFE4002
	.long   0xA6E03615, 0x5138C1D4, 0xBE216306, 0xB38B8890
	.long   0x3EA8B96B, 0x3299ACE4, 0x30924DD4, 0x55CB34A5
	.long   0xB405F031, 0xC4233EBA, 0xB3733979, 0xC0DD9D55
	.long   0xC51C28AE, 0xA327B8E1, 0x56C56167, 0xED614433
	.long   0x88B59D60, 0x60E2CEBA, 0x758B4B8B, 0x83E82A7F
	.long   0xBC968828, 0xE6E00BF7, 0xBA839E55, 0x9B491C60

	# output size = 288 bits
	.long   0x66653F11, 0xF9446EDB, 0x3E8568FB, 0xE519F95A
	.long   0xD6732054, 0xB2BDA0C5, 0x492F3CF8, 0x8FD63B96
	.long   0xB4EF4648, 0x7E5488E5, 0xA7ED170A, 0x5B25FA52
	.long   0xBBC636A1, 0xB87FB1E3, 0xED8EED32, 0x6D10E8D5
	.long   0x34868338, 0x56EB9A07, 0x27C8D6ED, 0x0283EF48
	.long   0x6B1AB349, 0x2619870B, 0xDDD0A56C, 0x7F72935B
	.long   0xD2B1A3C6, 0xD7DA5563, 0x58292E94, 0x363320A4
	.long   0xB8904378, 0xBC37135F, 0x2FCFECEC, 0xC74DF052
	.long   0xB70F01DE, 0x523EB809, 0xD8E14A78, 0x181A659D
	.long   0x90894E40, 0x48865BA0, 0x282C1F3F, 0x1C2488E9
	.long   0x0141B2E6, 0xFFD06EFB, 0xA1F15A68, 0xCFB06A87

	# output size = 320 bits
	.long   0x15FB76C8, 0x8BA3597D, 0x1B66A087, 0x65C5555B
	.long   0x3962E239, 0x4EA9C7C7, 0x93D9671B, 0x3074B7BE
	.long   0xA224ED96, 0x38A8A88B, 0x9A515FEA, 0x125ADBD4
	.long   0xF30E67D1, 0xCD0679E4, 0x0F373D5F, 0x1FCEF74D
	.long   0x5C80F3FB, 0xA6C70631, 0x0ECD6A69, 0x73885C8F
	.long   0x1B77FE69, 0xD487D25A, 0xC027EED1, 0x3F31BD8F
	.long   0xE275F714, 0x92D07441, 0x5B70FF10, 0x177AE160
	.long   0x6FCFA5FB, 0x67139322, 0x2D3109B2, 0x601A088C
	.long   0x293386A7, 0xF87584AB, 0x00862B83, 0x87EF6304
	.long   0xF69DA436, 0x6AA88022, 0x09D1EC75, 0x8099B045
	.long   0x59F8B2DE, 0x343456A8, 0x91F66DAE, 0xAF1D30ED

	# output size = 352 bits
	.long   0x1BD5653A, 0x63D79468, 0xAE5166FA, 0x3FA0E9CC
	.long   0x3578296E, 0xFFAA2881, 0xA07E89ED, 0xF38890CA
	.long   0x09111A9E, 0xF908F100, 0x8476444D, 0xFE33E1F8
	.long   0x5B228C89, 0x1982F710, 0x0C51EE3E, 0xCC6C68EE
	.long   0x5629BF34, 0xCDEA4F9F, 0x203C3ECA, 0xE352E40A
	.long   0x3FC6F380, 0x35E1DA2D, 0x8522A9DB, 0xB1DB66AE
	.long   0x07C002A7, 0xE495CA9B, 0x8D8B1D89, 0x1848DDFC
	.long   0x2B5E7D26, 0x1F25F07D, 0x9B9E6378, 0x8DC9DDA7
	.long   0x44221A9D, 0x193A59CB, 0xBB7ACA55, 0x432325AB
	.long   0x15CE4B4E, 0x6ED2B4DE, 0x42261A9F, 0x8DCADC14
	.long   0xEB49BC8E, 0xF566AEC0, 0x8A56D431, 0x216B0327

	# output size = 384 bits
	.long   0xC8FCA331, 0xE55C504E, 0x003EBF26, 0xBB6B8D83
	.long   0x7B0448C1, 0x41B82789, 0x0A7C9601, 0x8D659CFF
	.long   0xB6E2673E, 0xCA54C77B, 0x1460FD7E, 0x3FCB8F2D
	.long   0x527291FC, 0x2A16455F, 0x78E627E5, 0x944F169F
	.long   0x1CA6F016, 0xA854EA25, 0x8DB98ABE, 0xF2C62641
	.long   0x30117DCB, 0xCF5C4309, 0x93711A25, 0xF9F671B8
	.long   0xB01D2116, 0x333F4B89, 0xB285D165, 0x86829B36
	.long   0xF764B11A, 0x76172146, 0xCEF6934D, 0xC6D28399
	.long   0xFE095F61, 0x5E6018B4, 0x5048ECF5, 0x51353261
	.long   0x6E6E36DC, 0x63130DAD, 0xA9C69BD6, 0x1E90EA0C
	.long   0x7C35073B, 0x28D95E6D, 0xAA340E0D, 0xCB3DEE70

	# output size = 416 bits
	.long   0x352E0FF3, 0x6AC9C045, 0x2DB4EF9C, 0x77FA2ABD
	.long   0x338FDD70, 0x50534EFF, 0xFC4AEAD6, 0x7FD5E90F
	.long   0xD427A064, 0xAEEFEA00, 0xB2FE4468, 0xE0865D75
	.long   0xFBA3A3EF, 0x426D3426, 0xC3B9AD71, 0x75BB8771
	.long   0x641F7D90, 0xA4D1C61E, 0x50B1078C, 0xDFD10A17
	.long   0xCBF00300, 0x84931E03, 0xB3785FBB, 0xB2FE8C72
	.long   0xFBD8FFE0, 0x668069B2, 0x9389E26C, 0xA48A5538
	.long   0xF4FEB4AD, 0x53794AD4, 0xC7646768, 0x95CA7E26
	.long   0x4826175D, 0xFA1862B5, 0x3D4FD63C, 0x18D1D0B3
	.long   0x460F51F6, 0xB6B0FD70, 0xEB95B530, 0x36B38BC3
	.long   0x712EC1DD, 0xF1358202, 0x1C4B7E51, 0xF3B66BD6

	# output size = 448 bits
	.long   0x1D134615, 0xCE0B8376, 0x92A35944, 0x35D26466
	.long   0x1DB7AEDE, 0xDD3C360C, 0xAF5D0BE3, 0x5A3A3E91
	.long   0x9F0C34CD, 0xF520D9D4, 0xDDA48EC5, 0x4D3AE612
	.long   0x295698A4, 0x9240153D, 0xD6389B78, 0xC5CACDB0
	.long   0x2FBBFD90, 0x9F75A74A, 0x74DFABAC, 0xE79878CD
	.long   0x741DB421, 0xF4B2D818, 0xA0534AD1, 0x21DB4895
	.long   0x06706D7D, 0xD4E49AE5, 0x55CEA19F, 0xB2EEF72C
	.long   0x603BDEBA, 0xF3E69889, 0xA2F81607, 0xA7ECFE58
	.long   0x328401A7, 0x34056125, 0x275F0B5F, 0x73096110
	.long   0xD1B17548, 0x6285F95B, 0xC6847C12, 0xB1B2CC4D
	.long   0xE39A4C46, 0x666CBA43, 0xBF67FEDD, 0x6E6A920D

	# output size = 480 bits
	.long   0xE12D27A7, 0x043FF4FC, 0x381D9D67, 0xEE7CFD88
	.long   0xFAFDD1F4, 0x3FC132F0, 0x56D23777, 0xF1B02BFA
	.long   0x0F36802A, 0x5D56CF5A, 0x326F7F77, 0x15EC55C3
	.long   0x2DBA88B8, 0x8E28E38F, 0x8415932B, 0x34BC9908
	.long   0x387AE120, 0x38805CB7, 0xECBD45D3, 0x09923083
	.long   0x28077C3E, 0xCA2F47E3, 0x98AA2869, 0x55597017
	.long   0x9706DC5C, 0xD84853FA, 0x64B3D086, 0x2C175E1A
	.long   0x42C658E1, 0x1948A0AC, 0xEEDD9CAE, 0xFD90CE89
	.long   0xCBE143C9, 0x5755D779, 0x6DC174EC, 0xBBBDC5FD
	.long   0xD88C8760, 0xC052A9D6, 0x252D700E, 0x084A5959
	.long   0x78DC3463, 0x69C1402E, 0xEE5055CF, 0x5E909C8D

	# output size = 512 bits
	.long   0x20728DFD, 0x46C0BD53, 0xE782B699, 0x55304632
	.long   0x71B4EF90, 0x0EA9E82C, 0xDBB930F1, 0xFAD06B8B
	.long   0xBE0CAE40, 0x8BD14410, 0x76D2ADAC, 0x28ACAB7F
	.long   0xC1099CB7, 0x07B385F3, 0xE7442C26, 0xCC8AD640
	.long   0xEB6F56C7, 0x1EA81AA9, 0x73B9D314, 0x1DE85D08
	.long   0x48910A5A, 0x893B22DB, 0xC5A0DF44, 0xBBC4324E
	.long   0x72D2F240, 0x75941D99, 0x6D8BDE82, 0xA1A7502B
	.long   0xD9BF68D1, 0x58BAD750, 0x56028CB2, 0x8134F359
	.long   0xB5D469D8, 0x941A8CC2, 0x418B2A6E, 0x04052780
	.long   0x7F07D787, 0x5194358F, 0x3C60D665, 0xBE97D79A
	.long   0x950C3434, 0xAED9A06D, 0x2537DC8D, 0x7CDB5969

	.size   iv, .-iv

# reduced_memcpy(): enhanced 'rep movsb' with the same register
# conventions (but flags are altered).
	.type   reduced_memcpy, @function
reduced_memcpy:
	pushq   %rcx
	shrq    $3, %rcx
	repnz movsq
	popq    %rcx
	andl    $7, %ecx
	repnz movsb
	ret
	.size   reduced_memcpy, .-reduced_memcpy

# Align the context structure.
# Input:
#   %rbp   points to the source structure
#   %rdi   points to the destination buffer
# The destination buffer must be at an address which is equal to 16
# modulo 64. Its size is 184 bytes.
#
# If %rbp is such that the context A[0] address is already equal to
# 16 modulo 64, then this routine sets %rdi to &A[0] and returns.
# Otherwise, it copies the state words (the 44 A/B/C words and the
# W counter) into the destination buffer, and keeps %rdi unchanged.
# Either way, %rdi, upon exit, points to a properly aligned buffer.
#
# Registers other than %rdi are preserved.
#
	.type   align_structure_enter, @function
align_structure_enter:
	pushq   %rax
	leaq    116(%rbp), %rax
	testl   $63, %eax
	popq    %rax
	jnz     Lase1
	leaq    72(%rbp), %rdi
	ret
Lase1:
	pushq   %rdi
	pushq   %rsi
	pushq   %rcx
	leaq    72(%rbp), %rsi
	movl    $23, %ecx
	rep movsq
	popq    %rcx
	popq    %rsi
	popq    %rdi
	ret
	.size   align_structure_enter, .-align_structure_enter

# Copy the state back to the context structure, if necessary.
# Input:
#   %rbp   points to the context structure
#   %rsi   points to the state words
# The copy is performed unless %rdi already points to the state
# words within the source structure.
#
# All registers are preserved.
#
	.type   align_structure_leave, @function
align_structure_leave:
	pushq   %rdi
	leaq    72(%rbp), %rdi
	testq   %rsi, %rdi
	jz      Lasl1
	pushq   %rcx
	pushq   %rsi
	movl    $23, %ecx
	rep movsq
	popq    %rsi
	popq    %rcx
Lasl1:
	popq    %rdi
	ret
	.size   align_structure_leave, .-align_structure_leave

# shabal(sc, data, len)
#    sc     pointer to context structure
#    data   input data
#    len    input data length (in bytes)
#
	.globl  shabal
	.type   shabal, @function
shabal:
	pushq   %rbp

	# Conventions:
	#    %rbp   pointer to context structure
	#    %rsi   data pointer (already set)
	#    %rdx   data length (already set)
	movq    %rdi, %rbp

	# We reserve some room for a copy of the state words, with
	# proper alignment.
	movq    %rsp, %rax
	subq    $144, %rsp
	andq    $0xFFFFFFFFFFFFFFC0, %rsp
	subq    $48, %rsp
	movq    %rax, 184(%rsp)

	# We load the ptr field into %ecx, and see whether we have
	# enough data for a full block.
	movl    64(%rbp), %ecx
	movl    $64, %eax
	subl    %ecx, %eax
	cmpq    %rax, %rdx
	jae     Ls1

	# Not enough data (input and buffered) to get a full block;
	# we buffer the provided data, and exit.
	leaq    (%rbp, %rcx), %rdi
	movl    %edx, %ecx
	call    reduced_memcpy
	subq    %rbp, %rdi
	movl    %edi, 64(%rbp)
	jmp     Ls9

Ls1:
	# Since we will call shabal_inner(), we align the state words.
	movq    %rsp, %rdi
	call    align_structure_enter

	# At that point, %ecx contains ptr, and %eax contains 64-ptr.
	# If ptr != 0, then we fill the partial block and process it
	# alone.
	testl   %ecx, %ecx
	jz      Ls2
	subq    %rax, %rdx
	pushq   %rdi
	leaq    (%rbp, %rcx), %rdi
	movl    %eax, %ecx
	call    reduced_memcpy
	popq    %rdi

	pushq   %rdx
	pushq   %rsi
	pushq   %rdi
	movl    $1, %edx
	movq    %rbp, %rsi
	call    shabal_inner
	popq    %rdi
	popq    %rsi
	popq    %rdx

Ls2:
	# At that point, %rdx contains the remaining data length. If
	# the data contains at least one full block, then we call the
	# inner function.
	movq    %rdx, %rax
	shrq    $6, %rax
	jz      Ls3
	andl    $63, %edx

	pushq   %rdx
	movq    %rax, %rdx
	shlq    $6, %rax
	addq    %rsi, %rax
	pushq   %rax
	pushq   %rdi
	call    shabal_inner
	popq    %rdi
	popq    %rsi
	popq    %rdx

Ls3:
	# We copy the remaining partial block into the buffer (we do not
	# forget to set the ptr field as well). The address of the state
	# words is currently in %rdi; we save it ans restore it in %rsi.
	pushq   %rdi
	movq    %rbp, %rdi
	movl    %edx, %ecx
	movl    %edx, 64(%rbp)
	call    reduced_memcpy
	popq    %rsi

	# Make sure we propagate back the state words to the context
	# structure. The state words address is in %rsi.
	call    align_structure_leave

Ls9:
	# Release stack frame and exit.
	movq    184(%rsp), %rax
	movq    %rax, %rsp
	popq    %rbp
	ret
	.size   shabal, .-shabal

# shabal_close(sc, ub, n, dst)
#    sc    pointer to context structure
#    ub    extra bits
#    n     number of extra bits (0 to 7)
#    dst   destination buffer for output
#
	.globl  shabal_close
	.type   shabal_close, @function
shabal_close:
	pushq   %rbx
	pushq   %rbp

	# Conventions:
	#    %rbp   pointer to context structure
	#    %rdx   ub
	#    %rcx   n
	#    %rsi   output buffer
	movq    %rdi, %rbp
	pushq   %rcx
	pushq   %rdx
	pushq   %rsi
	popq    %rdx
	popq    %rcx
	popq    %rsi

	# Reserve some room for a 64-byte aligned copy of the state words.
	movq    %rsp, %rax
	subq    $144, %rsp
	andq    $0xFFFFFFFFFFFFFFC0, %rsp
	subq    $48, %rsp
	movq    %rax, 184(%rsp)

	# Compute extra byte.
	movl    $0x80, %eax
	shrl    %cl, %eax
	orl     %eax, %edx
	negl    %eax
	andl    %edx, %eax

	# Store extra byte, then some zeros. Note: we may store more
	# zeros than necessary, but this is harmless (this just overwrites
	# the 'ptr' field).
	movq    %rbp, %rdi
	movl    64(%rbp), %ecx
	addq    %rcx, %rdi
	stosb
	negl    %ecx
	xorl    %eax, %eax
	addl    $70, %ecx
	shrl    $3, %ecx
	repnz stosq

	# Make sure that the state words are properly aligned.
	movq    %rsp, %rdi
	call    align_structure_enter

	# We save the destination buffer pointer.
	pushq   %rsi

	# Call inner function four times (final block, and the
	# three finalization rounds). Since shabal_inner() increments
	# the block counter, we must reverse that increment (in the
	# copy structure, not the original, in case they are distinct).
	movl    $4, %ebx
Lc1:
	movl    $1, %edx
	movq    %rbp, %rsi
	pushq   %rdi
	call    shabal_inner
	popq    %rdi
	decq    176(%rdi)
	decl    %ebx
	jnz     Lc1

	# Copy the hash function output into the provided buffer.
	# We get out_size from the original structure (%rbp) and the C
	# words from the copy (%rdi). The destination buffer pointer
	# currently on top of the stack.
	movl    256(%rbp), %eax
	movl    %eax, %ecx
	shrl    $3, %eax
	shrl    $5, %ecx
	leaq    176(%rdi), %rsi
	popq    %rdi
	subq    %rax, %rsi
	rep movsl

	movq    184(%rsp), %rax
	movq    %rax, %rsp
	popq    %rbp
	popq    %rbx
	ret
	.size   shabal_close, .-shabal_close
