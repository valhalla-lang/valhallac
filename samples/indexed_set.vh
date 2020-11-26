-- You could maybe equate this with an 'array'.

-- Indexed set example:
S = [ "b"; "c"; "a" ]
I = [ 0; 1; 2 ]

-- A is the "array" here
A : I -> S
A 0 = "b"
A 1 = "c"
A 2 = "a"

-- or, equivalently
A = [
  (0, "b")
  (1, "c")
  (2, "a")
]


-- This is very cumbersome though, hence the alternate syntax
A = [| "b"; "c"; "a" |]

assert (A 1 == "c")
assert (A -< I * S)  -- A is a subset of the cartesian product of I and S.

-- An indexed ste is similar to,
-- but not the same as a tuple/ordered pair.
a = ("b", "c", "d")
assert (A /= a)
assert <| all [ nth i a == A i => i <- I ]

