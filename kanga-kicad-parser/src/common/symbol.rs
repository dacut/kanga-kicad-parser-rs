use {
    super::{
        deserialize_mm_to_nm, deserialize_mm_to_unsigned_nm, serialize_nm_to_mm, serialize_unsigned_nm_to_mm, Fill,
        Offset, Points, Position, Stroke, TextEffects,
    },
    crate::{impl_try_from_cons_value, LexprExt, ParseError},
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
    std::str::FromStr,
};

/// KiCad symbol or sub-unit of a parent symbol.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Symbol {
    /// The library id or unit id.
    pub id: String,

    /// If this symbol extends another symbol, this is the id of the parent symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// How to treat pin numbers in this symbol.
    #[serde(default, skip_serializing_if = "SymbolPinNumberDefaults::is_default")]
    pub pin_numbers: SymbolPinNumberDefaults,

    /// How to treat pin names in this symbol.
    #[serde(default, skip_serializing_if = "SymbolPinNameDefaults::is_default")]
    pub pin_names: SymbolPinNameDefaults,

    /// Whether this symbol should be excluded from simulation.
    ///
    /// KiCad does not document this, but it is present in some schematic files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_from_sim: Option<bool>,

    /// Whether this symbol is included in the BOM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_bom: Option<bool>,

    /// Whether this symbol is included on the PCB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_board: Option<bool>,

    /// Properties associated with this symbol. Note that these are extended from the regular
    /// [`Property`][crate::Property] type.
    #[serde(default)]
    pub properties: Vec<SymbolProperty>,

    /// Symbol graphics.
    #[serde(default)]
    pub graphics: Vec<SymbolGraphic>,

    /// Symbol pins.
    #[serde(default)]
    pub pins: Vec<SymbolPin>,
}

/// KiCad symbol graphic.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_graphic_items)///
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolGraphic {
    /// Graphical arc.
    Arc(SymbolGraphicArc),

    /// Bezier curve.
    Bezier(SymbolGraphicBezier),

    /// Graphical circle.
    Circle(SymbolGraphicCircle),

    /// Polyline.
    Polyline(SymbolGraphicPolyline),

    /// Rectangle
    Rectangle(SymbolGraphicRectangle),

    /// Text
    Text(SymbolGraphicText),
}

/// Symbol graphic arc.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_arc)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "arc")]
pub struct SymbolGraphicArc {
    /// The starting point of the arc.
    pub start: Position,

    /// The midpoint of the arc.
    pub mid: Position,

    /// The ending point of the arc.
    pub end: Position,

    /// The stroke definition of the arc.
    pub stroke: Stroke,

    /// The fill definition of the arc.
    pub fill: Fill,
}

/// Symbol graphic bezier curve.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_curve)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "bezier")]
pub struct SymbolGraphicBezier {
    /// The four X/Y coordinates of the curve.
    #[serde(rename = "pts")]
    pub points: Points,

    /// The stroke definition of the curve.
    pub stroke: Stroke,

    /// The fill definition of the curve.
    pub fill: Fill,
}

/// Symbol graphic circle.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_circle)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "circle")]
pub struct SymbolGraphicCircle {
    /// The center of the circle.
    pub center: Position,

    /// The radius of the circle in nm.
    #[serde(deserialize_with = "deserialize_mm_to_unsigned_nm", serialize_with = "serialize_unsigned_nm_to_mm")]
    pub radius: u64,

    /// The stroke definition of the circle.
    pub stroke: Stroke,

    /// The fill definition of the circle.
    pub fill: Fill,
}

/// Symbol graphic polyline, which is not necessarily a closed polygon.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_line)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "polyline")]
pub struct SymbolGraphicPolyline {
    /// The points of the polyline.
    #[serde(rename = "pts")]
    pub points: Points,

    /// The stroke definition of the polyline.
    pub stroke: Stroke,

    /// The fill definition of the polyline.
    pub fill: Fill,
}

