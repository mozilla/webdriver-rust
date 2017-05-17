use capabilities::{SpecNewSessionParameters, LegacyNewSessionParameters,
                   CapabilitiesMatching, BrowserCapabilities, Capabilities};
use common::{Date, WebElement, FrameId, LocatorStrategy};
use std::convert::From;
use error::{WebDriverResult, WebDriverError, ErrorStatus};
use httpapi::{Route, WebDriverExtensionRoute, VoidWebDriverExtensionRoute};
use regex::Captures;
use serde_json::{self, Value, Map};
use std::default::Default;

#[derive(PartialEq)]
pub enum WebDriverCommand<T: WebDriverExtensionCommand> {
    NewSession(NewSessionParameters),
    DeleteSession,
    Get(GetParameters),
    GetCurrentUrl,
    GoBack,
    GoForward,
    Refresh,
    GetTitle,
    GetPageSource,
    GetWindowHandle,
    GetWindowHandles,
    CloseWindow,
    GetWindowRect,
    SetWindowRect(WindowRectParameters),
    MaximizeWindow,
//    FullscreenWindow // Not supported in marionette
    SwitchToWindow(SwitchToWindowParameters),
    SwitchToFrame(SwitchToFrameParameters),
    SwitchToParentFrame,
    FindElement(LocatorParameters),
    FindElements(LocatorParameters),
    FindElementElement(WebElement, LocatorParameters),
    FindElementElements(WebElement, LocatorParameters),
    GetActiveElement,
    IsDisplayed(WebElement),
    IsSelected(WebElement),
    GetElementAttribute(WebElement, String),
    GetElementProperty(WebElement, String),
    GetCSSValue(WebElement, String),
    GetElementText(WebElement),
    GetElementTagName(WebElement),
    GetElementRect(WebElement),
    IsEnabled(WebElement),
    ExecuteScript(JavascriptCommandParameters),
    ExecuteAsyncScript(JavascriptCommandParameters),
    GetCookies,
    GetNamedCookie(String),
    AddCookie(AddCookieParameters),
    DeleteCookies,
    DeleteCookie(String),
    GetTimeouts,
    SetTimeouts(TimeoutsParameters),
    ElementClick(WebElement),
    ElementTap(WebElement),
    ElementClear(WebElement),
    ElementSendKeys(WebElement, SendKeysParameters),
    PerformActions(ActionsParameters),
    ReleaseActions,
    DismissAlert,
    AcceptAlert,
    GetAlertText,
    SendAlertText(SendKeysParameters),
    TakeScreenshot,
    TakeElementScreenshot(WebElement),
    Status,
    Extension(T)
}

pub trait WebDriverExtensionCommand : Clone + Send + PartialEq {
    fn parameters_json(&self) -> Option<Value>;
}

#[derive(Clone, PartialEq)]
pub struct VoidWebDriverExtensionCommand;

impl WebDriverExtensionCommand for VoidWebDriverExtensionCommand {
    fn parameters_json(&self) -> Option<Value> {
        panic!("No extensions implemented");
    }
}

#[derive(PartialEq)]
pub struct WebDriverMessage <U: WebDriverExtensionRoute=VoidWebDriverExtensionRoute> {
    pub session_id: Option<String>,
    pub command: WebDriverCommand<U::Command>,
}

impl<U: WebDriverExtensionRoute> WebDriverMessage<U> {
    pub fn new(session_id: Option<String>,
               command: WebDriverCommand<U::Command>)
               -> WebDriverMessage<U> {
        WebDriverMessage {
            session_id: session_id,
            command: command,
        }
    }

