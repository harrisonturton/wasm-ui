use math::{Vector2, Vector4};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Material {
    pub borders: Borders,
    pub fill: Color,
}

impl Material {
    #[must_use]
    pub fn filled(fill: Color) -> Material {
        Material {
            borders: Borders::none(),
            fill,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct BorderSide {
    pub color: Color,
    pub width: f32,
}

impl BorderSide {
    #[must_use]
    pub fn new(color: Color, width: f32) -> BorderSide {
        BorderSide { color, width }
    }
}

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Borders {
    pub top: Option<BorderSide>,
    pub bottom: Option<BorderSide>,
    pub left: Option<BorderSide>,
    pub right: Option<BorderSide>,
}

impl Borders {
    #[must_use]
    pub fn all(color: Color, width: f32) -> Borders {
        Borders {
            top: Some(BorderSide::new(color, width)),
            bottom: Some(BorderSide::new(color, width)),
            left: Some(BorderSide::new(color, width)),
            right: Some(BorderSide::new(color, width)),
        }
    }

    #[must_use]
    pub fn none() -> Borders {
        Borders {
            top: None,
            bottom: None,
            left: None,
            right: None,
        }
    }

    #[must_use]
    pub fn top(color: Color, width: f32) -> Borders {
        Borders {
            top: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn bottom(color: Color, width: f32) -> Borders {
        Borders {
            top: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn left(color: Color, width: f32) -> Borders {
        Borders {
            left: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn right(color: Color, width: f32) -> Borders {
        Borders {
            right: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn min(&self) -> Vector2 {
        let left = match self.left {
            Some(border) => border.width,
            None => 0.0,
        };
        let right = match self.right {
            Some(border) => border.width,
            None => 0.0,
        };
        Vector2::new(left, right)
    }

    #[must_use]
    pub fn max(&self) -> Vector2 {
        let top = match self.top {
            Some(border) => border.width,
            None => 0.0,
        };
        let bottom = match self.bottom {
            Some(border) => border.width,
            None => 0.0,
        };
        Vector2::new(top, bottom)
    }

    #[must_use]
    pub fn total_width(&self) -> f32 {
        let left = match self.left {
            Some(border) => border.width,
            None => 0.0,
        };
        let right = match self.right {
            Some(border) => border.width,
            None => 0.0,
        };
        left + right
    }

    #[must_use]
    pub fn total_height(&self) -> f32 {
        let top = match self.top {
            Some(border) => border.width,
            None => 0.0,
        };
        let bottom = match self.bottom {
            Some(border) => border.width,
            None => 0.0,
        };
        top + bottom
    }
}

impl Default for Material {
    fn default() -> Material {
        Material::filled(Color::transparent())
    }
}

/// A color stored as RGBA components, each ranging from 0 - 255.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Color {
        Color::transparent()
    }
}

impl Color {
    #[must_use]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[must_use]
    pub fn transparent() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn red() -> Color {
        Color::rgba(255.0, 0.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn green() -> Color {
        Color::rgba(0.0, 255.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn blue() -> Color {
        Color::rgba(0.0, 0.0, 255.0, 255.0)
    }

    #[must_use]
    pub fn yellow() -> Color {
        Color::rgba(255.0, 255.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn white() -> Color {
        Color::rgba(255.0, 255.0, 255.0, 255.0)
    }

    #[must_use]
    pub fn black() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 255.0)
    }

    // The alpha is between 0 and 1
    #[must_use]
    pub fn alpha(self, alpha: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, alpha * 255.0)
    }

    #[must_use]
    pub fn to_linear(&self) -> Vector4 {
        let r = self.r / 255.0;
        let g = self.g / 255.0;
        let b = self.b / 255.0;
        let a = self.a / 255.0;
        Vector4::new(r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_partial_eq_with_same_color_returns_true() {
        let red_lhs = Color::red();
        let red_rhs = Color::red();
        assert_eq!(red_lhs, red_rhs);
    }

    #[test]
    fn color_partial_eq_with_different_color_returns_false() {
        let red = Color::red();
        let green = Color::green();
        assert_ne!(red, green);
    }

    #[test]
    fn material_partial_eq_with_different_color_returns_false() {
        let red = Material::filled(Color::red());
        let green = Material::filled(Color::green());
        assert_ne!(red, green);
    }
}
