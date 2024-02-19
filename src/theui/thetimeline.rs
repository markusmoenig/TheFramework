pub use crate::prelude::*;
use std::collections::BTreeMap;

/// Represents a collection of TheValues.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TheTimeline {
    pub events: BTreeMap<TheTime, TheCollection>,
}

impl Default for TheTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl TheTimeline {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::default(),
        }
    }

    /// Returns true if the timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clears the timeline.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Gets the value for the given key at the given time.
    pub fn get(&self, key: String, at: &TheTime, inter: TheInterpolation) -> Option<TheValue> {
        let mut previous_time: Option<&TheTime> = None;
        let mut previous_value: Option<&TheValue> = None;

        for (time, collection) in &self.events {
            if let Some(value) = collection.get(&key) {
                if let Some(prev_time) = previous_time {
                    if at >= prev_time && at <= time {
                        let start = previous_value.unwrap();
                        let total_span =
                            time.to_total_seconds() as f32 - prev_time.to_total_seconds() as f32;
                        let time_position =
                            at.to_total_seconds() as f32 - prev_time.to_total_seconds() as f32;
                        let t = time_position / total_span;
                        return Some(inter.interpolate(start, value, t));
                    }
                }
                previous_time = Some(time);
                previous_value = Some(value);
            }
        }

        previous_value.cloned()
    }

    /// Gets the value for the given key at the given time.
    pub fn get_default(
        &self,
        key: String,
        at: &TheTime,
        default: TheValue,
        inter: TheInterpolation,
    ) -> TheValue {
        if let Some(value) = self.get(key, at, inter) {
            value
        } else {
            default
        }
    }

    /// Adds a collection of values at the given time.
    pub fn add(&mut self, time: TheTime, collection: TheCollection) {
        if let Some(existing) = self.events.get_mut(&time) {
            for (key, value) in collection.keys.iter() {
                existing.keys.insert(key.clone(), value.clone());
            }
            return;
        }
        self.events.insert(time, collection);
    }

    /// Adds a value at the given time.
    pub fn add_value(&mut self, time: TheTime, key: String, value: TheValue) {
        if let Some(existing) = self.events.get_mut(&time) {
            existing.keys.insert(key, value);
            return;
        }
        let mut collection = TheCollection::new();
        collection.keys.insert(key, value);
        self.add(time, collection);
    }
}

// TheInterpolation
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum TheInterpolation {
    Linear,
    Spline, // Smoothstep
    Switch,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl TheInterpolation {
    pub fn interpolate(&self, start: &TheValue, end: &TheValue, t: f32) -> TheValue {
        let t = t.clamp(0.0, 1.0);

        match (start.to_f32(), end.to_f32()) {
            (Some(s), Some(e)) => match self {
                TheInterpolation::Linear => TheValue::Float(s + (e - s) * t),
                TheInterpolation::Spline => {
                    let t = t * t * (3.0 - 2.0 * t); // Smoothstep
                    TheValue::Float(s + (e - s) * t)
                }
                TheInterpolation::Switch => {
                    if t < 0.5 {
                        start.clone()
                    } else {
                        end.clone()
                    }
                }
                TheInterpolation::EaseIn => {
                    let t = t * t;
                    TheValue::Float(s + (e - s) * t)
                }
                TheInterpolation::EaseOut => {
                    let t = t * (2.0 - t);
                    TheValue::Float(s + (e - s) * t)
                }
                TheInterpolation::EaseInOut => {
                    let t = if t < 0.5 {
                        2.0 * t * t
                    } else {
                        -1.0 + (4.0 - 2.0 * t) * t
                    };
                    TheValue::Float(s + (e - s) * t)
                }
            },
            _ => end.clone(),
        }
    }
}
