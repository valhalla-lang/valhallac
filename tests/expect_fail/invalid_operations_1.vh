f : Real -> Real -> Real

f a b = (1 + 1) * a + b  -- 2a + b

a : Nat
a = 3

( : ) b Int
b = 1 - 8  -- -7

c : Real
c = f (a + 1.0) (b - 0.0)

--* This should yield three errors *--

w : Beep -- Beep doesn't exits yet.
-- This is an error, it should be `<-` not `:`.
w = a + b : Int

2 = 3  -- Also an error.


