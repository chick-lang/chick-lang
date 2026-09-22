# chick Language Specification (Draft)

## 1. Overview & General Principles

chick is a programming language designed with the following core principles:

- **Entry Point**: The `main` function is **not required**. Code executes sequentially from top to bottom at the root scope.
- **Semicolons**: Mandatory in early implementations, with plans to make them optional in future syntax updates.

## 2. Functions

Functions utilize the `function` keyword for both standard declarations and inline/anonymous functional expressions.

### Named Functions
```chick
function add(a, b) {
    return a + b;
}
```

### Inline / Anonymous Functions
```chick
add = function(a, b) {
    return a + b;
};
```

## 3. Data Types

### Primitive Types

- Signed Integers: i8, i16, i32, i64, i128
- Unsigned Integers: u8, u16, u32, u64, u128
- Floating-point: f8, f16, f32, f64, f128
- Characters:
  - char: Multi-byte/Unicode support (e.g., Japanese text).
  - short_char: Single-byte / UTF-8 optimized character.

## 4. Variables, Control Flow & Arrays

### Variable Declarations & Control Flow
```chick
let A = 100;
let B: int;
B = 10;

// Range-based for loop
for i in 1..100 {
    print(A * i);
    println("{}", A * B * i);
}

// Conditional Statement
if a > b {
    a += b;
} else {
    a += b;
}
```

### Array Declarations (Fixed-size)

Right-hand side array initialization uses square brackets `[]`.

```chick
// Static array definition
int a[10] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3];

// Explicit type & sizing via let
let a[3] = [3, 1, 4];
let a[3]; // Zero-initialized by default

// Inferred size
let a = [3, 1, 4];
```

*Note: Dynamic arrays are handled by the Standard Library (STD).*

## 5. Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`, `**` (Exponentiation)
- Increment/Decrement: `++`, `--`
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Logical: `||`, `&&`

