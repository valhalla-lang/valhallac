-- What does the colon mean?
:sym        -- Symbol
:(x + y)    -- Quoted/Symbolic expression
f : A -> B  -- Type annotation
do: x y z   -- Indented code block
    a b c

-- Note:

--where:
--let:
--in:
--for:
-- etc.

-- Are just short for:

-- where do:
-- let do:
-- in do:
-- for do:

-- e.g.

x where:
	y = 3
	x = y + 4

-- same as,

x where: y = 3
         x = y + 4

-- same as,

x where do:
	y = 3
	x = y + 4

-- same as,

x where do
	y = 3
	x = y + 4
end

-- same as,

x where {
	y = 3
	x = y + 4
}

-- same as,

x where y = 3, x = y + 4
