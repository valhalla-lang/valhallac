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

-- a = n |-> (m |-> n - 2*m)
--           |_____________|
--                  |
--               func: `__a_final`
--     |_____________________|
--                |
--             func: `__a__0`
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
  [ Terminator:  "\n"                                              (9, 35):1 ],
  [ Terminator:  "\n"                                              (10, 31):1 ],
  [ Terminator:  "\n"                                              (11, 20):1 ],
  [ Terminator:  "\n"                                              (12, 30):1 ],
  [ Terminator:  "\n"                                              (13, 32):1 ],
  [ Terminator:  "\n"                                              (14, 14):1 ],
  [ End-Of-File: "\u{0}"                                           (15, 17):1 ] ]
```

---

From the token-stream, an abstract syntax tree (AST) is generated:
```hs
[|
  %file{ :filename samples/functions.vh }
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
|]
```

---

And the AST is compiled to bytecode.
The following is a disassembly of the generated bytecode:

```clojure
__a_final:
  |[meta]:
  |  stack-depth: 2
  |    file-name: samples/functions.vh
  |====Constants===============
  |   0 |  2             (Nat)
  |====Locals==================
  |   0 |  m
  |====Globals=================
  |   0 |  n
  |====Bytecodes===============
  | (0xfe):SET_LINE           2 (0x0002)
  | (0x02):PUSH_LOCAL         0 (0x0000)
  | (0x01):PUSH_CONST         0 (0x0000)
  | (0x31):N_MUL
  | (0x03):PUSH_SUPER         0 (0x0000)
  | (0x30):U_SUB
  | (0x0d):YIELD

__a_0:
  |[meta]:
  |  stack-depth: 2
  |    file-name: samples/functions.vh
  |====Constants===============
  |   0 |  __a_final     (Code)
  |   1 |  :__a_final    (Sym)
  |====Locals==================
  |   0 |  n
  |====Globals=================
  |====Bytecodes===============
  | (0x01):PUSH_CONST         0 (0x0000)
  | (0x01):PUSH_CONST         1 (0x0001)
  | (0x0c):MAKE_FUNC
  | (0x0d):YIELD

<main>:
  |[meta]:
  |  stack-depth: 3
  |    file-name: samples/functions.vh
  |====Constants===============
  |   0 |  __a_0         (Code)
  |   1 |  :a            (Sym)
  |   2 |  2             (Nat)
  |   3 |  1             (Nat)
  |====Locals==================
  |   0 |  a
  |====Globals=================
  |====Bytecodes===============
  | (0xfe):SET_LINE           2 (0x0002)
  | (0x01):PUSH_CONST         0 (0x0000)
  | (0x01):PUSH_CONST         1 (0x0001)
  | (0x0c):MAKE_FUNC
  | (0x05):STORE_LOCAL        0 (0x0000)
  | (0xfe):SET_LINE           4 (0x0004)
  | (0x01):PUSH_CONST         2 (0x0002)
  | (0x01):PUSH_CONST         3 (0x0003)
  | (0x02):PUSH_LOCAL         0 (0x0000)
  | (0x09):CALL_1
  | (0x09):CALL_1
  | (0x0d):YIELD
```

---

This bytecode you see visually represented gets marshalled into a string
of bytes according to a format (you may view it in `/src/compiler/marshal.rs`).

The marshalled code gets stored in a file, and the VM may execute it.
