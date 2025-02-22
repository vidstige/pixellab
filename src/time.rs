use std::{iter::Sum, ops::Add};

pub struct Duration {
    millis: u32,
}
impl Duration {
    pub fn from_secs(seconds: f32) -> Duration {
        Self { millis: (1000.0 * seconds) as u32 }
    }
    pub fn from_millis(millis: u32) -> Duration {
        Self { millis, }
    }
    pub fn as_millis(&self) -> u32 { self.millis }
}
impl Add for &Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration { millis: self.millis + rhs.millis }
    }
}
impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        Duration::from_millis(iter.map(|d| d.millis).sum())
    }
}
pub struct Instant {
    pub millis: u32, // TODO: should not be public probably
}
impl Default for Instant {
    fn default() -> Self {
        Self { millis: Default::default() }
    }
}
impl Instant {
    pub(crate) fn zero() -> Self { Self { millis: 0, } }
    
    pub(crate) fn after(&self, duration: &Duration) -> Instant {
        Self { millis: self.millis + duration.millis, }
    }
}