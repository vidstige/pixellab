use std::f32::consts::PI;

pub(crate) fn cubic_in(k: f32) -> f32 { k.powi(3) }
pub(crate) fn cubic_out(k: f32) -> f32 { (k - 1.0).powi(3) + 1.0 }

pub(crate) fn elastic_in(k: f32) -> f32 {
    if k == 0.0 { return 0.0; }
    if k == 1.0 { return 1.0; }    
    -(2.0_f32.powf(10.0 * (k - 1.0))) * ((k - 1.1) * 5.0 * PI).sin()
}
pub(crate) fn elastic_out(k: f32) -> f32 {
    if k == 0.0 { return 0.0; }
    if k == 1.0 { return 1.0; }    
    //Math.pow(2, -10 * k) * Math.sin((k - 0.1) * 5 * Math.PI) + 1;
    2.0_f32.powf(-10.0 * k) * ((k - 0.1) * 5.0 * PI).sin() + 1.0
}