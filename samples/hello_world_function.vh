io = import :IO

let: greeting : String -> String
     greeting name = "Hello, %{name.capitalise}."
in module Hello

io::puts <| Hello::greeting place
  where: place : String
         place = "World"

