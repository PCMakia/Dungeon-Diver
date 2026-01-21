//! Raylib helper functions
use raylib::prelude::*;
use rand::Rng;
use std::path::Path;
use std::ffi::CString;
use raylib::ffi;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};


pub fn check_collision_point_rect(point: &Vector2, rect: &Rectangle) -> bool {
    let in_x = point.x >= rect.x && point.x <= rect.x + rect.width;
    let in_y = point.y >= rect.y && point.y <= rect.y + rect.height;

    return in_x && in_y;
}

pub fn random_point(width: i32, height: i32) -> Vector2 {
    let mut rng = rand::rng();

    let x = rng.random_range(0..width);
    let y = rng.random_range(0..height);

    Vector2{x: x as f32, y: y as f32}
}

/// Resolve asset path by trying multiple relative path options
pub fn resolve_asset_path(path: &str) -> String {
    let paths_to_try = vec![
        path.to_string(),
        format!("../{}", path),
        format!("../../{}", path),
    ];
    
    for try_path in &paths_to_try {
        if Path::new(try_path).exists() {
            return try_path.clone();
        }
    }
    
    // If none found, return original path
    path.to_string()
}

 /// help loading in sfx
pub unsafe fn load_sound(path: &str) -> Option<ffi::Sound> {
    let c = CString::new(path).unwrap();
    let sound = ffi::LoadSound(c.as_ptr());
    if sound.frameCount == 0 {
        None
    } else {
        Some(sound)
    }
}

// #region agent log
pub fn agent_log(hypothesis_id: &str, location: &str, message: &str, data: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!(
        "{{\"sessionId\":\"debug-session\",\"runId\":\"run1\",\"hypothesisId\":\"{}\",\"location\":\"{}\",\"message\":\"{}\",\"data\":{},\"timestamp\":{}}}\n",
        hypothesis_id,
        location.replace('\\', "\\\\").replace('"', "\\\""),
        message.replace('\\', "\\\\").replace('"', "\\\""),
        data,
        ts
    );
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug_ndjson.log")
    {
        let _ = f.write_all(line.as_bytes());
    }
}
// #endregion agent log