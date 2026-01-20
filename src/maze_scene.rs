use raylib::prelude::*;

use crate::menu_scene::WinScene;
use crate::scenes::{Scene, SceneSwitch};
use crate::game_data::GameData;
use crate::{is_floor_tile, is_wall_tile};
use crate::npc::{Health, EntityState, ContactDamage, EntityStats, AIState};
use crate::projectile::{Projectile, ProjectileSystem};
use std::fs::File;
use std::io::Read;
use rand::Rng;
use std::env;
use std::path::Path;
use serde::Deserialize;

/// Animation state for sprite-based entities
struct AnimationState {
    current_frame: usize,
    frame_timer: f32,
    frame_duration: f32,  // seconds per frame
    total_frames: usize,
    is_playing: bool,     // false for death animations that play once
    frame_sequence: Option<Vec<usize>>, // Custom frame sequence (e.g., [0,1,2,3,2,1,0] for ping-pong)
}

impl AnimationState {
    
    /// Create animation with custom frame sequence
    fn with_sequence(frame_sequence: Vec<usize>, frame_duration: f32) -> Self {
        let total_frames = frame_sequence.len();
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            frame_duration,
            total_frames,
            is_playing: true,
            frame_sequence: Some(frame_sequence),
        }
    }
    
    fn update(&mut self, dt: f32) {
        if !self.is_playing {
            return;
        }
        self.frame_timer += dt;
        if self.frame_timer >= self.frame_duration {
            self.frame_timer = 0.0;
            self.current_frame = (self.current_frame + 1) % self.total_frames;
        }
    }
    
    /// Get the actual frame index to use
    fn get_frame_index(&self) -> usize {
        if let Some(ref sequence) = self.frame_sequence {
            sequence[self.current_frame]
        } else {
            self.current_frame
        }
    }
    
    
}

