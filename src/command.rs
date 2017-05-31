use actions::ActionSequence;
use capabilities::{SpecNewSessionParametersWrapper, SpecNewSessionParameters, LegacyNewSessionParameters,
                   CapabilitiesMatching, BrowserCapabilities, Capabilities};
use common::{Date, WebElement, FrameId, LocatorStrategy};
use error::{WebDriverResult, WebDriverError, ErrorStatus};
use httpapi::{Route, WebDriverExtensionRoute, VoidWebDriverExtensionRoute};
use regex::Captures;
use serde_json::{self, Value, Map};
use std::convert::From;

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
    TakeElementScreenshot(TakeScreenshotParameters),
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
                let parameters: NewSessionParametersWrapper = serde_json::from_str(raw_body)?;
                WebDriverCommand::NewSession(parameters.into())
            },
            Route::DeleteSession => WebDriverCommand::DeleteSession,
            Route::Get => {
                let parameters: GetParameters = serde_json::from_str(raw_body)?;
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
                let parameters: TimeoutsParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::SetTimeouts(parameters)
            },
            Route::GetWindowRect | Route::GetWindowPosition | Route::GetWindowSize => WebDriverCommand::GetWindowRect,
            Route::SetWindowRect | Route::SetWindowPosition | Route::SetWindowSize => {
                let parameters: WindowRectParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::SetWindowRect(parameters)
            },
            Route::MaximizeWindow => WebDriverCommand::MaximizeWindow,
            Route::SwitchToWindow => {
                let parameters: SwitchToWindowParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::SwitchToWindow(parameters)
            }
            Route::SwitchToFrame => {
                let parameters: SwitchToFrameParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::SwitchToFrame(parameters)
            },
            Route::SwitchToParentFrame => WebDriverCommand::SwitchToParentFrame,
            Route::FindElement => {
                let parameters: LocatorParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::FindElement(parameters)
            },
            Route::FindElements => {
                let parameters: LocatorParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::FindElements(parameters)
            },
            Route::FindElementElement => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let parameters: LocatorParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::FindElementElement(element, parameters)
            },
            Route::FindElementElements => {
                let element_id = try_opt!(params.name("elementId"),
                                          ErrorStatus::InvalidArgument,
                                          "Missing elementId parameter");
                let element = WebElement::new(element_id.as_str().into());
                let parameters: LocatorParameters = serde_json::from_str(raw_body)?;
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
                let parameters: SendKeysParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::ElementSendKeys(element, parameters)
            },
            Route::ExecuteScript => {
                let parameters: JavascriptCommandParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::ExecuteScript(parameters)
            },
            Route::ExecuteAsyncScript => {
                let parameters: JavascriptCommandParameters = serde_json::from_str(raw_body)?;
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
                let parameters: AddCookieParametersWrapper = serde_json::from_str(raw_body)?;
                WebDriverCommand::AddCookie(parameters.cookie)
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
                let parameters: ActionsParameters = serde_json::from_str(raw_body)?;
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
                let parameters: SendKeysParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::SendAlertText(parameters)
            },
            Route::TakeScreenshot => WebDriverCommand::TakeScreenshot,
            Route::TakeElementScreenshot =>  {
                let parameters: TakeScreenshotParameters = serde_json::from_str(raw_body)?;
                WebDriverCommand::TakeElementScreenshot(parameters)
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

/// Wrapper around the two supported variants of new session paramters
///
/// The Spec variant is used for storing spec-compliant parameters whereas
/// the legacy variant is used to store desiredCapabilities/requiredCapabilities
/// parameters, and is intended to minimise breakage as we transition users to
/// the spec design.

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NewSessionParametersWrapper {
    Spec(SpecNewSessionParametersWrapper),
    Legacy(LegacyNewSessionParameters),
}

#[derive(Debug, PartialEq)]
pub enum NewSessionParameters {
    Spec(SpecNewSessionParameters),
    Legacy(LegacyNewSessionParameters),
}

impl From<NewSessionParametersWrapper> for NewSessionParameters {
    fn from(wrapper: NewSessionParametersWrapper) -> NewSessionParameters {
        match wrapper {
            NewSessionParametersWrapper::Spec(x) => NewSessionParameters::Spec(x.into()),
            NewSessionParametersWrapper::Legacy(x) => NewSessionParameters::Legacy(x)
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


#[derive(PartialEq, Deserialize, Serialize)]
pub struct GetParameters {
    pub url: String
}

impl<'a> From<&'a GetParameters> for Value {
    fn from(params: &'a GetParameters) -> Value {
        let mut data = Map::new();
        data.insert("url".to_string(), Value::String(params.url.clone()));
        Value::Object(data)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct TimeoutsParameters {
    pub script: Option<u64>,
    #[serde(rename="pageLoad")]
    pub page_load: Option<u64>,
    pub implicit: Option<u64>,
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowRectParameters {
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<u64>,
    pub height: Option<u64>,
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

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct SwitchToWindowParameters {
    pub handle: String
}

impl<'a> From<&'a SwitchToWindowParameters> for Value {
    fn from(params: &'a SwitchToWindowParameters) -> Value {
        let mut data = Map::new();
        data.insert("handle".to_string(), params.handle.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct LocatorParameters {
    pub using: LocatorStrategy,
    pub value: String
}

impl<'a> From<&'a LocatorParameters> for Value {
    fn from(params: &'a LocatorParameters) -> Value {
        let mut data = Map::new();
        data.insert("using".to_string(), params.using.into());
        data.insert("value".to_string(), params.value.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct SwitchToFrameParameters {
    pub id: Option<FrameId>
}

impl<'a> From<&'a SwitchToFrameParameters> for Value {
    fn from(params: &'a SwitchToFrameParameters) -> Value {
        let mut data = Map::new();
        data.insert("id".to_string(), params.id.clone().map(|x| x.into()).unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct SendKeysParameters {
    pub text: String
}

impl<'a> From<&'a SendKeysParameters> for Value {
    fn from(params: &'a SendKeysParameters) -> Value {
        let mut data = Map::new();
        data.insert("value".to_string(), params.text.clone().into());
        Value::Object(data)
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct JavascriptCommandParameters {
    pub script: String,
    pub args: Option<Vec<Value>>
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

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct AddCookieParametersWrapper {
    cookie: AddCookieParameters
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct AddCookieParameters {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expiry: Option<Date>,
    pub secure: bool,
    pub httpOnly: bool
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

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct TakeScreenshotParameters {
    pub element: Option<WebElement>
}

impl<'a> From<&'a TakeScreenshotParameters> for Value {
    fn from(params: &'a TakeScreenshotParameters) -> Value {
        let mut data = Map::new();
        data.insert("element".to_string(), params.element.clone().map(|x| (&x).into()).unwrap_or(Value::Null));
        Value::Object(data)
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionsParameters {
    pub actions: Vec<ActionSequence>
}

impl<'a> From<&'a ActionsParameters> for Value {
    fn from(params: &'a ActionsParameters) -> Value {
        let mut data = Map::new();
        data.insert("actions".to_owned(),
                    params.actions.iter().map(|x| x.into()).collect::<Vec<Value>>().into());
        Value::Object(data)
    }
}
