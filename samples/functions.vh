plus : Nat -> Nat -> Nat

postulate do:
    plus n 0 = n
    plus 0 n = n
    plus (succ n) m = succ (plus n m)