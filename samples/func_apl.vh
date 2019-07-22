map (1 +) list
-- The same as
(map) ((1) +) (list)
-- which is the same as:
(((map) ((1) +)) (list))
-- because of currying, `a b c`  <=>  `((a b) c)`

-- Function application has highest binding, so:
a b c + 3
-- is the same as
(a b c) + 3