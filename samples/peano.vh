-- ZF Formulation.
Succ a = a | [ a ]
Zero = []
-- one = [ [] ]         = [ 0 ]
-- two = [ []; [ [] ] ] = [ 0; 1 ]
-- etc.

Peano = Tag [ Zero ] | [ Succ n => n <- Peano ]

-- Succ n forany n <- Peano, is also a Peano
assert <| all [ Succ n <- Peano => n <- Peano ]

(+) : Peano -> Peano -> Peano
Zero + n = n
(Succ n) + m = Succ (n + m)

(*) : Peano -> Peano -> Peano
n * Zero = Zero
n * (Succ m) = n + n * m

-- Printing Example:
show : Peano -> String
show Zero = "0"
show (Succ n) = (show n) + "+1"

one : Peano
one = succ zero

two : Peano
two = one + one

three : Peano
three = one + two

IO::puts three
	-- "0+1+1+1"


-- Indexing the Peano set is essentialy converting
-- between internal Nat to Peano.
index : Nat -> [S] -> S where S <: Peano
index 0 _ = Zero
index (n + 1) _ = Succ (index n)
--index n _ = Succ (index (n - 1))
--index n _ = n - 1 |> index |> Succ

assert (Peano[0] == zero)
assert (index 0 Peano == zero)
assert (Peano[1] == one)
assert (Peano[2] == one + one)
assert (Peano[3] == three)

to_nat : Peano -> Nat
to_nat Zero = 0
to_nat (Succ n) = to_nat n + 1

assert (Peano == [ Peano[n] => n <- Nat ])
assert (Nat == [ to_nat p => p <- Peano ])

