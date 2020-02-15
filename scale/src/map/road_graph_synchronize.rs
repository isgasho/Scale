use super::RoadGraph;
use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{MovedEvent, SelectedEntity};
use crate::map::road_graph_synchronize::ConnectState::{First, Inactive, Unselected};
use crate::map::IntersectionComponent;
use crate::map::{make_inter_entity, Intersection};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{LineRender, MeshRender, MeshRenderEnum};
use crate::rendering::RED;
use specs::prelude::*;
use specs::shred::{DynamicSystemData, PanicHandler};
use specs::shrev::{EventChannel, ReaderId};

#[derive(PartialEq, Eq, Clone, Copy)]
enum ConnectState {
    Inactive,
    Unselected,
    First(Entity),
}

pub struct RoadGraphSynchronize;

pub struct RoadGraphSynchronizeState {
    reader: ReaderId<MovedEvent>,
    connect_state: ConnectState,
    show_connect: Entity,
}

impl RoadGraphSynchronizeState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        let e = world
            .create_entity()
            .with(Transform::new([0.0, 0.0]))
            .with(MeshRender::simple(
                LineRender {
                    offset: [0.0, 0.0].into(),
                    color: RED,
                    thickness: 1.0,
                },
                9,
            ))
            .build();
        Self {
            reader,
            connect_state: Inactive,
            show_connect: e,
        }
    }
}

#[derive(SystemData)]
pub struct RGSData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    self_state: Write<'a, RoadGraphSynchronizeState, PanicHandler>,
    rg: Write<'a, RoadGraph, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    meshrenders: WriteStorage<'a, MeshRender>,
    transforms: WriteStorage<'a, Transform>,
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = RGSData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        // Moved events
        for event in data.moved.read(&mut data.self_state.reader) {
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.rg.set_intersection_position(rnc.id, event.new_pos);
                data.rg.calculate_nodes_positions(rnc.id);
            }
        }
        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data
                .rg
                .add_intersection(Intersection::new(data.mouseinfo.unprojected));
            let intersections = &data.intersections;
            if let Some(x) = data.selected.0.and_then(|x| intersections.get(x)) {
                data.rg.connect(id, x.id);
            }
            let e = make_inter_entity(id, data.mouseinfo.unprojected, &data.lazy, &data.entities);
            *data.selected = SelectedEntity(Some(e));
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.0 {
                if let Some(inter) = data.intersections.get(e) {
                    data.rg.delete_inter(inter.id);
                    data.entities.delete(e).unwrap();
                }
            }
            data.self_state.deactive_connect(&mut data.meshrenders);
        }

        // Connection handling
        if data.kbinfo.just_pressed.contains(&KeyCode::C) {
            match data.self_state.connect_state {
                First(_) => data.self_state.deactive_connect(&mut data.meshrenders),
                _ => data.self_state.connect_state = Unselected,
            }
        }

        if let Some(x) = data.selected.0 {
            if let Some(interc) = data.intersections.get(x) {
                match data.self_state.connect_state {
                    Unselected => {
                        data.self_state.connect_state = First(x);
                        data.meshrenders
                            .get_mut(data.self_state.show_connect)
                            .unwrap()
                            .hide = false;
                    }
                    First(y) => {
                        let interc2 = data.intersections.get(y).unwrap();
                        if y != x {
                            if !data.rg.intersections().is_neigh(interc.id, interc2.id) {
                                data.rg.connect(interc.id, interc2.id);
                            } else {
                                data.rg.disconnect(interc.id, interc2.id);
                            }
                            data.self_state.deactive_connect(&mut data.meshrenders);
                        }
                    }
                    _ => (),
                }
            } else {
                data.self_state.deactive_connect(&mut data.meshrenders);
            }
        }

        if let First(x) = data.self_state.connect_state {
            let trans = data.transforms.get(x).unwrap().clone();
            data.transforms
                .get_mut(data.self_state.show_connect)
                .unwrap()
                .set_position(trans.position());
            if let Some(MeshRenderEnum::Line(x)) = data
                .meshrenders
                .get_mut(data.self_state.show_connect)
                .and_then(|x| x.orders.get_mut(0))
            {
                x.offset = data.mouseinfo.unprojected - trans.position();
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        let state = RoadGraphSynchronizeState::new(world);
        world.insert(state);
    }
}

impl RoadGraphSynchronizeState {
    fn deactive_connect(&mut self, meshrenders: &mut WriteStorage<MeshRender>) {
        self.connect_state = Inactive;
        meshrenders.get_mut(self.show_connect).unwrap().hide = true;
    }
}
