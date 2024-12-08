// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

(MAINLOOP)
    @KBD
    D=M
    @NULLINPUT
    D;JEQ

// black case
    @n
    M=0
(BLACKLOOP)
    // if n == 256 * 32 goto MAINLOOP
    @n
    D=M
    @8192
    D=D-A
    @MAINLOOP
    D;JEQ
    // *(SCREEN + n) = -1
    @SCREEN
    D=A
    @n
    A=D+M
    M=-1
    // incriment
    @n
    M=M+1
    @BLACKLOOP
    0;JMP

// white case
(NULLINPUT)
    @n
    M=0
(WHITELOOP)
    // if n == 256 * 32 goto MAINLOOP
    @n
    D=M
    @8192
    D=D-A
    @MAINLOOP
    D;JEQ
    // *(SCREEN + n) = 0
    @SCREEN
    D=A
    @n
    A=D+M
    M=0
    // incriment
    @n
    M=M+1
    @WHITELOOP
    0;JMP