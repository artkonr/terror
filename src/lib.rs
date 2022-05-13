use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use serde_derive::Serialize;

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};
use serde_json::{Number, Value};
#[cfg(feature = "err_id")]
use uuid::Uuid;

/// A buildable error object, which suits
/// most cases of error reporting for web
/// services.
///
/// ### Returns
/// At the minimum, reports the following data:
/// * HTTP status associated with the error;
/// * error message.
///
/// With additional input provided, may also
/// include the data like:
/// * shortened version of error message;
/// * specific error code;
/// * arbitrary error details;
/// * a reference to MDN documentation regarding
///   the HTTP status reported;
/// * various error tags, which may be convenient
///   for internal logging.
///
/// If feature `time` is enabled, also reports
/// the present timestamp the object was created at.
///
/// if feature `err_id` is enabled, also assigns
/// a UUID to the error body.
///
/// ### Building
///
/// The object _may_ be constructed manually, as
/// all object fields are `pub`. However, for the
/// sake of convenience, object construction may
/// be done via [builder](Builder).
#[derive(Debug, Serialize)]
pub struct Terror {

    pub status: u16,
    pub message: String,
    pub short_message: Option<String>,
    pub error_code: Option<String>,
    pub details: HashMap<String, Value>,
    pub reference: Option<String>,

    #[cfg(feature = "time")]
    pub timestamp: DateTime<Utc>,

    #[cfg(feature = "err_id")]
    pub id: Uuid,

    #[serde(skip_serializing)]
    pub tags: Vec<String>

}

impl fmt::Display for Terror {

    /// Formats the object into a nicely
    /// looking log line. At the minimum,
    /// prints the HTTP status along with
    /// the error message.
    ///
    /// ### Examples
    ///
    /// ```text
    /// (409) :: failed to persist entity due to version conflict
    /// ```
    ///
    /// And this example demonstrates the
    /// use of log tags, which may be added
    /// for debugging purposes.
    /// ```text
    /// [op:persist ctx:none] (409) :: failed to persist entity due to version conflict
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut tags = String::new();

        if !self.tags.is_empty() {
            for tag in &self.tags {
                if tags.is_empty() {
                    tags.push('[');
                } else {
                    tags.push(' ');
                }

                tags.push_str(tag.as_str())
            }
            tags.push(' ');
        }

        write!(f, "{}({}) :: {}", tags, self.status, self.message)
    }
}

impl Terror {

    /// Constructs a new builder with the
    /// minimal data provided explicitly.
    pub fn new(status: u16, msg: String) -> Builder {
        Builder {
            status,
            message: msg,
            short_message: None,
            error_code: None,
            details: HashMap::new(),
            reference: None,

            #[cfg(feature = "time")]
            timestamp: Utc::now(),

            #[cfg(feature = "err_id")]
            id: Uuid::new_v4(),

            tags: Vec::new()
        }
    }

    /// Constructs a new builder from any
    /// [Error] subtype and assumes HTTP
    /// status of `500 Internal Server Error`.
    pub fn from_error<T: Error>(err: T) -> Builder {
        Terror::new(500, format!("{}", err))
    }


}

/// A builder for [Terror]. Intended
/// for one-time used, consumed after
/// calling [Builder::build].
pub struct Builder {

    status: u16,
    message: String,
    short_message: Option<String>,
    error_code: Option<String>,
    details: HashMap<String, Value>,
    reference: Option<String>,

    #[cfg(feature = "time")]
    timestamp: DateTime<Utc>,

    #[cfg(feature = "err_id")]
    id: Uuid,

    tags: Vec<String>

}

impl Builder {
    
    /// Adds a short error message.
    pub fn short_message(mut self, msg: String) -> Builder {
        self.short_message = Some(msg);
        self
    }

    /// Adds an error code.
    pub fn error_code(mut self, code: String) -> Builder {
        self.error_code = Some(code);
        self
    }

    /// Adds a text detail.
    ///
    /// ### Examples
    ///
    /// For instance,
    /// ```rust
    /// use terror::{Builder, Terror};
    /// let built = Terror::new(500, String::from("generic error"))
    ///     .add_text_detail(String::from("object_name"), String::from("server"))
    ///     .build();
    /// ```
    ///
    /// ... may be rendered into a JSON like below:
    ///
    /// ```json
    /// {
    ///     "status" : 500,
    ///     "message" : "generic error",
    ///     "details" : {
    ///         "object_name" : "server"
    ///     }
    /// }
    /// ```
    pub fn add_text_detail(mut self, name: String, value: String) -> Builder {
        self.details.insert(
            name,
            Value::String(value)
        );
        self
    }

