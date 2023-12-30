use crate::prelude::*;

pub mod thepattern;
pub mod thesdfcanvas;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TheSDF {
    Circle(TheDim),
    Hexagon(TheDim),
    Rhombus(TheDim),
    RoundedRect(TheDim, (f32, f32, f32, f32)),
}

use TheSDF::*;

impl TheSDF {
    pub fn distance(&self, p: Vec2f) -> f32 {
        match self {
            Circle(dim) => length(p - dim.center()) - dim.radius(),
            Hexagon(dim) => {
                let mut pp = abs(p - dim.center());
                let r = dim.radius() - dim.radius() / 8.0;

                let k = vec3f(-0.866_025_4, 0.5, 0.577_350_26);
                pp -= 2.0 * min(dot(k.xy(), pp), 0.0) * k.xy();
                pp -= vec2f(clamp(pp.x, -k.z * r, k.z * r), r);
                length(pp) * signum(pp.y)
            }
            Rhombus(dim) => {
                fn ndot(a: Vec2f, b: Vec2f) -> f32 {
                    a.x * b.x - a.y * b.y
                }
                let pp = abs(p - dim.center());
                let b = vec2f(dim.radius(), dim.radius());

                let h = clamp(ndot(b - 2.0 * pp, b) / dot(b, b), -1.0, 1.0);
                let mut d = length(pp - 0.5 * b * vec2f(1.0 - h, 1.0 + h));
                d *= signum(pp.x * b.y + pp.y * b.x - b.x * b.y);
                d
            }
            RoundedRect(dim, rounding) => {
                let pp = p - dim.center();
                let mut r: Vec2f;

                if pp.x > 0.0 {
                    r = vec2f(rounding.0, rounding.1);
                } else {
                    r = vec2f(rounding.2, rounding.3);
                }

                if p.y <= 0.0 {
                    r.x = r.y;
                }

                let q: (f32, f32) = (
                    pp.x.abs() - dim.width as f32 / 2.0 + r.x,
                    pp.y.abs() - dim.height as f32 / 2.0 + r.x,
                );

                f32::min(f32::max(q.0, q.1), 0.0)
                    + length(vec2f(f32::max(q.0, 0.0), f32::max(q.1, 0.0)))
                    - r.x
            }
        }
    }

    /// Returns a description of the SDF as string.
    pub fn describe(&self) -> String {
        match self {
            Circle(dim) => format!("Circle: {:?} {}", dim.center(), dim.radius()),
            Hexagon(dim) => format!("Hexagon: {:?} {}", dim.center(), dim.radius()),
            Rhombus(dim) => format!("Hexagon: {:?} {}", dim.center(), dim.radius()),
            RoundedRect(dim, rounding) => {
                format!(
                    "RoundedRect: {:?} {} {:?}",
                    dim.center(),
                    dim.radius(),
                    rounding
                )
            }
        }
    }
}
