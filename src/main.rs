use macroquad::prelude::*;
use std::time::Instant;
use std::fs;
use std::io;

// Window configuration
fn window_conf() -> Conf {    
    Conf {
        window_title: "Pawsse - Virtual Pet".to_owned(),
        window_width: 400,
        window_height: 300,
        window_resizable: false,  // Disable resize
        fullscreen: false,
        icon: None,  // Icon can be added later with proper Icon structure
        ..Default::default()
    }
}

// Enum to represent the pet's emotional state
#[derive(Clone, Copy, Debug, PartialEq)]
enum PawState {
    Happy,
    Serious,
    Sad,
}

// Struct to handle frame-based animations
struct Animation {
    frames: Vec<Texture2D>,  // List of textures (frames)
    current_frame: usize,     // Current frame index being displayed
    frame_duration: f32,       // Duration of each frame in seconds
    last_time: Instant,       // Last time the frame was changed
    cycles_completed: u32,     // Number of complete cycles played
}

// Enum to track petting animation state
enum PawAnimationState {
    None,                      // Not petting
    Petting(Animation),        // Currently petting (with active animation)
}

impl Animation {
    // Creates a new animation with given frames and duration
    fn new(frames: Vec<Texture2D>, frame_duration: f32) -> Self {
        Animation {
            frames,
            current_frame: 0,
            frame_duration,
            last_time: Instant::now(),
            cycles_completed: 0,
        }
    }
    
    // Updates the animation, switching frames based on elapsed time
    fn update(&mut self) {
        if self.last_time.elapsed().as_secs_f32() >= self.frame_duration {
            let previous_frame = self.current_frame;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_time = Instant::now();
            
            // Track completed cycles (when we loop back to frame 0)
            if previous_frame == self.frames.len() - 1 && self.current_frame == 0 {
                self.cycles_completed += 1;
            }
        }
    }
    
    // Returns the current frame texture
    fn current_frame(&self) -> &Texture2D {
        &self.frames[self.current_frame]
    }
    
    // Resets the animation to the first frame
    //fn reset(&mut self) {
    //    self.current_frame = 0;
    //    self.last_time = Instant::now();
    //    self.cycles_completed = 0;
    //}
    
    // Returns the number of completed cycles
    fn cycles_completed(&self) -> u32 {
        self.cycles_completed
    }
}

// Struct to save game data
#[derive(serde::Serialize, serde::Deserialize)]
struct GameData {
    total_pets: u32,
}

// Function to load saved game data
fn load_game_data() -> u32 {
    if let Ok(content) = fs::read_to_string("data/save_data.json") {
        if let Ok(data) = serde_json::from_str::<GameData>(&content) {
            return data.total_pets;
        }
    }
    0 // Returns 0 if unable to load
}

// Function to save game data
fn save_game_data(total_pets: u32) -> io::Result<()> {
    // Create data directory if it doesn't exist
    fs::create_dir_all("data")?;
    
    let data = GameData { total_pets };
    let json = serde_json::to_string_pretty(&data)?;
    fs::write("data/save_data.json", json)?;
    Ok(())
}

