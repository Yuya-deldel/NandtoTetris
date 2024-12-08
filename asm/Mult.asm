// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

@R0
D=M
@i
M=0
@mul
M=0

(LOOP)
// if (i == R1) goto STOP
@i
D=M
@R1
D=D-M
@STOP
D;JEQ
// mul += R0
@mul
D=M
@R0
D=D+M
@mul
M=D
// i += 1
@i
M=M+1
// goto LOOP
@LOOP
0;JMP

(STOP)
// R2 = mul
@mul
D=M
@R2
M=D

(END)
@END
0;JMP