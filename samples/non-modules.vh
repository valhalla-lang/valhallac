-- `Real' is the set of all real numbers, not a moudle.
-- But, it can use similar syntax/functions as module,
-- to represent so-called 'members' of the real numbers.

-- The mathematical constant e = 2.718...
Real::e
-- which is the same as:
member Real :e
--
assert ((round_to (3 dp) Real::e) == 2.718)
assert ((round_to (3 sf) Real::pi) == 3.14)
--

-- Adding a number is done as such:
--member : [Real] -> Real
member Real :my_number = 1337

assert (Real::my_number == 1337)


-- But for non-pure-number constant, you should
-- just use a module.
-- e.g.

module Universal where:
	c = 2.998E8
	G = 6.67408E-11
	e = -1.602E-19
	mu_0 = 4 * Real::pi * 1E-7

schwarzschild_radius m = 2 * Universal::G * m / Universal::c^2


