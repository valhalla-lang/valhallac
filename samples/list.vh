!allow redefinition

-- Code blocks are a series of statements, these statements
-- can be treated as items in a list. After all, that is how it is 
-- represented internally.

stmts = {
	a
	b
	c
}
stmts = { a; b; c }
stmts = (a, b, c)  -- Warning: could not find definition for a, b or c.

-- ^^ All the above are equivalent,
-- they are all code blocks, and are represented as functions
-- that take in *nothing* (i.e `()`), and execute a, b and c (returning c).

[| a; b; c |] = [| 11; 22; 33 |] -- assignment to tuples of equal size.
:(a; b; c) = :(11; 22; 33) -- assignment to quotes/ASTs of equal size.
(a, b, c) = (11, 22, 33) -- assignemt to blocks of equal size.

stmts -- is a function, with three statements.
stmts () -- is a function call, and executes a, b and c.
eval stmts -- same as above, calls the function.

assert (!nth 2 stmts == :b)

-- You can also write something similar
stmts_similar = :(a; b; c) -- This is, however, NOT a function.
stmts_similar -- is a quotation/AST of code, spanning three statements.
eval stmts_similar -- will execute the three stements, returning c.

stmts' = {
	j = i + 2
	3 * j
	j
}

assert (!nth 2 stmts' == :(3 * j))
assert (!nth 3 stmts' == :j)


items = !list { a; b; c } -- or
items = !list (a, b, c)

-- !list is a macro that evaluates a, b and c and places
-- them in a 'list'.

assert (items[2] == 22)

assert (eval :a == 22)
assert (eval :(:a) == :a) -- :xyz is essentially a way to escape/quote code.

-- :ident colon quote operator is slightly special on idents, but
-- works the same in every way.

s = input "Please enter a variable: "
sym = to_symbol s

eval sym |> IO::puts
-- This (unlike the rest) will only execute at
-- run-time (instead of compile-time), it may or may not
-- throw an error, depening on whether the input given as a variable
-- exists in this program.


