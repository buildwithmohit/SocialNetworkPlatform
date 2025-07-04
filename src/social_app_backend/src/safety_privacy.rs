use crate::state_handler;
use crate::types::*;
use crate::user_management;

pub fn report_content(
    reported_user_id: Option<String>,
    reported_post_id: Option<String>,
    reported_comment_id: Option<String>,
    reason: ReportReason,
    description: String,
) -> Result<Report, String> {
    let reporter_id = user_management::authenticate_user()?;

    // Validate that at least one target is provided
    if reported_user_id.is_none() && reported_post_id.is_none() && reported_comment_id.is_none() {
        return Err("Must specify what to report".to_string());
    }

    // Validate description
    if description.trim().is_empty() {
        return Err("Report description cannot be empty".to_string());
    }

    if description.len() > 1000 {
        return Err("Description must be 1000 characters or less".to_string());
    }

    let report_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();

    let report = Report {
        report_id: report_id.clone(),
        reporter_id,
        reported_user_id,
        reported_post_id,
        reported_comment_id,
        reason,
        description,
        status: ReportStatus::Pending,
        created_at: current_time,
        resolved_at: None,
    };

    // Store report in state_handler
    state_handler::insert_report(report_id, report.clone())?;

    Ok(report)
}

#[allow(dead_code)]
pub fn block_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::block_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn unblock_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::unblock_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn mute_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::mute_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn unmute_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::unmute_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn restrict_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::restrict_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn unrestrict_user(target_user_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    user_management::unrestrict_user(&user_id, &target_user_id)
}

#[allow(dead_code)]
pub fn get_blocked_users() -> Result<Vec<String>, String> {
    let user_id = user_management::authenticate_user()?;
    Ok(user_management::get_blocked_users(&user_id))
}

#[allow(dead_code)]
pub fn get_muted_users() -> Result<Vec<String>, String> {
    let user_id = user_management::authenticate_user()?;
    Ok(user_management::get_muted_users(&user_id))
}

#[allow(dead_code)]
pub fn get_restricted_users() -> Result<Vec<String>, String> {
    let user_id = user_management::authenticate_user()?;
    Ok(user_management::get_restricted_users(&user_id))
}

pub fn update_privacy_settings(
    is_private: bool,
    hide_activity_status: bool,
    hide_likes: bool,
) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    let mut user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    user_profile.is_private = is_private;
    user_profile.updated_at = state_handler::get_current_timestamp();

    state_handler::update_user(&user_id, user_profile)?;

    // Store additional privacy settings
    let privacy_settings = PrivacySettings {
        user_id: user_id.clone(),
        is_private,
        hide_activity_status,
        hide_likes,
        updated_at: state_handler::get_current_timestamp(),
    };

    state_handler::update_privacy_settings(user_id, privacy_settings)?;

    Ok(())
}

pub fn update_comment_controls(
    allow_comments: bool,
    filter_keywords: Vec<String>,
    hide_offensive: bool,
) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Validate keywords
    for keyword in &filter_keywords {
        if keyword.trim().is_empty() {
            return Err("Keywords cannot be empty".to_string());
        }
    }

    let comment_controls = CommentControls {
        user_id: user_id.clone(),
        allow_comments,
        filter_keywords,
        hide_offensive,
        updated_at: state_handler::get_current_timestamp(),
    };

    state_handler::update_comment_controls(user_id, comment_controls)?;

    Ok(())
}

pub fn add_keyword_filter(keyword: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    if keyword.trim().is_empty() {
        return Err("Keyword cannot be empty".to_string());
    }

    let clean_keyword = keyword.trim().to_lowercase();

    // Get existing filters
    let mut existing_filters = state_handler::get_keyword_filters(&user_id);

    // Check if keyword already exists
    if existing_filters.contains(&clean_keyword) {
        return Err("Keyword already in filter list".to_string());
    }

    // Check limit
    if existing_filters.len() >= 50 {
        return Err("Maximum of 50 keywords allowed".to_string());
    }

    existing_filters.push(clean_keyword);
    state_handler::update_keyword_filters(user_id, existing_filters)?;

    Ok(())
}

pub fn remove_keyword_filter(keyword: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    let clean_keyword = keyword.trim().to_lowercase();
    let mut existing_filters = state_handler::get_keyword_filters(&user_id);

    let initial_len = existing_filters.len();
    existing_filters.retain(|k| k != &clean_keyword);

    if existing_filters.len() == initial_len {
        return Err("Keyword not found in filter list".to_string());
    }

    state_handler::update_keyword_filters(user_id, existing_filters)?;

    Ok(())
}

pub fn get_keyword_filters() -> Result<Vec<String>, String> {
    let user_id = user_management::authenticate_user()?;

    Ok(state_handler::get_keyword_filters(&user_id))
}

