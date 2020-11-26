module Hello where:
	greet name = "Hello, :{name}!"

bye = module Bye where:
	greet name = "Bye, :{name}!"

Hello::greet "Sam"  --> "Hello, Sam!"
Bye::greet "Sam"  -- Error! No such module exists.

bye::greet "Sam"  --> "Bye, Sam!"
-- ^^ That works.

assert (bye <- Module) -- Type of `Module'.

import :IO
IO::puts "Hi."
io::puts "Hi." -- Error! `io' does not exist.
-- OR:
io = import :IO
io::puts "Hi."
IO::puts "Hi." -- Error! `IO' does not exist.

-- Syntax is general:
let:
	greet name = "Hi, :{name}!"
in module Hi

Hi::greet "Rostislav"  --> "Hi, Rostislav!"

--

member Hello :greet == Hello::greet
IO.memeber :greet == Hello::greet
-- (::) is syntactic sugar for this.
-- i.e.
member m e = m::(eval! e)
m::e = member m :(`e)
