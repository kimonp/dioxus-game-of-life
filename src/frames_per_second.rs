//! Calculates the frames per second and places the text in the given id.

use std::collections::VecDeque;
use crate::bindgen_glue::{document, window};

pub struct FramesPerSecond {
    last_timeframe_stamp: f64,
    frames: VecDeque<f64>,
    performance: web_sys::Performance,
    element_id: String,
}

impl FramesPerSecond {
    pub fn new(element_id: &str) -> FramesPerSecond {
        let window = window();
        let performance = window
            .performance()
            .expect("performance should be available");
        let start = performance.now();

        FramesPerSecond {
            last_timeframe_stamp: start,
            frames: VecDeque::new(),
            performance,
            element_id: element_id.to_string(),
        }
    }

    /// Display the current calculation for frames per second.
    fn text(&self) -> String {
        let mut sum = 0_f64;
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for frame in self.frames.iter() {
            sum += frame;
            min = min.min(*frame);
            max = max.max(*frame);
        }
        let mean = (sum / self.frames.len() as f64) as i64;
        let min = min.round() as i64;
        let max = max.round() as i64;

        let latest = if let Some(frame) = self.frames.get(0) {
            (*frame).round()
        } else {
            0_f64
        };

        format!(
            "\
Frames per second:
         latest = {latest}
avg of last 100 = {mean}
min of last 100 = {min}
max of last 100 = {max}
"
        )
        .to_string()
    }

    /// Update the number of frames.
    ///
    /// Call this every time a frame is presented.
    pub fn update_frame(&mut self) {
        let now = self.performance.now();
        let delta = now - self.last_timeframe_stamp;

        self.last_timeframe_stamp = now;

        let latest_fps = 1 as f64 / delta * 1000 as f64;
        self.frames.push_front(latest_fps);

        if self.frames.len() > 100 {
            self.frames.pop_back();
        }

        let element_id = &self.element_id;
        let element = document().get_element_by_id(element_id).expect(&format!("Could not find element {element_id}"));
        element.set_text_content(Some(&self.text()))
    }
}