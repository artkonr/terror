use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use serde_derive::{Serialize, Deserialize};

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};
use serde::Serialize;
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
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Terror {

    /// HTTP status code
    pub status: u16,

    /// Full error message
    pub message: String,

    /// Shortened error message; nullable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_message: Option<String>,

    /// Error code; nullable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    /// Arbitrary error details
    #[serde(default = "Terror::default_empty_map")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub details: HashMap<String, Value>,

    /// A reference to the MDN about the status code
    #[cfg(feature = "mdn")]
    pub reference: String,

    /// Error timestamp as captured by server
    #[cfg(feature = "time")]
    pub timestamp: DateTime<Utc>,

    /// Error ID
    #[cfg(feature = "err_id")]
    pub id: Uuid

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}) :: {}", self.status, self.message)
    }
}

impl Terror {

    /// Constructs a new builder with the
    /// minimal data provided explicitly.
    ///
    /// ### Examples
    ///
    /// For instance,
    /// ```rust
    /// use terror::{Terror, Builder};
    /// let built = Terror::new(429, String::from("some error"))
    ///     .build();
    /// ```
    ///
    /// Thanks to generic text parameter, the following also works:
    /// ```rust
    /// use terror::{Terror, Builder};
    /// let built = Terror::new(429, "some error")
    ///     .build();
    /// ```
    pub fn new<K: Into<String>>(status: u16, msg: K) -> Builder {
        let into: String = msg.into();
        Builder {
            status,
            message: into,
            short_message: None,
            error_code: None,
            details: HashMap::new(),

            #[cfg(feature = "mdn")]
            reference: format!("{}/{}", MDN_STATUS_REF, status),

            #[cfg(feature = "time")]
            timestamp: Utc::now(),

            #[cfg(feature = "err_id")]
            id: Uuid::new_v4(),
        }
    }

    /// Constructs a new builder from any
    /// [Error] subtype and assumes HTTP
    /// status of `500 Internal Server Error`.
    pub fn from_error<T: Error>(err: T) -> Builder {
        Terror::new(500, format!("{}", err))
    }

    /// Default handler for JSON map fields.
    fn default_empty_map() -> HashMap<String, Value> {
        HashMap::new()
    }

}

impl Default for Terror {
    fn default() -> Self {
        Terror::new(500, "generic error").build()
    }
}

/// A builder for [Terror]. Intended
/// for one-time use, consumed after
/// calling [Builder::build].
pub struct Builder {

    status: u16,
    message: String,
    short_message: Option<String>,
    error_code: Option<String>,
    details: HashMap<String, Value>,

    #[cfg(feature = "mdn")]
    reference: String,

    #[cfg(feature = "time")]
    timestamp: DateTime<Utc>,

    #[cfg(feature = "err_id")]
    id: Uuid

}

impl Builder {
    
    /// Adds a short error message.
    pub fn shorthand<K: Into<String>>(mut self, msg: K) -> Builder {
        let into: String = msg.into();
        self.short_message = Some(into);
        self
    }

    /// Adds an error code.
    pub fn error_code<K: Into<String>>(mut self, code: K) -> Builder {
        let into: String = code.into();
        self.error_code = Some(into);
        self
    }

    /// Adds a text detail.
    pub fn add_text_detail<K, V>(mut self,
                                 name: K,
                                 value: V) -> Builder
        where K: Into<String>,
              V: Into<String>
    {
        let name: String = name.into();
        let value: String = value.into();
        self.details.insert(name, Value::String(value));
        self
    }

    /// Adds a numeric detail.
    pub fn add_int_detail<K: Into<String>>(mut self,
                                           name: K,
                                           value: i64) -> Builder {
        let into: String = name.into();
        self.details.insert(into, Value::Number(Number::from(value)));
        self
    }

    /// Adds a boolean detail.
    pub fn add_bool_detail<K: Into<String>>(mut self,
                                            name: K,
                                            value: bool) -> Builder {
        let into: String = name.into();
        self.details.insert(into, Value::Bool(value));
        self
    }

    /// Adds an [Value] object as detail.
    pub fn add_value_detail<K: Into<String>>(mut self,
                                             name: K,
                                             value: Value) -> Builder {
        let into: String = name.into();
        self.details.insert(into, value);
        self
    }

    /// Adds a `null` object as detail.
    pub fn add_null_detail<K: Into<String>>(mut self, name: K) -> Builder {
        let into: String = name.into();
        self.details.insert(into, Value::Null);
        self
    }

