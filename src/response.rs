use common::Date;
use cookie;
use serde_json::{self, Value};
use std::convert::From;
use time;

#[derive(Debug)]
pub enum WebDriverResponse {
    CloseWindow(CloseWindowResponse),
    Cookie(CookieResponse),
    DeleteSession,
    ElementRect(ElementRectResponse),
    Generic(ValueResponse),
    NewSession(NewSessionResponse),
    Timeouts(TimeoutsResponse),
    Void,
    WindowRect(WindowRectResponse),
}

impl WebDriverResponse {
    pub fn to_json_string(self) -> String {
        let obj = match self {
            WebDriverResponse::CloseWindow(ref x) => serde_json::to_string(&Value::from(x)),
            WebDriverResponse::Cookie(ref x) => serde_json::to_string(x),
            WebDriverResponse::DeleteSession => Ok("{}".to_string()),
            WebDriverResponse::ElementRect(ref x) => serde_json::to_string(x),
            WebDriverResponse::Generic(ref x) => serde_json::to_string(x),
            WebDriverResponse::NewSession(ref x) => serde_json::to_string(x),
            WebDriverResponse::Timeouts(ref x) => serde_json::to_string(x),
            WebDriverResponse::Void => Ok("{}".to_string()),
            WebDriverResponse::WindowRect(ref x) => serde_json::to_string(x),
        }.unwrap();

        match self {
            WebDriverResponse::Generic(_) |
            WebDriverResponse::Cookie(_) => obj,
            _ => {
                let mut data = String::with_capacity(11 + obj.len());
                data.push_str("{\"value\": ");
                data.push_str(&*obj);
                data.push_str("}");
                data
            }
        }
    }
}

#[derive(Serialize, Debug)]
pub struct CloseWindowResponse {
    pub window_handles: Vec<String>,
}

impl CloseWindowResponse {
    pub fn new(handles: Vec<String>) -> CloseWindowResponse {
        CloseWindowResponse { window_handles: handles }
    }
}

impl <'a> From<&'a CloseWindowResponse> for Value {
    fn from(resp: &'a CloseWindowResponse) -> Value {
        Value::Array(resp.window_handles
                    .iter()
                    .map(|x| Value::String(x.clone()))
                    .collect::<Vec<Value>>())
    }
}

#[derive(Serialize, Debug)]
pub struct NewSessionResponse {
    pub sessionId: String,
    pub capabilities: Value
}

impl NewSessionResponse {
    pub fn new(session_id: String, capabilities: Value) -> NewSessionResponse {
        NewSessionResponse {
            capabilities: capabilities,
            sessionId: session_id
        }
    }
}

#[derive(Serialize, Debug)]
pub struct TimeoutsResponse {
    pub script: u64,
    pub pageLoad: u64,
    pub implicit: u64,
}

impl TimeoutsResponse {
    pub fn new(script: u64, page_load: u64, implicit: u64) -> TimeoutsResponse {
        TimeoutsResponse {
            script: script,
            pageLoad: page_load,
            implicit: implicit,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ValueResponse {
    pub value: Value
}

impl ValueResponse {
    pub fn new(value: Value) -> ValueResponse {
        ValueResponse {
            value: value
        }
    }
}

#[derive(Serialize, Debug)]
pub struct WindowRectResponse {
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
}

#[derive(Serialize, Debug)]
pub struct ElementRectResponse {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64
}

impl ElementRectResponse {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> ElementRectResponse {
        ElementRectResponse {
            x: x,
            y: y,
            width: width,
            height: height
        }
    }
}

//TODO: some of these fields are probably supposed to be optional
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expiry: Option<Date>,
    pub secure: bool,
    pub httpOnly: bool
}

impl Cookie {
    pub fn new(name: String, value: String, path: Option<String>, domain: Option<String>,
               expiry: Option<Date>, secure: bool, http_only: bool) -> Cookie {
        Cookie {
            name: name,
            value: value,
            path: path,
            domain: domain,
            expiry: expiry,
            secure: secure,
            httpOnly: http_only
        }
    }
}

impl Into<cookie::Cookie<'static>> for Cookie {
    fn into(self) -> cookie::Cookie<'static> {
        let cookie = cookie::Cookie::build(self.name, self.value)
            .secure(self.secure)
            .http_only(self.httpOnly);
        let cookie = match self.domain {
            Some(domain) => cookie.domain(domain),
            None => cookie,
        };
        let cookie = match self.path {
            Some(path) => cookie.path(path),
            None => cookie,
        };
        let cookie = match self.expiry {
            Some(Date(expiry)) => {
                cookie.expires(time::at(time::Timespec::new(expiry as i64, 0)))
            },
            None => cookie,
        };
        cookie.finish()
    }
}

