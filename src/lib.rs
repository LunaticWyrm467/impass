//! # impass
//!
//! `impass` is a utility crate that provides the `fatal!` macro for handling
//! unrecoverable errors in a declarative and safe way. It is built on the
//! philosophy that some errors are not meant to be handled gracefully, but
//! rather signal a critical failure in program logic or state.
//!
//! The `fatal!` macro provides a clear, concise, and idiomatic way to express
//! this "fail-fast" intent. It is analogous to the `assert!` macro, where a
//! failed assertion indicates a bug that should crash the program.
//!
//! ### Why `fatal!`?
//!
//! The idiomatic way to handle a `Result` that is expected to always be `Ok` is
//! to use `.unwrap()` or `.expect()`. While this works, it can become verbose
//! when chaining multiple fallible operations.
//!
//! This macro provides a single, cohesive block for this "fail-fast" behavior,
//! making your code more readable and your intent explicit.
//!
//! ### Quick Start
//! You can simply use the macro to wrap a block of code where you expect all
//! operations to succeed:
//!
//! ```rust,should_panic
//! use thiserror::Error;
//! use impass::fatal;
//! 
//! // Declare an error type for demonstration purposes.
//! #[derive(Error, Debug)]
//! pub enum MyError {
//!     #[error("This operation failed")]
//!     OperationFailed
//! }
//!
//! // A fallible function for demonstration.
//! fn might_fail(value: i32) -> Result<i32, MyError> {
//!     if value < 10 {
//!         Err(MyError::OperationFailed)
//!     } else {
//!         Ok(value * 2)
//!     }
//! }
//!
//! fn main() {
//!     // Usage with a custom message using the `#![reason]` attribute.
//!     let final_value = fatal! {
//!         #![reason("The crucial calculation failed. Program state is unrecoverable")]
//!         let value = might_fail(15)?;
//!         Ok(value)
//!     };
//!
//!     println!("Execution succeeded. Final value: {}", final_value);
//!
//!     // An example that will cause a panic.
//!     fatal! {
//!         let result = might_fail(5)?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ---

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse_macro_input;
use syn::parse::{Parse, ParseStream};
use syn::parse2;


/// A declarative macro for handling critical, unrecoverable errors.
///
/// The `fatal!` macro wraps a block of code, allowing for a clean and
/// expressive "fail-fast" pattern. It provides a highly visible boundary for
/// code that is expected to be infallible, and it signals a critical bug if a
/// `Result::Err` is returned.
///
/// This macro is a more ergonomic and readable alternative to
/// ```rust,ignore
/// (|| -> Result<_, anyhow::Error> { ... })().unwrap()
/// ```
///
/// ### Behavior
///
/// 1.  **Allows `?` Operator:** The macro wraps the provided block in a
///     closure, enabling the use of the `?` operator for seamless error
///     propagation.
/// 2.  **Detailed Panic:** If the block returns an `Err`, the macro catches the
///     error, prints a detailed error message which includes any context. For
///     this reason it is recommended to use the `context` function from
///     `anyhow`.
/// 3.  **Returns a Value:** The macro returns the inner value of the `Ok`
///     variant, allowing it to be used in assignments.
///
/// ### Usage
///
/// The macro accepts a code block that must return a `Result` type.
///
/// ```rust,should_panic
/// use thiserror::Error;
/// use impass::fatal;
/// 
/// // Declare an error type for demonstration purposes.
/// #[derive(Error, Debug)]
/// pub enum MyError {
///     #[error("This operation failed")]
///     OperationFailed
/// }
///
/// // A fallible function for demonstration.
/// fn might_fail(value: i32) -> Result<i32, MyError> {
///     if value < 10 {
///         Err(MyError::OperationFailed)
///     } else {
///         Ok(value * 2)
///     }
/// }
///
/// fn main() {
///     // Simple usage without a custom message.
///     let final_value = fatal! {
///         let value = might_fail(15)?;
///         Ok(value)
///     };
///     println!("Successfully computed: {}", final_value);
///
///     // Usage with a custom message using the `#![reason]` attribute.
///     fatal! {
///         #![reason("Failed to initialize system-critical component.")]
///         let result = might_fail(5)?; // This will cause a panic.
///         Ok(())
///     }
/// }
/// ```
#[proc_macro]
pub fn fatal(input: TokenStream) -> TokenStream {

    // Parse the input into a `FatalBlock` struct.
    let FatalBlock {
        stmts,
        reason_message,
    } = parse_macro_input!(input as FatalBlock);

    // The block is placed inside a closure that returns a `Result`.
    let result: TokenStream2 = quote! {
        (|| -> std::result::Result<_, anyhow::Error> {
            #(#stmts)*
        })()
    };

    // We generate an unwrap_or_else that formats the anyhow error and panics.
    let generated_code: TokenStream2 = if let Some(msg) = reason_message {
        quote! {
            #result.unwrap_or_else(|e| {
                panic!("\n{:?}", e.context(#msg));
            })
        }
    } else {
        quote! {
            #result.unwrap_or_else(|e| {
                panic!("\n{:?}", e.context("An unrecoverable error occurred"));
            })
        }
    };

    generated_code.into()
}


