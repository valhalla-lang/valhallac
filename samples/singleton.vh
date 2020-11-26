-- Say we want to represent the smallest non-zero
-- positive number.  Call it omega.

RealPos = filter (> 0) Real \ [0]

-- Try one (1):
omega_1 = index 0 <| collect [ n : Real => 0 < n < m => m <- RealPos ]
-- This is not quite so great, this will collect the elements
-- into a vector, and since its a singleton, get the first and only
-- element.  This elememt will be computed after a while, and we'll
-- get the smallest non-zero, positive 64bit floating-point number.

-- This is not satisfactory, we want to mathematically represent
-- this concept.

-- Try two (2):
omega_set = [ n : Real => 0 < n < m => m <- RealPos ]

-- Verify that this set is indeed a singleton.
assert (singleton? omega_set)  --> :true

-- Extract the singleton:
omega_2 = singleton omega_set

-- or even better, with a pattern match.
[ omega ] = [ n : Real => 0 < n < m => m <- RealPos ]

assert (omega_2 == omega)


-- Simpler examples:
a : Nat
[ a ] = [ n : Nat => 1 < n < 3 ]

assert (a == 2)


assert (singleton? [ n : Nat  => 2 < n < 4 ] == :true)
assert (singleton? [ n : Real => 2 < n < 4 ] == :false)







