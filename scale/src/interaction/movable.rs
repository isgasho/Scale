use crate::engine_interaction::{MouseButton, MouseInfo, TimeInfo};
use crate::interaction::SelectedEntity;
use crate::physics::{Kinematics, Transform};
use cgmath::num_traits::zero;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use specs::prelude::ResourceId;
use specs::shrev::EventChannel;
use specs::{
    Component, Entity, NullStorage, Read, ReadStorage, System, SystemData, World, Write,
    WriteStorage,
};
use std::f32;

#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Movable;
empty_inspect_impl!(Movable);

#[derive(Debug)]
pub struct MovedEvent {
    pub entity: Entity,
    pub new_pos: Vector2<f32>,
}

#[derive(Default)]
pub struct MovableSystem {
    offset: Option<Vector2<f32>>,
}

#[derive(SystemData)]
pub struct MovableSystemData<'a> {
    mouse: Read<'a, MouseInfo>,
    time: Read<'a, TimeInfo>,
    selected: Read<'a, SelectedEntity>,
    moved: Write<'a, EventChannel<MovedEvent>>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    movable: ReadStorage<'a, Movable>,
}

impl<'a> System<'a> for MovableSystem {
    type SystemData = MovableSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if data.mouse.buttons.contains(&MouseButton::Left)
            && data
                .selected
                .0
                .map_or(false, |e| data.movable.get(e).is_some())
        {
            let e = data.selected.0.unwrap();
            match self.offset {
                None => {
                    let p = data.transforms.get_mut(e).unwrap();
                    if let Some(kin) = data.kinematics.get_mut(e) {
                        kin.velocity = zero();
                        kin.acceleration = zero();
                    }
                    self.offset = Some(p.position() - data.mouse.unprojected);
                }
                Some(off) => {
                    let p = data.transforms.get_mut(e).unwrap();
                    let old_pos = p.position();
                    let new_pos = off + data.mouse.unprojected;
                    if new_pos != old_pos {
                        if let Some(kin) = data.kinematics.get_mut(e) {
                            kin.velocity = zero();
                            kin.acceleration = zero();
                        }
                        p.set_position(new_pos);
                        data.moved.single_write(MovedEvent { entity: e, new_pos });
                    }
                }
            }
        } else if let Some(off) = self.offset.take() {
            if let Some(e) = data.selected.0 {
                if let Some(kin) = data.kinematics.get_mut(e) {
                    let p = data.transforms.get(e).unwrap();
                    kin.velocity =
                        (data.mouse.unprojected - (p.position() - off)) / data.time.delta;
                }
            }
        }
    }
}
