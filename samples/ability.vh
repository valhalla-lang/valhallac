-- Most of the things defined here are already defined
-- in the Prelude.

id : 'A -> 'A
id x = x -- Identity function

show : String -> String
show = id  -- naturally.

-- Showable represents the
-- union of all types that implement
-- the show function on type 'A.
ability Showable on 'A {
	show : 'A -> String
}

-- Example
num_to_string : Showable & Number -> String
num_to_string n = show n
-- Converts only Numbers that are showable.
-- This gives us some early checking essentially.

stdout = IO::STDOUT ()
-- Same as `puts'/`put'
println : Showable -> ()
println s = [| s; "\n" |].map (|> show |> IO::write stdout)

-- partially applied composition operator :)

-- So if we try to print something that's
-- not showable, it won't accept it.
-- The error should tell you to consider implementing
-- `show' on that type.


