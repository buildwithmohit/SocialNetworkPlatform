use crate::state_handler;
use crate::types::*;
use crate::user_management;

// Use the get_current_user function from lib.rs instead of user_management::authenticate_user
fn get_current_user() -> Result<String, String> {
    crate::get_current_user()
}

pub fn create_user_profile(profile_data: CreateUserProfileRequest) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    // Check if user already has a profile
    // if state_handler::user_exists(&user_id) {
    //     return Err("User profile already exists".to_string());
    // }

    // Validate username
    user_management::validate_username(&profile_data.username)?;

    // Check if username is available
    if !user_management::check_username_availability(&profile_data.username) {
        return Err("Username is already taken".to_string());
    }

    // Validate email if provided
    if let Some(ref email) = profile_data.email {
        if !email.contains('@') || email.len() < 5 {
            return Err("Invalid email address".to_string());
        }
    }

    // Validate phone if provided
    if let Some(ref phone) = profile_data.phone {
        if phone.len() < 10 || phone.len() > 15 {
            return Err("Phone number must be between 10-15 digits".to_string());
        }
    }

    // Validate gender if provided
    if let Some(ref gender) = profile_data.gender {
        let valid_genders = ["male", "female", "non-binary", "prefer-not-to-say", "other"];
        if !valid_genders.contains(&gender.to_lowercase().as_str()) {
            return Err("Invalid gender option".to_string());
        }
    }

    // Validate date of birth if provided (must be at least 13 years old)
    if let Some(dob) = profile_data.date_of_birth {
        let current_time = state_handler::get_current_timestamp();
        let thirteen_years_ms = 13 * 365 * 24 * 60 * 60 * 1000; // 13 years in milliseconds
        if current_time - dob < thirteen_years_ms {
            return Err("You must be at least 13 years old to create an account".to_string());
        }
    }

    let current_time = state_handler::get_current_timestamp();

    let user_profile = UserProfile {
        user_id: user_id.clone(),
        username: profile_data.username,
        display_name: profile_data.display_name,
        bio: profile_data.bio.unwrap_or_default(),
        profile_picture: profile_data.profile_picture,
        website: profile_data.website,
        email: profile_data.email,
        phone: profile_data.phone,
        account_type: profile_data.account_type,
        is_verified: false,
        is_private: profile_data.is_private.unwrap_or(false),
        followers_count: 0,
        following_count: 0,
        posts_count: 0,
        created_at: current_time,
        updated_at: current_time,
        links: Vec::new(),
        location: profile_data.location,
        date_of_birth: profile_data.date_of_birth,
        gender: profile_data.gender,
    };

    state_handler::insert_user(user_id, user_profile.clone());
    Ok(user_profile)
}

pub fn update_user_profile(profile_data: UpdateUserProfileRequest) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // Update fields if provided
    if let Some(display_name) = profile_data.display_name {
        user_profile.display_name = display_name;
    }

    if let Some(bio) = profile_data.bio {
        user_profile.bio = bio;
    }

    if let Some(profile_picture) = profile_data.profile_picture {
        user_profile.profile_picture = Some(profile_picture);
    }

    if let Some(website) = profile_data.website {
        user_profile.website = Some(website);
    }

    if let Some(is_private) = profile_data.is_private {
        user_profile.is_private = is_private;
    }

    if let Some(links) = profile_data.links {
        user_profile.links = links;
    }

    if let Some(location) = profile_data.location {
        user_profile.location = Some(location);
    }

    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn get_user_profile(user_id: String) -> Result<UserProfile, String> {
    state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())
}

pub fn get_current_user_profile() -> Result<UserProfile, String> {
    let user_id = get_current_user()?;
    get_user_profile(user_id)
}

pub fn delete_user_profile() -> Result<(), String> {
    let user_id = get_current_user()?;

    // Verify user exists before deletion
    state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // 1. Delete all user's posts
    let user_posts = state_handler::get_posts_by_user(&user_id);
    for post in user_posts {
        // Delete post comments first
        state_handler::delete_post_comments(&post.post_id)?;
        // Delete the post itself
        state_handler::delete_post(&post.post_id)?;
    }

    // 2. Delete all user's comments on other posts
    state_handler::delete_comments_by_user(&user_id)?;

    // 3. Remove all relationships (followers and following)
    // Remove user from other users' followers lists
    let followers = state_handler::get_followers(&user_id);
    for follower_id in followers {
        state_handler::unfollow(&follower_id, &user_id)?;
    }
    
    // Remove user from other users' following lists
    let following = state_handler::get_following(&user_id);
    for followed_id in following {
        state_handler::unfollow(&user_id, &followed_id)?;
    }

    // 4. Delete all messages (sent and received)
    state_handler::delete_user_messages(&user_id)?;

    // Delete user's stories if they exist
    state_handler::delete_user_stories(&user_id)?;

    // Delete user's saved posts
    state_handler::delete_user_saved_posts(&user_id)?;

    // Delete user's notifications
    state_handler::delete_user_notifications(&user_id)?;

    // Remove user from any blocked/blocking relationships
    state_handler::remove_user_blocks(&user_id)?;

    // Delete analytics data
    state_handler::delete_user_analytics(&user_id)?;

    // 5. Delete user profile (this should be last)
    state_handler::delete_user(&user_id)?;

    Ok(())
}