pub fn enable_two_factor_auth() -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // In a real implementation, this would integrate with Internet Identity
    // For now, we'll just store the 2FA status
    let security_settings = SecuritySettings {
        user_id: user_id.clone(),
        two_factor_enabled: true,
        backup_codes: generate_backup_codes(),
        updated_at: state_handler::get_current_timestamp(),
    };

    state_handler::update_security_settings(user_id, security_settings)?;

    Ok(())
}

pub fn disable_two_factor_auth() -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    let security_settings = SecuritySettings {
        user_id: user_id.clone(),
        two_factor_enabled: false,
        backup_codes: Vec::new(),
        updated_at: state_handler::get_current_timestamp(),
    };

    state_handler::update_security_settings(user_id, security_settings)?;

    Ok(())
}

pub fn get_user_activity_insights() -> Result<ActivityInsights, String> {
    let user_id = user_management::authenticate_user()?;

    // Get user activities from the last week
    let current_time = state_handler::get_current_timestamp();
    let one_week_ago = current_time - (7 * 24 * 60 * 60 * 1000); // 7 days in milliseconds

    let activities = state_handler::get_user_activities(&user_id);
    let recent_activities: Vec<&UserActivity> = activities
        .iter()
        .filter(|activity| activity.created_at > one_week_ago)
        .collect();

    // Calculate statistics
    let posts_this_week = recent_activities
        .iter()
        .filter(|a| matches!(a.action, ActivityAction::PostCreated))
        .count() as u32;

    let stories_this_week = recent_activities
        .iter()
        .filter(|a| matches!(a.action, ActivityAction::StoryViewed))
        .count() as u32;

    let messages_this_week = recent_activities
        .iter()
        .filter(|a| matches!(a.action, ActivityAction::MessageSent))
        .count() as u32;

    // Calculate most active day and hour (simplified)
    let most_active_day = calculate_most_active_day(&recent_activities);
    let most_active_hour = calculate_most_active_hour(&recent_activities);

    // Estimate time spent (simplified calculation)
    let weekly_total_time = recent_activities.len() as u32 * 5; // Assume 5 minutes per activity
    let daily_average_time = weekly_total_time / 7;

    let insights = ActivityInsights {
        daily_average_time,
        weekly_total_time,
        posts_this_week,
        stories_this_week,
        messages_this_week,
        most_active_day,
        most_active_hour,
    };

    Ok(insights)
}

pub fn set_time_limit(daily_limit_minutes: u32) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    if daily_limit_minutes > 1440 {
        // 24 hours
        return Err("Daily limit cannot exceed 24 hours".to_string());
    }

    let time_limit_settings = TimeLimitSettings {
        user_id: user_id.clone(),
        daily_limit_minutes,
        break_reminders: true,
        updated_at: state_handler::get_current_timestamp(),
    };

    state_handler::update_time_limit_settings(user_id, time_limit_settings)?;

    Ok(())
}

pub fn get_time_limit() -> Result<Option<u32>, String> {
    let user_id = user_management::authenticate_user()?;

    match state_handler::get_time_limit_settings(&user_id) {
        Some(settings) => Ok(Some(settings.daily_limit_minutes)),
        None => Ok(None),
    }
}

// Helper functions
fn generate_backup_codes() -> Vec<String> {
    let mut codes = Vec::new();
    for _ in 0..8 {
        codes.push(state_handler::generate_id()[..8].to_string());
    }
    codes
}

fn calculate_most_active_day(activities: &[&UserActivity]) -> String {
    // Simplified: just return the most common day
    // In a real implementation, you'd calculate based on timestamps
    let days = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    if activities.is_empty() {
        return "Monday".to_string();
    }
    days[activities.len() % 7].to_string()
}

fn calculate_most_active_hour(activities: &[&UserActivity]) -> u32 {
    // Simplified: return hour based on activity count
    // In a real implementation, you'd parse timestamps
    if activities.is_empty() {
        return 12;
    }
    (activities.len() % 24) as u32
}

// Additional types for privacy and security settings
#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PrivacySettings {
    pub user_id: String,
    pub is_private: bool,
    pub hide_activity_status: bool,
    pub hide_likes: bool,
    pub updated_at: u64,
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CommentControls {
    pub user_id: String,
    pub allow_comments: bool,
    pub filter_keywords: Vec<String>,
    pub hide_offensive: bool,
    pub updated_at: u64,
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SecuritySettings {
    pub user_id: String,
    pub two_factor_enabled: bool,
    pub backup_codes: Vec<String>,
    pub updated_at: u64,
}

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TimeLimitSettings {
    pub user_id: String,
    pub daily_limit_minutes: u32,
    pub break_reminders: bool,
    pub updated_at: u64,
}

// Custom type for activity insights
#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ActivityInsights {
    pub daily_average_time: u32, // minutes
    pub weekly_total_time: u32,  // minutes
    pub posts_this_week: u32,
    pub stories_this_week: u32,
    pub messages_this_week: u32,
    pub most_active_day: String,
    pub most_active_hour: u32,
}
