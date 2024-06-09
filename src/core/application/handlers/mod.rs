use serde::{de::DeserializeOwned, Serialize};
use validator::{ValidationErrors, Validate};

/// This Rust code defines a trait named `RequestHandler` with a generic type `T`. Within the trait
/// definition:
/// - `type Input` is associated with the trait and represents a type that must implement the
/// `Validate`, `Clone`, `Default`, `Serialize`, and `DeserializeOwned` traits.
/// - The `validate` method takes a value of type `Self::Input` and returns a `Result<(), String>`,
/// where `()` indicates success and `String` represents an error message.
/// - The `handler` method takes a value of type `T` and returns a `Result<(), String>`, where `()`
/// indicates success and `String` represents an error message.
pub trait RequestHandler {
    type Input;
    type Output;
    type Validation: Clone + Serialize + Validate + DeserializeOwned;

    fn validate(&self, value: Self::Validation) -> Result<(), ValidationErrors>;

    fn handler(
        &self,
        request: Self::Input,
    ) -> impl std::future::Future<Output = Result<Self::Output, String>> + Send;
}
