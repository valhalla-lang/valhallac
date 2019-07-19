import :Prelude
import :puts from :IO as :put

open Prelude

div_by_three = [n : Nat => n mod 3 is 0]
div_by_five  = [n : Nat => n mod 5 is 0]

fizz : div_by_three -> String
fizz n = "Fizz"

fizz : div_by_five  -> String
fizz n = "Buzz"

fizz : div_by_five | div_by_three -> String
fizz n = "FizzBuzz"

# The order here doesn't matter, that's because if
# a number is part of both those sets, and neither set
# is a subset of eachother, it will give
# the number to the function that has a set that
# is a superset of both. If that function didn't exist,
# it would just run the first function.
# This is quite a unique scenario.

map fizz 1..100 |> each puts


# Type declarations don't have to be directly over
# the implementation of the function. Only the order
# matters.

int? : Real -> Boolean
int? : Int  -> Boolean

int? x = False
int? x = True

# The function that it runs is the more specific one.
# (By specific I mean the function that has a domain
#  that is a subset of the other functions domain.)


# Another way:

is_int? : Real -> Boolean
is_int x = x <- Int

# <-  is read as "is element of ...?".  It returns a bool.


