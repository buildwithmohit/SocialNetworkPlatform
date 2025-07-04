use candid::Principal;
use ic_cdk::{caller, query, update};
use std::collections::HashMap;

mod content_management;
mod discovery;
mod messaging;
mod notifications;
mod profile_management;
mod safety_privacy;
mod shopping;
mod social_features;
mod state_handler;
mod types;
mod user_management;

use types::*;

// Environment configuration
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Production,
}

static mut APP_CONFIG: Option<AppConfig> = None;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: Environment,
    pub allow_anonymous: bool,
    pub admin_principals: Vec<Principal>,
    pub development_users: HashMap<String, String>, // user_id -> display_name
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            allow_anonymous: true,
            admin_principals: vec![],
            development_users: HashMap::new(),
        }
    }
}

// Export main API functions
#[query]
fn get_user_profile(user_id: String) -> Result<UserProfile, String> {
    profile_management::get_user_profile(user_id)
}

#[update]
fn create_user_profile(profile_data: CreateUserProfileRequest) -> Result<UserProfile, String> {
    profile_management::create_user_profile(profile_data)
}

#[update]
fn create_multiple_profiles(count: u32) -> Result<Vec<UserProfile>, String> {
    user_management::create_multiple_profiles(count)
}

#[update]
fn update_user_profile(profile_data: UpdateUserProfileRequest) -> Result<UserProfile, String> {
    profile_management::update_user_profile(profile_data)
}

#[update]
fn create_post(post_data: CreatePostRequest) -> Result<Post, String> {
    content_management::create_post(post_data)
}

#[query]
fn get_feed(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Post>, String> {
    content_management::get_feed(limit.unwrap_or(20), offset.unwrap_or(0))
}

#[update]
fn like_post(post_id: String) -> Result<(), String> {
    social_features::like_post(post_id)
}

#[update]
fn comment_on_post(post_id: String, content: String) -> Result<Comment, String> {
    social_features::comment_on_post(post_id, content)
}

#[update]
fn follow_user(user_id: String) -> Result<(), String> {
    social_features::follow_user(user_id)
}

#[update]
fn send_message(
    recipient_id: String,
    content: String,
    message_type: MessageType,
) -> Result<Message, String> {
    messaging::send_message(recipient_id, content, message_type)
}

#[query]
fn get_messages(conversation_id: String, limit: Option<u32>) -> Result<Vec<Message>, String> {
    messaging::get_messages(conversation_id, limit.unwrap_or(50))
}

#[query]
fn search_content(query: String, search_type: SearchType) -> Result<SearchResults, String> {
    discovery::search_content(query, search_type)
}

#[query]
fn get_explore_content(limit: Option<u32>) -> Result<Vec<Post>, String> {
    discovery::get_explore_content(limit.unwrap_or(20))
}

// User Management Functions
#[update]
fn register_user(user_id: String) -> Result<(), String> {
    user_management::register_user(user_id)
}

#[query]
fn get_user_info(user_id: String) -> Result<UserProfile, String> {
    user_management::get_user_info(&user_id)
}

#[query]
fn search_users(query: String, limit: u32) -> Vec<UserProfile> {
    user_management::search_users(&query, limit)
}

#[query]
fn get_user_activity(user_id: String) -> Vec<UserActivity> {
    user_management::get_user_activity(&user_id)
}

#[update]
fn block_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::block_user(&current_user, &target_user_id)
}

#[update]
fn unblock_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::unblock_user(&current_user, &target_user_id)
}

#[update]
fn mute_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::mute_user(&current_user, &target_user_id)
}

#[update]
fn unmute_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::unmute_user(&current_user, &target_user_id)
}

#[update]
fn restrict_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::restrict_user(&current_user, &target_user_id)
}

#[update]
fn unrestrict_user(target_user_id: String) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::unrestrict_user(&current_user, &target_user_id)
}

#[query]
fn get_blocked_users() -> Vec<String> {
    let current_user = get_current_user().unwrap_or_default();
    user_management::get_blocked_users(&current_user)
}

#[query]
fn get_muted_users() -> Vec<String> {
    let current_user = get_current_user().unwrap_or_default();
    user_management::get_muted_users(&current_user)
}

#[query]
fn get_restricted_users() -> Vec<String> {
    let current_user = get_current_user().unwrap_or_default();
    user_management::get_restricted_users(&current_user)
}

#[update]
fn update_last_seen() -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::update_last_seen(&current_user)
}

#[update]
fn set_online_status(is_online: bool) -> Result<(), String> {
    let current_user = get_current_user()?;
    user_management::set_online_status(&current_user, is_online)
}

