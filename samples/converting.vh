Bool = [ :true
         :false ]

True = :true
False = :false

assert (1 == 1 <- Bool)
assert (1 == 2 <- Bool)
assert ((1 == 1) == :true)
assert ((1 == 2) == :false)

b : Int
b = :true -- Error.

from : Bool -> Nat
from :true  = 1
from :false = 0

b : Nat
b = :true -- Still an error.

-- Try again
b : Real
b = from :true  -- Works!

assert (b == 1)

-- What about other types?
from : Bool -> String  -- overloaded.
from :true  = "yes"
from :false = "no"

s : String
s = from :false

assert (s == "no")

-- What about:
c = from :true  -- Error!
-- We don't know what type we're converting to.

-- We have to use `as` instead.
c = :true as String
assert (c <- String)
assert (c == "yes")

-- This is how the `as` operator is defined:
(as) : 'A -> ['B] -> 'B
item as _ = from item  -- We know from the type signature that
					   -- `(from item) <- 'B`, so it is no
					   -- longer ambiguous!




