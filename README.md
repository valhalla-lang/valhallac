![Valhalla](https://raw.githubusercontent.com/Demonstrandum/valhalla/master/assets/logo.svg.png?token=AGMZNB7JZ47KSSVVXA2O2S25FXPWA)

# Valhalla Language

This repository contains the front-end (parser and
bytecode compilation) which understands the syntax and
semantics, as well as doing static type analysis and code
optimisation. The generated AST is then compiled to
Brokkr bytecode.
The execution of the subsequential bytecode
is handled by the langauge's VM (virtual machine) called
Brokkr, which exists seperately.

Valhalla is a set theoretic programming language.
That's to say, it's based on priciples from set theory,
in a way that all types are just sets, and hence everything
is just an element of a set. The language is meant to give a
new way to think about types, and provides an intuitive way to
think about types.  It may also be used to verify proofs and such
about set theory.

The language is a general purpose, but intead of being all OOP,
or functional, etc., it's just set theory based.  From what I've
gathered, it's not a very popular paradigme.
