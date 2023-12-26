//! Calculates the frames per second and places the text in the given id.

use crate::websys_utils::window;
use std::collections::VecDeque;

pub struct FramesPerSecond {
    last_timeframe_stamp: f64,
    frames: VecDeque<f64>,
    performance: web_sys::Performance,
}

impl Default for FramesPerSecond {
    fn default() -> Self {
        Self::new()
    }
}

impl FramesPerSecond {
    pub fn new() -> FramesPerSecond {
        let window = window();
        let performance = window
            .performance()
            .expect("performance should be available");
        let start = performance.now();

        FramesPerSecond {
            last_timeframe_stamp: start,
            frames: VecDeque::new(),
            performance,
        }
    }

    /// Display the current calculation for frames per second.
    pub fn text(&self) -> String {
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

        let latest_fps = 1_f64 / delta * 1000_f64;
        self.frames.push_front(latest_fps);

        if self.frames.len() > 100 {
            self.frames.pop_back();
        }
    }
}
