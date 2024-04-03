# Building a Minimal Compiler Front-end

This project consists of a small compiler front-end, including lexical analysis and syntax analysis.

The input is a source text file containing code in a minimal language designed for this project. This language supports integer constants, variable bindings, arithmetic and assignment operators, as well as printing.

The output if this program is a syntax tree, which can be traversed to give the order in which theoretical operations have to be performed.

Code generation is out of the scope of this paper.

The interface of this software is a simple CLI which takes a .txt file path as an argument and prints the operations in order. For example, a user would write "compiler.exe my_input.txt" to the command line and the program could print 
"ASSIGN(5,X) ADD(X, 3) PRINT(X)".


