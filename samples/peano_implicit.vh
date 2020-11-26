import :Prelude
io = import :IO

-- We don't need to define Zero and Succ, they can exist
-- as application of functions or intances of themselves,
-- and not mean anything more, they perform no computation,
-- unlike normal functions.

N = [ Zero ] | [ Succ n => n <- N ] where:
	Zero : N
	Succ : N -> N

(+) : N -> N -> N
n + Zero = n
n + (Succ m) = Succ (n + m)

(*) : N -> N -> N
n * Zero = Zero
n * (Succ m) = n + n * m

one = Succ Zero
two = Succ one
three = two + one

-- Should show: (Succ (Succ (Succ (Succ (Succ Zero)))))
io::puts <| three + two
