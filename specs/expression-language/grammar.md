# Concrete grammar

The parser ignores ASCII whitespace. Lexing and parse errors report a zero-based byte position. The grammar below describes accepted input; `identifier` is additionally constrained by the shared identifier rules.

```text
expression  = sum ;
sum         = product , { ( "+" | "-" ) , product } ;
product     = power , { ( "*" | "/" ) , power } ;
power       = unary , [ "^" , power ] ;
unary       = [ "-" ] , primary ;
primary     = number | identifier | function | "(" , sum , ")" ;
function    = ( "exp" | "log" | "sin" | "cos" ) , "(" , sum , ")" ;
number      = ( digit | "." ) , { digit | "." } , [ ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ] ;
identifier  = ( ASCII-letter | "_" ) , { ASCII-letter | digit | "_" } ;
```

`^` is right-associative. `*`, `/`, `+`, and `-` associate left-to-right. Unary minus binds inside the left operand of power, so `-x^2` parses as `(-x)^2`; write `-(x^2)` for the conventional alternative. A bare `-` is unary only where `unary` is expected. Hyphens are permitted in core identifiers and binary bundle symbols, but the expression parser intentionally does not lex them as identifier characters because `-` is an operator; use underscore names in textual expressions.

Only the four listed function spellings are functions. Any other `name(` is an unknown-function error. A parsed numeric literal must be finite.
