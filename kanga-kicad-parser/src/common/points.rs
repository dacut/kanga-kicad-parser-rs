use {
    super::Position,
    serde::{Deserialize, Serialize},
};

/// KiCad coordinate point list.
///
/// [Reference](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_coordinate_point_list)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "pts")]
pub struct Points {
    /// List of points.
    pub points: Vec<Position>,
}
