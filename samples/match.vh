--* Pattern matching into product types. *--

Product = Tagged Nat^2 * Int  -- Makes a constructor.

prod : Product
prod = [| 2; 7; -66 |]  -- Casts implicitly.
--prod = Product [| 2; 7; -66 |]

alpha : Boolean
alpha = match prod:
	[| 0; 0; i |] => i == 0,
	Product [| 2; n; _ |] => n == 2, -- Explicit, but not needed, type is know.
	[| _; _; i |] => i <- Nat | [ -1 ],
	[| n; m; i |] => do:
		n + m == -j where:
			j = i + m % 3


beta : Number
beta = match prod {
	[| _; _; i |] => {
		j = i + 2
		j * m where {
			k = 8 - i
			m = k ** 3
		}
	},
	[| _; n; _ |] => n * 8,
	_ => 42
}

-- Yes, these are messy on purpose.