/// Symbol graphic rectangle.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_rectangle)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "rectangle")]
pub struct SymbolGraphicRectangle {
    /// The start point of the rectangle.
    pub start: Position,

    /// The end point of the rectangle.
    pub end: Position,

    /// The stroke definition of the rectangle.
    pub stroke: Stroke,

    /// The fill definition of the rectangle.
    pub fill: Fill,
}

/// Symbol graphic text.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_text)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "text")]
pub struct SymbolGraphicText {
    /// The text to display.
    pub text: String,

    /// The position of the text.
    #[serde(rename = "at")]
    pub position: Position,

    /// Text effects for displaying the text.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,
}

/// Pin in a symbol definition.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_pin)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "pin")]
pub struct SymbolPin {
    /// The electrical type of the pin.
    pub electrical_type: SymbolPinElectricalType,

    /// The graphical style of the pin.
    pub graphical_style: SymbolPinGraphicalStyle,

    /// The position of the pin.
    #[serde(rename = "at")]
    pub position: Position,

    /// The length of the pin in nm.
    ///
    /// It is possible, though not exactly sensical, for this value to be negative.
    #[serde(deserialize_with = "deserialize_mm_to_nm", serialize_with = "serialize_nm_to_mm")]
    pub length: i64,

    /// The name of the pin.
    pub name: SymbolPinName,

    /// The number of the pin.
    pub number: SymbolPinNumber,
}

/// KiCad symbol pin electrical type.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_pin)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolPinElectricalType {
    /// Pin is an input
    Input,

    /// Pin is an output
    Output,

    /// Pin can be both input and output
    Bidirectional,

    /// Pin is a tri-state output
    TriState,

    /// Pin is electrically passive
    Passive,

    /// Not internally connected
    Free,

    /// Pin does not have a specified electrical type
    Unspecified,

    /// Pin is a power input
    PowerIn,

    /// Pin is a power output
    PowerOut,

    /// Pin is an open-collector output
    OpenCollector,

    /// Pin is an open-emitter output
    OpenEmitter,

    /// Pin has no electrical connection
    NoConnect,
}

/// KiCad symbol pin graphical style
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_pin)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolPinGraphicalStyle {
    /// Line: `|---`
    Line,

    /// Inverted: `|o--`
    Inverted,

    /// Clock: `<|---`
    Clock,

    /// Inverted Clock: `<|o--`
    InvertedClock,

    /// Input Low: `|/|--`
    InputLow,

    /// Clock Low: `<|/|--`
    ClockLow,

    /// Output Low: `|\\--`
    OutputLow,

    /// Edge Clock High: `<|/|--`
    EdgeClockHigh,

    /// Non-Logic: `x--`
    NonLogic,
}

/// The name of a symbol pin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "name")]
pub struct SymbolPinName {
    /// The name of the pin.
    pub name: String,

    /// Text effects for displaying the pin name.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,
}

/// The number of a symbol pin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "number")]
pub struct SymbolPinNumber {
    /// The number of the pin.
    pub number: String,

    /// Text effects for displaying the pin number.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,
}

/// How to treat pin names in this symbol by default.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SymbolPinNameDefaults {
    /// The offset of the pin name in nm.
    #[serde(default)]
    pub offset: i64,

    /// Whether pin names are hidden.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hide: bool,
}

/// How to treat pin numbers in this symbol by default.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SymbolPinNumberDefaults {
    /// Whether pin numbers are hidden.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub hide: bool,
}

/// KiCad symbol property.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_symbol_properties)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SymbolProperty {
    /// Property name.
    pub key: String,

    /// Property value.
    pub value: String,

    /// Unique integer identifier for the property.
    ///
    /// KiCad documents this as required, but it is not present in schematic lib_symbols symbols.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<i64>,

    /// Position of the property.
    #[serde(rename = "at")]
    pub position: Option<Position>,

    /// Text effects for displaying the property.
    #[serde(rename = "effects")]
    pub text_effects: Option<TextEffects>,
}

