//! IKI (Inter-Keystroke Interval) Adaptive Speed Sensor.
//!
//! Uses Exponential Moving Average (EMA) to compute real-time typing rhythm
//! and automatically calibrate backspace delays for Electron and browser DOMs.

/// Exponential Moving Average sensor tracking inter-keystroke intervals.
#[derive(Debug, Clone)]
pub struct IkiSensor {
    last_timestamp_ms: f64,
    ema_interval_ms: f64,
    alpha: f64, // smoothing factor (0.25)
}

impl IkiSensor {
    /// Creates a new IkiSensor.
    pub fn new() -> Self {
        Self {
            last_timestamp_ms: 0.0,
            ema_interval_ms: 120.0, // baseline average
            alpha: 0.25,
        }
    }

    /// Records a keystroke timestamp and returns the calculated adaptive delay in ms.
    pub fn record_stroke(&mut self, timestamp_ms: f64) -> u64 {
        if self.last_timestamp_ms > 0.0 {
            let delta = (timestamp_ms - self.last_timestamp_ms).max(1.0).min(2000.0);
            self.ema_interval_ms = (self.alpha * delta) + ((1.0 - self.alpha) * self.ema_interval_ms);
        }
        self.last_timestamp_ms = timestamp_ms;

        // Dynamic delay calculation:
        // High typing speed (< 80ms) -> slight safety barrier delay (4-8ms) for web DOMs
        // Slow typing (> 200ms) -> fast direct commit (1.5ms)
        if self.ema_interval_ms < 60.0 {
            8
        } else if self.ema_interval_ms < 100.0 {
            4
        } else {
            2
        }
    }

    /// Returns current moving average interval in ms.
    pub fn moving_average(&self) -> f64 {
        self.ema_interval_ms
    }

    /// Calculates estimated Words Per Minute (WPM).
    pub fn estimated_wpm(&self) -> f64 {
        if self.ema_interval_ms <= 0.0 {
            0.0
        } else {
            // standard 5 chars per word
            (60_000.0 / (self.ema_interval_ms * 5.0)).min(250.0)
        }
    }
}

impl Default for IkiSensor {
    fn default() -> Self {
        Self::new()
    }
}
