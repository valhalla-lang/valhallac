import :Prelude
io = import :IO

-- Primes is an infinte list.
Primes = [ p : 2... => p mod n /= 0 => n <- 2..p ]

-- The (..) and (...) operators mean:
-- n..m = [ i : Int => i >= n and i < m ]
-- n... = [ i : Int => i >= n ]
-- ...m = [ i : Int => i < m ]


firs_20_primes = take 20 Primes

io::puts first_20_primes