impl TryFrom<&Cons> for Symbol {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("symbol")?;
        let (id, mut rest) = rest.expect_cons_with_any_str_head()?;
        let id = id.to_string();

        let mut extends = None;
        let mut pin_numbers = SymbolPinNumberDefaults::default();
        let mut pin_names = SymbolPinNameDefaults::default();
        let mut exclude_from_sim = None;
        let mut in_bom = None;
        let mut on_board = None;
        let mut properties = Vec::new();
        let mut graphics = Vec::new();
        let mut pins = Vec::new();

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();

            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "exclude_from_sim" | "in_bom" | "on_board" => {
                    let v = cdr.expect_cons()?;
                    let v_car = v.car();
                    v.cdr().expect_null()?;

                    let value = v_car.expect_bool()?;
                    match key {
                        "exclude_from_sim" => exclude_from_sim = Some(value),
                        "in_bom" => in_bom = Some(value),
                        "on_board" => on_board = Some(value),
                        _ => unreachable!(),
                    }
                }

                "extends" => {
                    let (value, cdr) = cdr.expect_cons_with_any_str_head()?;
                    cdr.expect_null();
                    extends = Some(value.to_string());
                }

                "pin_names" => {
                    pin_names = SymbolPinNameDefaults::try_from(element)?;
                }

                "pin_numbers" => {
                    pin_numbers = SymbolPinNumberDefaults::try_from(element)?;
                }

                "property" => {
                    properties.push(SymbolProperty::try_from(element)?);
                }

                "arc" => {
                    graphics.push(SymbolGraphicArc::try_from(element)?.into());
                }

                "pin" => {
                    pins.push(SymbolPin::try_from(element)?);
                }

                _ => {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            }
        }

        Ok(Self {
            id,
            extends,
            pin_numbers,
            pin_names,
            exclude_from_sim,
            in_bom,
            on_board,
            properties,
            graphics,
            pins,
        })
    }
}

impl_try_from_cons_value!(Symbol);

impl From<SymbolGraphicArc> for SymbolGraphic {
    #[inline(always)]
    fn from(arc: SymbolGraphicArc) -> Self {
        Self::Arc(arc)
    }
}

impl From<SymbolGraphicBezier> for SymbolGraphic {
    #[inline(always)]
    fn from(bezier: SymbolGraphicBezier) -> Self {
        Self::Bezier(bezier)
    }
}

impl From<SymbolGraphicCircle> for SymbolGraphic {
    #[inline(always)]
    fn from(circle: SymbolGraphicCircle) -> Self {
        Self::Circle(circle)
    }
}

impl From<SymbolGraphicPolyline> for SymbolGraphic {
    #[inline(always)]
    fn from(line: SymbolGraphicPolyline) -> Self {
        Self::Polyline(line)
    }
}

impl From<SymbolGraphicRectangle> for SymbolGraphic {
    #[inline(always)]
    fn from(rect: SymbolGraphicRectangle) -> Self {
        Self::Rectangle(rect)
    }
}

impl From<SymbolGraphicText> for SymbolGraphic {
    #[inline(always)]
    fn from(text: SymbolGraphicText) -> Self {
        Self::Text(text)
    }
}

impl TryFrom<&Cons> for SymbolGraphicArc {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut start = None;
        let mut mid = None;
        let mut end = None;
        let mut stroke = None;
        let mut fill = None;

        let mut rest = cons.expect_cons_with_symbol_head("arc")?;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();

            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "start" => start = Some(Position::try_from_xy_cons(cdr)?),
                "mid" => mid = Some(Position::try_from_xy_cons(cdr)?),
                "end" => end = Some(Position::try_from_xy_cons(cdr)?),
                "stroke" => stroke = Some(Stroke::try_from(element)?),
                "fill" => fill = Some(Fill::try_from(element)?),
                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(start) = start else {
            return Err(ParseError::missing_field("arc", "start", cons.clone()));
        };

