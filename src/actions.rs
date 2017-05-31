use serde::{Deserialize, Deserializer};
use serde_json::{Value, Map};
use error::{WebDriverResult, WebDriverError, ErrorStatus};
use common::WebElement;

#[derive(PartialEq, Debug, Serialize)]
pub struct ActionSequence {
    pub id: Option<String>,
    pub actions: ActionsType
}

impl<'de> Deserialize<'de> for ActionSequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "lowercase")]
        enum Helper {
            Null {
                id: Option<String>,
                actions: Vec<NullActionItem>,
            },
            Key {
                id: Option<String>,
                actions: Vec<KeyActionItem>,
            },
            Pointer {
                id: Option<String>,
                #[serde(default)]
                parameters: PointerActionParameters,
                actions: Vec<PointerActionItem>,
            },
        }

        match Helper::deserialize(deserializer)? {
            Helper::Null { id, actions } => {
                Ok(ActionSequence {
                    id: id,
                    actions: ActionsType::Null{actions},
                })
            }
            Helper::Key { id, actions } => {
                Ok(ActionSequence {
                    id: id,
                    actions: ActionsType::Key{actions},
                })
            }
            Helper::Pointer { id, parameters, actions } => {
                Ok(ActionSequence {
                    id: id,
                    actions: ActionsType::Pointer{parameters, actions},
                })
            }
        }
    }
}

impl<'a> From<&'a ActionSequence> for Value {
    fn from(params: &'a ActionSequence) -> Value {
        let mut data: Map<String, Value> = Map::new();
        data.insert("id".into(), params.id.clone().map(|x| x.into()).unwrap_or(Value::Null));
        let (action_type, actions) = match params.actions {
            ActionsType::Null {ref actions} => {
                ("none",
                 actions.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
            ActionsType::Key {ref actions} => {
                ("key",
                 actions.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
            ActionsType::Pointer {ref parameters, ref actions} => {
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

#[derive(PartialEq, Debug, Serialize)]
pub enum ActionsType {
    Null {actions: Vec<NullActionItem>},
    Key {actions: Vec<KeyActionItem>},
    Pointer {parameters: PointerActionParameters, actions:Vec<PointerActionItem>}
}



#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all="lowercase")]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
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

#[derive(Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct PointerActionParameters {
    #[serde(rename="pointerType")]
    pub pointer_type: PointerType
}

impl<'a> From<&'a PointerActionParameters> for Value {
    fn from(params: &'a PointerActionParameters) -> Value {
        let mut data = Map::new();
        data.insert("pointerType".to_owned(),
                    (&params.pointer_type).into());
        Value::Object(data)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum NullActionItem {
    General(GeneralAction)
}

impl<'a> From<&'a NullActionItem> for Value {
    fn from(params: &'a NullActionItem) -> Value {
        match *params {
            NullActionItem::General(ref x) => x.into(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum KeyActionItem {
    General(GeneralAction),
    Key(KeyAction)
}

impl<'a> From<&'a KeyActionItem> for Value {
    fn from(params: &'a KeyActionItem) -> Value {
        match *params {
            KeyActionItem::General(ref x) => x.into(),
            KeyActionItem::Key(ref x) => x.into()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum PointerActionItem {
    General(GeneralAction),
    Pointer(PointerAction)
}

impl<'a> From<&'a PointerActionItem> for Value {
    fn from(params: &'a PointerActionItem) -> Value {
        match *params {
            PointerActionItem::General(ref x) => x.into(),
            PointerActionItem::Pointer(ref x) => x.into()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum GeneralAction {
    Pause(PauseAction)
}

impl<'a> From<&'a GeneralAction> for Value {
    fn from(params: &'a GeneralAction) -> Value {
        match *params {
            GeneralAction::Pause(ref x) => x.into()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PauseAction {
    pub duration: u64
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum KeyAction {
    #[serde(rename="keyUp")]
    Up(KeyUpAction),
    #[serde(rename="keyDown")]
    Down(KeyDownAction)
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct KeyUpAction {
    pub value: char
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct KeyDownAction {
    pub value: char
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

#[derive(PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged, rename_all="lowercase")]
pub enum PointerOrigin {
    Viewport,
    Pointer,
    Element(WebElement),
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum PointerAction {
    #[serde(rename="pointerUp")]
    Up(PointerUpAction),
    #[serde(rename="pointerDown")]
    Down(PointerDownAction),
    #[serde(rename="pointerMove")]
    Move(PointerMoveAction),
    #[serde(rename="pointerCancel")]
    Cancel
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PointerUpAction {
    pub button: u64,
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PointerDownAction {
    pub button: u64,
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PointerMoveAction {
    pub duration: Option<u64>,
    pub origin: PointerOrigin,
    pub x: Option<i64>,
    pub y: Option<i64>
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
mod test {
    use serde_json;
    use command::ActionsParameters;
    use common::WebElement;
    use super::*;

    #[test]
    fn test_pointer_no_parameters() {
        let expected = ActionsParameters {
            actions: vec![
                ActionSequence {
                    id: None,
                    actions: ActionsType::Pointer {
                        parameters: PointerActionParameters {
                            pointer_type: PointerType::Mouse
                        },
                        actions: vec!{
                            PointerActionItem::Pointer (
                                PointerAction::Down (
                                    PointerDownAction {
                                        button: 0
                                    }
                                )
                            ),
                            PointerActionItem::Pointer(
                                PointerAction::Move (
                                    PointerMoveAction {
                                        duration: Some(100),
                                        x: Some(5),
                                        y: Some(10),
                                        origin: PointerOrigin::Pointer
                                    }
                                )
                            ),
                            PointerActionItem::Pointer(
                                PointerAction::Move (
                                    PointerMoveAction {
                                        duration: Some(200),
                                        x: Some(10),
                                        y: Some(20),
                                        origin: PointerOrigin::Element(
                                            WebElement {
                                                id: "elem".into()
                                            }
                                        )
                                    }
                                )
                            ),
                            PointerActionItem::Pointer(
                                PointerAction::Up (
                                    PointerUpAction {
                                        button: 0
                                    }
                                )
                            ),
                            PointerActionItem::Pointer(
                                PointerAction::Cancel
                            ),
                        }
                    }
                }
            ]
        };
        let actual: ActionsParameters = serde_json::from_str(
r#"{"actions": [
  {"type": "pointer", "actions": [
    {"type": "pointerDown", "button": 0},
    {"type": "pointerMove", "x": 5, "y": 10, "origin": "relative"},
    {"type": "pointerMove", "x": 5, "y": 10, "origin": {"element-6066-11e4-a52e-4f735466cecf": "elem"}},
    {"type": "pointerUp", "button": 0},
    {"type": "pointerCancel"}
  ]
}]}"#).unwrap();
        assert_eq!(actual, expected);
    }
}
