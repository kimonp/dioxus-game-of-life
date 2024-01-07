//! Calculates the frames per second and places the text in the given id.

use dioxus::prelude::*;

use std::collections::VecDeque;

#[cfg(feature = "web")]
use crate::websys_utils::window;
#[cfg(feature = "desktop")]
use std::time::Instant;

/// Frames per second component that shows how quickly the app is rendering animation frames.
///
/// The web version does not have access to std::time, so we use web_sys::Performace hooks instead.
#[component]
pub fn FramesPerSecond(cx: Scope, frame_id: i32) -> Element {
    let frames_per_second = use_ref(cx, FramesPerSecond::new);
    let fps_text = use_state(cx, || frames_per_second.read().text());

    use_effect(cx, (frame_id,), |(_frame_id,)| {
        to_owned![frames_per_second, fps_text];
        async move {
            frames_per_second.with_mut(|fps| {
                fps.update_frame();
                fps_text.modify(|_old_text| fps.text());
            });
        }
    });

    render! {
        div { white_space: "pre", font_family: "monospace", fps_text.get().clone() }
    }
}

struct FramesPerSecond {
    #[cfg(feature = "web")]
    last_timeframe_stamp: f64,
    #[cfg(feature = "desktop")]
    last_timeframe_stamp: Instant,

    frames: VecDeque<f64>,
    #[cfg(feature = "web")]
    performance: web_sys::Performance,
}

impl Default for FramesPerSecond {
    fn default() -> Self {
        Self::new()
    }
}

impl FramesPerSecond {
    #[cfg(feature = "web")]
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

    #[cfg(feature = "desktop")]
    pub fn new() -> FramesPerSecond {
        let start = Instant::now();

        FramesPerSecond {
            last_timeframe_stamp: start,
            frames: VecDeque::new(),
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

    #[cfg(feature = "web")]
    fn calc_delta(&self) -> (f64, f64) {
        let now = self.performance.now();
        let delta = now - self.last_timeframe_stamp;

        (now, delta)
    }

    #[cfg(feature = "desktop")]
    fn calc_delta(&self) -> (Instant, f64) {
        let now = Instant::now();
        let delta = now - self.last_timeframe_stamp;
        let delta = delta.as_micros() as f64 / 1000_f64;

        (now, delta)
    }

    /// Update the number of frames.
    ///
    /// Call this every time a frame is presented.
    pub fn update_frame(&mut self) {
        let (now, delta) = self.calc_delta();

        self.last_timeframe_stamp = now;

        let latest_fps = 1_f64 / delta * 1000_f64;
        self.frames.push_front(latest_fps);

        if self.frames.len() > 100 {
            self.frames.pop_back();
        }
    }
}
