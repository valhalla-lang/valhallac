-- Declare equality
a = 3
-- Test equality
a == 3  -- True

-- Declare membership
a : Nat
-- Test memebership
a <- Nat  -- True


-- Subset (non-strict)
A <: Nat
A -< Nat  -- True


-- Any test can be made into a declaration:
b : Nat
b = 4
-- The Test:
3 < b < 5  -- True
-- Made into declaration:
[ b ] = [ n : Nat => 3 < n < 4 ]
-- ^^^ This sets (b=0), by pattent matching on a singleton set.
-- This can ve done for any test, making it into a declaration.
-- (As long as the test is limiting enough, narrowing it down to
--  only a singluar value.)
