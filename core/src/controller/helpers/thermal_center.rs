use core::f32::consts::PI;

use heapless::Vec;
#[allow(unused_imports)]
use micromath::F32Ext;

use crate::{
    model::GpsState,
    CoreModel,
};

const TWO_PI: f32 = 2.0 * PI;
const EARTH_RADIUS_M: f64 = 6_371_000.0;
const MAX_SAMPLES: usize = 128;
const MIN_SAMPLES_PER_CIRCLE: usize = 18;
const MIN_GROUND_SPEED_MPS: f32 = 5.0;
const MIN_WEIGHT_SUM: f32 = 0.5;
const SMOOTHING_ALPHA: f32 = 0.3;

#[derive(Clone, Copy, Default)]
struct CircleSample {
    east_m: f32,
    north_m: f32,
    climb: f32,
}

#[derive(Clone, Copy, Default)]
pub struct ThermalShiftEstimate {
    pub east_m: f32,
    pub north_m: f32,
    pub distance_m: f32,
    pub confidence: f32,
    pub valid: bool,
}

pub struct ThermalCenterTracker {
    samples: Vec<CircleSample, MAX_SAMPLES>,
    ref_lat_rad: f64,
    ref_lon_rad: f64,
    reference_valid: bool,
    last_yaw_rad: Option<f32>,
    accumulated_turn_rad: f32,
    smoothed_shift_east_m: f32,
    smoothed_shift_north_m: f32,
    initialized: bool,
}

impl Default for ThermalCenterTracker {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            ref_lat_rad: 0.0,
            ref_lon_rad: 0.0,
            reference_valid: false,
            last_yaw_rad: None,
            accumulated_turn_rad: 0.0,
            smoothed_shift_east_m: 0.0,
            smoothed_shift_north_m: 0.0,
            initialized: false,
        }
    }
}

impl ThermalCenterTracker {
    pub fn update(&mut self, cm: &CoreModel, is_circling: bool) -> ThermalShiftEstimate {
        if !is_circling
            || cm.sensor.gps_state == GpsState::NoGps
            || cm.sensor.gps_ground_speed.to_m_s() < MIN_GROUND_SPEED_MPS
        {
            self.reset_circle();
            return ThermalShiftEstimate::default();
        }

        let lat = cm.sensor.gps_lat.0.to_rad();
        let lon = cm.sensor.gps_lon.0.to_rad();
        let yaw = cm.sensor.euler_yaw.to_radians();
        let climb = cm.sensor.climb_rate.to_m_s();

        if !self.reference_valid {
            self.ref_lat_rad = lat;
            self.ref_lon_rad = lon;
            self.reference_valid = true;
            self.last_yaw_rad = Some(yaw);
            self.accumulated_turn_rad = 0.0;
            self.samples.clear();
        }

        let (east_m, north_m) = local_xy_m(self.ref_lat_rad, self.ref_lon_rad, lat, lon);
        let _ = self.samples.push(CircleSample {
            east_m,
            north_m,
            climb,
        });

        let mut circle_completed = false;
        if let Some(last_yaw) = self.last_yaw_rad {
            let delta = unwrap_angle(yaw - last_yaw);
            self.accumulated_turn_rad += delta;
            if self.accumulated_turn_rad.abs() >= TWO_PI {
                circle_completed = true;
            }
        }
        self.last_yaw_rad = Some(yaw);

        if !circle_completed {
            return ThermalShiftEstimate::default();
        }

        let result = self.finalize_circle();

        self.ref_lat_rad = lat;
        self.ref_lon_rad = lon;
        self.last_yaw_rad = Some(yaw);
        self.accumulated_turn_rad = 0.0;
        self.samples.clear();

        result
    }

    fn finalize_circle(&mut self) -> ThermalShiftEstimate {
        if self.samples.len() < MIN_SAMPLES_PER_CIRCLE {
            return ThermalShiftEstimate::default();
        }

        let count = self.samples.len() as f32;
        let mut mean_east = 0.0;
        let mut mean_north = 0.0;
        let mut mean_climb = 0.0;
        for sample in &self.samples {
            mean_east += sample.east_m;
            mean_north += sample.north_m;
            mean_climb += sample.climb;
        }
        mean_east /= count;
        mean_north /= count;
        mean_climb /= count;

        let mut weighted_east = 0.0;
        let mut weighted_north = 0.0;
        let mut weight_sum = 0.0;
        for sample in &self.samples {
            let weight = (sample.climb - mean_climb).max(0.0);
            if weight > 0.0 {
                weighted_east += weight * sample.east_m;
                weighted_north += weight * sample.north_m;
                weight_sum += weight;
            }
        }

        if weight_sum < MIN_WEIGHT_SUM {
            return ThermalShiftEstimate::default();
        }

        let best_east = weighted_east / weight_sum;
        let best_north = weighted_north / weight_sum;
        let shift_east = best_east - mean_east;
        let shift_north = best_north - mean_north;

        if !self.initialized {
            self.smoothed_shift_east_m = shift_east;
            self.smoothed_shift_north_m = shift_north;
            self.initialized = true;
        } else {
            self.smoothed_shift_east_m =
                self.smoothed_shift_east_m * (1.0 - SMOOTHING_ALPHA) + shift_east * SMOOTHING_ALPHA;
            self.smoothed_shift_north_m = self.smoothed_shift_north_m * (1.0 - SMOOTHING_ALPHA)
                + shift_north * SMOOTHING_ALPHA;
        }

        let distance_m = (self.smoothed_shift_east_m * self.smoothed_shift_east_m
            + self.smoothed_shift_north_m * self.smoothed_shift_north_m)
            .sqrt();
        let coverage = (self.samples.len() as f32 / 32.0).clamp(0.0, 1.0);
        let signal = (weight_sum / (count.max(1.0) * 0.6)).clamp(0.0, 1.0);
        let confidence = coverage.min(signal);

        ThermalShiftEstimate {
            east_m: self.smoothed_shift_east_m,
            north_m: self.smoothed_shift_north_m,
            distance_m,
            confidence,
            valid: confidence > 0.2 && distance_m > 3.0,
        }
    }

    fn reset_circle(&mut self) {
        self.samples.clear();
        self.reference_valid = false;
        self.last_yaw_rad = None;
        self.accumulated_turn_rad = 0.0;
    }
}

fn unwrap_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= TWO_PI;
    }
    while angle < -PI {
        angle += TWO_PI;
    }
    angle
}

fn local_xy_m(ref_lat: f64, ref_lon: f64, lat: f64, lon: f64) -> (f32, f32) {
    let d_lat = lat - ref_lat;
    let d_lon = lon - ref_lon;
    let mean_lat = (0.5 * (lat + ref_lat)) as f32;
    let north = d_lat * EARTH_RADIUS_M;
    let east = d_lon * EARTH_RADIUS_M * mean_lat.cos() as f64;
    (east as f32, north as f32)
}
