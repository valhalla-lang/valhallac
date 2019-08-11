# Compiler Example
This file contains a regularly updated code example.
The compiled code you will see is currently the best the compiler can do.
Obviously, there are many optimisations yet to come.

## Example 1
Given the source code:
```hs
a : Nat -> Nat -> Int
a n m = n - 2*m

a 1 2

-- a = n |-> (m |-> n + 2*m)
--           |_____________|
--                  |
--               func: `a__1`
--     |_____________________|
--                |
--             func: `a__0`
-- |__________________________|
--          |
--       func: a
```

---

The code is lexically analysed and generates the following tokens-stream:
```clojure
[ [ Identifier:  "a"                                               (1, 1):1 ],
  [ Operator:    ":"                                               (1, 3):1 ],
  [ Identifier:  "Nat"                                             (1, 5):3 ],
  [ Operator:    "->"                                              (1, 9):2 ],
  [ Identifier:  "Nat"                                             (1, 12):3 ],
  [ Operator:    "->"                                              (1, 16):2 ],
  [ Identifier:  "Int"                                             (1, 19):3 ],
  [ Terminator:  "\n"                                              (1, 22):1 ],
  [ Identifier:  "a"                                               (2, 1):1 ],
  [ Identifier:  "n"                                               (2, 3):1 ],
  [ Identifier:  "m"                                               (2, 5):1 ],
  [ Operator:    "="                                               (2, 7):1 ],
  [ Identifier:  "n"                                               (2, 9):1 ],
  [ Operator:    "-"                                               (2, 11):1 ],
  [ Numeric:     "2"                                               (2, 13):1 ],
  [ Operator:    "*"                                               (2, 14):1 ],
  [ Identifier:  "m"                                               (2, 15):1 ],
  [ Terminator:  "\n"                                              (2, 16):1 ],
  [ Terminator:  "\n"                                              (3, 1):1 ],
  [ Identifier:  "a"                                               (4, 1):1 ],
  [ Numeric:     "1"                                               (4, 3):1 ],
  [ Numeric:     "2"                                               (4, 5):1 ],
  [ Terminator:  "\n"                                              (4, 6):1 ],
  [ Terminator:  "\n"                                              (5, 1):1 ],
  [ Terminator:  "\n"                                              (6, 29):1 ],
  [ Terminator:  "\n"                                              (7, 29):1 ],
  [ Terminator:  "\n"                                              (8, 22):1 ],
  [ Terminator:  "\n"                                              (9, 30):1 ],
  [ Terminator:  "\n"                                              (10, 31):1 ],
  [ Terminator:  "\n"                                              (11, 20):1 ],
  [ Terminator:  "\n"                                              (12, 28):1 ],
  [ Terminator:  "\n"                                              (13, 32):1 ],
  [ Terminator:  "\n"                                              (14, 14):1 ],
  [ End-Of-File: "\u{0}"                                           (15, 17):1 ] ]
```

---

From the token-stream, an abstract syntax tree (AST) is generated:
```hs
[|
  %newfile{ :filename test.vh }
  %newline{ :line 1 }
  %call{
    :yield anything
    :callee (
      %call{
        :yield anything
        :callee (
          %ident{ :value ":"; :yield anything }
        )
        :operand [|
          %ident{ :value "a"; :yield (Nat ↦ (Nat 🡒 Int)) }
        |]
      }
    )
    :operand [|
      %call{
        :yield anything
        :callee (
          %call{
            :yield anything
            :callee (
              %ident{ :value "->"; :yield anything }
            )
            :operand [|
              %ident{ :value "Nat"; :yield Nat }
            |]
          }
        )
        :operand [|
          %call{
            :yield anything
            :callee (
              %call{
                :yield anything
                :callee (
                  %ident{ :value "->"; :yield anything }
                )
                :operand [|
                  %ident{ :value "Nat"; :yield Nat }
                |]
              }
            )
            :operand [|
              %ident{ :value "Int"; :yield Int }
            |]
          }
        |]
      }
    |]
  }
  %newline{ :line 2 }
  %call{
    :yield anything
    :callee (
      %call{
        :yield anything
        :callee (
          %ident{ :value "="; :yield anything }
        )
        :operand [|
          %call{
            :yield anything
            :callee (
              %call{
                :yield anything
                :callee (
                  %ident{ :value "a"; :yield anything }
                )
                :operand [|
                  %ident{ :value "n"; :yield anything }
                |]
              }
            )
            :operand [|
              %ident{ :value "m"; :yield anything }
            |]
          }
        |]
      }
    )
    :operand [|
      %call{
        :yield natural
        :callee (
          %call{
            :yield anything
            :callee (
              %ident{ :value "-"; :yield anything }
            )
            :operand [|
              %ident{ :value "n"; :yield natural }
            |]
          }
        )
        :operand [|
          %call{
            :yield anything
            :callee (
              %call{
                :yield anything
                :callee (
                  %ident{ :value "*"; :yield anything }
                )
                :operand [|
                  %num{ :value 2; :yield natural }
                |]
              }
            )
            :operand [|
              %ident{ :value "m"; :yield natural }
            |]
          }
        |]
      }
    |]
  }
  %newline{ :line 4 }
  %call{
    :yield integer
    :callee (
      %call{
        :yield (Nat ↦ Int)
        :callee (
          %ident{ :value "a"; :yield (Nat ↦ (Nat 🡒 Int)) }
        )
        :operand [|
          %num{ :value 1; :yield natural }
        |]
      }
    )
    :operand [|
      %num{ :value 2; :yield natural }
    |]
  }
  %newline{ :line 15 }
|]
```

---

And the AST is compiled to bytecode.
The following is a disassembly of the generated bytecode:

```lisp
a__1:
  | ===Constants===============
  |   0 |  2      => (Nat)    |
  | ===Locals==================
  |   0 |  m
  | ===Globals=================
  |   0 |  n
  | ===Bytecodes===============
  | (00000010):PUSH_LOCAL     0
  | (00000001):PUSH_CONST     0
  | (00110001):N_MUL
  | (00000011):PUSH_SUPER     0
  | (00110000):U_SUB

a__0:
  | ===Constants===============
  |   0 |  a__1   => (Block)  |
  |   1 |  :a__1  => (Sym)    |
  | ===Locals==================
  |   0 |  n
  | ===Globals=================
  | ===Bytecodes===============
  | (00000001):PUSH_CONST     0
  | (00000001):PUSH_CONST     1
  | (00001100):MAKE_FUNC

<main>:
  | ===Constants===============
  |   0 |  a__0   => (Block)  |
  |   1 |  :a     => (Sym)    |
  |   2 |  2      => (Nat)    |
  |   3 |  1      => (Nat)    |
  | ===Locals==================
  |   0 |  a
  | ===Globals=================
  | ===Bytecodes===============
  | (11111110):SET_LINE       2
  | (00000001):PUSH_CONST     0
  | (00000001):PUSH_CONST     1
  | (00001100):MAKE_FUNC
  | (00000101):STORE_LOCAL    0
  | (11111110):SET_LINE       4
  | (00000001):PUSH_CONST     2
  | (00000001):PUSH_CONST     3
  | (00000010):PUSH_LOCAL     0
  | (00001001):CALL_1
  | (00001001):CALL_1
  | (11111110):SET_LINE      15
```