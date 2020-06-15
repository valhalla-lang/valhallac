-- Values can be automatically wrapped in functions
-- if the return type matches.

--*--
  * Examples
--*--

f : [] -> Nat
f = 3         -- same as,
--f = _ |-> 3 -- same as,
--f a = 3     -- same as,
--f _ = 3     -- same as,
--f () = 3

g : Nat -> Int
g = -4 -- same as,
--g = _ |-> { -4 }


a = 3  -- a : Int

b : 'A -> 'B
b = 3  -- b : [] -> Int

-- This works everywhere:

xs = [| 1; 2; 3 |]
ys = map 3 xs  -- same as,
--ys = map (_ |-> 3) xs
assert (ys == [| 3; 3; 3 |])