    /// Adds a numeric detail.
    ///
    /// ### Examples
    ///
    /// For instance,
    /// ```rust
    /// use terror::{Builder, Terror};
    /// let built = Terror::new(500, String::from("generic error"))
    ///     .add_int_detail(String::from("object_id"), 922i64)
    ///     .build();
    /// ```
    ///
    /// ... may be rendered into a JSON like below:
    ///
    /// ```json
    /// {
    ///     "status" : 500,
    ///     "message" : "generic error",
    ///     "details" : {
    ///         "object_id" : 922
    ///     }
    /// }
    /// ```
    pub fn add_int_detail(mut self, name: String, value: i64) -> Builder {
        self.details.insert(
            name,
            Value::Number(Number::from(value))
        );
        self
    }

    /// Adds a boolean detail.
    ///
    /// ### Examples
    ///
    /// For instance,
    /// ```rust
    /// use terror::{Builder, Terror};
    /// let built = Terror::new(500, String::from("generic error"))
    ///     .add_bool_detail(String::from("object_up"), false)
    ///     .build();
    /// ```
    ///
    /// ... may be rendered into a JSON like below:
    ///
    /// ```json
    /// {
    ///     "status" : 500,
    ///     "message" : "generic error",
    ///     "details" : {
    ///         "object_up" : false
    ///     }
    /// }
    /// ```
    pub fn add_bool_detail(mut self, name: String, value: bool) -> Builder {
        self.details.insert(
            name,
            Value::Bool(value)
        );
        self
    }

    /// Adds an arbitrary object as detail. Requires
    /// to be passed as a pointer.
    ///
    /// ### Examples
    ///
    /// For instance,
    /// ```rust
    /// use terror::{Builder, Terror};
    /// use serde_json::{json, Value};
    ///
    /// let built = Terror::new(500, String::from("generic error"))
    ///     .add_value_detail(
    ///         String::from("object"),
    ///         Value::from(json!({
    ///             "id" : 94i32,
    ///             "name" : "server"
    ///         }))
    ///     )
    ///     .build();
    /// ```
    ///
    /// ... may be rendered into a JSON like below:
    ///
    /// ```json
    /// {
    ///     "status" : 500,
    ///     "message" : "generic error",
    ///     "details" : {
    ///         "object" : {
    ///             "id" : 94,
    ///             "name" : "server"
    ///         }
    ///     }
    /// }
    /// ```
    pub fn add_value_detail(mut self, name: String, value: Value) -> Builder {
        self.details.insert(
            name,
            value
        );
        self
    }

    /// Instructs the builder to attach
    /// a reference to MDN page explaining
    /// the HTTP status code.
    pub fn reference(mut self) -> Builder {
        let url = format!("{}/{}", MDN_STATUS_REF, self.status);
        self.reference = Some(url);
        self
    }

    /// Adds a log tag for debugging purposes.
    pub fn add_tag(mut self, tag: String) -> Builder {
        self.tags.push(tag);
        self
    }

    /// Concludes the configuration and produces
    /// a new [Terror] instance with all
    /// ownerships transferred, thus fully consuming
    /// `self`.
    pub fn build(self) -> Terror {
        Terror {
            status: self.status,
            message: self.message.clone(),
            short_message: self.short_message.clone(),
            error_code: self.error_code.clone(),
            details:self.details,
            reference: self.reference.clone(),
            tags: self.tags.clone(),

            #[cfg(feature = "time")]
            timestamp: self.timestamp.clone(),

            #[cfg(feature = "err_id")]
            id: self.id.clone(),
        }
    }

}

const MDN_STATUS_REF: &str = "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status";

#[cfg(test)]
mod no_feature_test {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use serde_json::{json, Value};
    use crate::{Builder, MDN_STATUS_REF, Terror};

    #[test]
    fn build_with_explicit_status() {
        let msg = "generic error";
        let built = Terror::new(
            404,
            String::from(msg)
        )
            .build();

        assert_eq!(404, built.status);
        assert_eq!(
            String::from(msg),
            built.message
        );
    }

