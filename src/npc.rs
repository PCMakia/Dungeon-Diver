//! NPC (Non-Player Character) module for HP and combat systems
//! 
//! This module handles:
//! - HP (Health Points) system for player and mobs
//! - Contact damage when player collides with enemies
//! - Entity state management (alive/dead)

/// HP and combat stats for entities
#[derive(Clone, Copy, Debug)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max_hp: i32) -> Self {
        Self {
            current: max_hp,
            max: max_hp,
        }
    }
    
    /// Take damage and return true if entity died
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.current = (self.current - damage).max(0);
        self.current == 0
    }
    
    /// Check if entity is alive
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }
    
    /// Get HP as percentage (0.0 to 1.0)
    pub fn hp_percentage(&self) -> f32 {
        if self.max > 0 {
            self.current as f32 / self.max as f32
        } else {
            0.0
        }
    }
}

/// Contact damage configuration for different entity types
pub struct ContactDamage {
    pub tank_damage: i32,      // Damage dealt by tank on contact
    pub shooter_damage: i32,    // Damage dealt by shooter on contact
    pub cooldown_time: f32,     // Seconds between damage ticks (prevents rapid damage)
}

impl ContactDamage {
    pub fn default() -> Self {
        Self {
            tank_damage: 1,
            shooter_damage: 1,
            cooldown_time: 0.5, // 0.5 seconds between damage ticks
        }
    }
}

/// Entity state with HP tracking
/// This extends MapEntity with runtime HP information
pub struct EntityState {
    pub kind: String,
    pub x: usize,
    pub y: usize,
    pub hp: Health,
    pub contact_damage_cooldown: f32, // Timer to prevent rapid damage
}

impl EntityState {
    pub fn new(kind: String, x: usize, y: usize, max_hp: i32) -> Self {
        Self {
            kind,
            x,
            y,
            hp: Health::new(max_hp),
            contact_damage_cooldown: 0.0,
        }
    }
    
    /// Update cooldown timer
    pub fn update_cooldown(&mut self, dt: f32) {
        if self.contact_damage_cooldown > 0.0 {
            self.contact_damage_cooldown = (self.contact_damage_cooldown - dt).max(0.0);
        }
    }
    
    /// Check if can deal contact damage (cooldown expired)
    pub fn can_deal_damage(&self) -> bool {
        self.contact_damage_cooldown <= 0.0 && self.hp.is_alive()
    }
    
    /// Reset damage cooldown after dealing damage
    pub fn reset_damage_cooldown(&mut self, cooldown: f32) {
        self.contact_damage_cooldown = cooldown;
    }
}

/// Default HP values for different entity types
pub struct EntityStats {
    pub player_max_hp: i32,
    pub tank_max_hp: i32,
    pub shooter_max_hp: i32,
}

impl EntityStats {
    pub fn default() -> Self {
        Self {
            player_max_hp: 10,
            tank_max_hp: 3,
            shooter_max_hp: 2,
        }
    }
    
    /// Get max HP for an entity type
    pub fn get_max_hp(&self, kind: &str) -> i32 {
        match kind {
            "player" => self.player_max_hp,
            "tank" => self.tank_max_hp,
            "shooter" => self.shooter_max_hp,
            _ => 1, // Default for unknown types
        }
    }
}