#[query]
fn get_online_status(user_id: String) -> bool {
    user_management::get_online_status(&user_id)
}

// Content Management Functions
#[query]
fn get_post(post_id: String) -> Result<Post, String> {
    content_management::get_post(&post_id)
}

#[update]
fn update_post(
    post_id: String,
    caption: Option<String>,
    hashtags: Option<Vec<String>>,
) -> Result<Post, String> {
    content_management::update_post(post_id, caption, hashtags)
}

#[update]
fn delete_post(post_id: String) -> Result<(), String> {
    content_management::delete_post(post_id)
}

#[query]
fn get_user_posts(user_id: String, limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    content_management::get_user_posts(&user_id, limit, offset)
}

#[update]
fn archive_post(post_id: String) -> Result<Post, String> {
    content_management::archive_post(post_id)
}

#[update]
fn unarchive_post(post_id: String) -> Result<Post, String> {
    content_management::unarchive_post(post_id)
}

#[query]
fn get_archived_posts(limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    content_management::get_archived_posts(limit, offset)
}

#[update]
fn create_story(
    media_url: String,
    story_type: StoryType,
    text_overlay: Option<String>,
    stickers: Vec<Sticker>,
    music_info: Option<MusicInfo>,
) -> Result<Story, String> {
    content_management::create_story(media_url, story_type, text_overlay, stickers, music_info)
}

#[update]
fn view_story(story_id: String) -> Result<Story, String> {
    content_management::view_story(story_id)
}

#[query]
fn get_user_stories(user_id: String) -> Result<Vec<Story>, String> {
    content_management::get_user_stories(&user_id)
}

#[query]
fn get_trending_posts(limit: u32) -> Result<Vec<Post>, String> {
    content_management::get_trending_posts(limit)
}

#[query]
fn get_posts_by_hashtag(hashtag: String, limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    content_management::get_posts_by_hashtag(&hashtag, limit, offset)
}

// Social Features Functions
#[update]
fn unlike_post(post_id: String) -> Result<(), String> {
    social_features::unlike_post(post_id)
}

#[update]
fn reply_to_comment(comment_id: String, content: String) -> Result<Comment, String> {
    social_features::reply_to_comment(comment_id, content)
}

#[query]
fn get_post_comments(post_id: String, limit: u32, offset: u32) -> Result<Vec<Comment>, String> {
    social_features::get_post_comments(post_id, limit, offset)
}