pub fn switch_account_type(account_type: AccountType) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.account_type = account_type;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn toggle_privacy_setting() -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.is_private = !user_profile.is_private;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn update_profile_picture(image_url: String) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.profile_picture = Some(image_url);
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn remove_profile_picture() -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.profile_picture = None;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn update_bio(bio: String) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    // Validate bio length
    if bio.len() > 150 {
        return Err("Bio must be 150 characters or less".to_string());
    }

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.bio = bio;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn add_website_link(website: String) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    // Basic URL validation
    if !website.starts_with("http://") && !website.starts_with("https://") {
        return Err("Website must be a valid URL starting with http:// or https://".to_string());
    }

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.website = Some(website);
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn add_profile_link(link: String) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // Limit number of links
    if user_profile.links.len() >= 5 {
        return Err("Maximum of 5 links allowed".to_string());
    }

    // Check if link already exists
    if user_profile.links.contains(&link) {
        return Err("Link already exists".to_string());
    }

    user_profile.links.push(link);
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn remove_profile_link(link: String) -> Result<UserProfile, String> {
    let user_id = get_current_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.links.retain(|l| l != &link);
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn get_profile_analytics(user_id: &str) -> Result<Analytics, String> {
    // Check if user can access analytics (must be own profile or business account)
    let current_user = get_current_user()?;
    let user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    if current_user != user_id
        && !matches!(
            user_profile.account_type,
            AccountType::Business | AccountType::Creator
        )
    {
        return Err("Access denied to analytics".to_string());
    }

    // In a real implementation, you would calculate actual analytics
    let analytics = Analytics {
        user_id: user_id.to_string(),
        post_id: None,
        story_id: None,
        views: 0,
        likes: 0,
        comments: 0,
        shares: 0,
        saves: 0,
        reach: 0,
        impressions: 0,
        profile_visits: 0,
        website_clicks: 0,
        date: state_handler::get_current_timestamp(),
    };

    Ok(analytics)
}

pub fn verify_account(user_id: &str) -> Result<UserProfile, String> {
    // This would typically be called by a system administrator
    let mut user_profile =
        state_handler::get_user(user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.is_verified = true;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(user_id, user_profile.clone())?;
    Ok(user_profile)
}

pub fn get_public_profile_info(user_id: &str) -> Result<UserProfile, String> {
    let user_profile =
        state_handler::get_user(user_id).ok_or_else(|| "User profile not found".to_string())?;

    // If account is private, only return basic info
    if user_profile.is_private {
        let current_user = user_management::authenticate_user().ok();
        if let Some(current_user_id) = current_user {
            // Check if current user follows this user
            if !state_handler::is_following(&current_user_id, user_id) && current_user_id != user_id
            {
                return Ok(UserProfile {
                    user_id: user_profile.user_id,
                    username: user_profile.username,
                    display_name: user_profile.display_name,
                    bio: "This account is private".to_string(),
                    profile_picture: user_profile.profile_picture,
                    website: None,
                    email: None,
                    phone: None,
                    account_type: user_profile.account_type,
                    is_verified: user_profile.is_verified,
                    is_private: user_profile.is_private,
                    followers_count: user_profile.followers_count,
                    following_count: 0, // Hidden for private accounts
                    posts_count: 0,     // Hidden for private accounts
                    created_at: user_profile.created_at,
                    updated_at: user_profile.updated_at,
                    links: Vec::new(),
                    location: None,
                    date_of_birth: None,
                    gender: None,
                });
            }
        } else {
            return Err("Private account".to_string());
        }
    }

    Ok(user_profile)
}

pub fn get_all_profiles() -> Result<Vec<UserProfile>, String> {
    let current_user = get_current_user()?;
    let all_profiles = state_handler::get_all_users();

    // Filter out private profiles if current user is not authenticated
    if current_user.is_empty() {
        Ok(all_profiles
            .into_iter()
            .filter(|profile| !profile.is_private)
            .collect())
    } else {
        Ok(all_profiles)
    }
}
