[ (:puts, puts) ] | io = import :IO

sum : Number^'N -> Number
sum () = 0
sum (x,xs) = x + sum xs

product : Number^'N -> Number
product () = 1
product (x,xs) = x * product xs

pi n = map ((2 /) <> f) (1..n) |> product |> (* 2)
    where: f : Nat -> Real
           f 0 = 0
           f k = sqrt (2 + f k)

puts <| pi 20

