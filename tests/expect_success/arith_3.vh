f : Int -> Int
f n = n - 20

b : Nat
b = 12

a = 2 + f b * 4 / 6
-- `b` will get automatically cast to an int.
