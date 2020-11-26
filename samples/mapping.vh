vec : Nat^3

vec = (1, 2, 3)
-- or
vec = 1,2,3,()
-- or
vec = 1,2,3,Empty


map (2 *) vec  -- (2, 4, 6)
map (n |-> 2*n - 1) vec  -- (1, 3, 5)

-- Image of A under f  <=>  map f A
assert <| image vec (2 *)                 == map (2 *) vec
assert <| image vec (n |-> 2*n - 1) vec   == map (n |-> 2*n - 1) vec
assert <| image vec ((+ 1) <> (2 *)) vec  == map ((+ 1) <> (2 *)) vec

-- Essentially:
map : ('A -> 'B) -> 'A^'N -> 'B^'N
map f () = ()
map f (x, xs) = (f x, map xs)

image : 'A^'N -> ('A -> 'B) -> 'B^'N
image = flip map

map f A    -- Read: Mapping of f on A
image A f  -- Read: Image if A under f


-- Of course, also works with sets (in the standard library):
set = [ 2; 1; 3 ]

image set (+ 10) == [ 13; 12; 11 ]
map (+ 10) set   == [ 11; 13; 12 ]

