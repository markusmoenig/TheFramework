pub use crate::prelude::*;
use std::collections::BTreeMap;

/// Represents a collection of TheValues.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TheTimeline {
    #[serde(with = "vectorize")]
    pub events: BTreeMap<TheTime, Vec<TheCollection>>,
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
    pub fn get(
        &self,
        name: String,
        key: String,
        at: &TheTime,
        inter: TheInterpolation,
    ) -> Option<TheValue> {
        let mut previous_time: Option<&TheTime> = None;
        let mut previous_value: Option<&TheValue> = None;

        for (time, list) in &self.events {
            for collection in list {
                if name == collection.name {
                    if let Some(value) = collection.get(&key) {
                        if let Some(prev_time) = previous_time {
                            if at >= prev_time && at <= time {
                                let start = previous_value.unwrap();
                                let total_span = time.to_total_seconds() as f32
                                    - prev_time.to_total_seconds() as f32;
                                let time_position = at.to_total_seconds() as f32
                                    - prev_time.to_total_seconds() as f32;
                                let t = time_position / total_span;
                                return Some(inter.interpolate(start, value, t));
                            }
                        }
                        previous_time = Some(time);
                        previous_value = Some(value);
                    }
                }
            }
        }

        previous_value.cloned()
    }

    /// Gets the value for the given key at the given time.
    pub fn get_default(
        &self,
        name: String,
        key: String,
        at: &TheTime,
        default: TheValue,
        inter: TheInterpolation,
    ) -> TheValue {
        if let Some(value) = self.get(name, key, at, inter) {
            value
        } else {
            default
        }
    }

    /// Adds a collection of values at the given time.
    pub fn add(&mut self, time: TheTime, collection: TheCollection) {
        if let Some(existing_list) = self.events.get_mut(&time) {
            for existing in existing_list.iter_mut() {
                if existing.name == collection.name {
                    // for (key, value) in collection.keys.iter() {
                    //     existing.keys.insert(key.clone(), value.clone());
                    // }
                    *existing = collection;
                    return;
                }
            }
            existing_list.push(collection);
            return;
        }
        self.events.insert(time, vec![collection]);
    }

    /// Replaces the keys of the collection with the keys at the given time.
    pub fn fill(&self, time: &TheTime, collection: &mut TheCollection) {
        let keys = collection.keys.keys().cloned().collect::<Vec<String>>();
        for k in keys {
            if let Some(value) = self.get(
                collection.name.clone(),
                k.clone(),
                time,
                TheInterpolation::Linear,
            ) {
                collection.keys.insert(k, value);
            }
        }
    }

    /// Returns the collection at the given time.
    pub fn get_collection_at(&self, time: &TheTime, name: String) -> Option<TheCollection> {
        for (t, list) in &self.events {
            for collection in list {
                if t <= time && collection.name == name {
                    return Some(collection.clone());
                }
            }
        }
        None
    }

    /// Checks if the timeline contains the given collection.
    pub fn contains_collection(&self, name: &str) -> bool {
        for (_, list) in self.events.iter() {
            for collection in list {
                if collection.name == name {
                    return true;
                }
            }
        }
        false
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
