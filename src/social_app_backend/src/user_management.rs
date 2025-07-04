use crate::state_handler;
use crate::types::*;
use candid::Principal;
use ic_cdk::caller;

// Remove the old authenticate_user function and use get_current_user from lib.rs
fn get_current_user() -> Result<String, String> {
    crate::get_current_user()
}

// Keep the old function for backward compatibility but make it use the new method
pub fn authenticate_user() -> Result<String, String> {
    get_current_user()
}

pub fn register_user(user_id: String) -> Result<(), String> {
    if state_handler::user_exists(&user_id) {
        Err("User already exists".to_string())
    } else {
        // Create default profile for new user
        let default_profile = UserProfile {
            user_id: user_id.clone(),
            username: format!("user_{}", &user_id[..8]), // Default username
            display_name: "New User".to_string(),
            bio: String::new(),
            profile_picture: None,
            website: None,
            email: None,
            phone: None,
            account_type: AccountType::Personal,
            is_verified: false,
            is_private: false,
            followers_count: 0,
            following_count: 0,
            posts_count: 0,
            created_at: state_handler::get_current_timestamp(),
            updated_at: state_handler::get_current_timestamp(),
            links: Vec::new(),
            location: None,
            date_of_birth: None,
            gender: None,
        };

        state_handler::insert_user(user_id, default_profile);
        Ok(())
    }
}

pub fn get_user_info(user_id: &str) -> Result<UserProfile, String> {
    state_handler::get_user(user_id).ok_or_else(|| "User not found".to_string())
}

pub fn check_username_availability(username: &str) -> bool {
    !state_handler::username_exists(username)
}

pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if username.len() > 30 {
        return Err("Username must be less than 30 characters long".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return Err(
            "Username can only contain letters, numbers, underscores, and periods".to_string(),
        );
    }

    if username.starts_with('.') || username.ends_with('.') {
        return Err("Username cannot start or end with a period".to_string());
    }

    if username.contains("..") {
        return Err("Username cannot contain consecutive periods".to_string());
    }

    Ok(())
}

pub fn search_users(query: &str, limit: u32) -> Vec<UserProfile> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    // Log search activity if user is authenticated
    if let Ok(user_id) = get_current_user() {
        log_user_activity(
            user_id,
            ActivityAction::ProfileVisited,
            None,
            Some("search".to_string()),
        );
    }

    state_handler::search_users(query, limit as usize)
}

pub fn get_user_activity(user_id: &str) -> Vec<UserActivity> {
    // Log that someone viewed activity (if authenticated and viewing others)
    if let Ok(current_user) = get_current_user() {
        if current_user != user_id {
            log_user_activity(
                current_user,
                ActivityAction::ProfileVisited,
                Some(user_id.to_string()),
                Some("activity".to_string()),
            );
        }
    }

    state_handler::get_user_activities(user_id)
}

pub fn log_user_activity(
    user_id: String,
    action: ActivityAction,
    target_id: Option<String>,
    target_type: Option<String>,
) {
    let activity = UserActivity {
        user_id: user_id.clone(),
        action,
        target_id,
        target_type,
        created_at: state_handler::get_current_timestamp(),
    };

    state_handler::add_user_activity(user_id, activity);
}

pub fn block_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Cannot block yourself
    if user_id == target_user_id {
        return Err("Cannot block yourself".to_string());
    }

    // Check if already blocked
    if state_handler::is_user_blocked(user_id, target_user_id) {
        return Err("User is already blocked".to_string());
    }

    // Remove any existing follow relationships
    let _ = state_handler::remove_follower(user_id, target_user_id);
    let _ = state_handler::remove_follower(target_user_id, user_id);

    // Add to blocked list
    state_handler::add_blocked_user(user_id.to_string(), target_user_id.to_string());

    // Log activity
    log_user_activity(
        user_id.to_string(),
        ActivityAction::UserFollowed, // We don't have a UserBlocked action, using closest
        Some(target_user_id.to_string()),
        Some("block".to_string()),
    );

    Ok(())
}

pub fn unblock_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Check if actually blocked
    if !state_handler::is_user_blocked(user_id, target_user_id) {
        return Err("User is not blocked".to_string());
    }

    // Remove from blocked list
    state_handler::remove_blocked_user(user_id, target_user_id);

    Ok(())
}

pub fn mute_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Cannot mute yourself
    if user_id == target_user_id {
        return Err("Cannot mute yourself".to_string());
    }

    // Check if already muted
    if state_handler::is_user_muted(user_id, target_user_id) {
        return Err("User is already muted".to_string());
    }

    // Add to muted list
    state_handler::add_muted_user(user_id.to_string(), target_user_id.to_string());

    Ok(())
}

pub fn unmute_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Check if actually muted
    if !state_handler::is_user_muted(user_id, target_user_id) {
        return Err("User is not muted".to_string());
    }

    // Remove from muted list
    state_handler::remove_muted_user(user_id, target_user_id);

    Ok(())
}