        let Some(mid) = mid else {
            return Err(ParseError::missing_field("arc", "mid", cons.clone()));
        };

        let Some(end) = end else {
            return Err(ParseError::missing_field("arc", "end", cons.clone()));
        };

        let Some(stroke) = stroke else {
            return Err(ParseError::missing_field("arc", "stroke", cons.clone()));
        };

        let Some(fill) = fill else {
            return Err(ParseError::missing_field("arc", "fill", cons.clone()));
        };

        Ok(Self {
            start,
            mid,
            end,
            stroke,
            fill,
        })
    }
}

impl_try_from_cons_value!(SymbolGraphicArc);

impl TryFrom<&Cons> for SymbolPin {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("pin")?;

        let (electrical_type, rest) = rest.expect_cons_with_any_symbol_head()?;
        let electrical_type = SymbolPinElectricalType::from_str(electrical_type)?;

        let (graphical_style, mut rest) = rest.expect_cons_with_any_symbol_head()?;
        let graphical_style = SymbolPinGraphicalStyle::from_str(graphical_style)?;

        let mut position = None;
        let mut length = None;
        let mut name = None;
        let mut number = None;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();

            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "at" => {
                    position = Some(Position::try_from(element)?);
                }

                "length" => {
                    let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                    cdr.expect_null()?;
                    length = Some((value * 1e6) as i64);
                }

                "name" => {
                    name = Some(SymbolPinName::try_from(element)?);
                }

                "number" => {
                    number = Some(SymbolPinNumber::try_from(element)?);
                }

                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(position) = position else {
            return Err(ParseError::missing_field("pin", "position", cons.clone()));
        };

        let Some(length) = length else {
            return Err(ParseError::missing_field("pin", "length", cons.clone()));
        };

        let Some(name) = name else {
            return Err(ParseError::missing_field("pin", "name", cons.clone()));
        };

        let Some(number) = number else {
            return Err(ParseError::missing_field("pin", "number", cons.clone()));
        };

        Ok(Self {
            electrical_type,
            graphical_style,
            position,
            length,
            name,
            number,
        })
    }
}

impl_try_from_cons_value!(SymbolPin);

impl FromStr for SymbolPinElectricalType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            "bidirectional" => Ok(Self::Bidirectional),
            "tri_state" => Ok(Self::TriState),
            "passive" => Ok(Self::Passive),
            "free" => Ok(Self::Free),
            "unspecified" => Ok(Self::Unspecified),
            "power_in" => Ok(Self::PowerIn),
            "power_out" => Ok(Self::PowerOut),
            "open_collector" => Ok(Self::OpenCollector),
            "open_emitter" => Ok(Self::OpenEmitter),
            "no_connect" => Ok(Self::NoConnect),
            _ => Err(ParseError::Unexpected(Value::Symbol(s.to_string().into_boxed_str()))),
        }
    }
}

impl FromStr for SymbolPinGraphicalStyle {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "line" => Ok(Self::Line),
            "inverted" => Ok(Self::Inverted),
            "clock" => Ok(Self::Clock),
            "inverted_clock" => Ok(Self::InvertedClock),
            "input_low" => Ok(Self::InputLow),
            "clock_low" => Ok(Self::ClockLow),
            "output_low" => Ok(Self::OutputLow),
            "edge_clock_high" => Ok(Self::EdgeClockHigh),
            "non_logic" => Ok(Self::NonLogic),
            _ => Err(ParseError::Unexpected(Value::Symbol(s.to_string().into_boxed_str()))),
        }
    }
}

impl TryFrom<&Cons> for SymbolPinName {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("name")?;
        let (name, mut rest) = rest.expect_cons_with_any_str_head()?;
        let name = name.to_string();
        let mut text_effects = None;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, _) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "effects" => text_effects = Some(TextEffects::try_from(element)?),
                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(text_effects) = text_effects else {
            return Err(ParseError::missing_field("name", "effects", cons.clone()));
        };

        Ok(Self {
            name,
            text_effects,
        })
    }
}