#[query]
fn get_comment_replies(
    comment_id: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<Comment>, String> {
    social_features::get_comment_replies(comment_id, limit, offset)
}

#[update]
fn unfollow_user(user_id: String) -> Result<(), String> {
    social_features::unfollow_user(user_id)
}

#[query]
fn get_followers(user_id: String, limit: u32, offset: u32) -> Result<Vec<UserProfile>, String> {
    social_features::get_followers(user_id, limit, offset)
}

#[query]
fn get_following(user_id: String, limit: u32, offset: u32) -> Result<Vec<UserProfile>, String> {
    social_features::get_following(user_id, limit, offset)
}

#[update]
fn save_post(post_id: String, collection_name: Option<String>) -> Result<(), String> {
    social_features::save_post(post_id, collection_name)
}

#[update]
fn unsave_post(post_id: String) -> Result<(), String> {
    social_features::unsave_post(post_id)
}

#[query]
fn get_saved_posts(limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    social_features::get_saved_posts(limit, offset)
}

#[update]
fn share_post(post_id: String, target_user_ids: Vec<String>) -> Result<(), String> {
    social_features::share_post(post_id, target_user_ids)
}

#[update]
fn create_close_friends_list(friend_ids: Vec<String>) -> Result<(), String> {
    social_features::create_close_friends_list(friend_ids)
}

#[update]
fn add_to_close_friends(friend_id: String) -> Result<(), String> {
    social_features::add_to_close_friends(friend_id)
}

#[update]
fn remove_from_close_friends(friend_id: String) -> Result<(), String> {
    social_features::remove_from_close_friends(friend_id)
}

#[query]
fn get_close_friends() -> Result<Vec<UserProfile>, String> {
    social_features::get_close_friends()
}

// Discovery Functions
#[query]
fn get_trending_hashtags(limit: u32) -> Result<Vec<Hashtag>, String> {
    discovery::get_trending_hashtags(limit)
}

#[query]
fn get_suggested_users(limit: u32) -> Result<Vec<UserProfile>, String> {
    discovery::get_suggested_users(limit)
}

#[query]
fn get_posts_by_location(
    location: LocationTag,
    limit: u32,
    offset: u32,
) -> Result<Vec<Post>, String> {
    discovery::get_posts_by_location(location, limit, offset)
}

#[query]
fn get_nearby_locations(
    latitude: f64,
    longitude: f64,
    radius_km: f64,
) -> Result<Vec<LocationTag>, String> {
    discovery::get_nearby_locations(latitude, longitude, radius_km)
}

// Messaging Functions
#[update]
fn send_media_message(
    recipient_id: String,
    media_url: String,
    message_type: MessageType,
    caption: Option<String>,
) -> Result<Message, String> {
    messaging::send_media_message(recipient_id, media_url, message_type, caption)
}

#[update]
fn reply_to_message(
    original_message_id: String,
    content: String,
    message_type: MessageType,
) -> Result<Message, String> {
    messaging::reply_to_message(original_message_id, content, message_type)
}

#[query]
fn get_conversations(limit: u32, offset: u32) -> Result<Vec<Conversation>, String> {
    messaging::get_conversations(limit, offset)
}

#[update]
fn mark_message_as_read(message_id: String) -> Result<(), String> {
    messaging::mark_message_as_read(message_id)
}

#[update]
fn mark_conversation_as_read(conversation_id: String) -> Result<(), String> {
    messaging::mark_conversation_as_read(conversation_id)
}

#[update]
fn add_reaction_to_message(message_id: String, emoji: String) -> Result<(), String> {
    messaging::add_reaction_to_message(message_id, emoji)
}

#[update]
fn remove_reaction_from_message(message_id: String, emoji: String) -> Result<(), String> {
    messaging::remove_reaction_from_message(message_id, emoji)
}

#[update]
fn delete_message(message_id: String) -> Result<(), String> {
    messaging::delete_message(message_id)
}

#[update]
fn create_group_chat(
    participants: Vec<String>,
    group_name: String,
    group_photo: Option<String>,
) -> Result<Conversation, String> {
    messaging::create_group_chat(participants, group_name, group_photo)
}

#[update]
fn add_participant_to_group(conversation_id: String, participant_id: String) -> Result<(), String> {
    messaging::add_participant_to_group(conversation_id, participant_id)
}

#[update]
fn remove_participant_from_group(
    conversation_id: String,
    participant_id: String,
) -> Result<(), String> {
    messaging::remove_participant_from_group(conversation_id, participant_id)
}

#[update]
fn leave_group(conversation_id: String) -> Result<(), String> {
    messaging::leave_group(conversation_id)
}

#[update]
fn make_group_admin(conversation_id: String, participant_id: String) -> Result<(), String> {
    messaging::make_group_admin(conversation_id, participant_id)
}

#[update]
fn enable_vanish_mode(conversation_id: String) -> Result<(), String> {
    messaging::enable_vanish_mode(conversation_id)
}

#[update]
fn disable_vanish_mode(conversation_id: String) -> Result<(), String> {
    messaging::disable_vanish_mode(conversation_id)
}

// Notification Functions
#[update]
fn send_notification(
    user_id: String,
    notification_type: NotificationType,
    title: String,
    message: String,
    action_user_id: Option<String>,
    post_id: Option<String>,
    comment_id: Option<String>,
) -> Result<Notification, String> {
    notifications::send_notification(
        user_id,
        notification_type,
        title,
        message,
        action_user_id,
        post_id,
        comment_id,
    )
}

#[query]
fn get_notifications(limit: u32, offset: u32) -> Result<Vec<Notification>, String> {
    notifications::get_notifications(limit, offset)
}

#[update]
fn mark_notification_as_read(notification_id: String) -> Result<(), String> {
    notifications::mark_notification_as_read(notification_id)
}

#[update]
fn mark_all_notifications_as_read() -> Result<(), String> {
    notifications::mark_all_notifications_as_read()
}

#[update]
fn delete_notification(notification_id: String) -> Result<(), String> {
    notifications::delete_notification(notification_id)
}

#[query]
fn get_unread_notification_count() -> Result<u32, String> {
    notifications::get_unread_notification_count()
}

#[update]
fn update_notification_settings(types: Vec<NotificationType>, enabled: bool) -> Result<(), String> {
    notifications::update_notification_settings(types, enabled)
}

#[query]
fn get_notification_settings() -> Result<NotificationSettings, String> {
    notifications::get_notification_settings()
}

// Profile Management Functions
#[query]
fn get_current_user_profile() -> Result<UserProfile, String> {
    profile_management::get_current_user_profile()
}

#[update]
fn delete_user_profile() -> Result<(), String> {
    profile_management::delete_user_profile()
}

#[update]
fn switch_account_type(account_type: AccountType) -> Result<UserProfile, String> {
    profile_management::switch_account_type(account_type)
}

#[update]
fn toggle_privacy_setting() -> Result<UserProfile, String> {
    profile_management::toggle_privacy_setting()
}

#[update]
fn update_profile_picture(image_url: String) -> Result<UserProfile, String> {
    profile_management::update_profile_picture(image_url)
}

#[update]
fn remove_profile_picture() -> Result<UserProfile, String> {
    profile_management::remove_profile_picture()
}

#[update]
fn update_bio(bio: String) -> Result<UserProfile, String> {
    profile_management::update_bio(bio)
}

#[update]
fn add_website_link(website: String) -> Result<UserProfile, String> {
    profile_management::add_website_link(website)
}

#[update]
fn add_profile_link(link: String) -> Result<UserProfile, String> {
    profile_management::add_profile_link(link)
}

#[update]
fn remove_profile_link(link: String) -> Result<UserProfile, String> {
    profile_management::remove_profile_link(link)
}

#[query]
fn get_profile_analytics(user_id: String) -> Result<Analytics, String> {
    profile_management::get_profile_analytics(&user_id)
}

#[update]
fn verify_account(user_id: String) -> Result<UserProfile, String> {
    profile_management::verify_account(&user_id)
}

#[query]
fn get_public_profile_info(user_id: String) -> Result<UserProfile, String> {
    profile_management::get_public_profile_info(&user_id)
}

// Safety & Privacy Functions
#[update]
fn report_content(
    reported_user_id: Option<String>,
    reported_post_id: Option<String>,
    reported_comment_id: Option<String>,
    reason: ReportReason,
    description: String,
) -> Result<Report, String> {
    safety_privacy::report_content(
        reported_user_id,
        reported_post_id,
        reported_comment_id,
        reason,
        description,
    )
}

#[update]
fn update_privacy_settings(
    is_private: bool,
    hide_activity_status: bool,
    hide_likes: bool,
) -> Result<(), String> {
    safety_privacy::update_privacy_settings(is_private, hide_activity_status, hide_likes)
}

#[update]
fn update_comment_controls(
    allow_comments: bool,
    filter_keywords: Vec<String>,
    hide_offensive: bool,
) -> Result<(), String> {
    safety_privacy::update_comment_controls(allow_comments, filter_keywords, hide_offensive)
}

#[update]
fn add_keyword_filter(keyword: String) -> Result<(), String> {
    safety_privacy::add_keyword_filter(keyword)
}

#[update]
fn remove_keyword_filter(keyword: String) -> Result<(), String> {
    safety_privacy::remove_keyword_filter(keyword)
}

#[query]
fn get_keyword_filters() -> Result<Vec<String>, String> {
    safety_privacy::get_keyword_filters()
}

#[update]
fn enable_two_factor_auth() -> Result<(), String> {
    safety_privacy::enable_two_factor_auth()
}

#[update]
fn disable_two_factor_auth() -> Result<(), String> {
    safety_privacy::disable_two_factor_auth()
}

#[query]
fn get_user_activity_insights() -> Result<types::ActivityInsights, String> {
    let current_user = get_current_user()?;
    safety_privacy::get_user_activity_insights().map(|insights| types::ActivityInsights {
        user_id: current_user,
        total_time_spent: insights.weekly_total_time as u64,
        daily_average: insights.daily_average_time as u64,
        posts_created: insights.posts_this_week,
        stories_created: insights.stories_this_week,
        messages_sent: insights.messages_this_week,
        likes_given: 0,      // Not available in safety_privacy::ActivityInsights
        comments_made: 0,    // Not available in safety_privacy::ActivityInsights
        most_active_hour: 0, // Not available in safety_privacy::ActivityInsights
        weekly_summary: Vec::new(), // Not available in safety_privacy::ActivityInsights
    })
}

#[update]
fn set_time_limit(daily_limit_minutes: u32) -> Result<(), String> {
    safety_privacy::set_time_limit(daily_limit_minutes)
}

#[query]
fn get_time_limit() -> Result<Option<u32>, String> {
    safety_privacy::get_time_limit()
}

// Shopping Functions
#[update]
fn create_shop(
    name: String,
    description: String,
    website: Option<String>,
    contact_email: String,
) -> Result<Shop, String> {
    shopping::create_shop(name, description, website, contact_email)
}

#[update]
fn add_product(
    shop_id: String,
    name: String,
    description: String,
    price: String,
    currency: String,
    images: Vec<String>,
    category: String,
    inventory_count: Option<u32>,
) -> Result<Product, String> {
    shopping::add_product(
        shop_id,
        name,
        description,
        price,
        currency,
        images,
        category,
        inventory_count,
    )
}

#[update]
fn update_product(
    product_id: String,
    name: Option<String>,
    description: Option<String>,
    price: Option<String>,
    is_available: Option<bool>,
    inventory_count: Option<u32>,
) -> Result<Product, String> {
    shopping::update_product(
        product_id,
        name,
        description,
        price,
        is_available,
        inventory_count,
    )
}

#[update]
fn delete_product(product_id: String) -> Result<(), String> {
    shopping::delete_product(product_id)
}

#[query]
fn get_shop_products(shop_id: String, limit: u32, offset: u32) -> Result<Vec<Product>, String> {
    shopping::get_shop_products(shop_id, limit, offset)
}

#[query]
fn search_products(
    query: String,
    category: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
    limit: u32,
) -> Result<Vec<Product>, String> {
    shopping::search_products(query, category, min_price, max_price, limit)
}

#[query]
fn get_product_details(product_id: String) -> Result<Product, String> {
    shopping::get_product_details(product_id)
}

// Initialize canister with default development configuration
#[ic_cdk::init]
fn init() {
    unsafe {
        APP_CONFIG = Some(AppConfig::default());
    }
    state_handler::init_state();
}

// Pre-upgrade hook
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    state_handler::save_state();
}

