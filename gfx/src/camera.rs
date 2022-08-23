use geometry::transform::Transform;
use geometry::Point3f;
use math::{Matrix4f, Vec3f};

/// A struct representing a camera looking at a scene. The camera's own
/// coordinate system is:
/// - Positive x points right
/// - Positive y points down
/// - Positive z points forward
///
/// World coordinate system is right-handed, positive z pointing up
pub struct Camera {
    /// Camera->World transform
    pub xform: Transform,
    /// Projection transform
    pub projection: Transform,
}

impl Camera {
    /// Look at given point, keeping the `right` direction perpendicular to the world's xy plane.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geometry::Point3f;
    /// use gfx::camera::Camera;
    /// use geometry::transform::Transform;
    /// use math::{assert_eq_eps, Vec3f};
    ///
    /// let mut camera = Camera {
    ///   xform: Transform::translation(Vec3f::new(0.0, 0.0, 0.0)),
    ///   projection: Transform::infinite_projection(1.0, 1.0, 0.1, 0.001)
    /// };
    ///
    /// camera.look_at(Point3f::new(1.0, 0.0, 0.0));
    /// assert_eq!(camera.forward(), Vec3f::new(1.0, 0.0, 0.0));
    /// ```
    pub fn look_at(&mut self, p: Point3f) {
        let forward = p - self.location();
        let right = Vec3f::new(forward.y(), -forward.x(), 0.);
        let down = right.cross(forward);
        self.xform = Transform::from(Matrix4f::from_columns(
            right.xyz0().unit(),
            down.xyz0().unit(),
            forward.xyz0().unit(),
            self.xform.as_matrix().col(3),
        ));
    }

    pub fn move_by(&mut self, direction: Vec3f) {
        let new_location = self.location() + direction;
        self.set_location(new_location)
    }

    pub fn set_location(&mut self, p: Point3f) {
        let m = self.xform.as_matrix_mut();
        m.set(0, 3, p.x());
        m.set(1, 3, p.y());
        m.set(2, 3, p.z());
    }

    pub fn dolly(&mut self, amount: f32) {
        let forward = self.forward();
        self.move_by(forward * amount)
    }

    pub fn truck(&mut self, amount: f32) {
        let right = self.right();
        self.move_by(right * amount);
    }

    /// Return the up direction of the camera as a unit vector, in world coordinates
    pub fn up(&self) -> Vec3f {
        -self.down()
    }
    /// Return the down direction of the camera as a unit vector, in world coordinates
    pub fn down(&self) -> Vec3f {
        self.xform.as_matrix().col(1).xyz()
    }

    /// Return the forward direction of the camera as a unit vector, in world coordinates
    pub fn forward(&self) -> Vec3f {
        self.xform.as_matrix().col(2).xyz()
    }
    /// Return the backward direction of the camera as a unit vector, in world coordinates
    pub fn backward(&self) -> Vec3f {
        -self.forward()
    }

    /// Return the left direction of the camera as a unit vector, in world coordinates
    pub fn left(&self) -> Vec3f {
        -self.right()
    }
    /// Return the right direction of the camera as a unit vector, in world coordinates
    pub fn right(&self) -> Vec3f {
        self.xform.as_matrix().col(0).xyz()
    }

    pub fn location(&self) -> Point3f {
        self.xform.as_matrix().col(3).xyz().into()
    }

    /// Return the View transform (World->Camera)
    pub fn view(&self) -> Option<Transform> {
        self.xform.inverse()
    }
}