/// Resolve asset path by trying multiple relative path options
fn resolve_asset_path(path: &str) -> String {
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

#[derive(Deserialize)]
pub struct MapData {
    pub grid_w: usize,
    pub grid_h: usize,
    pub tile_size_px: i32,
    pub tiles: Vec<Vec<i32>>,
    pub entities: Vec<MapEntity>,
}

#[derive(Deserialize)]
pub struct MapEntity {
    pub kind: String,
    pub x: usize,
    pub y: usize,
}


pub fn load_map(path: &str) -> MapData {
    let resolved_path = resolve_asset_path(path);
    
    let mut file = File::open(&resolved_path)
        .unwrap_or_else(|e| {
            let current_dir = env::current_dir().unwrap_or_default();
            panic!("Failed to open map file at '{}' (resolved from '{}'): {}. Current working directory: {:?}", 
                   resolved_path, path, e, current_dir);
        });
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .unwrap_or_else(|e| {
            panic!("Failed to read map file '{}': {}", resolved_path, e);
        });
    serde_json::from_str(&contents)
        .unwrap_or_else(|e| {
            panic!("Failed to parse map JSON from '{}': {}", resolved_path, e);
        })
}

pub struct MazeScene {
    pub map_path: String,   

    map: MapData,

    tileset: Option<Texture2D>, // Use Option since we can't load it in from_map
    tile_size: i32,

    player_x: usize,
    player_y: usize,
    player_direction: usize, // 0=North, 1=East, 2=South, 3=West
    
    // Camera system
    camera: Camera2D,
    fov_radius: i32, // tiles
    
    // Tick-based game logic
    tick_timer: f32,
    tick_rate: f32, // seconds per tick
    
    // Queued movement for tick system
    queued_move: Option<(usize, usize)>,
    
    // Gamepad input tracking
    last_gamepad_direction: Option<(i32, i32)>,
    
    // Sprite sheets for animated entities
    player_sprite: Option<Texture2D>,
    tank_sprite: Option<Texture2D>,
    shooter_sprite: Option<Texture2D>,
    
    // Animation states
    player_anim: AnimationState,
    tank_anim: AnimationState,
    shooter_anim: AnimationState,
    
    
    // HP and combat system
    player_hp: Health,
    entity_states: Vec<EntityState>,
    contact_damage: ContactDamage,
    entity_stats: EntityStats,
    player_stun_ticks: i32, // Number of ticks player is stunned (0 = not stunned)
    
    // Projectile system for shooting
    projectile_system: ProjectileSystem,

    // Temporary death animations for defeated enemies
    death_anims: Vec<DeathAnim>,
}

struct DeathAnim {
    kind: String,
    x: usize,
    y: usize,
    timer: f32,
}





impl MazeScene {
    pub fn from_map(path: String) -> Self {
        Self {
            map_path: path.clone(),
            map: load_map(&path),
            tileset: None, 
            tile_size: 32,
            player_x: 0,
            player_y: 0,
            player_direction: 2, // Start facing South
            // Initialize camera centered on origin 
            camera: Camera2D {
                target: Vector2::zero(),
                offset: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            },
            fov_radius: 7, 
            tick_timer: 0.0,
            tick_rate: 0.22, // increase for slower tick game
            queued_move: None,
            last_gamepad_direction: None,
            player_sprite: None,
            tank_sprite: None,
            shooter_sprite: None,
            // Player: 3 frames per row, 4 rows (directions)
            player_anim: AnimationState::with_sequence(vec![0, 1, 2, 2, 1, 0], 0.15),
            // Tank: 3 frames per row, varies by direction
            tank_anim: AnimationState::with_sequence(vec![0, 1, 2, 2, 1, 0], 0.15),
            // Shooter: 4 frames idle:
            shooter_anim: AnimationState::with_sequence(vec![0, 1, 2, 3, 3, 2, 1, 0], 0.2),
         
            // Initialize HP system
            player_hp: Health::new(10), 
            entity_states: Vec::new(),
            contact_damage: ContactDamage::default(),
            entity_stats: EntityStats::default(),
            player_stun_ticks: 0,
            
            // Initialize projectile system
            projectile_system: ProjectileSystem::new(),

        
            death_anims: Vec::new(),
        }
    }


    
    
    
    // Check if a move to the given position is valid
    fn is_valid_move(&self, x: usize, y: usize) -> bool {
        if x >= self.map.grid_w || y >= self.map.grid_h {
            return false;
        }
        let tid = self.map.tiles[y][x];
        tid >= 0 && !is_wall_tile(tid)
    }
    
    /// Check if a tile is within the player's field of view
    fn in_fov(&self, x: usize, y: usize) -> bool {
        // Bounds check first
        if x >= self.map.grid_w || y >= self.map.grid_h {
            return false;
        }
        
        // Calculate squared distance
        let dx = x as i32 - self.player_x as i32;
        let dy = y as i32 - self.player_y as i32;
        let dist_squared = dx * dx + dy * dy;
        let radius_squared = self.fov_radius * self.fov_radius;
        
        dist_squared <= radius_squared
    }
    
    /// Calculate visible tile bounds for optimized drawing
    /// Returns (min_x, max_x, min_y, max_y) clamped to map bounds
    fn get_visible_bounds(&self) -> (usize, usize, usize, usize) {
        let min_x = self.player_x.saturating_sub(self.fov_radius as usize);
        let max_x = (self.player_x + self.fov_radius as usize + 1).min(self.map.grid_w);
        let min_y = self.player_y.saturating_sub(self.fov_radius as usize);
        let max_y = (self.player_y + self.fov_radius as usize + 1).min(self.map.grid_h);
        
        (min_x, max_x, min_y, max_y)
    }
    
    /// Update camera to follow player (centered on screen)
    fn update_camera(&mut self, data: &GameData) {
        // Convert player tile position to world pixel position (center of tile)
        self.camera.target = Vector2::new(
            (self.player_x as i32 * self.tile_size + self.tile_size / 2) as f32,
            (self.player_y as i32 * self.tile_size + self.tile_size / 2) as f32,
        );
        
        // Offset camera so player appears centered on screen
        self.camera.offset = Vector2::new(
            (data.screen_width / 2) as f32,
            (data.screen_height / 2) as f32,
        );
    }
    
    /// Process player movement on game tick
    fn update_player(&mut self) {
        // Decrease stun counter
        if self.player_stun_ticks > 0 {
            self.player_stun_ticks -= 1;
            // Clear queued move if player is stunned
            self.queued_move = None;
            return;
        }
        
        if let Some((new_x, new_y)) = self.queued_move.take() {
            // Check if target tile is blocked by a monster
            // Find monster index first to avoid borrowing issues
            let monster_index = self.entity_states.iter()
                .position(|e| e.x == new_x && e.y == new_y && e.hp.is_alive() && (e.kind == "tank" || e.kind == "shooter"));
            
            if let Some(idx) = monster_index {
                let monster_kind = self.entity_states[idx].kind.clone();
                
                // Deal contact damage
                let damage = match monster_kind.as_str() {
                    "tank" => self.contact_damage.tank_damage,
                    "shooter" => self.contact_damage.shooter_damage,
                    _ => 0,
                };
                
                if damage > 0 {
                    let player_died = self.player_hp.take_damage(damage);
                    
                    // Reset monster's cooldown
                    self.entity_states[idx].reset_damage_cooldown(self.contact_damage.cooldown_time);
                    
                    // Stun player for 2 ticks
                    self.player_stun_ticks = 2;
                    
                    // Check for player death
                    if player_died {
                        // TODO: Handle player death (game over screen)
                        println!("Player died! HP: {}/{}", self.player_hp.current, self.player_hp.max);
                    }
                }
                
                // Block movement - don't move to that tile
                return;
            }
            
            // No monster blocking, proceed with normal movement check
            if self.is_valid_move(new_x, new_y) {
                // Update player direction based on movement
                if new_x > self.player_x {
                    self.player_direction = 1; // East
                } else if new_x < self.player_x {
                    self.player_direction = 3; // West
                } else if new_y > self.player_y {
                    self.player_direction = 2; // South
                } else if new_y < self.player_y {
                    self.player_direction = 0; // North
                }
                
                self.player_x = new_x;
                self.player_y = new_y;
            }
        }
    }
    
    /// Draw animated sprite from sprite sheet
    fn draw_animated_sprite(
        &self,
        d: &mut RaylibDrawHandle,
        sprite: &Texture2D,
        frame_x: usize,
        frame_y: usize,
        frame_width: i32,
        frame_height: i32,
        x: usize,
        y: usize,
        sprite_width: i32,
        sprite_height: i32,
    ) {
        let src = Rectangle {
            x: (frame_x * frame_width as usize) as f32,
            y: (frame_y * frame_height as usize) as f32,
            width: frame_width as f32,
            height: frame_height as f32,
        };
        
        // Center sprite on tile
        let tile_center_x = (x as i32 * self.tile_size + self.tile_size / 2) as f32;
        let tile_center_y = (y as i32 * self.tile_size + self.tile_size / 2) as f32;
        
        let dst = Rectangle {
            x: tile_center_x - sprite_width as f32 / 2.0,
            y: tile_center_y - sprite_height as f32 / 2.0,
            width: sprite_width as f32,
            height: sprite_height as f32,
        };
        
        d.draw_texture_pro(sprite, src, dst, Vector2::zero(), 0.0, Color::WHITE);
    }
    
    /// Static helper: Check if there's a clear path (line of sight) between two points
    /// Returns true if player and entity are in the same hallway (no walls blocking)
    fn has_line_of_sight_static(
        x1: usize, y1: usize, x2: usize, y2: usize,
        grid_w: usize, grid_h: usize, tiles: &[Vec<i32>]
    ) -> bool {
        let dx = (x2 as i32 - x1 as i32).signum();
        let dy = (y2 as i32 - y1 as i32).signum();
        
        // Check horizontal and vertical paths
        if dx == 0 {
            // Vertical path
            let start_y = y1.min(y2);
            let end_y = y1.max(y2);
            for y in start_y..=end_y {
                if y >= grid_h {
                    return false;
                }
                let tile_id = tiles[y][x1];
                if is_wall_tile(tile_id) {
                    return false;
                }
            }
            return true;
        } else if dy == 0 {
            // Horizontal path
            let start_x = x1.min(x2);
            let end_x = x1.max(x2);
            for x in start_x..=end_x {
                if x >= grid_w {
                    return false;
                }
                let tile_id = tiles[y1][x];
                if is_wall_tile(tile_id) {
                    return false;
                }
            }
            return true;
        }
        
        // Diagonal or complex path - check if it's a valid L-shaped path (same hallway)
        // Check horizontal then vertical
        let mut valid = true;
        for x in (x1.min(x2))..=(x1.max(x2)) {
            if x >= grid_w || y1 >= grid_h {
                valid = false;
                break;
            }
            let tile_id = tiles[y1][x];
            if is_wall_tile(tile_id) {
                valid = false;
                break;
            }
        }
        if valid {
            for y in (y1.min(y2))..=(y1.max(y2)) {
                if y >= grid_h || x2 >= grid_w {
                    valid = false;
                    break;
                }
                let tile_id = tiles[y][x2];
                if is_wall_tile(tile_id) {
                    valid = false;
                    break;
                }
            }
        }
        if valid {
            return true;
        }
        
        // Check vertical then horizontal
        valid = true;
        for y in (y1.min(y2))..=(y1.max(y2)) {
            if y >= grid_h || x1 >= grid_w {
                valid = false;
                break;
            }
            let tile_id = tiles[y][x1];
            if is_wall_tile(tile_id) {
                valid = false;
                break;
            }
        }
        if valid {
            for x in (x1.min(x2))..=(x1.max(x2)) {
                if x >= grid_w || y2 >= grid_h {
                    valid = false;
                    break;
                }
                let tile_id = tiles[y2][x];
                if is_wall_tile(tile_id) {
                    valid = false;
                    break;
                }
            }
        }
        
        valid
    }
    
    
    /// Calculate Manhattan distance between two points
    fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
        ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as usize
    }
    
    
    /// Static helper: Get valid adjacent tiles
    fn get_valid_adjacent_tiles_static(
        x: usize, y: usize,
        grid_w: usize, grid_h: usize,
        is_valid_move: &dyn Fn(usize, usize) -> bool,
    ) -> Vec<(usize, usize)> {
        let mut valid = Vec::new();
        
        // Check all 4 directions
        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        for (dx, dy) in &directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            
            if new_x >= 0 && new_y >= 0 && 
               new_x < grid_w as i32 && new_y < grid_h as i32 {
                let nx = new_x as usize;
                let ny = new_y as usize;
                
                if is_valid_move(nx, ny) {
                    valid.push((nx, ny));
                }
            }
        }
        
        valid
    }
    
    /// Update enemy AI on game tick
    fn update_enemies(&mut self) {
        // Enemies update even outside FOV - simulation is separate from rendering
        
        // Store actions to apply after iteration (to avoid borrowing conflicts)
        let mut projectiles_to_spawn = Vec::new();
        let player_x = self.player_x;
        let player_y = self.player_y;
        let tile_size = self.tile_size;
        let grid_w = self.map.grid_w;
        let grid_h = self.map.grid_h;
        let tiles = &self.map.tiles; // Reference to tiles for line-of-sight checks
        
        // First pass: calculate aggression zones and update states/cooldowns
        for entity in &mut self.entity_states {
            if !entity.hp.is_alive() {
                continue;
            }
            
            // Calculate distance to player
            let distance = Self::manhattan_distance(entity.x, entity.y, player_x, player_y);
            
            // Check if player is in aggression zone (6 tiles, with line of sight)
            let in_aggression_zone = distance <= 6 && 
                                     Self::has_line_of_sight_static(
                                         entity.x, entity.y, player_x, player_y,
                                         grid_w, grid_h, tiles);
            
            // Update AI state
            if in_aggression_zone && entity.ai_state == AIState::Wandering {
                entity.ai_state = AIState::Attacking;
            } else if !in_aggression_zone && entity.ai_state == AIState::Attacking {
                entity.ai_state = AIState::Wandering;
            }
            
            // Decrease cooldowns
            if entity.movement_cooldown > 0 {
                entity.movement_cooldown -= 1;
            }
            if entity.shoot_cooldown > 0 {
                entity.shoot_cooldown -= 1;
            }
            
           
           
        }
        
        // Second pass: execute AI behaviors
        // Collect entity positions for collision checking
        let entity_positions: Vec<(usize, usize)> = self.entity_states.iter()
            .filter(|e| e.hp.is_alive())
            .map(|e| (e.x, e.y))
            .collect();
        
        // Extract map data for helper functions
        let is_valid_move_fn = |x: usize, y: usize| -> bool {
            if x >= grid_w || y >= grid_h {
                return false;
            }
            let tile_id = tiles[y][x];
            tile_id >= 0 && !is_wall_tile(tile_id)
        };
        
        for entity in &mut self.entity_states {
            if !entity.hp.is_alive() {
                continue;
            }
            
            // Execute AI behavior based on state
            match entity.ai_state {
                AIState::Wandering => {
                    Self::handle_wandering_static(
                        entity, &entity_positions, 
                        grid_w, grid_h, &is_valid_move_fn);
                }
                AIState::Attacking => {
                    match entity.kind.as_str() {
                        "shooter" => {
                            if let Some(projectile) = Self::handle_shooter_attack_static(
                                entity, player_x, player_y, &entity_positions, 
                                tile_size, grid_w, grid_h, tiles, &is_valid_move_fn) {
                                projectiles_to_spawn.push(projectile);
                            }
                        }
                        "tank" => {
                            // Check if tank would move to player position and deal damage if so
                            let path = Self::find_path_static(
                                entity.x, entity.y, player_x, player_y,
                                grid_w, grid_h, &is_valid_move_fn);
                            
                            if let Some(&next_pos) = path.first() {
                                // If tank would move to player's position, deal damage but don't move
                                if next_pos.0 == player_x && next_pos.1 == player_y {
                                    if entity.can_deal_damage() {
                                        // Double damage for tank attacking player
                                        let damage = self.contact_damage.tank_damage * 2;
                                        let player_died = self.player_hp.take_damage(damage);
                                        entity.reset_damage_cooldown(self.contact_damage.cooldown_time);
                                        
                                        // Stun player for 1 ticks
                                        self.player_stun_ticks = 1;
                                        
                                        if player_died {
                                            println!("Player died from tank contact! HP: {}/{}", 
                                                    self.player_hp.current, self.player_hp.max);
                                        }
                                    }
                                    // Don't move - set cooldown but keep tank in current position
                                    if entity.movement_cooldown == 0 {
                                        entity.movement_cooldown = 5; // Move every 4 ticks
                                    }
                                } else {
                                    // Normal movement - tank not trying to step on player
                                    Self::handle_tank_attack_static(
                                        entity, player_x, player_y, &entity_positions,
                                        grid_w, grid_h, &is_valid_move_fn);
                                }
                            } else {
                                // No valid path, use normal attack behavior
                                Self::handle_tank_attack_static(
                                    entity, player_x, player_y, &entity_positions,
                                    grid_w, grid_h, &is_valid_move_fn);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Spawn projectiles after iteration
        for projectile in projectiles_to_spawn {
            self.projectile_system.fire_projectile(projectile);
        }
    }
    
    /// Static helper: Handle wandering behavior: random movement within 2 tiles of spawn
    fn handle_wandering_static(
        entity: &mut EntityState, 
        entity_positions: &[(usize, usize)],
        grid_w: usize,
        grid_h: usize,
        is_valid_move: &dyn Fn(usize, usize) -> bool,
    ) {
        // Can only move once every 10 game ticks
        if entity.movement_cooldown > 0 {
            return;
        }
        
        // Find random tile within 2 tiles radius of spawn
        let mut candidates = Vec::new();
        for dy in -2..=2 {
            for dx in -2..=2 {
                let new_x = entity.spawn_x as i32 + dx;
                let new_y = entity.spawn_y as i32 + dy;
                let distance = Self::manhattan_distance(
                    entity.spawn_x, entity.spawn_y,
                    new_x.max(0) as usize, new_y.max(0) as usize
                );
                
                if distance <= 2 && 
                   new_x >= 0 && new_y >= 0 &&
                   new_x < grid_w as i32 && 
                   new_y < grid_h as i32 {
                    let nx = new_x as usize;
                    let ny = new_y as usize;
                    if is_valid_move(nx, ny) {
                        candidates.push((nx, ny));
                    }
                }
            }
        }
        
        if !candidates.is_empty() {
            // Randomly select a candidate tile
            let mut rng = rand::rng();
            let target = candidates[rng.random_range(0..candidates.len())];
            
            // If target is blocked by a wall, find closest valid adjacent tile
            if !is_valid_move(target.0, target.1) {
                let adjacent = Self::get_valid_adjacent_tiles_static(
                    entity.x, entity.y, grid_w, grid_h, is_valid_move);
                // Randomly select from adjacent tiles
                if !adjacent.is_empty() {
                    let new_pos = adjacent[rng.random_range(0..adjacent.len())];
                    let blocked = entity_positions.contains(&new_pos);
                    if !blocked {
                        entity.x = new_pos.0;
                        entity.y = new_pos.1;
                        entity.movement_cooldown = 13;
                    }
                }
            } else {
                // Move one step towards target if valid
                let path = Self::find_path_static(
                    entity.x, entity.y, target.0, target.1,
                    grid_w, grid_h, is_valid_move);
                if let Some(next_pos) = path.get(0) {
                    // Check if next position is valid and not blocked by other entity
                    let blocked = entity_positions.contains(&(next_pos.0, next_pos.1));
                    if !blocked {
                        entity.x = next_pos.0;
                        entity.y = next_pos.1;
                        entity.movement_cooldown = 13;
                    }
                }
            }
        } else {
            // No valid candidates, try to move to a random valid adjacent tile
            let adjacent = Self::get_valid_adjacent_tiles_static(
                entity.x, entity.y, grid_w, grid_h, is_valid_move);
            if !adjacent.is_empty() {
                let mut rng = rand::rng();
                let new_pos = adjacent[rng.random_range(0..adjacent.len())];
                let blocked = entity_positions.contains(&new_pos);
                if !blocked {
                    entity.x = new_pos.0;
                    entity.y = new_pos.1;
                    entity.movement_cooldown = 13;
                }
            }
        }
    }
    
    /// Static helper: Simple pathfinding: move one step towards target
    fn find_path_static(
        start_x: usize, start_y: usize, target_x: usize, target_y: usize,
        grid_w: usize, grid_h: usize,
        is_valid_move: &dyn Fn(usize, usize) -> bool,
    ) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        let mut current_x = start_x;
        let mut current_y = start_y;
        
        // Simple greedy: move towards target
        while (current_x != target_x || current_y != target_y) && path.len() < 10 {
            let dx = target_x as i32 - current_x as i32;
            let dy = target_y as i32 - current_y as i32;
            
            let next_x = if dx != 0 {
                current_x as i32 + dx.signum()
            } else {
                current_x as i32
            };
            
            let next_y = if dy != 0 {
                current_y as i32 + dy.signum()
            } else {
                current_y as i32
            };
            
            if next_x >= 0 && next_y >= 0 &&
               next_x < grid_w as i32 &&
               next_y < grid_h as i32 &&
               is_valid_move(next_x as usize, next_y as usize) {
                path.push((next_x as usize, next_y as usize));
                current_x = next_x as usize;
                current_y = next_y as usize;
            } else {
                break;
            }
        }
        
        path
    }
    

    
    /// Static helper: Handle shooter attack behavior: position 5 tiles away and shoot
    /// Returns Some(Projectile) if a shot should be fired, None otherwise
    fn handle_shooter_attack_static(
        entity: &mut EntityState, 
        player_x: usize, 
        player_y: usize, 
        entity_positions: &[(usize, usize)],
        tile_size: i32,
        grid_w: usize,
        grid_h: usize,
        tiles: &[Vec<i32>],
        is_valid_move: &dyn Fn(usize, usize) -> bool,
    ) -> Option<Projectile> {
        let distance = Self::manhattan_distance(entity.x, entity.y, player_x, player_y);
        let desired_distance = 5;
        let mut projectile_to_fire = None;
        
        // Movement: can move once per game tick in attacking mode
        if entity.movement_cooldown == 0 {
            let dx = player_x as i32 - entity.x as i32;
            let dy = player_y as i32 - entity.y as i32;
            
            // Determine alignment direction (which axis has larger difference)
            let mut new_x = entity.x;
            let mut new_y = entity.y;
            
            if dx.abs() > dy.abs() {
                // Align Y first, then adjust X to maintain distance
                if dy != 0 {
                    new_y = (entity.y as i32 + dy.signum()).max(0) as usize;
                    if new_y >= grid_h {
                        new_y = entity.y;
                    }
                }
                // Adjust X to maintain ~5 tile distance
                if distance > desired_distance && dx > 0 {
                    new_x = entity.x.saturating_add(1).min(grid_w.saturating_sub(1));
                } else if distance > desired_distance && dx < 0 {
                    new_x = entity.x.saturating_sub(1);
                } else if distance < desired_distance && dx > 0 {
                    new_x = entity.x.saturating_sub(1);
                } else if distance < desired_distance && dx < 0 {
                    new_x = entity.x.saturating_add(1).min(grid_w.saturating_sub(1));
                }
            } else {
                // Align X first, then adjust Y
                if dx != 0 {
                    new_x = (entity.x as i32 + dx.signum()).max(0) as usize;
                    if new_x >= grid_w {
                        new_x = entity.x;
                    }
                }
                // Adjust Y to maintain ~5 tile distance
                if distance > desired_distance && dy > 0 {
                    new_y = entity.y.saturating_add(1).min(grid_h.saturating_sub(1));
                } else if distance > desired_distance && dy < 0 {
                    new_y = entity.y.saturating_sub(1);
                } else if distance < desired_distance && dy > 0 {
                    new_y = entity.y.saturating_sub(1);
                } else if distance < desired_distance && dy < 0 {
                    new_y = entity.y.saturating_add(1).min(grid_h.saturating_sub(1));
                }
            }
            
            // Validate move
            if is_valid_move(new_x, new_y) {
                let blocked = entity_positions.contains(&(new_x, new_y));
                if !blocked {
                    entity.x = new_x;
                    entity.y = new_y;
                    entity.movement_cooldown = 2; // Can move every 2 ticks 
                }
            }
        }
        
        // Shooting: once every 6 game ticks, and only if aligned with player
        if entity.shoot_cooldown == 0 {
            let dx = player_x as i32 - entity.x as i32;
            let dy = player_y as i32 - entity.y as i32;
            
            // Check if aligned (same row or same column)
            let is_aligned = (dx == 0 && dy != 0) || (dx != 0 && dy == 0);
            
            if is_aligned && Self::has_line_of_sight_static(
                entity.x, entity.y, player_x, player_y,
                grid_w, grid_h, tiles) {
                // Determine direction
                let (dir_x, dir_y) = if dx.abs() > dy.abs() {
                    (dx.signum() as f32, 0.0)
                } else {
                    (0.0, dy.signum() as f32)
                };
                
                // Create projectile
                let center_x = (entity.x as i32 * tile_size + tile_size / 2) as f32;
                let center_y = (entity.y as i32 * tile_size + tile_size / 2) as f32;
                
                let mut projectile = Projectile::new(
                    center_x,
                    center_y,
                    dir_x,
                    dir_y,
                    100.0,  // Speed
                    1,      // Damage
                    false,  // Not player projectile (mage bullet)
                );
                // Extend lifetime to 10 seconds for shooter projectiles
                projectile.lifetime = 10.0;
                
                entity.shoot_cooldown = 6; // Shoot every 6 ticks
                projectile_to_fire = Some(projectile);
            }
        }
        
        projectile_to_fire
    }
    
    /// Static helper: Handle tank attack behavior: chase player and deal contact damage
    fn handle_tank_attack_static(
        entity: &mut EntityState, 
        player_x: usize, 
        player_y: usize, 
        entity_positions: &[(usize, usize)],
        grid_w: usize,
        grid_h: usize,
        is_valid_move: &dyn Fn(usize, usize) -> bool,
    ) {
        // Can only move once every 6 game ticks
        if entity.movement_cooldown > 0 {
            return;
        }
        
        // Move towards player
        let path = Self::find_path_static(
            entity.x, entity.y, player_x, player_y,
            grid_w, grid_h, is_valid_move);
        if let Some(&next_pos) = path.first() {
            // Don't allow tank to step on player's position
            if next_pos.0 == player_x && next_pos.1 == player_y {
                // Skip movement - tank will deal damage, but not move here
                return;
            }
            
            // Check if blocked by another entity
            let blocked = entity_positions.contains(&(next_pos.0, next_pos.1));
            
            if !blocked {
                entity.x = next_pos.0;
                entity.y = next_pos.1;
                entity.movement_cooldown = 6; // Move every 6 ticks
            }
        }
    }
    
    /// Handle player shooting
    fn handle_shooting(&mut self, rl: &RaylibHandle) -> bool {
        // Check for shooting input (Space, Enter, or gamepad button)
        let should_shoot = rl.is_key_pressed(KeyboardKey::KEY_SPACE) || 
                           rl.is_key_pressed(KeyboardKey::KEY_ENTER) ||
                           (rl.is_gamepad_available(0) && (
                               rl.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN) || // A/X button
                               rl.is_gamepad_button_pressed(0, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT)   // B/Circle button
                           ));
        
        if should_shoot && self.player_stun_ticks <= 0 {
            // Create projectile based on player position and direction
            let projectile = Projectile::from_player(
                self.player_x,
                self.player_y,
                self.player_direction,
                self.tile_size
            );
            
            // Try to fire projectile (returns true if successful, false if on cooldown)
            return self.projectile_system.fire_projectile(projectile);
        }
        
        false
    }
    
    /// Update projectiles and check for collisions
    /// `data` is used to award points when enemies are defeated.
    fn update_projectiles(&mut self, dt: f32, data: &mut GameData) {
        // Update all projectiles
        self.projectile_system.update(dt);
        
        // Check for collisions with walls and enemies
        let projectiles = &mut self.projectile_system.projectiles;
        let mut projectiles_to_remove = Vec::new();
        

        use std::collections::HashSet;
        let mut enemies_with_death_anim = HashSet::new();
        
        for (i, projectile) in projectiles.iter().enumerate() {
            // Get tile position of projectile
            let (tile_x, tile_y) = projectile.get_tile_position(self.tile_size);
            
            // Check if out of bounds
            if tile_x >= self.map.grid_w || tile_y >= self.map.grid_h {
                projectiles_to_remove.push(i);
                continue;
            }
            
            // Check collision with walls
            let tile_id = self.map.tiles[tile_y][tile_x];
            if is_wall_tile(tile_id) {
                projectiles_to_remove.push(i);
                continue;
            }
            
            // Check collision with enemies (only for player projectiles)
            if projectile.is_player_projectile {
                let projectile_rect = projectile.get_collision_rect();
                
                for (j, entity) in self.entity_states.iter_mut().enumerate() {
                    if !entity.hp.is_alive() {
                        continue;
                    }
                    
                    // Skip non-enemy entities
                    if entity.kind != "tank" && entity.kind != "shooter" {
                        continue;
                    }
                    
                    // Create entity rectangle
                    let entity_rect = Rectangle {
                        x: (entity.x as i32 * self.tile_size + self.tile_size / 4) as f32,
                        y: (entity.y as i32 * self.tile_size + self.tile_size / 4) as f32,
                        width: (self.tile_size / 2) as f32,
                        height: (self.tile_size / 2) as f32,
                    };
                    
                    // Check for collision
                    if projectile_rect.check_collision_recs(&entity_rect) {
                        // Deal damage to enemy
                        let enemy_died = entity.hp.take_damage(projectile.damage);
                        
                        // Mark projectile for removal
                        projectiles_to_remove.push(i);
                        
                        // If enemy died, add points and spawn death animation
                        // Use entity index to uniquely identify each enemy
                        if enemy_died {
                            // Use entity index as unique identifier (not position, since multiple can share position)
                            if !enemies_with_death_anim.contains(&j) {
                                enemies_with_death_anim.insert(j);
                                
                                // Award different points per enemy type
                                match entity.kind.as_str() {
                                    "tank" => data.add_points(100),
                                    "shooter" => data.add_points(250),
                                    _ => {}
                                }

                                // Spawn a transient death animation at this tile
                                self.death_anims.push(DeathAnim {
                                    kind: entity.kind.clone(),
                                    x: entity.x,
                                    y: entity.y,
                                    timer: 0.0,
                                });
                            }
                        }
                        
                        break;
                    }
                }
            } else {
                // Check collision with player (for enemy projectiles)
                let projectile_rect = projectile.get_collision_rect();
                
                // Create player rectangle
                let player_rect = Rectangle {
                    x: (self.player_x as i32 * self.tile_size + self.tile_size / 4) as f32,
                    y: (self.player_y as i32 * self.tile_size + self.tile_size / 4) as f32,
                    width: (self.tile_size / 2) as f32,
                    height: (self.tile_size / 2) as f32,
                };
                
                // Check for collision
                if projectile_rect.check_collision_recs(&player_rect) {
                    // Deal damage to player
                    let player_died = self.player_hp.take_damage(projectile.damage);
                    
                    // Mark projectile for removal
                    projectiles_to_remove.push(i);
                    
                    // Stun player for 2 ticks
                    self.player_stun_ticks = 2;
                    
                    // Check for player death
                    if player_died {
                        println!("Player died from projectile! HP: {}/{}", 
                                self.player_hp.current, self.player_hp.max);
                    }
                }
            }
        }
        
        // Remove projectiles in reverse order to avoid index issues
        projectiles_to_remove.sort_by(|a, b| b.cmp(a));
        for i in projectiles_to_remove {
            if i < projectiles.len() {
                projectiles.remove(i);
            }
        }
    }
    fn draw_tile(&self, d: &mut RaylibDrawHandle, tile_id: i32, x: usize, y: usize) {
        let tileset = match &self.tileset {
            Some(t) => t,
            None => return, 
        };
        let cols = tileset.width() / self.tile_size;
        let src = Rectangle {
            x: ((tile_id % cols) * self.tile_size) as f32,
            y: ((tile_id / cols) * self.tile_size) as f32,
            width: self.tile_size as f32,
            height: self.tile_size as f32,
        };

        let dst = Rectangle {
            x: (x as i32 * self.tile_size) as f32,
            y: (y as i32 * self.tile_size) as f32,
            width: self.tile_size as f32,
            height: self.tile_size as f32,
        };

        d.draw_texture_pro(tileset, 
            src, 
            dst, Vector2::zero(), 0.0, Color::WHITE);
    }


}

impl Scene for MazeScene {
    fn on_enter(&mut self, rl: &mut RaylibHandle, data: &mut GameData) {
        self.map = load_map(&self.map_path);
        self.tile_size = self.map.tile_size_px;

        // Load texture using the thread from GameData
        if let Some(ref thread) = data.thread {
            let texture_path = resolve_asset_path("assets/textures/tileset0.png");
            self.tileset = Some(
                rl.load_texture(thread, &texture_path)
                    .unwrap_or_else(|e| {
                        let current_dir = env::current_dir().unwrap_or_default();
                        panic!("Failed to load tileset at '{}' (resolved from 'assets/textures/tileset0.png'): {:?}. Current working directory: {:?}", 
                               texture_path, e, current_dir);
                    })
            );
            
            // Load entity sprite sheets
            let player_path = resolve_asset_path("assets/models/P-Ranger.png");
            self.player_sprite = Some(
                rl.load_texture(thread, &player_path)
                    .unwrap_or_else(|e| {
                        panic!("Failed to load player sprite: {:?}", e);
                    })
            );
            
            let tank_path = resolve_asset_path("assets/models/Tank-45x66.png");
            self.tank_sprite = Some(
                rl.load_texture(thread, &tank_path)
                    .unwrap_or_else(|e| {
                        panic!("Failed to load tank sprite: {:?}", e);
                    })
            );
            
            let shooter_path = resolve_asset_path("assets/models/Shooter-45x51.png");
            self.shooter_sprite = Some(
                rl.load_texture(thread, &shooter_path)
                    .unwrap_or_else(|e| {
                        panic!("Failed to load shooter sprite: {:?}", e);
                    })
            );
            
            // Load projectile textures
            let arrow_path = resolve_asset_path("assets/textures/Arrow.png");
            let arrow_texture = rl.load_texture(thread, &arrow_path)
                .unwrap_or_else(|e| {
                    panic!("Failed to load arrow texture: {:?}", e);
                });
            self.projectile_system.set_arrow_texture(arrow_texture);
            
            let mage_bullet_path = resolve_asset_path("assets/textures/mage-bullet-13x13.png");
            let mage_bullet_texture = rl.load_texture(thread, &mage_bullet_path)
                .unwrap_or_else(|e| {
                    panic!("Failed to load mage bullet texture: {:?}", e);
                });
            self.projectile_system.set_mage_bullet_texture(mage_bullet_texture);
        }

        // Initialize player position from map entities
        let mut player_initialized = false;
        for e in &self.map.entities {
            if e.kind == "player" {
                self.player_x = e.x;
                self.player_y = e.y;
                player_initialized = true;
                break;
            }
        }
        
        // If no player entity found, try to find first valid floor tile (fool checks)
        if !player_initialized {
            'outer: for y in 0..self.map.grid_h {
                for x in 0..self.map.grid_w {
                    if self.is_valid_move(x, y) {
                        self.player_x = x;
                        self.player_y = y;
                        break 'outer;
                    }
                }
            }
        }
        
        // Filter out player entities from map (player is now separate)
        self.map.entities.retain(|e| e.kind != "player");
        
        // Initialize player HP
        self.player_hp = Health::new(self.entity_stats.player_max_hp);
        self.player_stun_ticks = 0;
        
        // Convert map entities to entity states with HP
        //
        // IMPORTANT: `assets/maps/mapTest.json` currently contains many duplicate enemy entries
        // at the same (x,y) positions (stacked enemies). That makes it look like a single enemy
        // that takes many more hits to kill, because we're actually killing multiple enemies
        // on the same tile. (problem of MapMaker_v3)
        //
        // To match the intended behavior (1 enemy per tile), dedupe enemies by tile here.
        self.entity_states.clear();
        use std::collections::{HashMap, HashSet};
        let mut position_counts: HashMap<(usize, usize), usize> = HashMap::new();
        let mut seen_enemy_tiles: HashSet<(usize, usize)> = HashSet::new();
        
        for e in &self.map.entities {
            // Only create entity states for enemies (tank, shooter)
            if e.kind == "tank" || e.kind == "shooter" {
                // Track how many entities are on each tile (for diagnostics)
                let pos = (e.x, e.y);
                *position_counts.entry(pos).or_insert(0) += 1;

                // Dedupe: keep only the first enemy encountered per tile
                if !seen_enemy_tiles.insert(pos) {
                    // Duplicate enemy on same tile; skip spawning it.
                    continue;
                }
                
                let max_hp = self.entity_stats.get_max_hp(&e.kind);
                self.entity_states.push(EntityState::new(
                    e.kind.clone(),
                    e.x,
                    e.y,
                    max_hp
                ));
            }
        }
        
        // Warn about multiple enemies on the same tile in the map file
        for ((x, y), count) in &position_counts {
            if *count > 1 {
                eprintln!(
                    "WARNING: map has {} enemies on tile ({}, {}). Only 1 will be spawned due to dedupe.",
                    count, x, y
                );
            }
        }
        
        // Initialize camera position
        self.update_camera(data);
        
        // Start level timer when entering the maze
        data.start_level_timer();
    }



    fn handle_input(&mut self, rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) || rl.is_key_pressed(KeyboardKey::KEY_P) {
            use crate::menu_scene::PauseScene;
            return SceneSwitch::Push(Box::new(PauseScene));
        }
        
        // Handle shooting input (works even when stunned for better responsiveness)
        self.handle_shooting(rl);
        
        // Don't queue movement if player is stunned
        if self.player_stun_ticks > 0 {
            return SceneSwitch::None;
        }
        
        // Queue movement for tick-based updates (only queue if no move is already queued)
        if self.queued_move.is_none() {
            let mut new_x = self.player_x;
            let mut new_y = self.player_y;
            let mut movement_queued = false;
            
            // ===== KEYBOARD INPUT =====
            if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
                new_x = new_x.saturating_add(1).min(self.map.grid_w.saturating_sub(1));
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
                new_x = new_x.saturating_sub(1);
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
                new_y = new_y.saturating_add(1).min(self.map.grid_h.saturating_sub(1));
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
                new_y = new_y.saturating_sub(1);
                movement_queued = true;
            }
            
            // ===== GAMEPAD INPUT =====
            // Check if gamepad is available
            if rl.is_gamepad_available(0) {
                // ===== D-PAD INPUT =====
                // D-pad buttons provide discrete directional input
                if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT) {
                    // D-pad right
                    new_x = new_x.saturating_add(1).min(self.map.grid_w.saturating_sub(1));
                    movement_queued = true;
                }
                if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
                    // D-pad left
                    new_x = new_x.saturating_sub(1);
                    movement_queued = true;
                }
                if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
                    // D-pad down
                    new_y = new_y.saturating_add(1).min(self.map.grid_h.saturating_sub(1));
                    movement_queued = true;
                }
                if rl.is_gamepad_button_down(0, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
                    // D-pad up
                    new_y = new_y.saturating_sub(1);
                    movement_queued = true;
                }
                
                // ===== ANALOG STICK INPUT =====
                let x_axis = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
                let y_axis = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
                
                // Convert analog stick input to discrete directions
                // Threshold: stick must be pushed at least 0.5 to register input (deadzone)
                let deadzone = 0.5;
                
                // Determine discrete direction from analog input
                // Prioritize the axis with greater magnitude for diagonal movement
                let abs_x = x_axis.abs();
                let abs_y = y_axis.abs();
                
                if abs_x > deadzone || abs_y > deadzone {
                    // Determine which direction to move
                    let mut gamepad_x_dir = 0;
                    let mut gamepad_y_dir = 0;
                    
                    if abs_x > abs_y {
                        // Horizontal movement takes priority
                        gamepad_x_dir = if x_axis > 0.0 { 1 } else { -1 };
                    } else if abs_y > abs_x {
                        // Vertical movement takes priority
                        gamepad_y_dir = if y_axis > 0.0 { 1 } else { -1 };
                    } else {
                        // Equal magnitude - allow diagonal movement
                        if abs_x > deadzone {
                            gamepad_x_dir = if x_axis > 0.0 { 1 } else { -1 };
                        }
                        if abs_y > deadzone {
                            gamepad_y_dir = if y_axis > 0.0 { 1 } else { -1 };
                        }
                    }
                    
                    // Apply gamepad movement
                    if gamepad_x_dir != 0 {
                        new_x = if gamepad_x_dir > 0 {
                            new_x.saturating_add(1).min(self.map.grid_w.saturating_sub(1))
                        } else {
                            new_x.saturating_sub(1)
                        };
                        movement_queued = true;
                    }
                    if gamepad_y_dir != 0 {
                        new_y = if gamepad_y_dir > 0 {
                            new_y.saturating_add(1).min(self.map.grid_h.saturating_sub(1))
                        } else {
                            new_y.saturating_sub(1)
                        };
                        movement_queued = true;
                    }
                } else {
                    // Stick is in deadzone - reset tracking
                    self.last_gamepad_direction = None;
                }
            }
    
            // Only queue if position changed
            if movement_queued && (new_x != self.player_x || new_y != self.player_y) {
                self.queued_move = Some((new_x, new_y));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, dt: f32, data: &mut GameData) -> SceneSwitch {
        // Update camera every frame
        self.update_camera(data);
        
        // Update animations every frame
        self.player_anim.update(dt);
        self.tank_anim.update(dt);
        self.shooter_anim.update(dt);
        // Note: shooter_death_anim only plays when shooter dies
        
        // Update entity cooldowns every frame
        for entity in &mut self.entity_states {
            entity.update_cooldown(dt);
        }
        
        // Update projectiles every frame (not tick-based for smooth movement)
        self.update_projectiles(dt, data);

        // Update active death animations (timers only)
        for anim in &mut self.death_anims {
            anim.timer += dt;
        }

        // Remove finished death animations based on per-enemy duration
        self.death_anims.retain(|anim| {
            match anim.kind.as_str() {
                // Shooter: 7-frame sequence at ~0.1s per frame ≈ 0.7s total
                "shooter" => anim.timer < 0.7,
                // Tank: ping-pong 3-frame sequence, let it play for a bit longer
                "tank" => anim.timer < 0.8,
                _ => anim.timer < 0.5,
            }
        });
        
        // Tick-based game logic
        self.tick_timer += dt;
        
        // Process game tick when timer exceeds tick_rate
        if self.tick_timer >= self.tick_rate {
            self.tick_timer = 0.0;
            
            // Update player movement (grid-locked, tick-based)
            self.update_player();
            
            // Update enemy AI
            self.update_enemies();
            
            // Remove dead entities (goals are kept separately in map.entities)
            self.entity_states.retain(|e| e.hp.is_alive());
        }
        
        // Check if player has died (HP <= 0)
        if !self.player_hp.is_alive() {
            use crate::menu_scene::LoseScene;
            return SceneSwitch::Push(Box::new(LoseScene::new(self.map_path.clone())));
        }
        
        // Check if player has reached the goal 
        for e in &self.map.entities {
            if e.kind == "goal" && e.x == self.player_x && e.y == self.player_y {
                // Record completion time
                data.complete_level();
                return SceneSwitch::Replace(Box::new(WinScene));
            }
        }
        
        SceneSwitch::None
    }



    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::BLACK);
        
        // Begin 2D camera mode
        let mut d2d = d.begin_mode2D(self.camera);
        
        // only draw tiles in FOV
        let (min_x, max_x, min_y, max_y) = self.get_visible_bounds();
        
        // ===== FLOOR LAYER =====
        // Only iterate over visible tiles
        for y in min_y..max_y {
            for x in min_x..max_x {
                // skip tiles outside circular FOV
                if !self.in_fov(x, y) {
                    continue;
                }
                
                let tid = self.map.tiles[y][x];
                // Draw floor tiles, or any non-wall tile as floor
                if tid >= 0 && (is_floor_tile(tid) || !is_wall_tile(tid)) {
                    self.draw_tile(&mut d2d, tid, x, y);
                }
            }
        }
        
        // ===== WALL LAYER =====
        // Only iterate over visible tiles
        for y in min_y..max_y {
            for x in min_x..max_x {
                // skip tiles outside circular FOV
                if !self.in_fov(x, y) {
                    continue;
                }
                
                let tid = self.map.tiles[y][x];
                if tid >= 0 && is_wall_tile(tid) {
                    self.draw_tile(&mut d2d, tid, x, y);
                }
            }
        }
        
        // ===== ENTITIES LAYER =====
        // Draw goals from map.entities
        for e in &self.map.entities {
            if e.kind == "goal" && self.in_fov(e.x, e.y) {
                self.draw_tile(&mut d2d, 81, e.x, e.y);
            }
        }
        
        // Draw enemies from entity_states (only alive ones)
        for e in &self.entity_states {
            // FOV culling: skip entities outside circular FOV
            if !self.in_fov(e.x, e.y) {
                continue;
            }
            
            // Only draw alive entities
            if !e.hp.is_alive() {
                continue;
            }
            
            match e.kind.as_str() {
                "tank" => {
                    if let Some(ref sprite) = self.tank_sprite {
                        // Tank: Top row (3 frames) = East movement
                        let frame_x = self.tank_anim.get_frame_index().min(2); // Clamp to 3 frames
                        let frame_y = 0; // Top row for starting
                        self.draw_animated_sprite(
                            &mut d2d,
                            sprite,
                            frame_x,
                            frame_y,
                            45, // frame width
                            66, // frame height
                            e.x,
                            e.y,
                            45, // sprite width
                            66, // sprite height
                        );
                    }
                }
                "shooter" => {
                    if let Some(ref sprite) = self.shooter_sprite {
                        // Shooter: First row (4 frames) = idle animation
                        let frame_x = self.shooter_anim.get_frame_index();
                        let frame_y = 0; // First row for idle
                        self.draw_animated_sprite(
                            &mut d2d,
                            sprite,
                            frame_x,
                            frame_y,
                            45, // frame width
                            51, // frame height
                            e.x,
                            e.y,
                            45, // sprite width
                            51, // sprite height
                        );
                    }
                }
                _ => {}
            }
        }

        // Draw death animations for defeated enemies
        for anim in &self.death_anims {
            // FOV culling for death animations as well
            if !self.in_fov(anim.x, anim.y) {
                continue;
            }

            match anim.kind.as_str() {
                "shooter" => {
                    if let Some(ref sprite) = self.shooter_sprite {
                        let frame_duration = 0.1;
                        let mut frame = (anim.timer / frame_duration).floor() as usize;
                        if frame > 6 {
                            frame = 6;
                        }

                        let (frame_x, frame_y) = if frame < 4 {
                            (frame, 1)
                        } else {
                            (frame - 4, 2)
                        };

                        self.draw_animated_sprite(
                            &mut d2d,
                            sprite,
                            frame_x,
                            frame_y,
                            45, // frame width
                            51, // frame height
                            anim.x,
                            anim.y,
                            45, // sprite width
                            51, // sprite height
                        );
                    }
                }
                "tank" => {
                    if let Some(ref sprite) = self.tank_sprite {

                        
                        let seq: &[(usize, usize)] = &[
                            (2, 0), 
                            (1, 1), 
                            (0, 1), 
                            (1, 1), 
                        ];

                        let frame_duration = 0.1;
                        let frame_count = (anim.timer / frame_duration).floor() as usize;
                        let idx = (frame_count % seq.len()).min(seq.len() - 1);
                        let (frame_x, frame_y) = seq[idx];

                        self.draw_animated_sprite(
                            &mut d2d,
                            sprite,
                            frame_x,
                            frame_y,
                            45, // frame width
                            66, // frame height
                            anim.x,
                            anim.y,
                            45, // sprite width
                            66, // sprite height
                        );
                    }
                }
                _ => {}
            }
        }
        
        // ===== PLAYER (always visible, drawn on top) =====
        // Player is always drawn, even if outside FOV (shouldn't happen, but safe)
        if let Some(ref sprite) = self.player_sprite {
            // Player: 3 frames per row, 4 rows (directions)
            // Row 0 = North, Row 1 = East, Row 2 = South, Row 3 = West
            let frame_x = self.player_anim.get_frame_index().min(2); // Clamp to 3 frames (0-2)
            let frame_y = self.player_direction.min(3); // Clamp to 4 directions (0-3)
            let player_frame_width = sprite.width() / 3; // 3 frames per row
            let player_frame_height = sprite.height() / 4; // 4 rows
            self.draw_animated_sprite(
                &mut d2d,
                sprite,
                frame_x,
                frame_y,
                player_frame_width,
                player_frame_height,
                self.player_x,
                self.player_y,
                player_frame_width,
                player_frame_height,
            );
        } else {
            // Fallback to circle if sprite not loaded
            let player_px = self.player_x as i32 * self.tile_size + self.tile_size / 2;
            let player_py = self.player_y as i32 * self.tile_size + self.tile_size / 2;
            d2d.draw_circle(
                player_px,
                player_py,
                self.tile_size as f32 * 0.4,
                Color::BLUE,
            );
        }
        
        self.projectile_system
            .draw(&mut d2d, self.tile_size, min_x, max_x, min_y, max_y);
        
        // End 2D camera mode
        drop(d2d);
        
        // ===== UI LAYER (screen space, not affected by camera) =====
        // Draw player HP
        let hp_text = format!("HP: {}/{}", self.player_hp.current, self.player_hp.max);
        d.draw_text(hp_text.as_str(), 10, 10, 20, Color::WHITE);
        
        // Draw stun indicator if player is stunned
        if self.player_stun_ticks > 0 {
            d.draw_text("STUNNED!", 10, 35, 20, Color::RED);
        }
        
        d.draw_text(
            &format!("Score: {}", data.points),
            10,
            data.screen_height - 24,
            20,
            Color::WHITE,
        );
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}

