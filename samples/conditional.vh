-- For example, a piecewise function:
f : 'A -> 'B -> 'C
f a b = piecewise do:
  C a b,  a == 1
  C b a,  b == 1
  C 1 1,  otherwise

-- or
piecewise {
  x, p
  y, q
  z, otherwise
}

-- or write `cond' instead of `piecewise'
cond do:
  x, p
  y, q
  z, otherwise


-- for more trivial branching, (with pattern matching)

-- Exactly the same as the previous f.
f : 'A -> 'B -> 'C
f a b = match (a, b) do:
  (1, _) => C a b
  (_, 1) => C b a
  (_, _) => C 1 1

-- Or, the same again
f : (a : 'A) -> (b : 'B) -> 'C
f = curry f' where
  f' : 'A * 'B -> 'C
  f' (1, _) = C a b
  f' (_, 1) = C a b
  f' (_, _) = C 1 1

-- Again!
f : (a : 'A) -> (b : 'B) -> 'C
f 1 _ = C a b
f _ 1 = C b a
f _ _ = C 1 1

-- And again,
f : 'A -> 'B -> 'C
f a@1 b = C a b
f a b@1 = C b a
f 1 1 = C 1 1

-- Example of the special (syntactic) => operator

S = [ x : Nat => x > 3 ]
Z = filter f Any where f : Any -> Bool
                       f a = match a do:
                         x : Nat => x > 3 -- This is exactly like in S.
                         _       => False

-- This is how set builder notation works.
-- and how the uses of the => operators are related.

assert (S == Z)

