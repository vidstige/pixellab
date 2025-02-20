#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::PixelLab;

mod fields;

mod hex;

mod time;
mod nodes {
    pub mod node;
    pub mod bezier;
}
