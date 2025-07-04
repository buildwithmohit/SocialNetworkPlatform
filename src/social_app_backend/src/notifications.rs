use crate::state_handler;
use crate::types::*;
use crate::user_management;

pub fn send_notification(
    user_id: String,
    notification_type: NotificationType,
    title: String,
    message: String,
    action_user_id: Option<String>,
    post_id: Option<String>,
    comment_id: Option<String>,
) -> Result<Notification, String> {
    let notification_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();

    let notification = Notification {
        notification_id: notification_id.clone(),
        user_id: user_id.clone(),
        notification_type,
        title,
        message,
        action_user_id,
        post_id,
        comment_id,
        is_read: false,
        created_at: current_time,
        read_at: None,
    };

    state_handler::add_notification(user_id, notification.clone());
    Ok(notification)
}

pub fn get_notifications(limit: u32, offset: u32) -> Result<Vec<Notification>, String> {
    let user_id = user_management::authenticate_user()?;

    let notifications = state_handler::get_user_notifications(&user_id, limit + offset);

    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;

    if start >= notifications.len() {
        return Ok(Vec::new());
    }

    let end = end.min(notifications.len());
    Ok(notifications[start..end].to_vec())
}

pub fn mark_notification_as_read(notification_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    state_handler::mark_notification_as_read(&user_id, &notification_id)
        .map_err(|e| format!("Failed to mark notification as read: {}", e))?;

    Ok(())
}

pub fn mark_all_notifications_as_read() -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    state_handler::mark_all_notifications_as_read(&user_id)
        .map_err(|e| format!("Failed to mark all notifications as read: {}", e))?;

    Ok(())
}

pub fn delete_notification(notification_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    state_handler::delete_notification(&user_id, &notification_id)
        .map_err(|e| format!("Failed to delete notification: {}", e))?;

    Ok(())
}

pub fn get_unread_notification_count() -> Result<u32, String> {
    let user_id = user_management::authenticate_user()?;

    let notifications = state_handler::get_user_notifications(&user_id, 1000); // Get recent notifications
    let unread_count = notifications.iter().filter(|n| !n.is_read).count() as u32;

    Ok(unread_count)
}

pub fn update_notification_settings(
    types: Vec<NotificationType>,
    enabled: bool,
) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Get current settings or create default ones
    let mut settings = match state_handler::get_notification_settings(&user_id) {
        Some(settings) => settings,
        None => NotificationSettings {
            likes_enabled: true,
            comments_enabled: true,
            follows_enabled: true,
            mentions_enabled: true,
            tags_enabled: true,
            messages_enabled: true,
            user_id: user_id.clone(),
            story_views_enabled: true,
            push_notifications: true,
            email_notifications: false,
        }
    };

    // Update the specified notification types
    for notification_type in types {
        match notification_type {
            NotificationType::Like => settings.likes_enabled = enabled,
            NotificationType::Comment => settings.comments_enabled = enabled,
            NotificationType::Follow => settings.follows_enabled = enabled,
            NotificationType::Mention => settings.mentions_enabled = enabled,
            NotificationType::Tag => settings.tags_enabled = enabled,
            NotificationType::Message => settings.messages_enabled = enabled,
            NotificationType::Story => settings.story_views_enabled = enabled,
            NotificationType::Live => todo!(),
            NotificationType::Request => todo!(),
            NotificationType::System => todo!(),
        }
    }

    state_handler::update_notification_settings(user_id, settings)
        .map_err(|e| format!("Failed to update notification settings: {}", e))?;

    Ok(())
}

pub fn get_notification_settings() -> Result<NotificationSettings, String> {
    let user_id = user_management::authenticate_user()?;

    match state_handler::get_notification_settings(&user_id) {
        Some(settings) => Ok(settings),
        None => {
            // Return default settings if none exist
            let default_settings = NotificationSettings {
                likes_enabled: true,
                comments_enabled: true,
                follows_enabled: true,
                mentions_enabled: true,
                tags_enabled: true,
                messages_enabled: true,
                user_id: user_id.clone(),
                story_views_enabled: true,
                push_notifications: true,
                email_notifications: false,
            };

            // Save default settings for future use
            state_handler::save_notification_settings(user_id.clone(), default_settings.clone())
                .map_err(|e| format!("Failed to save default notification settings: {}", e))?;

            Ok(default_settings)
        }
    }
}

// Helper functions to create specific notifications
#[allow(dead_code)]
pub fn notify_like(post_owner_id: String, liker_id: String, post_id: String) -> Result<(), String> {
    let liker_profile =
        state_handler::get_user(&liker_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        post_owner_id,
        NotificationType::Like,
        "New Like".to_string(),
        format!("{} liked your post", liker_profile.username),
        Some(liker_id),
        Some(post_id),
        None,
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_comment(
    post_owner_id: String,
    commenter_id: String,
    post_id: String,
    comment_id: String,
) -> Result<(), String> {
    let commenter_profile =
        state_handler::get_user(&commenter_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        post_owner_id,
        NotificationType::Comment,
        "New Comment".to_string(),
        format!("{} commented on your post", commenter_profile.username),
        Some(commenter_id),
        Some(post_id),
        Some(comment_id),
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_follow(followed_user_id: String, follower_id: String) -> Result<(), String> {
    let follower_profile =
        state_handler::get_user(&follower_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        followed_user_id,
        NotificationType::Follow,
        "New Follower".to_string(),
        format!("{} started following you", follower_profile.username),
        Some(follower_id),
        None,
        None,
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_mention(
    mentioned_user_id: String,
    mentioner_id: String,
    post_id: Option<String>,
    comment_id: Option<String>,
) -> Result<(), String> {
    let mentioner_profile =
        state_handler::get_user(&mentioner_id).ok_or_else(|| "User not found".to_string())?;

    let message = if post_id.is_some() {
        format!("{} mentioned you in a post", mentioner_profile.username)
    } else if comment_id.is_some() {
        format!("{} mentioned you in a comment", mentioner_profile.username)
    } else {
        format!("{} mentioned you", mentioner_profile.username)
    };

    send_notification(
        mentioned_user_id,
        NotificationType::Mention,
        "You were mentioned".to_string(),
        message,
        Some(mentioner_id),
        post_id,
        comment_id,
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_tag(
    tagged_user_id: String,
    tagger_id: String,
    post_id: String,
) -> Result<(), String> {
    let tagger_profile =
        state_handler::get_user(&tagger_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        tagged_user_id,
        NotificationType::Tag,
        "You were tagged".to_string(),
        format!("{} tagged you in a post", tagger_profile.username),
        Some(tagger_id),
        Some(post_id),
        None,
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_story_view(story_owner_id: String, viewer_id: String) -> Result<(), String> {
    let viewer_profile =
        state_handler::get_user(&viewer_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        story_owner_id,
        NotificationType::Story,
        "Story View".to_string(),
        format!("{} viewed your story", viewer_profile.username),
        Some(viewer_id),
        None,
        None,
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn notify_message(recipient_id: String, sender_id: String) -> Result<(), String> {
    let sender_profile =
        state_handler::get_user(&sender_id).ok_or_else(|| "User not found".to_string())?;

    send_notification(
        recipient_id,
        NotificationType::Message,
        "New Message".to_string(),
        format!("New message from {}", sender_profile.username),
        Some(sender_id),
        None,
        None,
    )?;

    Ok(())
}