    #[test]
    fn build_from_error() {
        let error = TestError;
        let built = Terror::from_error(error)
            .build();

        assert_eq!(500, built.status);
        assert_eq!(
            String::from("generic error"),
            built.message
        )
    }

    #[test]
    fn build_no_short_message_set() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .build();

        assert!(built.short_message.is_none())
    }

    #[test]
    fn build_short_message_set() {
        let short_message = "generic";
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .short_message(String::from(short_message))
            .build();

        assert!(built.short_message.is_some());
        assert_eq!(
            String::from(short_message),
            built.short_message.unwrap()
        );
    }

    #[test]
    fn build_no_error_code_set() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .build();

        assert!(built.error_code.is_none())
    }

    #[test]
    fn build_error_code_set() {
        let error_code = "generic.failure";
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .error_code(String::from(error_code))
            .build();

        assert!(built.error_code.is_some());
        assert_eq!(
            String::from(error_code),
            built.error_code.unwrap()
        );
    }

    #[test]
    fn build_no_reference_set() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .build();

        assert!(built.reference.is_none())
    }

    #[test]
    fn build_reference_set() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .reference()
            .build();

        assert!(built.reference.is_some());
        assert_eq!(
            format!("{}/{}", MDN_STATUS_REF, 404),
            built.reference.unwrap()
        );
    }

    #[test]
    fn build_no_details() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .build();

        assert!(built.details.is_empty());
    }

    #[test]
    fn build_with_string_detail() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .add_text_detail(
                String::from("key"),
                String::from("val")
            )
            .build();

        assert!(!built.details.is_empty());
        assert_eq!(1, built.details.len());
        assert!(built.details.get("key").is_some());
        assert_eq!(
            Value::String(String::from("val")),
            *built.details.get("key")
                .unwrap()
        );
    }

    #[test]
    fn build_with_number_detail() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .add_int_detail(
                String::from("key"),
                53i64
            )
            .build();

        assert!(!built.details.is_empty());
        assert_eq!(1, built.details.len());
        assert!(built.details.get("key").is_some());
        assert_eq!(
            Value::from(53i64),
            *built.details.get("key")
                .unwrap()
        );
    }

    #[test]
    fn build_with_bool_detail() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .add_bool_detail(
                String::from("key"),
                true
            )
            .build();

        assert!(!built.details.is_empty());
        assert_eq!(1, built.details.len());
        assert!(built.details.get("key").is_some());
        assert_eq!(
            Value::Bool(true),
            *built.details.get("key")
                .unwrap()
        );
    }

    #[test]
    fn build_with_struct_detail() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .add_value_detail(
                String::from("key"),
                Value::from(json!({
                    "id" : 25,
                    "name" : "server"
                }))
            )
            .build();

        assert!(!built.details.is_empty());
        assert_eq!(1, built.details.len());
        assert!(built.details.get("key").is_some());
    }

    #[test]
    fn build_with_several_details() {
        let built = Terror::new(
            404,
            String::from("generic error")
        )
            .add_text_detail(
                String::from("str"),
                String::from("val")
            )
            .add_int_detail(
                String::from("num"),
                53i64
            )
            .add_bool_detail(
                String::from("flg"),
                true
            )
            .build();


        assert!(!built.details.is_empty());
        assert_eq!(3, built.details.len());
        assert!(built.details.contains_key("str"));
        assert!(built.details.contains_key("num"));
        assert!(built.details.contains_key("flg"));
    }

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "generic error")
        }
    }

    impl Error for TestError {}

}

#[cfg(all(test, feature = "err_id", feature = "time"))]
mod with_features_test {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use chrono::{Utc};
    use crate::{Builder, Terror};

    #[test]
    fn build_with_explicit_status() {
        let built = Terror::new(
            404,
            String::from("generic error")
        );

        assert_eq!(4, built.id.get_version_num());
        assert_eq!(
            Utc::now().date(),
            built.timestamp.date()
        )
    }

    #[test]
    fn build_from_error() {
        let error = TestError;
        let built = Terror::from_error(error)
            .build();

        assert_eq!(4, built.id.get_version_num());
        assert_eq!(
            Utc::now().date(),
            built.timestamp.date()
        )
    }

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "generic error")
        }
    }

    impl Error for TestError {}

}