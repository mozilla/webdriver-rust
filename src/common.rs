use serde_json::{Value, Map};
use std::convert::From;

use error::{WebDriverResult, WebDriverError, ErrorStatus};

pub static ELEMENT_KEY: &'static str = "element-6066-11e4-a52e-4f735466cecf";

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Date(pub u64);

impl Date {
    pub fn new(timestamp: u64) -> Date {
        Date(timestamp)
    }
}

impl From<Date> for Value {
    fn from(date: Date) -> Value {
        let Date(x) = date;
        x.into()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebElement {
    #[serde(rename="element-6066-11e4-a52e-4f735466cecf")]
    pub id: String
}

impl WebElement {
    pub fn new(id: String) -> WebElement {
        WebElement {
            id: id
        }
    }

    pub fn from_json(data: &Value) -> WebDriverResult<WebElement> {
        let object = try_opt!(data.as_object(),
                              ErrorStatus::InvalidArgument,
                              "Could not convert webelement to object");
        let id_value = try_opt!(object.get(ELEMENT_KEY),
                                ErrorStatus::InvalidArgument,
                                "Could not find webelement key");

        let id = try_opt!(id_value.as_str(),
                          ErrorStatus::InvalidArgument,
                          "Could not convert web element to string").to_string();

        Ok(WebElement::new(id))
    }
}

impl <'a> From<&'a WebElement> for Value {
    fn from(elem: &'a WebElement) -> Value {
        let mut data = Map::new();
        data.insert(ELEMENT_KEY.to_string(), elem.id.clone().into());
        Value::Object(data)
    }
}

impl <T> From<T> for WebElement
    where T: Into<String> {
    fn from(data: T) -> WebElement {
        WebElement::new(data.into())
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameId {
    Short(u16),
    Element(WebElement)
}

impl From<FrameId> for Value {
    fn from(frame_id: FrameId) -> Value {
        match frame_id {
            FrameId::Short(x) => {
                Value::Number(x.into())
            },
            FrameId::Element(ref x) => {
                Value::String(x.id.clone())
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum LocatorStrategy {
    #[serde(rename = "css selector")]
    CSSSelector,
    #[serde(rename = "link text")]
    LinkText,
    #[serde(rename = "partial link text")]
    PartialLinkText,
    #[serde(rename = "xpath")]
    XPath
}

impl LocatorStrategy {
    pub fn from_json(body: &Value) -> WebDriverResult<LocatorStrategy> {
        match try_opt!(body.as_str(),
                       ErrorStatus::InvalidArgument,
                       "Cound not convert strategy to string") {
            "css selector" => Ok(LocatorStrategy::CSSSelector),
            "link text" => Ok(LocatorStrategy::LinkText),
            "partial link text" => Ok(LocatorStrategy::PartialLinkText),
            "xpath" => Ok(LocatorStrategy::XPath),
            x => Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                         format!("Unknown locator strategy {}", x)))
        }
    }
}

impl From<LocatorStrategy> for Value {
    fn from(strategy: LocatorStrategy) -> Value {
        match strategy {
            LocatorStrategy::CSSSelector => "css selector",
            LocatorStrategy::LinkText => "link text",
            LocatorStrategy::PartialLinkText => "partial link text",
            LocatorStrategy::XPath => "xpath"
        }.into()
    }
}