// Post-upgrade hook
#[ic_cdk::post_upgrade]
fn post_upgrade() {
    state_handler::restore_state();
}

// Helper function to get current user with environment-aware authentication
pub fn get_current_user() -> Result<String, String> {
    let caller_principal = caller();

    unsafe {
        let default_config = AppConfig::default();
        let config = APP_CONFIG.as_ref().unwrap_or(&default_config);

        match config.environment {
            Environment::Development => {
                if caller_principal == Principal::anonymous() {
                    if config.allow_anonymous {
                        // In development, generate a unique user ID for anonymous calls
                        Ok(state_handler::generate_id())
                    } else {
                        Err("Anonymous access disabled".to_string())
                    }
                } else {
                    Ok(caller_principal.to_string())
                }
            }
            Environment::Production => {
                if caller_principal == Principal::anonymous() {
                    Err("Anonymous access not allowed in production".to_string())
                } else {
                    Ok(caller_principal.to_string())
                }
            }
        }
    }
}

// Environment management functions
#[update]
pub fn reset_to_development() -> Result<(), String> {
    unsafe {
        APP_CONFIG = Some(AppConfig {
            environment: Environment::Development,
            allow_anonymous: true,
            admin_principals: vec![caller()],
            development_users: {
                let mut users = HashMap::new();
                users.insert(
                    state_handler::generate_id(),
                    "Anonymous Dev User".to_string(),
                );
                users.insert(state_handler::generate_id(), "Test User 1".to_string());
                users.insert(state_handler::generate_id(), "Test User 2".to_string());
                users
            },
        });
    }
    Ok(())
}

