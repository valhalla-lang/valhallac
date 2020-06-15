io = import :IO

name : String
name = "Sammy"

io::puts "Hello, ${name}!"
	-- expands to: `"Hello, " + name.to_string + "!"`
