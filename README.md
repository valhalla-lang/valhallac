<p align="center">
  <img alt="Valhalla Flag" height=230 src="https://github.com/Demonstrandum/valhalla/raw/master/assets/logo.svg.png" />
</p>

# Valhalla Programming Language
This is the parser and compiler for Valhalla, which excludes the virtual
machine that the compiled bytecode runs on, which is,
[Brokkr VM](https://github.com/Demonstrandum/brokkr).

## IN (HEAVY) DEVELOPMENT


What's been done so far on the front-end:

- [ ] Parser
  - [x] Lexical analysis, full UTF-8 support, handling: identifiers,
        symbols, numbers, strings (utf-8 with a good number of escapes),
        all sorts of braces for: vectors, sets and grouping, etc.
  - [x] Infix, prefix and suffix notation.
  - [x] Correct parsing of precedence, arity and associativity.
  - [x] The base operators for the language.
  - [x] Proper function calls, Currying / partial application
        of functions is properly implemented (Haskell-like functions).
  - [x] Error messages, with fancy line and column number and read-out of the source line.
  - [x] Constant folding optimisations on trivially deducible
        numeric computations at compile time.
  - [x] Implicit compile-time type-casting in specific situations.
  - [ ] Macros (including macro definitions and macro application).
  - [ ] User-defined binary operators as aliases to functions.
- [ ] Compiler (generating bytecode to assemble an executable file).
  - [x] Table of constants and locals with basic PUSH & POP
        instructions as well as basic arithmetic.
  - [x] Access, assignment and retrieval of local variables within
        code-block scope.
  - [x] Generating curried functions.
  - [ ] Optimise functions to not curry when currying is not neccesary (by tracking arity of
        function's definition and function's call).
  - [ ] Optimise functions to not search globally for variables when they
        come from nested closures (nested closures implement currying).
  - [ ] Optimise functions for tail calls.
  - [ ] Track variable and function types.
  - [x] Marshaling, i.e. serialising the bytecode and storing it in a file
        for future interpretation and execution by the virtual machine.
  - [ ] ...

The VM, i.e. the backend for the language, is being developed independently
and will have its own progress and check-list updates.

### Compile & Run

In your shell, in the root of this repository, you may write:
```
cargo run [source-file-to-compile]
```

For example, you can run.
```sh
cargo run test_source.vh
```
to demonstrate compilation with the included test-file (`test_source.vh`).
The argument of a source-file to compile is, of course, necessary.

### Example of what the compiler currently does:
[current_compiler_test.md](https://github.com/valhalla-lang/valhallac/blob/master/current_compiler_test.md)

### Description

This repository contains the front-end (parser and
bytecode compilation) which processes the syntax and
semantics of the source code. The generated AST is then
compiled to [Brokkr VM](https://github.com/Demonstrandum/brokkr) bytecode.
The execution of the subsequent bytecode
is handled by the language's VM (virtual machine) called
Brokkr, which exists separately from this repository.

Valhalla is a set-theoretic programming language.
That's to say, it's based on principles from set theory,
in a way that all types are just sets, and hence everything
is just an element of a set. The language is meant to give a
new way to interact with types, and provides an intuitive way to
think about them.  A goal is that it may also be used to
verify proofs and such in and around set theory.

The language is a general purpose, but instead of being totally object-oriented,
or functional, etc., it's just set theory based.  From what I've
gathered, it's not a very popular paradigm...  Likely for good reason, but hey,
it might be interesting.

### Dependencies
Yikes...

![deps](https://github.com/Demonstrandum/valhalla/raw/master/graph.png)