#[update]
pub fn set_production_mode(admin_principals: Vec<String>) -> Result<(), String> {
    let caller_principal = caller();

    // Convert string principals to Principal type
    let admin_principals: Result<Vec<Principal>, _> = admin_principals
        .into_iter()
        .map(|p| Principal::from_text(p))
        .collect();

    let admin_principals = admin_principals.map_err(|e| format!("Invalid principal: {}", e))?;

    unsafe {
        APP_CONFIG = Some(AppConfig {
            environment: Environment::Production,
            allow_anonymous: false,
            admin_principals: {
                let mut admins = admin_principals;
                if !admins.contains(&caller_principal) {
                    admins.push(caller_principal);
                }
                admins
            },
            development_users: HashMap::new(),
        });
    }
    Ok(())
}

#[query]
pub fn is_development_mode() -> bool {
    unsafe {
        match &APP_CONFIG {
            Some(config) => matches!(config.environment, Environment::Development),
            None => true, // Default to development
        }
    }
}

// Greet function
#[query]
fn greet(name: String) -> String {
    let env_info = unsafe {
        match &APP_CONFIG {
            Some(config) => format!(" (Environment: {:?})", config.environment),
            None => " (Environment: Not configured)".to_string(),
        }
    };
    format!("Hello, {}!{}", name, env_info)
}

#[query]
pub fn get_all_profiles() -> Result<Vec<UserProfile>, String> {
    profile_management::get_all_profiles()
}

// Export Candid interface
ic_cdk::export_candid!();
