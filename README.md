# impass
`impass` is a utility crate that provides the `fatal!` macro for handling
unrecoverable errors in a declarative and safe way. It is built on the
philosophy that some errors are not meant to be handled gracefully, but rather
signal a critical failure in program logic or state.

The `fatal!` macro provides a clear, concise, and idiomatic way to express this
"fail-fast" intent. It is analogous to the `assert!` macro, where a failed
assertion indicates a bug that should crash the program.

## Why `fatal!`?
The idiomatic way to handle a Result that is expected to always be Ok is to use
.unwrap() or .expect(). While this works, it can become verbose when chaining
multiple fallible operations.

This macro provides a single, cohesive block for this "fail-fast" behavior,
making your code more readable and your intent explicit.

```rust
// A common but verbose pattern:
(|| -> Result<(), anyhow::Error> {
    let result = some_fallible_function()?;
    Ok(())
})().expect("Fatal error occurred.");
```

The `fatal!` macro replaces this with a single, clear, and more expressive
statement

## Quick Start
Simply Add `impass` and `anyhow` as dependencies in your `Cargo.toml`.

```toml
[dependencies]
impass = "X.X"
anyhow = "X.X"
```

You can then simply use the macro to wrap a block of code where you expect all
operations to succeed:

```rust
use impass::fatal;

fn main() {
    let final_value = fatal! {

        // You can declare the error message on panic like so.
        // This is completely optional.
        #![reason("This is a critical error!")]

        // Use the '?' operator freely inside this block.
        let value1 = fallible_function_a()?;
        let value2 = fallible_function_b(value1)?;
        
        // This must return a `Result`. The macro will unwrap it.
        Ok(value2 * 2)
    };
    
    println!("Execution succeeded. Final value: {}", final_value);
}
```

If an error occurs, the program will terminate with a report providing any
context from `anyhow`, helping you quickly identify the root cause of the bug.

This also provides a function attribute:
```rust
use impass::fatal_fn;

#[fatal_fn(reason = "Critical failure in function execution")] // `reason` is optional.
fn example_function() -> Result<i32, anyhow::Error> {
    let value = fallible_function_a()?;
    Ok(value)
}
```

**Note that any error types must implement `std::error::Error`.**