/// Handles the parsing of the `fatal!` macro's input.
struct FatalBlock {
    stmts:          Vec<syn::Stmt>,
    reason_message: Option<syn::LitStr>,
}

impl Parse for FatalBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        // Find the `reason` attribute, if it exists.
        let mut reason_message: Option<syn::LitStr> = None;
        let     attribs:        Vec<syn::Attribute> = input.call(syn::Attribute::parse_inner)?;

        for attr in attribs {
            if attr.path.is_ident("reason") {
                if let Ok(value) = attr.parse_args::<syn::LitStr>() {
                    reason_message = Some(value);
                }
            }
        }

        // Return the parsed block.
        Ok(FatalBlock {
            stmts: input.call(syn::Block::parse_within)?,
            reason_message,
        })
    }
}

/// An attribute macro that wraps a function's body in the `fatal!` macro.
///
/// This macro allows you to specify an optional reason for the fatal error
/// using the attribute argument.
///
/// ### Example
/// ```rust
/// use thiserror::Error;
/// use impass::{fatal, fatal_fn};
/// 
/// // Declare an error type for demonstration purposes.
/// #[derive(Error, Debug)]
/// pub enum MyError {
///     #[error("This operation failed")]
///     OperationFailed
/// }
///
/// // A fallible function for demonstration.
/// fn might_fail(value: i32) -> Result<i32, MyError> {
///     if value < 10 {
///         Err(MyError::OperationFailed)
///     } else {
///         Ok(value * 2)
///     }
/// }
/// 
/// // Using the `fatal_fn` macro to wrap a function.
/// #[fatal_fn(reason = "Critical failure in function execution")]
/// fn example_function() -> i32 {
///     let value = might_fail(15)?;
///     Ok(value)
/// }
/// 
/// // The above function is equivalent to:
/// fn example_function_actual() -> i32 {
///     fatal! {
///         #![reason("Critical failure in function execution")]
///         let value = might_fail(15)?;
///         Ok(value)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn fatal_fn(args: TokenStream, input: TokenStream) -> TokenStream {

    // Parse the attribute arguments and the function.
    let     args:     syn::AttributeArgs = parse_macro_input!(args as syn::AttributeArgs);
    let mut input_fn: syn::ItemFn        = parse_macro_input!(input as syn::ItemFn);

    // Extract the reason argument, if provided.
    let reason_message = args.iter().find_map(|arg| {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(meta)) = arg {
            if meta.path.is_ident("reason") {
                if let syn::Lit::Str(lit_str) = &meta.lit {
                    return Some(lit_str.value());
                }
            }
        }
        None
    });

    // Get the original function body.
    let original_body: &[syn::Stmt] = &input_fn.block.stmts;

    // Construct the new body wrapped in the `fatal!` macro.
    let new_body: TokenStream2 = if let Some(reason) = reason_message {
        quote! {
            impass::fatal! {
                #![reason(#reason)]
                #(#original_body)*
            }
        }
    } else {
        quote! {
            impass::fatal! {
                #(#original_body)*
            }
        }
    };

    // Replace the function's body with the new wrapped body.
    input_fn.block = parse2(quote! { { #new_body } })
        .expect("Failed to parse the new body into a block.");

    // Return the modified function as a TokenStream.
    TokenStream::from(input_fn.to_token_stream())
}