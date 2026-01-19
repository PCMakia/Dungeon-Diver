//! Projectile system for player shooting
//! 
//! This module handles projectiles fired by the player and enemies

use raylib::prelude::*;

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
}

impl Projectile {
    /// Create a new projectile
    pub fn new(x: f32, y: f32, direction_x: f32, direction_y: f32, speed: f32, damage: i32, is_player_projectile: bool) -> Self {
        Self {
            x,
            y,
            direction_x,
            direction_y,
            speed,
            damage,
            lifetime: 2.0, // 2 seconds default lifetime
            is_player_projectile,
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
    
    /// Update projectile position
    pub fn update(&mut self, dt: f32) {
        self.x += self.direction_x * self.speed * dt;
        self.y += self.direction_y * self.speed * dt;
        self.lifetime -= dt;
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
}

/// Manages all projectiles in the game
pub struct ProjectileSystem {
    pub projectiles: Vec<Projectile>,
    pub cooldown: f32,
    pub max_cooldown: f32,
}

impl ProjectileSystem {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            cooldown: 0.0,
            max_cooldown: 0.3, // 0.3 seconds between shots
        }
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
    
    /// Draw all projectiles
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for projectile in &self.projectiles {
            let color = if projectile.is_player_projectile {
                Color::BLUE
            } else {
                Color::RED
            };
            
            d.draw_circle(
                projectile.x as i32,
                projectile.y as i32,
                4.0,
                color,
            );
        }
    }
}