pub fn restrict_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Cannot restrict yourself
    if user_id == target_user_id {
        return Err("Cannot restrict yourself".to_string());
    }

    // Check if already restricted
    if state_handler::is_user_restricted(user_id, target_user_id) {
        return Err("User is already restricted".to_string());
    }

    // Add to restricted list
    state_handler::add_restricted_user(user_id.to_string(), target_user_id.to_string());

    Ok(())
}

pub fn unrestrict_user(user_id: &str, target_user_id: &str) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(target_user_id) {
        return Err("User not found".to_string());
    }

    // Check if actually restricted
    if !state_handler::is_user_restricted(user_id, target_user_id) {
        return Err("User is not restricted".to_string());
    }

    // Remove from restricted list
    state_handler::remove_restricted_user(user_id, target_user_id);

    Ok(())
}

pub fn is_user_blocked(user_id: &str, target_user_id: &str) -> bool {
    state_handler::is_user_blocked(user_id, target_user_id)
}

pub fn is_user_muted(user_id: &str, target_user_id: &str) -> bool {
    state_handler::is_user_muted(user_id, target_user_id)
}

pub fn is_user_restricted(user_id: &str, target_user_id: &str) -> bool {
    state_handler::is_user_restricted(user_id, target_user_id)
}

pub fn get_blocked_users(user_id: &str) -> Vec<String> {
    state_handler::get_blocked_users_list(user_id)
}

pub fn get_muted_users(user_id: &str) -> Vec<String> {
    state_handler::get_muted_users_list(user_id)
}

pub fn get_restricted_users(user_id: &str) -> Vec<String> {
    state_handler::get_restricted_users_list(user_id)
}

pub fn update_last_seen(user_id: &str) -> Result<(), String> {
    // Update user's last seen timestamp
    if let Some(mut user) = state_handler::get_user(user_id) {
        user.updated_at = state_handler::get_current_timestamp();
        state_handler::update_user(user_id, user)?;
        Ok(())
    } else {
        Err("User not found".to_string())
    }
}

pub fn set_online_status(user_id: &str, is_online: bool) -> Result<(), String> {
    // Check if user exists
    if !state_handler::user_exists(user_id) {
        return Err("User not found".to_string());
    }

    // Set online status
    state_handler::set_user_online_status(user_id.to_string(), is_online);

    // Update last seen if going online
    if is_online {
        update_last_seen(user_id)?;
    }

    Ok(())
}

pub fn get_online_status(user_id: &str) -> bool {
    state_handler::get_user_online_status(user_id)
}

pub fn create_multiple_profiles(count: u32) -> Result<Vec<UserProfile>, String> {
    if count == 0 || count > 100 {
        return Err("Count must be between 1 and 100".to_string());
    }

    let mut created_profiles = Vec::new();
    let sample_names = vec![
        "Alex", "Jordan", "Casey", "Taylor", "Morgan", "Avery", "Riley", "Parker", "Quinn", "Sage",
        "River", "Blake", "Cameron", "Drew", "Emery", "Finley", "Harper", "Hayden", "Jamie",
        "Kennedy", "Logan", "Marley", "Peyton", "Reese", "Rowan", "Skylar", "Sydney", "Dakota",
        "Ellis", "Jessie",
    ];

    let sample_bios = vec![
        "Living life to the fullest ✨",
        "Coffee lover ☕ | Dog parent 🐕",
        "Adventure seeker 🌍 | Photographer 📸",
        "Foodie 🍕 | Traveler ✈️",
        "Fitness enthusiast 💪 | Health advocate",
        "Artist 🎨 | Creative soul",
        "Music lover 🎵 | Concert goer",
        "Book worm 📚 | Learning every day",
        "Tech enthusiast 💻 | Innovation lover",
        "Nature lover 🌿 | Hiking addict",
    ];

    for i in 0..count {
        let user_id = state_handler::generate_id();
        let random_name = sample_names[i as usize % sample_names.len()];
        let username = format!(
            "{}_{}",
            random_name.to_lowercase(),
            state_handler::generate_id()[..6].to_lowercase()
        );
        let display_name = format!("{} {}", random_name, i + 1);
        let bio = sample_bios[i as usize % sample_bios.len()].to_string();

        let current_time = state_handler::get_current_timestamp();

        let user_profile = UserProfile {
            user_id: user_id.clone(),
            username,
            display_name,
            bio,
            profile_picture: None,
            website: None,
            email: None,
            phone: None,
            account_type: if i % 5 == 0 {
                AccountType::Creator
            } else if i % 7 == 0 {
                AccountType::Business
            } else {
                AccountType::Personal
            },
            is_verified: i % 10 == 0, // Every 10th user is verified
            is_private: i % 4 == 0,   // Every 4th user is private
            followers_count: (i as u64) * 10,
            following_count: (i as u64) * 5,
            posts_count: i as u64,
            created_at: current_time,
            updated_at: current_time,
            links: Vec::new(),
            location: if i % 3 == 0 {
                Some("San Francisco, CA".to_string())
            } else {
                None
            },
            date_of_birth: None,
            gender: None,
        };

        state_handler::insert_user(user_id, user_profile.clone());
        created_profiles.push(user_profile);
    }

    Ok(created_profiles)
}
