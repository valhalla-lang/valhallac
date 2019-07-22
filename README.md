<p align="center">	
  <img alt="Valhalla Flag" height=230 src="https://github.com/Demonstrandum/valhalla/raw/master/assets/logo.svg.png" />
</p>

# Valhalla Language

## IN DEVELOPMENT

What's been done so far on the front end:
- [ ] Parser
  - [x] Lexical analysis, full UTF-8 support, handling: identifiers, symbols, numbers, strings (utf-8 with good number of escapes), all sorts of braces for vectors, sets, grouping, etc.
  - [x] Infix, prefix and suffix notation.
  - [x] Correct parsing of precedence, arity and associativity.
  - [x] The base operators for the language.
  - [x] Proper function calls, Currying is properly implemented for functions (Haskell-like syntax).
  - [x] (Cool) error messages, with line and column number and read-out of the line.
  - [ ] Macros (incl. macro definitions and macro application).
- [ ] Compiler (generating bytecode to assemble an executable file).

The VM, i.e. the backend for the language, is being developed independently
and will have its own progress and check-list updates.

### Description

This repository contains the front-end (parser and
bytecode compilation) which understands the syntax and
semantics, as well as doing static type analysis and code
optimisation. The generated AST is then compiled to
Brokkr bytecode.
The execution of the subsequent bytecode
is handled by the language's VM (virtual machine) called
Brokkr, which exists separately.

Valhalla is a set theoretic programming language.
That's to say, it's based on principles from set theory,
in a way that all types are just sets, and hence everything
is just an element of a set. The language is meant to give a
new way to think about types, and provides an intuitive way to
think about types.  It may also be used to verify proofs and such
about set theory.

The language is a general purpose, but instead of being all OOP,
or functional, etc., it's just set theory based.  From what I've
gathered, it's not a very popular paradigm.