    pub fn from_http(match_type: Route<U>,
                     params: &Captures,
                     raw_body: &str,
                     requires_body: bool)
                     -> WebDriverResult<WebDriverMessage<U>> {
        let session_id = WebDriverMessage::<U>::get_session_id(params);
        let body_data = try!(WebDriverMessage::<U>::decode_body(raw_body, requires_body));
        let command = match match_type {
            Route::NewSession => {
                let parameters: NewSessionParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::NewSession(parameters)
            },
            Route::DeleteSession => WebDriverCommand::DeleteSession,
            Route::Get => {
                let parameters: GetParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::Get(parameters)
            },
            Route::GetCurrentUrl => WebDriverCommand::GetCurrentUrl,
            Route::GoBack => WebDriverCommand::GoBack,
            Route::GoForward => WebDriverCommand::GoForward,
            Route::Refresh => WebDriverCommand::Refresh,
            Route::GetTitle => WebDriverCommand::GetTitle,
            Route::GetPageSource => WebDriverCommand::GetPageSource,
            Route::GetWindowHandle => WebDriverCommand::GetWindowHandle,
            Route::GetWindowHandles => WebDriverCommand::GetWindowHandles,
            Route::CloseWindow => WebDriverCommand::CloseWindow,
            Route::GetTimeouts => WebDriverCommand::GetTimeouts,
            Route::SetTimeouts => {
                let parameters: TimeoutsParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SetTimeouts(parameters)
            },
            Route::GetWindowRect | Route::GetWindowPosition | Route::GetWindowSize => WebDriverCommand::GetWindowRect,
            Route::SetWindowRect | Route::SetWindowPosition | Route::SetWindowSize => {
                let parameters: WindowRectParameters = Parameters::from_json(&body_data)?;
                WebDriverCommand::SetWindowRect(parameters)
            },
            Route::MaximizeWindow => WebDriverCommand::MaximizeWindow,
            Route::SwitchToWindow => {
                let parameters: SwitchToWindowParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SwitchToWindow(parameters)
            }
            Route::SwitchToFrame => {
                let parameters: SwitchToFrameParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SwitchToFrame(parameters)
            },
            Route::SwitchToParentFrame => WebDriverCommand::SwitchToParentFrame,
            Route::FindElement => {
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElement(parameters)
            },
            Route::FindElements => {
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElements(parameters)
            },
            Route::FindElementElement => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElementElement(element, parameters)
            },
            Route::FindElementElements => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let parameters: LocatorParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::FindElementElements(element, parameters)
            },
            Route::GetActiveElement => WebDriverCommand::GetActiveElement,
            Route::IsDisplayed => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::IsDisplayed(element)
            },
            Route::IsSelected => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::IsSelected(element)
            },
            Route::GetElementAttribute => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let attr = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing name parameter").as_str();
                WebDriverCommand::GetElementAttribute(element, attr.into())
            },
            Route::GetElementProperty => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let property = try_opt!(params.name("name"),
                                        ErrorStatus::InvalidArgument,
                                        "Missing name parameter").as_str();
                WebDriverCommand::GetElementProperty(element, property.into())
            },
            Route::GetCSSValue => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let property = try_opt!(params.name("propertyName"),
                                        ErrorStatus::InvalidArgument,
                                        "Missing propertyName parameter").as_str();
                WebDriverCommand::GetCSSValue(element, property.into())
            },
            Route::GetElementText => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::GetElementText(element)
            },
            Route::GetElementTagName => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::GetElementTagName(element)
            },
            Route::GetElementRect => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::GetElementRect(element)
            },
            Route::IsEnabled => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::IsEnabled(element)
            },
            Route::ElementClick => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::ElementClick(element)
            },
            Route::ElementTap => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::ElementTap(element)
            },
            Route::ElementClear => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::ElementClear(element)
            },
            Route::ElementSendKeys => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let parameters: SendKeysParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ElementSendKeys(element, parameters)
            },
            Route::ExecuteScript => {
                let parameters: JavascriptCommandParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ExecuteScript(parameters)
            },
            Route::ExecuteAsyncScript => {
                let parameters: JavascriptCommandParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::ExecuteAsyncScript(parameters)
            },
            Route::GetCookies => {
                WebDriverCommand::GetCookies
            },
            Route::GetNamedCookie => {
                let name = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing 'name' parameter").as_str().into();
                WebDriverCommand::GetNamedCookie(name)
            },
            Route::AddCookie => {
                let parameters: AddCookieParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::AddCookie(parameters)
            },
            Route::DeleteCookies => {
                WebDriverCommand::DeleteCookies
            },
            Route::DeleteCookie => {
                let name = try_opt!(params.name("name"),
                                    ErrorStatus::InvalidArgument,
                                    "Missing name parameter").as_str().into();
                WebDriverCommand::DeleteCookie(name)
            },
            Route::PerformActions => {
                let parameters: ActionsParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::PerformActions(parameters)
            },
            Route::ReleaseActions => {
                WebDriverCommand::ReleaseActions
            },
            Route::DismissAlert => {
                WebDriverCommand::DismissAlert
            },
            Route::AcceptAlert => {
                WebDriverCommand::AcceptAlert
            },
            Route::GetAlertText => {
                WebDriverCommand::GetAlertText
            },
            Route::SendAlertText => {
                let parameters: SendKeysParameters = try!(Parameters::from_json(&body_data));
                WebDriverCommand::SendAlertText(parameters)
            },
            Route::TakeScreenshot => WebDriverCommand::TakeScreenshot,
            Route::TakeElementScreenshot =>  {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                WebDriverCommand::TakeElementScreenshot(element)
            },
            Route::Status => WebDriverCommand::Status,
            Route::Extension(ref extension) => {
                try!(extension.command(params, &body_data))
            }
        };
        Ok(WebDriverMessage::new(session_id, command))
    }

    fn get_session_id(params: &Captures) -> Option<String> {
        params.name("sessionId").map(|x| x.as_str().into())
    }

    fn decode_body(body: &str, requires_body: bool) -> WebDriverResult<Value> {
        if requires_body {
            match serde_json::from_str(body) {
                Ok(x @ Value::Object(_)) => Ok(x),
                Ok(_) => {
                    Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                            "Body was not a JSON Object"))
                }
                Err(e) => {
                    if e.is_io() {
                        Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                format!("I/O error whilst decoding body: {}", e)))
                    } else {
                        let msg = format!("Failed to decode request as JSON: {}", body);
                        let stack = format!("Syntax error at :{}:{}", e.line(), e.column());
                        Err(WebDriverError::new_with_stack(ErrorStatus::InvalidArgument, msg, stack))
                    }
                }
            }
        } else {
            Ok(Value::Null)
        }
    }
}

