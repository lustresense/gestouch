use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GestureKind { Swipe, Tap }
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction { Left, Right, Up, Down, None }

#[derive(Clone, Copy, Debug)]
pub struct Contact { pub id: u32, pub x: i32, pub y: i32 }
#[derive(Clone, Debug)]
pub struct Frame { pub device: usize, pub timestamp_ms: u64, pub contacts: Vec<Contact>, pub complete: bool }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Recognized { pub kind: GestureKind, pub fingers: u8, pub direction: Direction }

struct Candidate {
    device: usize,
    begun_ms: u64,
    last_ms: u64,
    start: HashMap<u32, (i32, i32)>,
    end: HashMap<u32, (i32, i32)>,
    count: usize,
    invalid: bool,
}

/// Coordinates are normalized to 0..10000 by the HID layer. Only a complete
/// release of *all* tracked contacts can emit one gesture. Never repeat while held.
pub struct Recognizer {
    candidate: Option<Candidate>,
    last_fire_ms: Option<u64>,
    swipe_threshold: i32,
    tap_tolerance: i32,
    timeout_ms: u64,
    debounce_ms: u64,
}
impl Default for Recognizer {
    fn default() -> Self { Self::new() }
}
impl Recognizer {
    pub fn new() -> Self {
        Self { candidate: None, last_fire_ms: None, swipe_threshold: 1150,
            tap_tolerance: 280, timeout_ms: 1600, debounce_ms: 180 }
    }
    pub fn reset(&mut self) { self.candidate = None; }
    pub fn update(&mut self, frame: Frame) -> Option<Recognized> {
        if !frame.complete { self.reset(); return None; }
        if frame.contacts.len() > 5 { self.reset(); return None; }
        let unique: HashSet<_> = frame.contacts.iter().map(|c| c.id).collect();
        if unique.len() != frame.contacts.len() { self.reset(); return None; }
        if frame.contacts.is_empty() {
            let c = self.candidate.take()?;
            if c.invalid || c.count < 2 || c.count > 5 ||
                frame.timestamp_ms.saturating_sub(c.begun_ms) > self.timeout_ms ||
                frame.timestamp_ms.saturating_sub(c.last_ms) > 500 { return None; }
            if self.last_fire_ms.is_some_and(|last| frame.timestamp_ms.saturating_sub(last) < self.debounce_ms) { return None; }
            let mut dx_sum: i64 = 0;
            let mut dy_sum: i64 = 0;
            let mut max_motion: i32 = 0;
            let mut disjoint = false;
            for (id, &(sx, sy)) in &c.start {
                let Some(&(ex, ey)) = c.end.get(id) else { disjoint = true; break; };
                let dx = ex - sx; let dy = ey - sy;
                dx_sum += i64::from(dx); dy_sum += i64::from(dy);
                max_motion = max_motion.max(dx.abs().max(dy.abs()));
            }
            if disjoint || c.start.len() != c.count || c.end.len() != c.count { return None; }
            let duration = frame.timestamp_ms.saturating_sub(c.begun_ms);
            let (kind, direction) = if max_motion <= self.tap_tolerance && duration <= 350 {
                (GestureKind::Tap, Direction::None)
            } else {
                let dx = dx_sum / c.count as i64;
                let dy = dy_sum / c.count as i64;
                let threshold = i64::from(self.swipe_threshold);
                if dx.abs().max(dy.abs()) < threshold { return None; }
                // Each participating finger must move coherently; no one-finger triggers.
                if c.start.iter().any(|(id, &(sx, sy))| {
                    let (ex, ey) = c.end[id];
                    let delta = if dx.abs() >= dy.abs() { i64::from(ex - sx) } else { i64::from(ey - sy) };
                    delta.signum() != (if dx.abs() >= dy.abs() { dx } else { dy }).signum() || delta.abs() < threshold / 2
                }) { return None; }
                let d = if dx.abs() >= dy.abs() {
                    if dx < 0 { Direction::Left } else { Direction::Right }
                } else if dy < 0 { Direction::Up } else { Direction::Down };
                (GestureKind::Swipe, d)
            };
            self.last_fire_ms = Some(frame.timestamp_ms);
            return Some(Recognized { kind, fingers: c.count as u8, direction });
        }
        let points: HashMap<u32, (i32, i32)> = frame.contacts.iter().map(|c| (c.id, (c.x, c.y))).collect();
        match &mut self.candidate {
            None => self.candidate = Some(Candidate { device: frame.device, begun_ms: frame.timestamp_ms,
                last_ms: frame.timestamp_ms, count: points.len(), start: points.clone(), end: points, invalid: false }),
            Some(c) => {
                if c.device != frame.device || frame.timestamp_ms < c.last_ms ||
                    frame.timestamp_ms.saturating_sub(c.begun_ms) > self.timeout_ms {
                    self.reset(); return None;
                }
                // Staggered finger-down is normal, but after 200 ms new contacts invalidate it.
                if points.len() > c.count {
                    if frame.timestamp_ms.saturating_sub(c.begun_ms) > 200 { c.invalid = true; }
                    c.count = points.len(); c.start = points.clone();
                } else if points.len() < c.count { c.invalid = true; }
                if points.keys().any(|id| !c.start.contains_key(id)) { c.invalid = true; }
                c.last_ms = frame.timestamp_ms;
                c.end = points;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(t: u64, dx: i32, count: u32) -> Frame {
        Frame { device: 1, timestamp_ms: t, complete: true,
            contacts: (0..count).map(|id| Contact { id, x: 1000 + id as i32 * 100 + dx, y: 3000 }).collect() }
    }
    #[test] fn three_finger_swipe_right_once() {
        let mut r = Recognizer::new();
        assert_eq!(r.update(f(0, 0, 3)), None);
        assert_eq!(r.update(f(50, 1600, 3)), None);
        assert_eq!(r.update(f(100, 1600, 0)), Some(Recognized { kind: GestureKind::Swipe, fingers: 3, direction: Direction::Right }));
        assert_eq!(r.update(f(120, 1600, 0)), None);
    }
    #[test] fn one_finger_and_malformed_never_fire() {
        let mut r = Recognizer::new(); r.update(f(0, 0, 1));r.update(f(50, 2000, 1));assert_eq!(r.update(f(70, 0, 0)), None);
        let mut x = f(200, 0, 3);x.complete = false; assert_eq!(r.update(x), None);
    }
    #[test] fn tap_is_distinct_from_swipe() {
        let mut r = Recognizer::new();r.update(f(0, 0, 4));r.update(f(100, 40, 4));
        assert_eq!(r.update(f(120, 0, 0)), Some(Recognized {kind:GestureKind::Tap,fingers:4,direction:Direction::None}));
    }
    #[test] fn staggered_finger_contact() {
        let mut r = Recognizer::new();r.update(f(0,0,1));r.update(f(30,0,2));r.update(f(60,0,3));r.update(f(90,1400,3));
        assert_eq!(r.update(f(120,0,0)).unwrap().fingers,3);
    }
    #[test] fn conflicting_motion_does_not_trigger() {
        let mut r=Recognizer::new();r.update(f(0,0,3));let mut mixed=f(100,1800,3);mixed.contacts[0].x=1000;
        r.update(mixed);assert_eq!(r.update(f(110,0,0)),None);
    }
}
