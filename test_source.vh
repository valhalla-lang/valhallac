-- TODO: Test overloading with `f`.
-- TODO: Test casting from subsets of Real, upto Real.

R = Real

BinaryType = R -> R -> R
(*) : BinaryType
(+) : BinaryType
(-) : BinaryType

f : Real -> Real -> Real
f a b = (1 + 1) * a + b  -- 2a + b

a : Nat
a = 3

b : Int
b = 1 - 8  -- -7

c : Real
c = f (a + 1.0) (b - 0.0)



