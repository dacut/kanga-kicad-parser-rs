use {
    crate::{
        common::{
            deserialize_mm_to_unsigned_nm, serialize_unsigned_nm_to_mm, Color, Paper, Points, Position, Size, Stroke,
            Symbol, SymbolProperty, TextEffects, TitleBlock,
        },
        impl_try_from_cons_value, LexprExt, ParseError,
    },
    lexpr::{Cons, Value},
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

/// KiCad schematic file format.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "kicad_sch")]
pub struct Schematic {
    /// The schematic version, as a YYYYMMDD integer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,

    /// The program used to generate this schematic (`eeschema` for KiCad's schematic editor).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub generator: String,

    /// The version of the program used to generate this schematic.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub generator_version: String,

    /// The UUID of the schematic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,

    /// The paper size.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub paper: String,

    /// The title block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_block: Option<TitleBlock>,

    /// All of the symbols used
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lib_symbols: Vec<Symbol>,

    /// Junctions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub junctions: Vec<SchematicJunction>,

    /// Unused pins
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub no_connects: Vec<SchematicNoConnect>,

    /// Bus entries
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bus_entries: Vec<SchematicBusEntry>,

    /// Wires
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wires: Vec<SchematicWire>,

    /// Buses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buses: Vec<SchematicBus>,

    /// Graphical polylines
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub polylines: Vec<SchematicGraphicPolyline>,

    /// Graphical text elements
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<SchematicGraphicText>,

    /// Net labels
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<SchematicLabel>,

    /// Global labels (sheet pins)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub global_labels: Vec<SchematicGlobalLabel>,
}

/// Schematic Bus
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_wire_and_bus_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "bus")]
pub struct SchematicBus {
    /// The coordinates of the bus.
    pub points: Points,

    /// The stroke to use for the bus.
    pub stroke: Stroke,

    /// A unique identifier for the bus
    pub uuid: Uuid,
}

/// Schematic bus entry
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_bus_entry_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "bus_entry")]
pub struct SchematicBusEntry {
    /// The position of the bus entry
    #[serde(rename = "at")]
    pub position: Position,

    /// The size of the bus entry. This defines the end point.
    pub size: Size,

    /// The stroke to use for the bus entry.
    pub stroke: Stroke,

    /// A unique identifier for the bus entry
    pub uuid: Uuid,
}


/// Global schematic label (sheet pin), visible across all schematics in a design.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_global_label_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "global_label")]
pub struct SchematicGlobalLabel {
    /// The net name.
    pub text: String,

    /// The shape of the label.
    pub shape: SchematicGlobalLabelShape,

    /// Whether fields have been automatically placed
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fields_autoplaced: bool,

    /// The position of the label.
    #[serde(rename = "at")]
    pub position: Position,

    /// Effects to apply to the label.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,

    /// A unique identifier for the label
    pub uuid: Uuid,

    /// Properties of the label
    pub properties: Vec<SymbolProperty>,
}

/// Global schematic label shape
///
/// [Reference](https://gitlab.com/kicad/code/kicad/-/blob/cbccf6f027002577b1268371cf031a490a6f38f1/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp#L2358)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "shape", rename_all = "snake_case")]
pub enum SchematicGlobalLabelShape {
    /// Input
    Input,

    /// Output
    Output,

    /// Bidirectional
    Bidirectional,

    /// Tri-state
    TriState,

    /// Passive
    Passive,
}

/// Schematic graphical lines, which may not necessarily represnt a closed polygon.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_graphical_line_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "polyline")]
pub struct SchematicGraphicPolyline {
    /// The coordinates of the polyline.
    #[serde(rename = "pts")]
    pub points: Points,

    /// The stroke to use for the polyline.
    pub stroke: Stroke,

    /// A unique identifier for the polyline
    pub uuid: Uuid,
}

/// Schematic text
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_graphical_text_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "text")]
pub struct SchematicGraphicText {
    /// The text to display.
    pub text: String,

    /// The position of the text.
    #[serde(rename = "at")]
    pub position: Position,

    /// Effects to apply to the text.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,

    /// A unique identifier for the text
    pub uuid: Uuid,
}


/// Schematic wire junction
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_no_connect_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "junction")]
pub struct SchematicJunction {
    /// The position of the junction
    #[serde(rename = "at")]
    pub position: Position,

    /// The diameter in nm
    #[serde(deserialize_with = "deserialize_mm_to_unsigned_nm", serialize_with = "serialize_unsigned_nm_to_mm")]
    pub diameter: u64,

    /// The color of the junction.
    pub color: Color,

    /// A unique identifier for the junction
    pub uuid: Uuid,
}

/// Unused schematic pin.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_no_connect_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "no_connect")]
pub struct SchematicNoConnect {
    /// The position of the no connect
    #[serde(rename = "at")]
    pub position: Position,

    /// A unique identifier for the no connect
    pub uuid: Uuid,
}

/// Schematic wire or bus label
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_local_label_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "label")]
pub struct SchematicLabel {
    /// The net name.
    pub text: String,

    /// The position of the label.
    #[serde(rename = "at")]
    pub position: Position,

    /// Effects to apply to the label.
    #[serde(rename = "effects")]
    pub text_effects: TextEffects,

    /// A unique identifier for the label
    pub uuid: Uuid,
}