impl_try_from_cons_value!(SymbolPinName);

impl TryFrom<&Cons> for SymbolPinNameDefaults {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut rest = cons.expect_cons_with_symbol_head("pin_names")?;
        let mut offset = 0;
        let mut hide = false;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let element = cons.car();
            rest = cons.cdr();

            if let Some(econs) = element.as_cons() {
                let (key, cdr) = econs.expect_cons_with_any_symbol_head()?;
                match key {
                    "offset" => {
                        let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                        cdr.expect_null()?;
                        offset = (value * 1e6) as i64;
                    }

                    _ => return Err(ParseError::Unexpected(element.clone())),
                }
            } else if let Some(key) = element.as_symbol() {
                if key == "hide" {
                    hide = true
                } else {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            } else {
                return Err(ParseError::Unexpected(element.clone()));
            }
        }

        Ok(Self {
            offset,
            hide,
        })
    }
}

impl_try_from_cons_value!(SymbolPinNameDefaults);

impl TryFrom<&Cons> for SymbolPinNumber {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("number")?;
        let (number, mut rest) = rest.expect_cons_with_any_str_head()?;
        let number = number.to_string();
        let mut text_effects = None;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, _) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "effects" => text_effects = Some(TextEffects::try_from(element)?),
                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(text_effects) = text_effects else {
            return Err(ParseError::missing_field("number", "effects", cons.clone()));
        };

        Ok(Self {
            number,
            text_effects,
        })
    }
}

impl_try_from_cons_value!(SymbolPinNumber);

impl SymbolPinNameDefaults {
    /// Indicates whether this is the default pin name treatment.
    #[inline(always)]
    pub fn is_default(&self) -> bool {
        self.offset == 0 && !self.hide
    }
}

impl TryFrom<&Cons> for SymbolPinNumberDefaults {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut rest = cons.expect_cons_with_symbol_head("pin_numbers")?;
        let mut hide = false;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let element = cons.car();
            rest = cons.cdr();

            if let Some(key) = element.as_symbol() {
                if key == "hide" {
                    hide = true
                } else {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            } else {
                return Err(ParseError::Unexpected(element.clone()));
            }
        }

        Ok(Self {
            hide,
        })
    }
}

impl_try_from_cons_value!(SymbolPinNumberDefaults);

impl SymbolPinNumberDefaults {
    /// Indicates whether this is the default pin number treatment.
    #[inline(always)]
    pub fn is_default(&self) -> bool {
        !self.hide
    }
}

impl TryFrom<&Cons> for SymbolProperty {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let rest = cons.expect_cons_with_symbol_head("property")?;
        let (key, rest) = rest.expect_cons_with_any_str_head()?;
        let (value, mut rest) = rest.expect_cons_with_any_str_head()?;
        let key = key.to_string();
        let value = value.to_string();

        let mut identifier = None;
        let mut position = None;
        let mut text_effects = None;

        while !rest.is_null() {
            let cons = rest.expect_cons()?;
            let element = cons.car();
            rest = cons.cdr();
            let (id, cdr) = element.expect_cons_with_any_symbol_head()?;

            match id {
                "id" => {
                    let (value, cdr) = cdr.expect_cons_with_any_int_head()?;
                    cdr.expect_null()?;
                    identifier = Some(value);
                }

                "at" => {
                    position = Some(Position::try_from(element)?);
                }

                "effects" => {
                    text_effects = Some(TextEffects::try_from(element)?);
                }

                _ => {
                    return Err(ParseError::Unexpected(element.clone()));
                }
            }
        }

        Ok(Self {
            key,
            value,
            identifier,
            position,
            text_effects,
        })
    }
}

impl_try_from_cons_value!(SymbolProperty);
