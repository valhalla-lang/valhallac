-- The `add' function return the sum of two inputs:

add : Nat -> Nat -> Nat
add n m = n + m

-- This can be rewritten as a singleton set return value:
add : (n : Nat) -> (m : Nat) -> [ n + m ]

-- and so no direct definition is required, because it can
-- only possibly return one exact value, based on the inputs.
-- (The return set only holds one value).

-- This is not always applicable, for pattern/case matches and such.