impl <U:WebDriverExtensionRoute> From<WebDriverMessage<U>> for Value {
    fn from(msg: WebDriverMessage<U>) -> Value {
        let parameters = match msg.command {
            WebDriverCommand::AcceptAlert |
            WebDriverCommand::CloseWindow |
            WebDriverCommand::ReleaseActions |
            WebDriverCommand::DeleteCookie(_) |
            WebDriverCommand::DeleteCookies |
            WebDriverCommand::DeleteSession |
            WebDriverCommand::DismissAlert |
            WebDriverCommand::ElementClear(_) |
            WebDriverCommand::ElementClick(_) |
            WebDriverCommand::ElementTap(_) |
            WebDriverCommand::GetActiveElement |
            WebDriverCommand::GetAlertText |
            WebDriverCommand::GetNamedCookie(_) |
            WebDriverCommand::GetCookies |
            WebDriverCommand::GetCSSValue(_, _) |
            WebDriverCommand::GetCurrentUrl |
            WebDriverCommand::GetElementAttribute(_, _) |
            WebDriverCommand::GetElementProperty(_, _) |
            WebDriverCommand::GetElementRect(_) |
            WebDriverCommand::GetElementTagName(_) |
            WebDriverCommand::GetElementText(_) |
            WebDriverCommand::GetPageSource |
            WebDriverCommand::GetTimeouts |
            WebDriverCommand::GetTitle |
            WebDriverCommand::GetWindowHandle |
            WebDriverCommand::GetWindowHandles |
            WebDriverCommand::GetWindowRect |
            WebDriverCommand::GoBack |
            WebDriverCommand::GoForward |
            WebDriverCommand::IsDisplayed(_) |
            WebDriverCommand::IsEnabled(_) |
            WebDriverCommand::IsSelected(_) |
            WebDriverCommand::MaximizeWindow |
            WebDriverCommand::NewSession(_) |
            WebDriverCommand::Refresh |
            WebDriverCommand::Status |
            WebDriverCommand::SwitchToParentFrame |
            WebDriverCommand::TakeElementScreenshot(_) |
            WebDriverCommand::TakeScreenshot => {
                None
            },

            WebDriverCommand::AddCookie(ref x) => Some(x.into()),
            WebDriverCommand::ElementSendKeys(_, ref x) => Some(x.into()),
            WebDriverCommand::ExecuteAsyncScript(ref x) |
            WebDriverCommand::ExecuteScript(ref x) => Some(x.into()),
            WebDriverCommand::FindElementElement(_, ref x) => Some(x.into()),
            WebDriverCommand::FindElementElements(_, ref x) => Some(x.into()),
            WebDriverCommand::FindElement(ref x) => Some(x.into()),
            WebDriverCommand::FindElements(ref x) => Some(x.into()),
            WebDriverCommand::Get(ref x) => Some(x.into()),
            WebDriverCommand::PerformActions(ref x) => Some(x.into()),
            WebDriverCommand::SendAlertText(ref x) => Some(x.into()),
            WebDriverCommand::SetTimeouts(ref x) => Some(x.into()),
            WebDriverCommand::SetWindowRect(ref x) => Some(x.into()),
            WebDriverCommand::SwitchToFrame(ref x) => Some(x.into()),
            WebDriverCommand::SwitchToWindow(ref x) => Some(x.into()),
            WebDriverCommand::Extension(ref x) => x.parameters_json(),
        };

        let mut data = Map::new();
        if let Some(parameters) = parameters {
            data.insert("parameters".to_string(), parameters);
        }
        Value::Object(data)
    }
}

pub trait Parameters: Sized {
    fn from_json(body: &Value) -> WebDriverResult<Self>;
}

/// Wrapper around the two supported variants of new session paramters
///
/// The Spec variant is used for storing spec-compliant parameters whereas
/// the legacy variant is used to store desiredCapabilities/requiredCapabilities
/// parameters, and is intended to minimise breakage as we transition users to
/// the spec design.
#[derive(Debug, PartialEq)]
pub enum NewSessionParameters {
    Spec(SpecNewSessionParameters),
    Legacy(LegacyNewSessionParameters),
}

impl Parameters for NewSessionParameters {
    fn from_json(body: &Value) -> WebDriverResult<NewSessionParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");
        if data.get("capabilities").is_some() {
            Ok(NewSessionParameters::Spec(try!(SpecNewSessionParameters::from_json(body))))
        } else {
            Ok(NewSessionParameters::Legacy(try!(LegacyNewSessionParameters::from_json(body))))
        }
    }
}

impl <'a> From<&'a NewSessionParameters> for Value {
    fn from(params: &'a NewSessionParameters) -> Value {
        match *params {
            NewSessionParameters::Spec(ref x) => x.into(),
            NewSessionParameters::Legacy(ref x) => x.into()
        }
    }
}

impl CapabilitiesMatching for NewSessionParameters {
    fn match_browser<T: BrowserCapabilities>(&self, browser_capabilities: &mut T)
                                             -> WebDriverResult<Option<Capabilities>> {
        match self {
            &NewSessionParameters::Spec(ref x) => x.match_browser(browser_capabilities),
            &NewSessionParameters::Legacy(ref x) => x.match_browser(browser_capabilities)
        }
    }
}


#[derive(PartialEq)]
pub struct GetParameters {
    pub url: String
}

impl Parameters for GetParameters {
    fn from_json(body: &Value) -> WebDriverResult<GetParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let url = try_opt!(
            try_opt!(data.get("url"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'url' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "'url' not a string");
        Ok(GetParameters {
            url: url.to_string()
        })
    }
}

impl<'a> From<&'a GetParameters> for Value {
    fn from(params: &'a GetParameters) -> Value {
        let mut data = Map::new();
        data.insert("url".to_string(), Value::String(params.url.clone()));
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct TimeoutsParameters {
    pub script: Option<u64>,
    pub page_load: Option<u64>,
    pub implicit: Option<u64>,
}

impl Parameters for TimeoutsParameters {
    fn from_json(body: &Value) -> WebDriverResult<TimeoutsParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");

        let script = match data.get("script") {
            Some(json) => {
                Some(try_opt!(json.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "Script timeout duration was not a signed integer"))
            }
            None => None,
        };

        let page_load = match data.get("pageLoad") {
            Some(json) => {
                Some(try_opt!(json.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "Page load timeout duration was not a signed integer"))
            }
            None => None,
        };

        let implicit = match data.get("implicit") {
            Some(json) => {
                Some(try_opt!(json.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "Implicit timeout duration was not a signed integer"))
            }
            None => None,
        };

        Ok(TimeoutsParameters {
            script: script,
            page_load: page_load,
            implicit: implicit,
        })
    }
}

impl<'a> From<&'a TimeoutsParameters> for Value {
    fn from(params: &'a TimeoutsParameters) -> Value {
        let mut data = Map::new();
        if let Some(ms) = params.script {
            data.insert("script".into(), Value::Number(ms.into()));
        }
        if let Some(ms) = params.page_load {
            data.insert("pageLoad".into(), Value::Number(ms.into()));
        }
        if let Some(ms) = params.implicit {
            data.insert("implicit".into(), Value::Number(ms.into()));
        }
        Value::Object(data)
    }
}

#[derive(Debug, PartialEq)]
pub struct WindowRectParameters {
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

impl Parameters for WindowRectParameters {
    fn from_json(body: &Value) -> WebDriverResult<WindowRectParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");

