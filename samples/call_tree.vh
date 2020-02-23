A <> B
(<>) A B
((<>) A) B

--      CALL
--      /  \
--     /    \
--   CALL    B
--   /  \
--  /    \
-- <>     A


-- say we have the function definition:

f : A -> B -> C
f a b = c where c = a + b

-- ...is really saying...

(((:) f) (((->) A) (((->) B) C)))
(((=) ((f a) b)) (((where) c) (((=) c) (((+) a) b))))

-- which is...

--      CALL
--     /    \
--   CALL    \
--  /    \    \
-- :      f    \
--            CALL
--           /    \
--         CALL    \
--        /    \    \
--       ->     A    \
--                  CALL
--                 /    \
--               CALL    C
--              /    \
--             ->     B
--
--       CALL
--      /    \
--    CALL    \
--   /    \    \
--  =    CALL   \
--      /    \   \
--    CALL    b   \
--   /   \        CALL
--  f     a      /    \
--             CALL    \
--            /    \    \
--         where    c    \
--                      CALL
--                     /    \
--                   CALL    \
--                  /    \    \
--                 =      c    \
--                             CALL
--                            /    \
--                          CALL    b
--                         /    \
--                        +      a
