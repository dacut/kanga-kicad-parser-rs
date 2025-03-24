use kanga_sexpr::sexpr;

sexpr! {
    /// Color
    /// 
    /// An RGB color with an optional alpha channel. Each value is in the range 0.0 to 1.0.
    /// The format of this is `(color <red> <green> <blue> [<alpha>])`.
    #[derive(Debug)]
    pub struct Color {
        (color
            red: f64
            green: f64
            blue: f64
            [alpha: f64]
        )
    }
}

sexpr! {
    /// Font
    /// 
    /// The font to use for text. The format of this is
    /// `(font [(face <string>)] (size <height_mm> <width_mm>) (thickness <mm>) [bold] [italic] [(line_spacing <mm>)])`.
    #[derive(Debug)]
    pub struct Font {
        (font
            [(face: String)]
            (size
                height: f64
                width: f64
            )
            (thickness: f64)
            [bold]
            [italic]
            [(line_spacing:f64)]
        )
    }
}

sexpr! {
    /// Coordinate Point List
    /// 
    /// A list of X/Y coordinate points formatted as `(pts (xy <x> <y>)...)`.

    #[derive(Debug)]
    pub struct Points {
        (pts (xy:XY)*)
    }
}

sexpr! {
    /// Position
    /// 
    /// A two-dimensional position (in millimeters) and optional rotation (in degrees) of an object
    /// formatted as `(at <x> <y> [<angle>])`.
    #[derive(Debug)]
    pub struct Position {
        (at
            /// The X position in millimeters.
            x: f64

            /// The Y position in millimeters.
            y: f64

            /// The rotation angle in degrees.
            [angle: f64]
        )
    }
}

sexpr! {
    /// Stroke definition
    /// 
    /// Defines how the outline of a graphical object is drawn. The format of this is
    /// `(stroke (width <mm>) (type <StrokeType>) (color <red> <green> <blue> [<alpha>]))`.
    #[derive(Debug)]
    pub struct Stroke {
        (stroke
            /// The width of the stroke in millimeters.
            (width: f64)

            /// The type of stroke.
            (r#type => stroke_type: StrokeType)

            /// The color of the stroke.
            (color: Color)
        )
    }
}

sexpr! {
    /// Stroke line type
    /// 
    /// Defines the style of line to draw for a stroked outline. This is one of the following
    /// symbol values: `dash`, `dash_dot`, `dash_dot_dot`, `dot`, `default`, or `solid`.
    #[derive(Debug, Default)]
    pub enum StrokeType {
        dash => Dash,
        dash_dot => DashDot,
        dash_dot_dot => DashDotDot,
        dot => Dot,
        #[default]
        default => Default,
        solid => Solid,
    }
}

sexpr! {
    /// Text effects
    /// 
    /// Defines how text is displayed.
    /// 
    /// ## Format
    /// ```
    /// (effects
    ///   (font <[Font]>)
    ///   (justify [left|right] [top|bottom] [mirror])
    /// )
    /// ```
    #[derive(Debug)]
    pub struct TextEffect {
        (effects
            /// The font to use for the text.
            (font: Font)

            /// The justification of the text.
            [(justify: TextJustify)]

            /// Whether the text is hidden.
            [hide]
        )
    }
}

sexpr! {
    /// Test justification
    ///
    /// Defines how text is justified. Formatted as `(justify [left|right] [top|bottom] [mirror])`.
    #[derive(Debug)]
    pub struct TextJustify {
        (justify
            [h_justify: HJustify]
            [v_justify: VJustify]
            [mirror]
        )
    }
}

sexpr! {
    #[derive(Debug)]
    pub enum HJustify {
        left => Left,
        right => Right,
    }
}

sexpr! {
    #[derive(Debug)]
    pub enum VJustify {
        top => Top,
        bottom => Bottom,
    }
}

sexpr! {
    #[derive(Debug)]
    pub struct XY {
        (xy
            x: f64
            y: f64
        )
    }
}

#[cfg(test)]
mod tests {
    use {super::*, lexpr::sexp};

    #[test]
    fn test_color() {
        let color = Color::try_from(&sexp!((color 0.1 0.2 0.3 0.4))).unwrap();
        assert_eq!(color.red, 0.1);
        assert_eq!(color.green, 0.2);
        assert_eq!(color.blue, 0.3);
        assert_eq!(color.alpha, Some(0.4));

        let color = Color::try_from(&sexp!((color 0.1 0.2 0.3))).unwrap();
        assert_eq!(color.red, 0.1);
        assert_eq!(color.green, 0.2);
        assert_eq!(color.blue, 0.3);
        assert!(color.alpha.is_none());
    }

        #[test]
        fn test_position() {
            let pos = Position::try_from(&sexp!((at 1.0 2.0 3.0))).unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
            assert_eq!(pos.angle, Some(3.0));

            let pos = Position::try_from(&sexp!((at 1.0 2.0))).unwrap();
            assert_eq!(pos.x, 1.0);
            assert_eq!(pos.y, 2.0);
            assert!(pos.angle.is_none());
        }

        #[test]
        fn test_points() {
            let pts = Points::try_from(&sexp!((pts (xy 1.0 2.0) (xy 3.0 4.0)))).unwrap();
            assert_eq!(pts.xy.len(), 2);
            assert_eq!(pts.xy[0].x, 1.0);
            assert_eq!(pts.xy[0].y, 2.0);
            assert_eq!(pts.xy[1].x, 3.0);
            assert_eq!(pts.xy[1].y, 4.0);
        }
}
