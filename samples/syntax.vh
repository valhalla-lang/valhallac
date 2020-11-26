if_then_else : Bool -> Code -> Code -> 'Value where Code = Quote 'Value
if_then_else condition consequence alternative
  = !eval <| piecewise { consequence, condition; alternative, otherwise }

syntax :(if #p then #c else #a) = if_then_else #p #c #a
--     ^^^ The expression to match   ^^^ what it evaluates to
-- Alt.
mixfix (if # then # else #) if_the_else 10
-- ^^^ ^^^                  ^^^         ^^ Mixfix precedence.
-- |   |                    |The function whose parameters are given by the #s.
-- |   |The expression to match.
-- |Alternative to {infix, prefix, postfix/suffix} when there are >2 paramters.

x = if 1 == 1
      then "Hi."
      else "Bye."

assert <| x == "Hi."

-- Maybe you'd want French syntax, you can define that.

syntax :(si #p alors #c sinon #a) = if_then_else #p #c #a
-- Alt.
mixfix (si#alors#sinon#) if_then_else 10

y = si 1 == 1
      alors "Bonjour."
      sinon "Au revoir."

affirmer = assert
affirmer <| y == "Bonjour."
