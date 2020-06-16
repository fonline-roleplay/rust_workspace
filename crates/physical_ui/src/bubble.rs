use ncollide2d::{
    shape::{ShapeHandle, Shape, SupportMap, FeatureId},
    bounding_volume::{self, AABB},
    math::{Vector, Isometry},
};
use nalgebra::{Vector2, Point2, Isometry2, Unit};

#[derive(Clone)]
pub struct Bubble {
    a: f32, // The first radius.
    b: f32  // The second radius.
}
impl Bubble {
    pub fn new(a: f32, b: f32) -> Self {
        Self{a, b}
    }
}

impl SupportMap<f32> for Bubble {
    fn support_point(&self, transform: &Isometry2<f32>, dir: &Vector2<f32>) -> Point2<f32> {
        // Bring `dir` into the Bubble's local frame.
        let local_dir = transform.inverse_transform_vector(dir);

        // Compute the denominator.
        let denom = f32::sqrt(
            local_dir.x * local_dir.x * self.a * self.a
                + local_dir.y * local_dir.y * self.b * self.b,
        );

        // Compute the support point into the Bubble's local frame.
        let local_support_point = Point2::new(
            self.a * self.a * local_dir.x / denom,
            self.b * self.b * local_dir.y / denom,
        );

        // Return the support point transformed back into the global frame.
        *transform * local_support_point
    }
}

impl Shape<f32> for Bubble {
    fn aabb(&self, m: &Isometry2<f32>) -> AABB<f32> {
        // Generic method to compute the aabb of a support-mapped shape.
        bounding_volume::support_map_aabb(m, self)
    }

    fn as_support_map(&self) -> Option<&SupportMap<f32>> {
        Some(self)
    }

    fn tangent_cone_contains_dir(
        &self,
        _: FeatureId,
        _: &Isometry<f32>,
        _: Option<&[f32]>,
        _: &Unit<Vector<f32>>,
    ) -> bool {
        false
    }
}
/*
/// The volume of a ball.
#[inline]
pub fn bubble_volume<N: RealField>(radius: N) -> N {
    if DIM == 2 {
        let _pi = N::pi();
        _pi * radius * radius
    } else {
        let _pi = N::pi();
        _pi * radius * radius * radius * na::convert(4.0f64 / 3.0)
    }
}

/// The area of a ball.
#[inline]
pub fn bubble_area<N: RealField>(a: N, b: N) -> N {
    if DIM == 2 {
        let _pi = N::pi();
        _pi * radius * na::convert(2.0f64)
    } else {
        unimplemented!();
    }
}

/// The unit angular inertia of a ball.
#[inline]
pub fn bubble_unit_angular_inertia<N: RealField>(radius: N) -> AngularInertia<N> {
    let diag = if DIM == 2 {
        radius * radius / na::convert(2.0f64)
    } else {
        radius * radius * na::convert(2.0f64 / 5.0)
    };

    AngularInertia::from_diagonal_element(diag)
}

impl<N: RealField> Volumetric<N> for Ball<N> {
    fn area(&self) -> N {
        ball_area(self.radius())
    }

    fn volume(&self) -> N {
        ball_volume(self.radius())
    }

    fn center_of_mass(&self) -> Point<N> {
        Point::origin()
    }

    fn unit_angular_inertia(&self) -> AngularInertia<N> {
        ball_unit_angular_inertia(self.radius())
    }
}
*/