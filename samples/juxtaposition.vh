-- Alow for multiplication by juxtaposition

jux : Real -> Real -> Real
jux n m = n * m


-- when the type resolver sees juxtaposition of two
-- values, and the first is not a function, it will look for
-- a definition of `jux` on both of them, if found, it will expand
-- (n m) into (jux n m), if not it will throw an error (e.g.
-- "juxtapostion not defined for `T' on `U'").

-- now
3 2 == 6

a : Nat
a = 3

b : Int
b = -3

assert <| a b == -9

(0.5)a == 1.5
