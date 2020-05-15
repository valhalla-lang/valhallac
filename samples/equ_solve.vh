-- Notationally powerful set comprhension requiers some
-- sort of eqation solver / symbol manipulation to be
-- computationally understood and useful.

-- e.g. finding the solutions to a quadratic is done as so:

[ x : Real => x^2 - 3*x - 4 == 0 ]
--> [ -1; 4 ]

-- x^2 + (1/2)x -3 = 0 has roots (x = -2) and (x = 1.5)
-- so,
[ x : Int => x^2 + 0.5*x == 3 ]
--> [ -2 ]

-- Likewise
[ x : Nat => x^2 + 0.5*x == 3 ]
--> []
-- or
--> Empty


-- Same for FizzBuzz

Multiples3 = [ 3*n => n <- Nat ]
Multiples5 = [ 5*n => n <- Nat ]

fizzbuzz : Nat -> String
fizzbuzz n = show n

fizzbuzz : Multiples3 -> String
fizzbuzz _ = "Fizz"

fizzbuzz : Multiples5 -> String
fizzbuzz _ = "Buzz"

fizzbuzz : Mutiples3 & Multipes5 -> String
fizzbuzz _ = "FizzBuzz"

image f [| n : Nat => 0 < n < 101 |]
-- or
image f (1..100)
-- or
map (1..100) fizzbuzz
-- or
(1..100).map fizzbuzz
-- or
1..100 |> map fizzbuzz

-- a.b.c == (a.b).c == c (b a)
-- a |> b |> c == (a |> b) |> c == c (b a)

-- s.t.
-- !infix operator precedenc associativity
!infix (.) 200 :left
!infix (|>) 50 :left

(.) : 'A -> ('A -> 'B) -> 'B
(|>) : 'A -> ('A -> 'B) -> 'B

-- 'X is a generic type. Last resort if overloaded.

arg . f = f arg
arg |> f = f arg   -- Different precedence.

