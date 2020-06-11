import :Prelude

Vect : (n : Nat) -> ['A] -> 'Self where:
	Nil  :  Vect 0 'A
	Cons : 'A -> Vect n 'A -> Vect (n + 1) 'A

extend : Vect 'N 'A -> Vect 'M 'A -> Vect ('N + 'M) 'A
extend Nil         ys = ys
extend (Cons x xs) ys = Cons x (extend xs ys)

zip_with : ('A -> 'B -> 'C) -> Vect 'N 'A -> Vect 'N 'B -> Vect 'N 'C
zip_with f Nil _ = Nil
zip_with f (Cons x xs) (Cons y ys) = Cons (f x y) (zip_with f xs ys)

v : Vect 3 Int
v = Cons -7 (Cons -6 (Cons -5 Nil))

-- Or, since product type A^n is A
-- vector of type a with n elements

Vect : (n : Nat) -> ['A] -> ['A^n]
-- i.e.: Vect n a = a^n
-- so, we can also just say:
Vect : Nat -> ['A] -> 'Self
Vect n a = a^n