        let x = match data.get("x") {
            Some(n) => {
                Some(try_opt!(n.as_i64(),
                              ErrorStatus::InvalidArgument,
                              "'x' is not an integer"))
            }
            None => None,
        };
        let y = match data.get("y") {
            Some(n) => {
                Some(try_opt!(n.as_i64(),
                              ErrorStatus::InvalidArgument,
                              "'y' is not an integer"))
            }
            None => None,
        };
        let width = match data.get("width") {
            Some(n) => {
                Some(try_opt!(n.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "'width' is not a positive integer"))
            }
            None => None,
        };
        let height = match data.get("height") {
            Some(n) => {
                Some(try_opt!(n.as_u64(),
                              ErrorStatus::InvalidArgument,
                              "'height' is not a positive integer"))
            }
            None => None,
        };

        Ok(WindowRectParameters {
               x: x,
               y: y,
               width: width,
               height: height,
           })
    }
}

impl<'a> From<&'a WindowRectParameters> for Value {
    fn from(params: &'a WindowRectParameters) -> Value {
        let mut data = Map::new();
        data.insert("x".to_string(), params.x.map(|x| Value::Number(x.into())).unwrap_or(Value::Null));
        data.insert("y".to_string(), params.y.map(|x| Value::Number(x.into())).unwrap_or(Value::Null));
        data.insert("width".to_string(), params.width.map(|x| Value::Number(x.into())).unwrap_or(Value::Null));
        data.insert("height".to_string(), params.height.map(|x| Value::Number(x.into())).unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SwitchToWindowParameters {
    pub handle: String
}

impl Parameters for SwitchToWindowParameters {
    fn from_json(body: &Value) -> WebDriverResult<SwitchToWindowParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");
        let handle = try_opt!(
            try_opt!(data.get("handle"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'handle' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "'handle' not a string");
        return Ok(SwitchToWindowParameters {
            handle: handle.to_string()
        })
    }
}

impl<'a> From<&'a SwitchToWindowParameters> for Value {
    fn from(params: &'a SwitchToWindowParameters) -> Value {
        let mut data = Map::new();
        data.insert("handle".to_string(), params.handle.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct LocatorParameters {
    pub using: LocatorStrategy,
    pub value: String
}

impl Parameters for LocatorParameters {
    fn from_json(body: &Value) -> WebDriverResult<LocatorParameters> {
        let data = try_opt!(body.as_object(), ErrorStatus::UnknownError,
                            "Message body was not an object");

        let using = try!(LocatorStrategy::from_json(
            try_opt!(data.get("using"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'using' parameter")));

        let value = try_opt!(
            try_opt!(data.get("value"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'value' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "Could not convert using to string").to_string();

        return Ok(LocatorParameters {
            using: using,
            value: value
        })
    }
}

impl<'a> From<&'a LocatorParameters> for Value {
    fn from(params: &'a LocatorParameters) -> Value {
        let mut data = Map::new();
        data.insert("using".to_string(), params.using.into());
        data.insert("value".to_string(), params.value.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SwitchToFrameParameters {
    pub id: FrameId
}

impl Parameters for SwitchToFrameParameters {
    fn from_json(body: &Value) -> WebDriverResult<SwitchToFrameParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::UnknownError,
                            "Message body was not an object");
        let id = try!(FrameId::from_json(try_opt!(data.get("id"),
                                                  ErrorStatus::UnknownError,
                                                  "Missing 'id' parameter")));

        Ok(SwitchToFrameParameters {
            id: id
        })
    }
}

impl<'a> From<&'a SwitchToFrameParameters> for Value {
    fn from(params: &'a SwitchToFrameParameters) -> Value {
        let mut data = Map::new();
        data.insert("id".to_string(), params.id.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct SendKeysParameters {
    pub text: String
}

impl Parameters for SendKeysParameters {
    fn from_json(body: &Value) -> WebDriverResult<SendKeysParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let text = try_opt!(try_opt!(data.get("text"),
                                     ErrorStatus::InvalidArgument,
                                     "Missing 'text' parameter").as_str(),
                            ErrorStatus::InvalidArgument,
                            "Could not convert 'text' to string");

        Ok(SendKeysParameters {
            text: text.into()
        })
    }
}

impl<'a> From<&'a SendKeysParameters> for Value {
    fn from(params: &'a SendKeysParameters) -> Value {
        let mut data = Map::new();
        data.insert("value".to_string(), params.text.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct JavascriptCommandParameters {
    pub script: String,
    pub args: Option<Vec<Value>>
}

impl Parameters for JavascriptCommandParameters {
    fn from_json(body: &Value) -> WebDriverResult<JavascriptCommandParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");

        let args_json = try_opt!(data.get("args"),
                                 ErrorStatus::InvalidArgument,
                                 "Missing args parameter");

        let args = Some(try_opt!(args_json.as_array(),
                                 ErrorStatus::InvalidArgument,
                                 "Failed to convert args to Array").clone());

         //TODO: Look for WebElements in args?
        let script = try_opt!(
            try_opt!(data.get("script"),
                     ErrorStatus::InvalidArgument,
                     "Missing script parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "Failed to convert script to String");
        Ok(JavascriptCommandParameters {
            script: script.to_string(),
            args: args.clone()
        })
    }
}

impl<'a> From<&'a JavascriptCommandParameters> for Value {
    fn from(params: &'a JavascriptCommandParameters) -> Value {
        let mut data = Map::new();
        //TODO: Wrap script so that it becomes marionette-compatible
        data.insert("script".to_string(), params.script.clone().into());
        data.insert("args".to_string(), params.args.clone()
                    .map(|x| Value::Array(x))
                    .unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct GetNamedCookieParameters {
    pub name: Option<String>,
}

impl Parameters for GetNamedCookieParameters {
    fn from_json(body: &Value) -> WebDriverResult<GetNamedCookieParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let name_json = try_opt!(data.get("name"),
                                 ErrorStatus::InvalidArgument,
                                 "Missing 'name' parameter");
        let name = Some(try_opt!(name_json.as_str(),
                                 ErrorStatus::InvalidArgument,
                                 "Failed to convert name to string")
                        .into());
        return Ok(GetNamedCookieParameters { name: name });
    }
}

impl<'a> From<&'a GetNamedCookieParameters> for Value {
    fn from(params: &'a GetNamedCookieParameters) -> Value {
        let mut data = Map::new();
        data.insert("name".to_string(), params.name.clone().map(|x| x.into()).unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct AddCookieParameters {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expiry: Option<Date>,
    pub secure: bool,
    pub httpOnly: bool
}

impl Parameters for AddCookieParameters {
    fn from_json(body: &Value) -> WebDriverResult<AddCookieParameters> {
        if !body.is_object() {
            return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                           "Message body was not an object"));
        }

        let data = try_opt!(body.get("cookie").and_then(|x| x.as_object()),
                            ErrorStatus::UnableToSetCookie,
                            "Cookie parameter not found or not an object");

        let name = try_opt!(
            try_opt!(data.get("name"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'name' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "'name' is not a string").to_string();

        let value = try_opt!(
            try_opt!(data.get("value"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'value' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "'value' is not a string").to_string();

        let path = match data.get("path") {
            Some(path_json) => {
                Some(try_opt!(path_json.as_str(),
                              ErrorStatus::InvalidArgument,
                              "Failed to convert path to String").to_string())
            },
            None => None
        };

        let domain = match data.get("domain") {
            Some(domain_json) => {
                Some(try_opt!(domain_json.as_str(),
                              ErrorStatus::InvalidArgument,
                              "Failed to convert domain to String").to_string())
            },
            None => None
        };

        let expiry = match data.get("expiry") {
            Some(expiry_json) => {
                Some(Date::new(try_opt!(expiry_json.as_u64(),
                                        ErrorStatus::InvalidArgument,
                                        "Failed to convert expiry to Date")))
            },
            None => None
        };

        let secure = match data.get("secure") {
            Some(x) => try_opt!(x.as_bool(),
                                ErrorStatus::InvalidArgument,
                                "Failed to convert secure to boolean"),
            None => false
        };

        let http_only = match data.get("httpOnly") {
            Some(x) => try_opt!(x.as_bool(),
                                ErrorStatus::InvalidArgument,
                                "Failed to convert httpOnly to boolean"),
            None => false
        };

        return Ok(AddCookieParameters {
            name: name,
            value: value,
            path: path,
            domain: domain,
            expiry: expiry,
            secure: secure,
            httpOnly: http_only
        })
    }
}

impl<'a> From<&'a AddCookieParameters> for Value {
    fn from(params: &'a AddCookieParameters) -> Value {
        let mut data = Map::new();
        data.insert("name".to_string(), params.name.clone().into());
        data.insert("value".to_string(), params.value.clone().into());
        data.insert("path".to_string(), params.path.clone().map(|x| x.into()).unwrap_or(Value::Null));
        data.insert("domain".to_string(), params.domain.clone().map(|x| x.into()).unwrap_or(Value::Null));
        data.insert("expiry".to_string(), params.expiry.clone().map(|x| x.into()).unwrap_or(Value::Null));
        data.insert("secure".to_string(), params.secure.into());
        data.insert("httpOnly".to_string(), params.httpOnly.into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct TakeScreenshotParameters {
    pub element: Option<WebElement>
}

impl Parameters for TakeScreenshotParameters {
    fn from_json(body: &Value) -> WebDriverResult<TakeScreenshotParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Message body was not an object");
        let element = match data.get("element") {
            Some(element_json) => Some(try!(WebElement::from_json(element_json))),
            None => None
        };

        return Ok(TakeScreenshotParameters {
            element: element
        })
    }
}

impl<'a> From<&'a TakeScreenshotParameters> for Value {
    fn from(params: &'a TakeScreenshotParameters) -> Value {
        let mut data = Map::new();
        data.insert("element".to_string(), params.element.clone().map(|x| (&x).into()).unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct ActionsParameters {
    pub actions: Vec<ActionSequence>
}

impl Parameters for ActionsParameters {
    fn from_json(body: &Value) -> WebDriverResult<ActionsParameters> {
        try_opt!(body.as_object(),
                 ErrorStatus::InvalidArgument,
                 "Message body was not an object");
        let actions = try_opt!(
            try_opt!(body.get("actions"),
                     ErrorStatus::InvalidArgument,
                     "No actions parameter found").as_array(),
            ErrorStatus::InvalidArgument,
            "Parameter 'actions' was not an array");

        let mut result = Vec::with_capacity(actions.len());
        for chain in actions.iter() {
            result.push(try!(ActionSequence::from_json(chain)));
        }
        Ok(ActionsParameters {
            actions: result
        })
    }
}

impl<'a> From<&'a ActionsParameters> for Value {
    fn from(params: &'a ActionsParameters) -> Value {
        let mut data = Map::new();
        data.insert("actions".to_owned(),
                    params.actions.iter().map(|x| x.into()).collect::<Vec<Value>>().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct ActionSequence {
    pub id: Option<String>,
    pub actions: ActionsType
}

impl Parameters for ActionSequence {
    fn from_json(body: &Value) -> WebDriverResult<ActionSequence> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Actions chain was not an object");

        let type_name = try_opt!(try_opt!(data.get("type"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing type parameter").as_str(),
                                 ErrorStatus::InvalidArgument,
                                 "Parameter ;type' was not a string");

        let id = match data.get("id") {
            Some(x) => Some(try_opt!(x.as_str(),
                                     ErrorStatus::InvalidArgument,
                                     "Parameter 'id' was not a string").to_owned()),
            None => None
        };


        // Note that unlike the spec we get the pointer parameters in ActionsType::from_json

        let actions = match type_name {
            "none" | "key" | "pointer" => try!(ActionsType::from_json(&body)),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                "Invalid action type"))
        };

        Ok(ActionSequence {
            id: id.into(),
            actions: actions
        })
    }
}

impl<'a> From<&'a ActionSequence> for Value {
    fn from(params: &'a ActionSequence) -> Value {
        let mut data: Map<String, Value> = Map::new();
        data.insert("id".into(), params.id.clone().map(|x| x.into()).unwrap_or(Value::Null));
        let (action_type, actions) = match params.actions {
            ActionsType::Null(ref actions) => {
                ("none",
                 actions.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
            ActionsType::Key(ref actions) => {
                ("key",
                 actions.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
            ActionsType::Pointer(ref parameters, ref actions) => {
                data.insert("parameters".into(), parameters.into());
                ("pointer",
                 actions.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
        };
        data.insert("type".into(), action_type.into());
        data.insert("actions".into(), actions.into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub enum ActionsType {
    Null(Vec<NullActionItem>),
    Key(Vec<KeyActionItem>),
    Pointer(PointerActionParameters, Vec<PointerActionItem>)
}

impl Parameters for ActionsType {
    fn from_json(body: &Value) -> WebDriverResult<ActionsType> {
        // These unwraps are OK as long as this is only called from ActionSequence::from_json
        let data = body.as_object().expect("Body should be a JSON Object");
        let actions_type = body.get("type").and_then(|x| x.as_str()).expect("Type should be a string");
        let actions_chain = try_opt!(try_opt!(data.get("actions"),
                                              ErrorStatus::InvalidArgument,
                                              "Missing actions parameter").as_array(),
                                     ErrorStatus::InvalidArgument,
                                     "Parameter 'actions' was not an array");
        match actions_type {
            "none" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                for action_body in actions_chain.iter() {
                    actions.push(try!(NullActionItem::from_json(action_body)));
                };
                Ok(ActionsType::Null(actions))
            },
            "key" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                for action_body in actions_chain.iter() {
                    actions.push(try!(KeyActionItem::from_json(action_body)));
                };
                Ok(ActionsType::Key(actions))
            },
            "pointer" => {
                let mut actions = Vec::with_capacity(actions_chain.len());
                let parameters = match data.get("parameters") {
                    Some(x) => try!(PointerActionParameters::from_json(x)),
                    None => Default::default()
                };

                for action_body in actions_chain.iter() {
                    actions.push(try!(PointerActionItem::from_json(action_body)));
                }
                Ok(ActionsType::Pointer(parameters, actions))
            }
            _ => panic!("Got unexpected action type after checking type")
        }
    }
}

#[derive(PartialEq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

impl Parameters for PointerType {
    fn from_json(body: &Value) -> WebDriverResult<PointerType> {
        match body.as_str() {
            Some("mouse") => Ok(PointerType::Mouse),
            Some("pen") => Ok(PointerType::Pen),
            Some("touch") => Ok(PointerType::Touch),
            Some(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Unsupported pointer type"
            )),
            None => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Pointer type was not a string"
            ))
        }
    }
}

impl<'a> From<&'a PointerType> for Value {
    fn from(params: &'a PointerType) -> Value {
        match *params {
            PointerType::Mouse => "mouse".into(),
            PointerType::Pen => "pen".into(),
            PointerType::Touch => "touch".into(),
        }
    }
}

impl Default for PointerType {
    fn default() -> PointerType {
        PointerType::Mouse
    }
}

#[derive(Default, PartialEq)]
pub struct PointerActionParameters {
    pub pointer_type: PointerType
}

impl Parameters for PointerActionParameters {
    fn from_json(body: &Value) -> WebDriverResult<PointerActionParameters> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Parameter 'parameters' was not an object");
        let pointer_type = match data.get("pointerType") {
            Some(x) => try!(PointerType::from_json(x)),
            None => PointerType::default()
        };
        Ok(PointerActionParameters {
            pointer_type: pointer_type
        })
    }
}

impl<'a> From<&'a PointerActionParameters> for Value {
    fn from(params: &'a PointerActionParameters) -> Value {
        let mut data = Map::new();
        data.insert("pointerType".to_owned(),
                    (&params.pointer_type).into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub enum NullActionItem {
    General(GeneralAction)
}

impl Parameters for NullActionItem {
    fn from_json(body: &Value) -> WebDriverResult<NullActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Actions chain was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");
        match type_name {
            "pause" => Ok(NullActionItem::General(
                try!(GeneralAction::from_json(body)))),
            _ => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                "Invalid type attribute"))
        }
    }
}

impl<'a> From<&'a NullActionItem> for Value {
    fn from(params: &'a NullActionItem) -> Value {
        match *params {
            NullActionItem::General(ref x) => x.into(),
        }
    }
}

#[derive(PartialEq)]
pub enum KeyActionItem {
    General(GeneralAction),
    Key(KeyAction)
}

impl Parameters for KeyActionItem {
    fn from_json(body: &Value) -> WebDriverResult<KeyActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Key action item was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");
        match type_name {
            "pause" => Ok(KeyActionItem::General(
                try!(GeneralAction::from_json(body)))),
            _ => Ok(KeyActionItem::Key(
                try!(KeyAction::from_json(body))))
        }
    }
}

impl<'a> From<&'a KeyActionItem> for Value {
    fn from(params: &'a KeyActionItem) -> Value {
        match *params {
            KeyActionItem::General(ref x) => x.into(),
            KeyActionItem::Key(ref x) => x.into()
        }
    }
}

#[derive(PartialEq)]
pub enum PointerActionItem {
    General(GeneralAction),
    Pointer(PointerAction)
}

impl Parameters for PointerActionItem {
    fn from_json(body: &Value) -> WebDriverResult<PointerActionItem> {
        let data = try_opt!(body.as_object(),
                            ErrorStatus::InvalidArgument,
                            "Pointer action item was not an object");
        let type_name = try_opt!(
            try_opt!(data.get("type"),
                     ErrorStatus::InvalidArgument,
                     "Missing 'type' parameter").as_str(),
            ErrorStatus::InvalidArgument,
            "Parameter 'type' was not a string");

        match type_name {
            "pause" => Ok(PointerActionItem::General(try!(GeneralAction::from_json(body)))),
            _ => Ok(PointerActionItem::Pointer(try!(PointerAction::from_json(body))))
        }
    }
}

impl<'a> From<&'a PointerActionItem> for Value {
    fn from(params: &'a PointerActionItem) -> Value {
        match *params {
            PointerActionItem::General(ref x) => x.into(),
            PointerActionItem::Pointer(ref x) => x.into()
        }
    }
}

#[derive(PartialEq)]
pub enum GeneralAction {
    Pause(PauseAction)
}

impl Parameters for GeneralAction {
    fn from_json(body: &Value) -> WebDriverResult<GeneralAction> {
        match body.get("type").and_then(|x| x.as_str()) {
            Some("pause") => Ok(GeneralAction::Pause(try!(PauseAction::from_json(body)))),
            _ => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                         "Invalid or missing type attribute"))
        }
    }
}

impl<'a> From<&'a GeneralAction> for Value {
    fn from(params: &'a GeneralAction) -> Value {
        match *params {
            GeneralAction::Pause(ref x) => x.into()
        }
    }
}

#[derive(PartialEq)]
pub struct PauseAction {
    pub duration: u64
}

impl Parameters for PauseAction {
    fn from_json(body: &Value) -> WebDriverResult<PauseAction> {
        let default = Value::Number(0.into());
        Ok(PauseAction {
            duration: try_opt!(body.get("duration").unwrap_or(&default).as_u64(),
                               ErrorStatus::InvalidArgument,
                               "Parameter 'duration' was not a positive integer")
        })
    }
}

impl<'a> From<&'a PauseAction> for Value {
    fn from(params: &'a PauseAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(),
                    "pause".into());
        data.insert("duration".to_owned(),
                    params.duration.into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub enum KeyAction {
    Up(KeyUpAction),
    Down(KeyDownAction)
}

impl Parameters for KeyAction {
    fn from_json(body: &Value) -> WebDriverResult<KeyAction> {
        match body.get("type").and_then(|x| x.as_str()) {
            Some("keyDown") => Ok(KeyAction::Down(try!(KeyDownAction::from_json(body)))),
            Some("keyUp") => Ok(KeyAction::Up(try!(KeyUpAction::from_json(body)))),
            Some(_) | None => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                      "Invalid type attribute value for key action"))
        }
    }
}

impl<'a> From<&'a KeyAction> for Value {
    fn from(params: &'a KeyAction) -> Value {
        match *params {
            KeyAction::Down(ref x) => x.into(),
            KeyAction::Up(ref x) => x.into(),
        }
    }
}

fn validate_key_value(value_str: &str) -> WebDriverResult<char> {
    let mut chars = value_str.chars();
    let value = if let Some(c) = chars.next() {
        c
    } else {
        return Err(WebDriverError::new(
            ErrorStatus::InvalidArgument,
            "Parameter 'value' was an empty string"))
    };
    if chars.next().is_some() {
        return Err(WebDriverError::new(
            ErrorStatus::InvalidArgument,
            "Parameter 'value' contained multiple characters"))
    };
    Ok(value)
}

#[derive(PartialEq)]
pub struct KeyUpAction {
    pub value: char
}

impl Parameters for KeyUpAction {
    fn from_json(body: &Value) -> WebDriverResult<KeyUpAction> {
        let value_str = try_opt!(
                try_opt!(body.get("value"),
                         ErrorStatus::InvalidArgument,
                         "Missing value parameter").as_str(),
                ErrorStatus::InvalidArgument,
            "Parameter 'value' was not a string");

        let value = try!(validate_key_value(value_str));
        Ok(KeyUpAction {
            value: value
        })
    }
}

impl<'a> From<&'a KeyUpAction> for Value {
    fn from(params: &'a KeyUpAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(),
                    "keyUp".into());
        data.insert("value".to_string(),
                    params.value.to_string().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct KeyDownAction {
    pub value: char
}

impl Parameters for KeyDownAction {
    fn from_json(body: &Value) -> WebDriverResult<KeyDownAction> {
        let value_str = try_opt!(
                try_opt!(body.get("value"),
                         ErrorStatus::InvalidArgument,
                         "Missing value parameter").as_str(),
                ErrorStatus::InvalidArgument,
            "Parameter 'value' was not a string");
        let value = try!(validate_key_value(value_str));
        Ok(KeyDownAction {
            value: value
        })
    }
}

impl<'a> From<&'a KeyDownAction> for Value {
    fn from(params: &'a KeyDownAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(),
                    "keyDown".into());
        data.insert("value".to_owned(),
                    params.value.to_string().into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub enum PointerOrigin {
    Viewport,
    Pointer,
    Element(WebElement),
}

impl Parameters for PointerOrigin {
    fn from_json(body: &Value) -> WebDriverResult<PointerOrigin> {
        match *body {
            Value::String(ref x) => {
                match &**x {
                    "viewport" => Ok(PointerOrigin::Viewport),
                    "pointer" => Ok(PointerOrigin::Pointer),
                    _ => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                                 "Unknown pointer origin"))
                }
            },
            Value::Object(_) => Ok(PointerOrigin::Element(try!(WebElement::from_json(body)))),
            _ => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                        "Pointer origin was not a string or an object"))
        }
    }
}

impl<'a> From<&'a PointerOrigin> for Value {
    fn from(params: &'a PointerOrigin) -> Value {
        match *params {
            PointerOrigin::Viewport => "viewport".into(),
            PointerOrigin::Pointer => "pointer".into(),
            PointerOrigin::Element(ref x) => x.into(),
        }
    }
}

impl Default for PointerOrigin {
    fn default() -> PointerOrigin {
        PointerOrigin::Viewport
    }
}

#[derive(PartialEq)]
pub enum PointerAction {
    Up(PointerUpAction),
    Down(PointerDownAction),
    Move(PointerMoveAction),
    Cancel
}

impl Parameters for PointerAction {
    fn from_json(body: &Value) -> WebDriverResult<PointerAction> {
        match body.get("type").and_then(|x| x.as_str()) {
            Some("pointerUp") => Ok(PointerAction::Up(try!(PointerUpAction::from_json(body)))),
            Some("pointerDown") => Ok(PointerAction::Down(try!(PointerDownAction::from_json(body)))),
            Some("pointerMove") => Ok(PointerAction::Move(try!(PointerMoveAction::from_json(body)))),
            Some("pointerCancel") => Ok(PointerAction::Cancel),
            Some(_) | None => Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Missing or invalid type argument for pointer action"))
        }
    }
}

impl<'a> From<&'a PointerAction> for Value {
    fn from(params: &'a PointerAction) -> Value {
        match *params {
            PointerAction::Down(ref x) => x.into(),
            PointerAction::Up(ref x) => x.into(),
            PointerAction::Move(ref x) => x.into(),
            PointerAction::Cancel => {
                let mut data = Map::new();
                data.insert("type".to_owned(),
                            "pointerCancel".into());
                Value::Object(data)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct PointerUpAction {
    pub button: u64,
}

impl Parameters for PointerUpAction {
    fn from_json(body: &Value) -> WebDriverResult<PointerUpAction> {
        let button = try_opt!(
            try_opt!(body.get("button"),
                     ErrorStatus::InvalidArgument,
                     "Missing button parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "Parameter 'button' was not a positive integer");

        Ok(PointerUpAction {
            button: button
        })
    }
}

impl<'a> From<&'a PointerUpAction> for Value {
    fn from(params: &'a PointerUpAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(),
                    "pointerUp".into());
        data.insert("button".to_owned(), params.button.into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct PointerDownAction {
    pub button: u64,
}

impl Parameters for PointerDownAction {
    fn from_json(body: &Value) -> WebDriverResult<PointerDownAction> {
        let button = try_opt!(
            try_opt!(body.get("button"),
                     ErrorStatus::InvalidArgument,
                     "Missing button parameter").as_u64(),
            ErrorStatus::InvalidArgument,
            "Parameter 'button' was not a positive integer");

        Ok(PointerDownAction {
            button: button
        })
    }
}

impl<'a> From<&'a PointerDownAction> for Value {
    fn from(params: &'a PointerDownAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(),
                    "pointerDown".into());
        data.insert("button".to_owned(), params.button.into());
        Value::Object(data)
    }
}

#[derive(PartialEq)]
pub struct PointerMoveAction {
    pub duration: Option<u64>,
    pub origin: PointerOrigin,
    pub x: Option<i64>,
    pub y: Option<i64>
}

impl Parameters for PointerMoveAction {
    fn from_json(body: &Value) -> WebDriverResult<PointerMoveAction> {
        let duration = match body.get("duration") {
            Some(duration) => Some(try_opt!(duration.as_u64(),
                                            ErrorStatus::InvalidArgument,
                                            "Parameter 'duration' was not a positive integer")),
            None => None

        };

        let origin = match body.get("origin") {
            Some(o) => try!(PointerOrigin::from_json(o)),
            None => PointerOrigin::default()
        };

        let x = match body.get("x") {
            Some(x) => {
                Some(try_opt!(x.as_i64(),
                              ErrorStatus::InvalidArgument,
                              "Parameter 'x' was not an integer"))
            },
            None => None
        };

        let y = match body.get("y") {
            Some(y) => {
                Some(try_opt!(y.as_i64(),
                              ErrorStatus::InvalidArgument,
                              "Parameter 'y' was not an integer"))
            },
            None => None
        };

        Ok(PointerMoveAction {
            duration: duration.into(),
            origin: origin.into(),
            x: x.into(),
            y: y.into(),
        })
    }
}

impl<'a> From<&'a PointerMoveAction> for Value {
    fn from(params: &'a PointerMoveAction) -> Value {
        let mut data = Map::new();
        data.insert("type".to_owned(), "pointerMove".into());
        if let Some(duration) = params.duration {
            data.insert("duration".to_owned(),
                        duration.into());
        }

        data.insert("origin".to_owned(), (&params.origin).into());

        if let Some(x) = params.x {
            data.insert("x".to_owned(), x.into());
        }
        if let Some(y) = params.y {
            data.insert("y".to_owned(), y.into());
        }
        Value::Object(data)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{self, Value};
    use super::{Parameters, WindowRectParameters};

    #[test]
    fn test_window_rect() {
        let expected = WindowRectParameters {
            x: Some(0i64),
            y: Some(1i64),
            width: Some(2u64),
            height: Some(3u64),
        };
        let actual = serde_json::from_str(r#"{"x": 0, "y": 1, "width": 2, "height": 3}"#).unwrap();
        assert_eq!(expected, Parameters::from_json(&actual).unwrap());
    }

    #[test]
    fn test_window_rect_nullable() {
        let expected = WindowRectParameters {
            x: Some(0i64),
            y: None,
            width: Some(2u64),
            height: None,
        };
        let actual = serde_json::from_str(r#"{"x": 0, "y": null, "width": 2, "height": null}"#).unwrap();
        assert_eq!(expected, Parameters::from_json(&actual).unwrap());
    }

    #[test]
    fn test_window_rect_missing_fields() {
        let expected = WindowRectParameters {
            x: Some(0i64),
            y: None,
            width: Some(2u64),
            height: None,
        };
        let actual = serde_json::from_str(r#"{"x": 0, "width": 2}"#).unwrap();
        assert_eq!(expected, Parameters::from_json(&actual).unwrap());
    }
}
