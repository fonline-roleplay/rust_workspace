//use std::collections::btree_map::{BTreeMap, Entry};
use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;

//mod bubble;

pub struct Region<PD> {
    updated: bool,
    size: Size,
    size_changed: bool,
    anchor: Point,
    anchor_changed: bool,
    location: Point,
    physics_data: PD,
}

impl<PD> Region<PD> {
    fn new(size: Size, anchor: Point, physics_data: PD) -> Self {
        Self {
            updated: true,
            size,
            size_changed: false,
            anchor,
            anchor_changed: false,
            location: anchor,
            physics_data,
        }
    }
    pub fn rect(&self) -> (Point, Size) {
        (self.location, self.size)
    }
    pub fn anchor(&self) -> Point {
        self.anchor
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

mod physics {
    use crate::{Point, Region, Size};
    use nalgebra::{Point2, RealField, Unit, Vector2};
    use ncollide2d::shape::{Capsule, ConvexPolygon, Cuboid, ShapeHandle};
    use nphysics2d::{
        force_generator::DefaultForceGeneratorSet,
        joint::DefaultJointConstraintSet,
        material::{BasicMaterial, MaterialHandle},
        math::{Force, ForceType},
        object::{
            ActivationStatus, Body, BodyPartHandle, ColliderDesc, DefaultBodyHandle,
            DefaultBodySet, DefaultColliderHandle, DefaultColliderSet, RigidBodyDesc,
        },
        solver::SignoriniModel,
        world::{DefaultGeometricalWorld, DefaultMechanicalWorld},
    };

    fn size_to_shape(size: &Size) -> ShapeHandle<f32> {
        let shape = Cuboid::new(Vector2::new(size.width / 2.0, size.height / 2.0));
        //let shape = Capsule::new(size.height/2.0, size.width/2.0);
        //let shape = crate::bubble::Bubble::new(size.width/2.0, size.height/2.0);
        /*let half_width = size.width*0.5;
        let half_height = size.height*0.5;
        let x_bump = size.width*0.1;
        let y_bump = size.height*0.1;
        let points = vec![
            Point2::new(-half_width, -half_height),
            Point2::new(0.0, -half_height-y_bump),
            Point2::new(half_width, -half_height),
            Point2::new(half_width+x_bump, 0.0),
            Point2::new(half_width, half_height),
            Point2::new(0.0, half_height+y_bump),
            Point2::new(-half_width, half_height),
            Point2::new(-half_width-x_bump, 0.0),
        ];
        let shape = ConvexPolygon::try_new(points).expect("Invalid convex polygon.");
        */
        ShapeHandle::new(shape)
    }

    pub struct NPhysics<N: RealField> {
        mechanical_world: DefaultMechanicalWorld<N>,
        geometrical_world: DefaultGeometricalWorld<N>,
        body_set: DefaultBodySet<N>,
        collider_set: DefaultColliderSet<N>,
        constraint_set: DefaultJointConstraintSet<N>,
        force_generator_set: DefaultForceGeneratorSet<N>,
    }

    impl<N: RealField> NPhysics<N> {
        pub fn new() -> Self {
            let mut mechanical_world = DefaultMechanicalWorld::new(Vector2::zeros());
            let contact_model = Box::new(SignoriniModel::new());
            mechanical_world.solver.set_contact_model(contact_model);
            //mechanical_world.integration_parameters.restitution_velocity_threshold = nalgebra::convert(0.01);
            let mut geometrical_world = DefaultGeometricalWorld::new();

            let mut body_set = DefaultBodySet::new();
            let mut collider_set = DefaultColliderSet::new();
            let mut constraint_set = DefaultJointConstraintSet::new();
            let mut force_generator_set = DefaultForceGeneratorSet::new();
            Self {
                mechanical_world,
                geometrical_world,
                body_set,
                collider_set,
                constraint_set,
                force_generator_set,
            }
        }
    }
    pub struct NPhysicsData {
        body_handle: DefaultBodyHandle,
        collider_handle: DefaultColliderHandle,
    }
    pub trait Physics {
        type Data;
        fn new_region(&mut self, size: Size, anchor: Point) -> Region<Self::Data>;
        fn upload_region<'a, K: 'a + Ord, I: Iterator<Item = (&'a K, &'a Region<Self::Data>)>>(
            &mut self,
            key: &'a K,
            region: &'a Region<Self::Data>,
            others: I,
        );
        fn delete_region(&mut self, region: &Region<Self::Data>);
        fn step(&mut self);
        fn download_region(&self, region: &mut Region<Self::Data>);
    }
    impl Physics for NPhysics<f32> {
        type Data = NPhysicsData;
        fn new_region(&mut self, size: Size, anchor: Point) -> Region<Self::Data> {
            //let mut activation = ActivationStatus::new_active();
            //activation.set_deactivation_threshold(Some(1.0));
            let body = RigidBodyDesc::new()
                .translation(Vector2::new(anchor.x, anchor.y))
                .gravity_enabled(false)
                .linear_damping(5.0)
                .kinematic_rotations(true)
                .sleep_threshold(Some(2000.0))
                .build();
            let body_handle = self.body_set.insert(body);

            let shape_handle = size_to_shape(&size);
            let body_part_handle = BodyPartHandle(body_handle, 0);
            let collider = ColliderDesc::new(shape_handle)
                .density(1.0)
                .margin(2.0)
                .linear_prediction(10.0)
                //.ccd_enabled(true)
                //.material(MaterialHandle::new(BasicMaterial::new(50.0, 0.05)))
                .build(body_part_handle);
            let collider_handle = self.collider_set.insert(collider);
            let physics_data = NPhysicsData {
                body_handle,
                collider_handle,
            };
            Region::new(size, anchor, physics_data)
        }
        fn upload_region<'a, K: 'a + Ord, I: Iterator<Item = (&'a K, &'a Region<Self::Data>)>>(
            &mut self,
            key: &'a K,
            region: &'a Region<Self::Data>,
            others: I,
        ) {
            let rigid_body = self
                .body_set
                .rigid_body_mut(region.physics_data.body_handle)
                .expect("Get rigid body for region's location update");
            let position = rigid_body.position();
            let translation_vector = position.translation.vector;

            if region.size_changed {
                let collider = self
                    .collider_set
                    .get_mut(region.physics_data.collider_handle)
                    .expect("Get collider for region's size update");
                let shape_handle = size_to_shape(&region.size);
                collider.set_shape(shape_handle);
            }

            let mut max_x = None;
            let mut max_y = None;
            let mut wake_up = region.size_changed || region.anchor_changed;
            for (other_key, other) in others {
                if other_key == key {
                    continue;
                }
                if other.size_changed {
                    wake_up = true;
                }
                /*
                let other_vector = Vector2::new(other.location.x, other.location.y);
                let delta_pos = translation_vector - other_vector;
                let dist = delta_pos.norm();
                let min = other.size.width.min(other.size.height);
                let max = other.size.width.max(other.size.height);
                if dist > 0.0001 && dist < max {
                    let force = Force::linear((min-dist) * 5.0 * delta_pos/dist);
                    rigid_body.apply_force(0, &force, ForceType::Force, false);
                }
                */
                fn calc_axis_force(x0: f32, x1: f32, size0: f32, size1: f32) -> Option<f32> {
                    let diff = x0 - x1;
                    let mut dist = diff.abs().max(0.01);
                    let max = (size0 + size1) * 0.5;
                    if dist > max {
                        return None;
                    }
                    /*
                    let sign = diff.signum();
                    let desired = max;
                    let force = 1.0; //1.0 - dist/desired;
                    Some(sign * force * 1000.0)
                    */
                    let sign = diff.signum();
                    let desired = max;
                    let force = (desired - dist) / desired;
                    //Some(sign * force)
                    Some(sign * (force + 1.0))
                }
                fn option_max(some_max: &mut Option<f32>, some_current: Option<f32>) {
                    *some_max = match (some_max.as_ref(), some_current) {
                        (None, Some(_)) => some_current,
                        (Some(max), Some(current)) if current.abs() > max.abs() => some_current,
                        _ => return,
                    };
                }
                let x = calc_axis_force(
                    region.location.x,
                    other.location.x,
                    region.size.width,
                    other.size.width,
                );
                let y = calc_axis_force(
                    region.location.y,
                    other.location.y,
                    region.size.height,
                    other.size.height,
                );
                if x.is_some() && y.is_some() {
                    option_max(&mut max_x, x);
                    option_max(&mut max_y, y);
                }
            }
            if let (Some(max_x), Some(max_y)) = (max_x, max_y) {
                let mul = 4_000_000.0;
                let vec = Vector2::new(max_x * mul, max_y * mul);
                let force = Force::linear(vec);
                rigid_body.apply_force(0, &force, ForceType::Force, true);
            } else {
                let anchor_vector = Vector2::new(region.anchor.x, region.anchor.y);
                let delta_pos = translation_vector - anchor_vector;
                let force = Force::linear(delta_pos * -5_000.0);
                rigid_body.apply_force(0, &force, ForceType::Force, wake_up);
            }
        }
        fn delete_region(&mut self, region: &Region<Self::Data>) {
            self.body_set.remove(region.physics_data.body_handle);
            self.collider_set
                .remove(region.physics_data.collider_handle);
        }
        fn step(&mut self) {
            self.mechanical_world.step(
                &mut self.geometrical_world,
                &mut self.body_set,
                &mut self.collider_set,
                &mut self.constraint_set,
                &mut self.force_generator_set,
            );
        }
        fn download_region(&self, region: &mut Region<Self::Data>) {
            let rigid_body = self
                .body_set
                .rigid_body(region.physics_data.body_handle)
                .expect("Get rigid body for region's location update");
            let position = rigid_body.position();
            let translation_vector = position.translation.vector;
            region.location.x = translation_vector.x;
            region.location.y = translation_vector.y;
        }
    }
}
use physics::{NPhysics, Physics};

pub struct Layer<K: Ord + Hash, P: Physics> {
    regions: HashMap<K, Region<P::Data>>,
    physics: P,
}

impl<K: Ord + Hash, P: Physics> Layer<K, P> {
    pub fn new(physics: P) -> Self {
        Self {
            regions: HashMap::new(),
            physics,
        }
    }
    pub fn upsert(&mut self, key: K, size: Size, anchor: Point) -> Point {
        match self.regions.entry(key) {
            Entry::Vacant(vacant) => {
                let region = self.physics.new_region(size, anchor);
                vacant.insert(region);
                anchor
            }
            Entry::Occupied(mut occupied) => {
                let region = occupied.get_mut();
                if size != region.size {
                    region.size = size;
                    region.size_changed = true;
                }
                if anchor != region.anchor {
                    region.anchor = anchor;
                    region.anchor_changed = true;
                }
                region.updated = true;
                region.location
            }
        }
    }
    pub fn update(&mut self, remove_old: bool) {
        if remove_old {
            let Layer { regions, physics } = self;
            regions.retain(|key, region| {
                if region.updated {
                    true
                } else {
                    physics.delete_region(region);
                    false
                }
            });
        }
        for (key, region) in &self.regions {
            self.physics.upload_region(key, region, self.regions.iter());
        }
        self.physics.step();
        for (_key, region) in &mut self.regions {
            self.physics.download_region(region);
            region.size_changed = false;
            region.anchor_changed = false;
            region.updated = false;
        }
    }
    pub fn regions(&self) -> impl Iterator<Item = (&K, &Region<P::Data>)> {
        self.regions.iter()
    }
}

pub type NPhysicsLayer<K> = Layer<K, NPhysics<f32>>;
pub fn nphysics_layer<K: Ord + Hash>() -> NPhysicsLayer<K> {
    let physics = NPhysics::new();
    Layer::new(physics)
}
