//! Projectile system for player shooting
//! 
//! This module handles projectiles fired by the player and enemies

use raylib::prelude::*;

/// Animation state for projectile sprites
struct ProjectileAnimation {
    current_frame: usize,
    frame_timer: f32,
    frame_duration: f32,
    total_frames: usize,
    frame_sequence: Option<Vec<usize>>, 
}

impl ProjectileAnimation {
    
    fn new(total_frames: usize, frame_duration: f32) -> Self {
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration,
            total_frames,
            frame_sequence: None,
        }
    }
    
    /// Create animation with custom frame sequence
    fn with_sequence(frame_sequence: Vec<usize>, frame_duration: f32) -> Self {
        let total_frames = frame_sequence.len();
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration,
            total_frames,
            frame_sequence: Some(frame_sequence),
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.total_frames;
        }
    }
    
    fn get_frame_index(&self) -> usize {
        if let Some(ref sequence) = self.frame_sequence {
            sequence[self.current_frame]
        } else {
            self.current_frame
        }
    }
}

/// Represents a projectile in the game
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub direction_x: f32,
    pub direction_y: f32,
    pub speed: f32,
    pub damage: i32,
    pub lifetime: f32,
    pub is_player_projectile: bool,
    animation: ProjectileAnimation,
}

impl Projectile {
    /// Create a new projectile
    pub fn new(x: f32, y: f32, direction_x: f32, direction_y: f32, speed: f32, damage: i32, is_player_projectile: bool) -> Self {
        let frame_duration = 0.05;
        
       
        let animation = if is_player_projectile {
            ProjectileAnimation::with_sequence(vec![0, 1, 2, 3], frame_duration)
        } else {
           
            ProjectileAnimation::with_sequence(vec![0, 1, 2, 3, 4, 3, 2, 1], frame_duration)
        };
        
        Self {
            x,
            y,
            direction_x,
            direction_y,
            speed,
            damage,
            lifetime: 2.0, // 2 seconds default lifetime
            is_player_projectile,
            animation,
        }
    }
    
    /// Create a new projectile with custom frame sequence
    pub fn with_frame_sequence(x: f32, y: f32, direction_x: f32, direction_y: f32, speed: f32, damage: i32, is_player_projectile: bool, frame_sequence: Vec<usize>, frame_duration: f32) -> Self {
        let animation = ProjectileAnimation::with_sequence(frame_sequence, frame_duration);
        
        Self {
            x,
            y,
            direction_x,
            direction_y,
            speed,
            damage,
            lifetime: 2.0,
            is_player_projectile,
            animation,
        }
    }
    
    /// Create a projectile from player position and direction
    pub fn from_player(player_x: usize, player_y: usize, player_direction: usize, tile_size: i32) -> Self {
        // Convert tile position to center of tile in pixels
        let center_x = (player_x as i32 * tile_size + tile_size / 2) as f32;
        let center_y = (player_y as i32 * tile_size + tile_size / 2) as f32;
        
        // Set direction based on player facing
        let (dir_x, dir_y) = match player_direction {
            0 => (0.0, -1.0),  // North
            1 => (1.0, 0.0),   // East
            2 => (0.0, 1.0),   // South
            3 => (-1.0, 0.0),  // West
            _ => (0.0, 1.0),   // Default South
        };
        
        Self::new(
            center_x,
            center_y,
            dir_x,
            dir_y,
            300.0,  // Speed in pixels per second
            1,      // Damage
            true,   // Is player projectile
        )
    }
    
    /// Update projectile position and animation
    pub fn update(&mut self, dt: f32) {
        self.x += self.direction_x * self.speed * dt;
        self.y += self.direction_y * self.speed * dt;
        self.lifetime -= dt;
        self.animation.update(dt);
    }
    
    /// Get the current animation frame index
    pub fn get_frame_index(&self) -> usize {
        self.animation.get_frame_index()
    }
    
    /// Get frame position in sprite sheet (row, column)
    pub fn get_frame_position(&self) -> (usize, usize) {
        if self.is_player_projectile {
            let frame = self.get_frame_index();
            let row = frame / 2;
            let col = frame % 2;
            (row, col)
        } else {
            // mage-bullet-13x13.png: 1 row x 5 frames
            let frame = self.get_frame_index();
            (0, frame)
        }
    }
    
    /// Check if projectile is still active
    pub fn is_active(&self) -> bool {
        self.lifetime > 0.0
    }
    
    /// Get projectile position in tile coordinates
    pub fn get_tile_position(&self, tile_size: i32) -> (usize, usize) {
        (
            (self.x / tile_size as f32) as usize,
            (self.y / tile_size as f32) as usize,
        )
    }
    
    /// Get projectile collision rectangle
    pub fn get_collision_rect(&self) -> Rectangle {
        Rectangle {
            x: self.x - 4.0,
            y: self.y - 4.0,
            width: 8.0,
            height: 8.0,
        }
    }
    

