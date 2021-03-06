use crate::rendering::{Color, GREEN, ORANGE, RED};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TrafficBehavior {
    RED,
    ORANGE,
    GREEN,
    STOP,
}

impl TrafficBehavior {
    pub fn as_render_color(self) -> Color {
        match self {
            TrafficBehavior::RED | TrafficBehavior::STOP => RED,
            TrafficBehavior::ORANGE => ORANGE,
            TrafficBehavior::GREEN => GREEN,
        }
    }

    pub fn is_red(self) -> bool {
        match self {
            TrafficBehavior::RED => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TrafficLightSchedule {
    period: usize,
    green: usize,
    orange: usize,
    red: usize,
    offset: usize,
}

impl TrafficLightSchedule {
    pub fn from_basic(green: usize, orange: usize, red: usize, offset: usize) -> Self {
        Self {
            period: green + orange + red,
            green,
            orange,
            red,
            offset,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TrafficControl {
    Always,
    Periodic(TrafficLightSchedule),
    StopSign,
}

impl TrafficControl {
    pub fn is_always(&self) -> bool {
        match self {
            TrafficControl::Always => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match self {
            TrafficControl::StopSign => true,
            _ => false,
        }
    }

    pub fn get_behavior(&self, time_seconds: u64) -> TrafficBehavior {
        match self {
            TrafficControl::Always => TrafficBehavior::GREEN,
            TrafficControl::Periodic(schedule) => {
                let remainder = (time_seconds as usize + schedule.offset) % schedule.period;
                if remainder < schedule.green {
                    TrafficBehavior::GREEN
                } else if remainder < schedule.green + schedule.orange {
                    TrafficBehavior::ORANGE
                } else {
                    TrafficBehavior::RED
                }
            }
            TrafficControl::StopSign => TrafficBehavior::STOP,
        }
    }
}
