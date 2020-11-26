!allow redefinition
-- Consider the C code:
{-
	#include <stdio.h>

	struct Prod {
		int x, y;
		char *name;
		double r;
	};

	int main(void)
	{
		struct Prod prod = { 3, 9, "John", -2.6f };
		printf("%d, %d, %s, %f\n",
			prod.x, prod.y, prod.name, prod.r);
		return 0;
	}
-}

-- How do we write this in Valhalla?

-- Product types are explicitly cartesian products.

Prod = Int * Int * String * Real
-- or
Prod = Int^2 * String * Real
-- Prod is a type (set) of the cartesian product of:
--   Two Ints, String and Real.

prod : Prod -- prod is an element of Prod (prod has type Prod).
prod = (3, 9,"John", -2.6)  -- way to initialise, by just assigning.

-- Now, to access elements of `prod`, we may index normally.
assert (prod[0] is 3)

-- If we want names, we can do that with functions:
x : Prod -> Int
x p = p[0]

-- Hence,
assert ((x prod) is 3)
-- or using the `.` composition operator.
assert (prod.x is 3)

-- We can define `x` function like this too:
x p = index 0 p
-- since `p[0]` is the same as `index 0 p`
-- so, since this is Curried, we can also write:
x = index 0

-- So, lets define the rest.
x    : Prod -> Int
y    : Prod -> Int
name : Prod -> String
r    : Prod -> Real

x    = index 0
y    = index 1
name = index 2
r    = index 3

-- If we don't include the type signatures here, then
-- the functions (x, y, name, etc) would be defined generally
-- for any type overloading the `index` function.

IO::put "#{prod.x}, #{prod.y}, #{prod.name} #{prod.r}"

-- And we've done the same as the C program, but
-- all this can be automated by macros, and we were
-- overly verbose here, for the sake of being educational.



-- More examples:

-- 2 dimensional vector type.
Vec2D = Real^2   -- or, Real * Real
x : Vec2D -> Real
y : Vec2D -> Real
x = index 0
y = index 1


v : Vec2D
v = (1, 2)  -- Create a Vec2D.

(n, m) : Real^2
(n, m) : Vec2D -- these two type signatures mean the same thing,
-- Neither type signature are necessary, because we have type inference.
(n, m) = v  -- pattern matching assignment.

assert (v.x == n)
assert (v.y == m)


show : Vec2D -> String -- overload `show'.
show v = "(#{v.x}, #{v.y})"

IO::puts v -- this calls `show' impicitly,
           --  (within the definition of `puts').