/// Schematic Wire
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/index.html#_wire_and_bus_section)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "wire")]
pub struct SchematicWire {
    /// The coordinates of the wire.
    pub points: Points,

    /// The stroke to use for the wire.
    pub stroke: Stroke,

    /// A unique identifier for the wire
    pub uuid: Uuid,
}

impl TryFrom<&Cons> for Schematic {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, ParseError> {
        let mut version = None;
        let mut generator = None;
        let mut generator_version = None;
        let mut uuid = None;
        let mut paper = None;
        let mut title_block = None;
        let mut lib_symbols = Vec::new();
        let mut junctions = Vec::new();
        let mut no_connects = Vec::new();
        let mut bus_entries = Vec::new();
        let mut wires = Vec::new();
        let mut buses = Vec::new();
        let mut polylines = Vec::new();
        let mut texts = Vec::new();

        let mut rest = cons.expect_cons_with_symbol_head("kicad_sch")?;

        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, mut cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "version" => {
                    let (value, cdr) = cdr.expect_cons_with_any_int_head()?;
                    cdr.expect_null();
                    version = Some(value);
                }

                "generator" => {
                    let (value, cdr) = cdr.expect_cons_with_any_symbol_head()?;
                    cdr.expect_null()?;
                    generator = Some(value.to_string());
                }

                "generator_version" => {
                    let (value, cdr) = cdr.expect_cons_with_any_symbol_head()?;
                    cdr.expect_null();
                    generator_version = Some(value.to_string());
                }

                "uuid" => {
                    let (value, cdr) = cdr.expect_cons_with_any_symbol_head()?;
                    cdr.expect_null();
                    uuid = Some(Uuid::parse_str(value).map_err(|_| ParseError::InvalidUuid(value.to_string()))?);
                }

                "paper" => {
                    paper = Some(Paper::try_from(r_cons));
                }

                "title_block" => {
                    title_block = Some(TitleBlock::try_from(r_cons));
                }

                "lib_symbols" => {
                    while !cdr.is_null() {
                        let r_cons = cdr.expect_cons()?;
                        let element = r_cons.car();
                        cdr = r_cons.cdr();
                        lib_symbols.push(Symbol::try_from(element)?);
                    }
                }

                "junction" => {
                    junctions.push(SchematicJunction::try_from(element)?);
                }

                "no_connect" => {
                    no_connects.push(SchematicNoConnect::try_from(element)?);
                }
            }
        }

        todo!()
    }
}

impl_try_from_cons_value!(Schematic);

impl TryFrom<&Cons> for SchematicBusEntry {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        
    }
}

impl TryFrom<&Cons> for SchematicJunction {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut position = None;
        let mut diameter = None;
        let mut color = None;
        let mut uuid = None;

        let mut rest = cons.expect_cons_with_symbol_head("junction")?;
        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "at" => {
                    position = Some(Position::try_from(element)?);
                }

                "diameter" => {
                    let (value, cdr) = cdr.expect_cons_with_any_float_head()?;
                    cdr.expect_null()?;
                    diameter = Some((value * 1e6) as u64);
                }

                "color" => {
                    color = Some(Color::try_from(element)?);
                }

                "uuid" => {
                    let (value, cdr) = cdr.expect_cons_with_any_symbol_head()?;
                    cdr.expect_null()?;
                    uuid = Some(Uuid::parse_str(value).map_err(|_| ParseError::InvalidUuid(value.to_string()))?);
                }

                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(position) = position else {
            return Err(ParseError::missing_field("junction", "at", cons.clone()));
        };

        let Some(diameter) = diameter else {
            return Err(ParseError::missing_field("junction", "diameter", cons.clone()));
        };

        let Some(color) = color else {
            return Err(ParseError::missing_field("junction", "color", cons.clone()));
        };

        let Some(uuid) = uuid else {
            return Err(ParseError::missing_field("junction", "uuid", cons.clone()));
        };

        Ok(Self {
            position,
            diameter,
            color,
            uuid,
        })
    }
}

impl_try_from_cons_value!(SchematicJunction);

impl TryFrom<&Cons> for SchematicNoConnect {
    type Error = ParseError;

    fn try_from(cons: &Cons) -> Result<Self, Self::Error> {
        let mut position = None;
        let mut uuid = None;

        let mut rest = cons.expect_cons_with_symbol_head("no_connect")?;
        while !rest.is_null() {
            let r_cons = rest.expect_cons()?;
            let element = r_cons.car();
            rest = r_cons.cdr();
            let (key, cdr) = element.expect_cons_with_any_symbol_head()?;

            match key {
                "at" => {
                    position = Some(Position::try_from(element)?);
                }

                "uuid" => {
                    let (value, cdr) = cdr.expect_cons_with_any_symbol_head()?;
                    cdr.expect_null()?;
                    uuid = Some(Uuid::parse_str(value).map_err(|_| ParseError::InvalidUuid(value.to_string()))?);
                }

                _ => return Err(ParseError::Unexpected(element.clone())),
            }
        }

        let Some(position) = position else {
            return Err(ParseError::missing_field("no_connect", "at", cons.clone()));
        };

        let Some(uuid) = uuid else {
            return Err(ParseError::missing_field("no_connect", "uuid", cons.clone()));
        };

        Ok(Self { position, uuid })
    }
}

impl_try_from_cons_value!(SchematicNoConnect);