    pub fn get_rotation_angle(&self) -> f32 {
        if !self.is_player_projectile {
            return 0.0; // Mage bullets don't need rotation
        }
        
        // Direction vectors:
        //   North (0, -1): atan2(-1, 0) = -90° → need 180° → -90° - 90° = -180° (same as 180°)
        //   East (1, 0):   atan2(0, 1) = 0° → need -90° → 0° - 90° = -90°
        //   South (0, 1):  atan2(1, 0) = 90° → need 0° → 90° - 90° = 0°
        //   West (-1, 0):  atan2(0, -1) = 180° → need 90° → 180° - 90° = 90°
        let angle_rad = f32::atan2(self.direction_y, self.direction_x);
        let angle_deg = angle_rad.to_degrees();
        
        // Arrow points south by default, so subtract 90° to align rotations
        // This gives us: North = 180°, East = -90°, South = 0°, West = 90°
        angle_deg - 90.0
    }
}

/// Manages all projectiles in the game
pub struct ProjectileSystem {
    pub projectiles: Vec<Projectile>,
    pub cooldown: f32,
    pub max_cooldown: f32,
    pub arrow_texture: Option<Texture2D>,
    pub mage_bullet_texture: Option<Texture2D>,
}

impl ProjectileSystem {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            cooldown: 0.0,
            max_cooldown: 0.4,
            arrow_texture: None,
            mage_bullet_texture: None,
        }
    }
    
    /// Set the arrow texture for player projectiles
    pub fn set_arrow_texture(&mut self, texture: Texture2D) {
        self.arrow_texture = Some(texture);
    }
    
    /// Set the mage bullet texture for enemy projectiles
    pub fn set_mage_bullet_texture(&mut self, texture: Texture2D) {
        self.mage_bullet_texture = Some(texture);
    }
    
    /// Update all projectiles
    pub fn update(&mut self, dt: f32) {
        // Update cooldown
        if self.cooldown > 0.0 {
            self.cooldown -= dt;
        }
        
        // Update projectiles
        for projectile in &mut self.projectiles {
            projectile.update(dt);
        }
        
        // Remove inactive projectiles
        self.projectiles.retain(|p| p.is_active());
    }
    
    /// Fire a projectile if cooldown allows
    pub fn fire_projectile(&mut self, projectile: Projectile) -> bool {
        if self.cooldown <= 0.0 {
            self.projectiles.push(projectile);
            self.cooldown = self.max_cooldown;
            true
        } else {
            false
        }
    }
    
    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        tile_size: i32,
        min_x: usize,
        max_x: usize,
        min_y: usize,
        max_y: usize,
    ) {
        for projectile in &self.projectiles {
            let (tile_x, tile_y) = projectile.get_tile_position(tile_size);
            if tile_x < min_x || tile_x >= max_x || tile_y < min_y || tile_y >= max_y {
                continue;
            }

            if projectile.is_player_projectile {
                // Draw arrow texture
                if let Some(ref arrow_tex) = self.arrow_texture {
                    let (frame_row, frame_col) = projectile.get_frame_position();
                    let frame_width = arrow_tex.width() / 2; // 2 frames per row
                    let frame_height = arrow_tex.height() / 2; // 2 rows
                    
                    let src = Rectangle {
                        x: (frame_col as i32 * frame_width) as f32,
                        y: (frame_row as i32 * frame_height) as f32,
                        width: frame_width as f32,
                        height: frame_height as f32,
                    };
                    
                    let dst = Rectangle {
                        x: projectile.x - frame_width as f32 / 2.0 + 14.0,
                        y: projectile.y - frame_height as f32 / 2.0 + 14.0,
                        width: frame_width as f32,
                        height: frame_height as f32,
                    };
                    
                    let rotation_origin = Vector2 {
                        x: frame_width as f32 / 2.0,
                        y: frame_height as f32 / 2.0,
                    };
                    
                    let rotation_angle = projectile.get_rotation_angle();
                    
                    d.draw_texture_pro(arrow_tex, src, dst, rotation_origin, rotation_angle, Color::WHITE);
                } else {
                    // Fallback to circle if texture not loaded
                    d.draw_circle(projectile.x as i32, projectile.y as i32, 4.0, Color::BLUE);
                }
            } else {
                // Draw mage bullet texture
                if let Some(ref bullet_tex) = self.mage_bullet_texture {
                    let (frame_row, frame_col) = projectile.get_frame_position();
                    let frame_width = bullet_tex.width() / 5; // 5 frames per row
                    let frame_height = bullet_tex.height(); // 1 row
                    
                    // Double the scale for mage bullet
                    let scaled_width = frame_width as f32 * 1.5;
                    let scaled_height = frame_height as f32 * 1.5;
                    
                    let src = Rectangle {
                        x: (frame_col as i32 * frame_width) as f32,
                        y: (frame_row as i32 * frame_height) as f32,
                        width: frame_width as f32,
                        height: frame_height as f32,
                    };
                    
                    let dst = Rectangle {
                        x: projectile.x - scaled_width / 2.0,
                        y: projectile.y - scaled_height / 2.0,
                        width: scaled_width,
                        height: scaled_height,
                    };
                    
                    d.draw_texture_pro(bullet_tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                } else {
                    // Fallback to circle if texture not loaded
                    d.draw_circle(projectile.x as i32, projectile.y as i32, 4.0, Color::RED);
                }
            }
        }
    }
}