#[derive(Serialize, Debug)]
pub struct CookieResponse {
    pub value: Vec<Cookie>
}

impl CookieResponse {
    pub fn new(value: Vec<Cookie>) -> CookieResponse {
        CookieResponse {
            value: value
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::{self, Map, Value};
    use super::{WebDriverResponse,
                CloseWindowResponse,
                CookieResponse,
                ElementRectResponse,
                NewSessionResponse,
                ValueResponse,
                TimeoutsResponse,
                WindowRectResponse,
                Cookie};

    fn test(resp: WebDriverResponse, expected_str: &str) {
        let data = resp.to_json_string();
        let actual: Value = serde_json::from_str(&*data).unwrap();
        let expected: Value = serde_json::from_str(expected_str).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_close_window() {
        let resp = WebDriverResponse::CloseWindow(
            CloseWindowResponse::new(vec!["test".into()]));
        let expected = r#"{"value": ["test"]}"#;
        test(resp, expected);
    }

    #[test]
    fn test_cookie() {
        let resp = WebDriverResponse::Cookie(CookieResponse::new(
            vec![
                Cookie::new("test".into(),
                            "test_value".into(),
                            Some("/".into()),
                            None,
                            None,
                            true,
                            false)
            ]));
        let expected = r#"{"value": [{"name": "test", "value": "test_value", "path": "/",
"domain": null, "expiry": null, "secure": true, "httpOnly": false}]}"#;
        test(resp, expected);
    }

    #[test]
    fn test_element_rect() {
        let resp = WebDriverResponse::ElementRect(ElementRectResponse::new(
            0f64, 1f64, 2f64, 3f64));
        let expected = r#"{"value": {"x": 0.0, "y": 1.0, "width": 2.0, "height": 3.0}}"#;
        test(resp, expected);
    }

    #[test]
    fn test_window_rect() {
        let resp = WebDriverResponse::WindowRect(WindowRectResponse {
            x: 0i64,
            y: 1i64,
            width: 2u64,
            height: 3u64,
        });
        let expected = r#"{"value": {"x": 0, "y": 1, "width": 2, "height": 3}}"#;
        test(resp, expected);
    }

    #[test]
    fn test_new_session() {
        let resp = WebDriverResponse::NewSession(
            NewSessionResponse::new("test".into(),
                                    Value::Object(Map::new())));
        let expected = r#"{"value": {"sessionId": "test", "capabilities": {}}}"#;
        test(resp, expected);
    }

    #[test]
    fn test_timeouts() {
         let resp = WebDriverResponse::Timeouts(TimeoutsResponse::new(
            1, 2, 3));
        let expected = r#"{"value": {"script": 1, "pageLoad": 2, "implicit": 3}}"#;
        test(resp, expected);
    }

    #[test]
    fn test_value() {
        let mut value = Map::new();
        value.insert("example".into(), Value::Array(vec![Value::String("test".into())]));
        let resp = WebDriverResponse::Generic(ValueResponse::new(
            Value::Object(value)));
        let expected = r#"{"value": {"example": ["test"]}}"#;
        test(resp, expected);
    }
}