#[macroquad::main(window_conf)]
async fn main() {
    // Load background textures
    let background_1 = load_texture("assets/400x300/background_1.png").await.unwrap();
    let background_2 = load_texture("assets/400x300/background_2.png").await.unwrap();
    
    // Load pet state textures
    let paw_happy = load_texture("assets/400x300/paw_happy.png").await.unwrap();
    let paw_serious = load_texture("assets/400x300/paw_serious.png").await.unwrap();
    let paw_sad = load_texture("assets/400x300/paw_sad.png").await.unwrap();
    
    // Load petting animation textures
    let hand_happy_1 = load_texture("assets/400x300/hand_paw_happy_1.png").await.unwrap();
    let hand_happy_2 = load_texture("assets/400x300/hand_paw_happy_2.png").await.unwrap();
    let hand_serious_1 = load_texture("assets/400x300/hand_paw_serious_1.png").await.unwrap();
    let hand_serious_2 = load_texture("assets/400x300/hand_paw_serious_2.png").await.unwrap();
    let hand_sad_1 = load_texture("assets/400x300/hand_paw_sad_1.png").await.unwrap();
    let hand_sad_2 = load_texture("assets/400x300/hand_paw_sad_2.png").await.unwrap();
    
    // Create background animation (alternates between 2 frames)
    // Clone textures so they can be reused
    let mut background_animation = Animation::new(
        vec![background_1.clone(), background_2.clone()],
        0.5, // 0.5 seconds per frame
    );
    
    // Initial state
    let mut state = PawState::Happy;
    let mut last_pet = Instant::now();
    
    // Petting counters - load saved data
    let mut total_pets: u32 = load_game_data();  // Load from file
    let mut pets_in_current_state: u32 = 0;  // Number of pets in current state
    
    // Petting animation states
    let mut animation_state = PawAnimationState::None;
    
    // Main game loop
    loop {
        // Clear the window with white background
        clear_background(Color::from_rgba(97, 103, 117, 255));
        
        // Update and draw background animation
        background_animation.update();
        draw_texture(background_animation.current_frame(), 0.0, 0.0, WHITE);
        
        // Calculate pet state based on time since last pet
        // Time can make the pet sadder, but petting is needed to improve
        let time_since_last_pet = last_pet.elapsed().as_secs();
        let time_based_state = if time_since_last_pet > 900 {
            PawState::Sad
        } else if time_since_last_pet > 1800 {
            PawState::Serious
        } else {
            PawState::Happy
        };
        
        // If time made pet sadder, update state and reset pet counter
        if time_based_state != state {
            match (state, time_based_state) {
                (PawState::Happy, PawState::Serious) | 
                (PawState::Serious, PawState::Sad) |
                (PawState::Happy, PawState::Sad) => {
                    state = time_based_state;
                    pets_in_current_state = 0; // Reset counter when state worsens
                }
                _ => {}
            }
        }
        
        // Handle petting animation state
        match &mut animation_state {
            PawAnimationState::None => {
                // Draw pet based on current state
                let paw_texture = match state {
                    PawState::Happy => &paw_happy,
                    PawState::Serious => &paw_serious,
                    PawState::Sad => &paw_sad,
                };
                draw_texture(paw_texture, 0.0, 0.0, WHITE);
                
                // Detect mouse click to start petting animation
                if is_mouse_button_pressed(MouseButton::Left) {
                    // Create petting animation based on current state
                    // Clone textures so they can be reused in future iterations
                    let petting_frames = match state {
                        PawState::Happy => vec![hand_happy_1.clone(), hand_happy_2.clone()],
                        PawState::Serious => vec![hand_serious_1.clone(), hand_serious_2.clone()],
                        PawState::Sad => vec![hand_sad_1.clone(), hand_sad_2.clone()],
                    };
                    
                    animation_state = PawAnimationState::Petting(
                        Animation::new(petting_frames, 0.1) // 0.3 seconds per frame
                    );
                    
                    // Update last pet time (but don't reset state - petting is needed to improve)
                    last_pet = Instant::now();
                }
            }
            PawAnimationState::Petting(anim) => {
                // Update and draw petting animation
                anim.update();
                draw_texture(anim.current_frame(), 0.0, 0.0, WHITE);
                
                // After completing 3 full cycles, finish petting and update state
                if anim.cycles_completed() >= 3 {
                    // Increment counters
                    total_pets += 1;
                    save_game_data(total_pets).unwrap();
                    pets_in_current_state += 1;
                    
                    // Check if enough pets to improve state
                    match state {
                        PawState::Sad => {
                            // Need 3 pets to go from Sad to Serious
                            if pets_in_current_state >= 3 {
                                state = PawState::Serious;
                                pets_in_current_state = 0; // Reset counter for new state
                            }
                        }
                        PawState::Serious => {
                            // Need 2 pets to go from Serious to Happy
                            if pets_in_current_state >= 2 {
                                state = PawState::Happy;
                                pets_in_current_state = 0; // Reset counter for new state
                            }
                        }
                        PawState::Happy => {
                            // Happy state - pets keep it happy but don't change state
                            // Counter can accumulate but doesn't need to reset
                        }
                    }
                    
                    animation_state = PawAnimationState::None;
                }
            }
        }
        
        // Draw "bold" text by simulating with multiple layers (subtle effect)
        let text = &format!("Total Pets: {}", total_pets);
        let x = 10.0;
        let y = 280.0;
        let size = 20.0;
        let color = WHITE;

        // Draw only in the 4 main directions for a more subtle effect
        draw_text(text, x - 0.5, y, size, color);  // Left
        draw_text(text, x + 0.5, y, size, color);  // Right
        draw_text(text, x, y - 0.5, size, color);  // Top
        draw_text(text, x, y + 0.5, size, color);  // Bottom
        // Draw on top with original color
        draw_text(text, x, y, size, color);
        
        // Wait for next frame (60 FPS)
        next_frame().await;
    }
}
