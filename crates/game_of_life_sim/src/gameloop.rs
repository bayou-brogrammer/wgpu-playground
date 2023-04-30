#![allow(dead_code)]

use instant::{Duration, Instant};

/// A clock that tracks how much it has advanced (and how much real time has elapsed) since
/// its previous update and since its creation.
#[derive(Debug, Clone)]
pub struct Time {
    // pausing
    paused: bool,

    startup: Instant,
    first_update: Option<Instant>,
    last_update: Option<Instant>,

    // scaling
    delta: Duration,
    delta_seconds: f32,
    delta_seconds_f64: f64,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_update: None,
            first_update: None,
            delta: Duration::ZERO,
            startup: Instant::now(),

            paused: false,
            delta_seconds: 0.0,
            delta_seconds_f64: 0.0,
        }
    }
}

impl Time {
    /// Updates the internal time measurements.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.update_with_instant(now);
    }

    /// Updates time with a specified [`Instant`].
    pub fn update_with_instant(&mut self, instant: Instant) {
        let delta = instant - self.last_update.unwrap_or(self.startup);

        if self.last_update.is_some() {
            self.delta = delta;
            self.delta_seconds = self.delta.as_secs_f32();
            self.delta_seconds_f64 = self.delta.as_secs_f64();
        } else {
            self.first_update = Some(instant);
        }

        self.last_update = Some(instant);
    }

    /// Returns the [`Instant`] the clock was created.
    ///
    /// This usually represents when the app was started.
    #[inline]
    pub fn startup(&self) -> Instant {
        self.startup
    }

    /// Returns the [`Instant`] when [`update`](#method.update) was first called, if it exists.
    ///
    /// This usually represents when the first app update started.
    #[inline]
    pub fn first_update(&self) -> Option<Instant> {
        self.first_update
    }

    /// Returns the [`Instant`] when [`update`](#method.update) was last called, if it exists.
    ///
    /// This usually represents when the current app update started.
    #[inline]
    pub fn last_update(&self) -> Option<Instant> {
        self.last_update
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as a [`Duration`].
    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as [`f32`] seconds.
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Returns how much time has advanced since the last [`update`](#method.update), as [`f64`] seconds.
    #[inline]
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_seconds_f64
    }

    /// Stops the clock, preventing it from advancing until resumed.
    ///
    /// **Note:** This does not affect the `raw_*` measurements.
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the clock if paused.
    #[inline]
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// Returns `true` if the clock is currently paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