    /// Adds a serialised struct detail from a
    /// provided [Serialize]-annotated object.
    ///
    /// ### Panics
    ///
    /// This method expects that `obj` parameter
    /// can be correctly serialised into JSON
    /// and panics if [serde_json::to_value]
    /// fails.
    pub fn add_struct_detail<K, S>(mut self,
                                   name: K,
                                   obj: S) -> Builder
        where K: Into<String>,
              S: Serialize + Debug
    {
        let into: String = name.into();
        let value = serde_json::to_value(obj)
            .expect(format!("failed to serialise: {:?}", obj).as_str());
        self.details.insert(into, value);
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
            details: self.details,
            reference: self.reference.clone(),

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
    use crate::{Builder, Terror};

    type R = anyhow::Result<()>;

    #[test]
    fn build_with_explicit_status() -> R {
        let built = builder().build();

        let expected = json!({
            "status": 404,
            "message": "generic error"
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_from_error() -> R {
        let error = TestError;
        let built = Terror::from_error(error)
            .build();

        let expected = json!({
            "status": 500,
            "message": "generic error"
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_shorthand() -> R {
        let built = builder()
            .shorthand("generic")
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "short_message": "generic"
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_error_code() -> R {
        let built = builder()
            .error_code("generic.failure")
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "error_code": "generic.failure"
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_reference() -> R {
        let built = builder()
            .reference()
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "reference": "https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404"
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_string_detail() -> R {
        let built = builder()
            .add_text_detail("key", "val")
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "details": {
                "key": "val"
            }
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_number_detail() -> R {
        let built = builder()
            .add_int_detail("key", 53i64)
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "details": {
                "key": 53
            }
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_bool_detail() -> R {
        let built = builder()
            .add_bool_detail("key", true)
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "details": {
                "key": true
            }
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_w_value_detail() -> R {
        let detail = json!({
            "id" : 25,
            "name" : "server"
        });

        let built = builder()
            .add_value_detail("key", detail)
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "details": {
                "key": {
                    "id": 25,
                    "name": "server"
                }
            }
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_with_several_details() -> R {
        let built = builder()
            .add_text_detail("str", "val")
            .add_int_detail("num", 53i64)
            .add_bool_detail("flg", true)
            .build();

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "details": {
                "str": "val",
                "num": 53,
                "flg": true
            }
        });

        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    #[cfg(not(feature = "err_id"))]
    #[cfg(not(feature = "time"))]
    fn deserialize_all_fields() {
        let inbound = json!({
            "status" : 405u16,
            "message" : "Method not allowed; use GET",
            "short_message" : "Not allowed",
            "error_code" : "web.generic",
            "details" : {
                "allowed" : [ "GET" ],
                "got" : "OPTIONS"
            },
            "reference" : format!("{}/{}", MDN_STATUS_REF, 405)
        });

        let as_struct = serde_json::from_value(inbound);
        assert!(as_struct.is_ok());

        let expected = Terror::new(
            405,
            String::from("Method not allowed; use GET")
        )
            .shorthand(String::from("Not allowed"))
            .error_code(String::from("web.generic"))
            .add_text_detail(
                String::from("got"),
                String::from("OPTIONS")
            )
            .add_value_detail(
                String::from("allowed"),
                Value::Array(vec![
                    Value::String(String::from("GET"))
                ])
            )
            .reference()
            .build();

        assert_eq!(expected, as_struct.unwrap());
    }

    #[test]
    #[cfg(not(feature = "err_id"))]
    #[cfg(not(feature = "time"))]
    fn deserialize_some_fields() {
        let inbound = json!({
            "status" : 405u16,
            "message" : "Method not allowed; use GET",
            "error_code" : "web.generic",
        });

        let as_struct = serde_json::from_value(inbound);
        assert!(as_struct.is_ok());

        let expected = Terror::new(
            405,
            String::from("Method not allowed; use GET")
        )
            .error_code(String::from("web.generic"))
            .build();

        assert_eq!(expected, as_struct.unwrap());
    }

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "generic error")
        }
    }

    impl Error for TestError {}

    fn compare(expected: Value, mut actual: Value) -> R {

        #[cfg(feature = "time")]
        actual.as_object_mut().unwrap().remove("timestamp");

        #[cfg(feature = "err_id")]
        actual.as_object_mut().unwrap().remove("id");

        assert_eq!(expected, actual);
        Ok(())
    }

    fn builder() -> Builder {
        Terror::new(404, "generic error")
    }

}

#[cfg(all(test, feature = "err_id", feature = "time"))]
mod with_features_test {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::str::FromStr;
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde_json::{json, Value};
    use uuid::Uuid;
    use crate::{Builder, Terror};

    type R = anyhow::Result<()>;

    #[test]
    fn build_w_explicit_status() -> R {
        let mut built = builder().build();

        let now = Utc::now();

        assert_eq!(4, built.id.get_version_num());
        assert_eq!(
            now.date_naive(),
            built.timestamp.date_naive()
        );

        // overwrite to check json values
        let uuid = Uuid::new_v4();
        built.id = uuid;
        built.timestamp = now;

        let expected = json!({
            "status": 404,
            "message": "generic error",
            "id": uuid,
            "timestamp": now
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn build_from_error() -> R {
        let error = TestError;
        let mut built = Terror::from_error(error)
            .build();

        let now = Utc::now();

        assert_eq!(4, built.id.get_version_num());
        assert_eq!(
            now.date_naive(),
            built.timestamp.date_naive()
        );

        // overwrite to check json values
        let uuid = Uuid::new_v4();
        built.id = uuid;
        built.timestamp = now;

        let expected = json!({
            "status": 500,
            "message": "generic error",
            "id": uuid,
            "timestamp": now
        });
        let actual = serde_json::to_value(built)?;
        compare(expected, actual)
    }

    #[test]
    fn deserialize_some_fields() {
        let inbound = json!({
            "status" : 405u16,
            "message" : "Method not allowed; use GET",
            "id" : "2d10a950-d6f4-11ec-ab97-00155d887325",
            "timestamp" : "2022-01-01T21:00:00Z"
        });

        let as_struct = serde_json::from_value(inbound);
        assert!(as_struct.is_ok());

        let mut expected = Terror::new(
            405,
            String::from("Method not allowed; use GET")
        )
            .build();

        expected.id = Uuid::from_str("2d10a950-d6f4-11ec-ab97-00155d887325")
            .unwrap();
        expected.timestamp = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(21, 0, 0).unwrap()
            ),
            Utc
        );

        assert_eq!(expected, as_struct.unwrap());
    }

    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "generic error")
        }
    }

    impl Error for TestError {}

    fn compare(expected: Value, actual: Value) -> R {
        assert_eq!(expected, actual);
        Ok(())
    }

    fn builder() -> Builder {
        Terror::new(404, "generic error")
    }

}
