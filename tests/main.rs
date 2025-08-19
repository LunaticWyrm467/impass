use thiserror::Error;
use impass::{fatal, fatal_fn};


// Declare an error type for demonstration purposes.
#[derive(Error, Debug)]
pub enum MyError {
    #[error("This operation failed")]
    OperationFailed
}


// A dummy fallible function to test with.
fn might_fail(should_fail: bool) -> Result<i32, MyError> {
    if should_fail {
        Err(MyError::OperationFailed)
    } else {
        Ok(42)
    }
}


// This test uses the macro in a way that should succeed.
#[test]
fn test_fatal_success() {
    let result: i32 = fatal! {
        let value: i32 = might_fail(false)?;
        Ok(value)
    };
    assert_eq!(result, 42);
}

// This test uses the macro in a way that should fail and panic.
#[test]
#[should_panic]
#[fatal_fn]
fn test_fatal_panic() {
    let _: i32 = might_fail(true)?;
    Ok(())
}

// This test uses the macro in a way that should fail and panic with a reason.
#[test]
#[should_panic]
#[fatal_fn(reason = "Failed with a specific error")]
fn test_fatal_reason() {
    let _: i32 = might_fail(true)?;
    Ok(())
}