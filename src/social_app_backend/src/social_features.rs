use crate::notifications;
use crate::state_handler;
use crate::types::*;
use crate::user_management;

pub fn like_post(post_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists
    let post = state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    // Check if user can view this post
    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    // Check if user already liked this post
    if state_handler::has_user_liked_post(&post_id, &user_id) {
        return Err("Post already liked".to_string());
    }

    state_handler::add_post_like(post_id.clone(), user_id.clone())?;

    // Log user activity
    user_management::log_user_activity(
        user_id.clone(),
        ActivityAction::PostLiked,
        Some(post_id.clone()),
        Some("post".to_string()),
    );

    // Send notification to post owner (if not liking own post)
    if post.user_id != user_id {
        let _ = notifications::notify_like(post.user_id, user_id, post_id);
    }

    Ok(())
}

pub fn unlike_post(post_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists
    state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    // Check if user has liked this post
    if !state_handler::has_user_liked_post(&post_id, &user_id) {
        return Err("Post not liked".to_string());
    }

    state_handler::remove_post_like(&post_id, &user_id)?;
    Ok(())
}

pub fn comment_on_post(post_id: String, content: String) -> Result<Comment, String> {
    let user_id = user_management::authenticate_user()?;

    // Get user profile to get username
    let user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // Check if post exists
    let post = state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    // Check if user can view this post
    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    // Validate comment content
    if content.trim().is_empty() {
        return Err("Comment cannot be empty".to_string());
    }

    if content.len() > 2200 {
        return Err("Comment must be 2200 characters or less".to_string());
    }

    let comment_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();

    // Extract mentions from comment
    let mentions = extract_mentions(&content);

    let comment = Comment {
        comment_id: comment_id.clone(),
        post_id: post_id.clone(),
        user_id: user_id.clone(),
        username: user_profile.username,
        content: content.clone(),
        likes_count: 0,
        replies_count: 0,
        parent_comment_id: None,
        created_at: current_time,
        updated_at: current_time,
        is_pinned: false,
        mentions: mentions.clone(),
    };

    state_handler::insert_comment(comment_id.clone(), comment.clone());

    // Log user activity
    user_management::log_user_activity(
        user_id.clone(),
        ActivityAction::PostCommented,
        Some(post_id.clone()),
        Some("comment".to_string()),
    );

    // Send notification to post owner (if not commenting on own post)
    if post.user_id != user_id {
        let _ = notifications::notify_comment(
            post.user_id,
            user_id.clone(),
            post_id.clone(),
            comment_id.clone(),
        );
    }

    // Send notifications to mentioned users
    for mention in mentions {
        if let Some(_mentioned_user) = state_handler::get_user(&mention) {
            let _ = notifications::notify_mention(
                mention,
                user_id.clone(),
                Some(post_id.clone()),
                Some(comment_id.clone()),
            );
        }
    }

    Ok(comment)
}

pub fn reply_to_comment(comment_id: String, content: String) -> Result<Comment, String> {
    let user_id = user_management::authenticate_user()?;

    // Get user profile to get username
    let user_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // Check if parent comment exists
    let parent_comment =
        state_handler::get_comment(&comment_id).ok_or_else(|| "Comment not found".to_string())?;

    // Check if post exists and user can view it
    let post = state_handler::get_post(&parent_comment.post_id)
        .ok_or_else(|| "Post not found".to_string())?;

    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    // Validate comment content
    if content.trim().is_empty() {
        return Err("Reply cannot be empty".to_string());
    }

    if content.len() > 2200 {
        return Err("Reply must be 2200 characters or less".to_string());
    }

    let reply_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();

    // Extract mentions from reply
    let mentions = extract_mentions(&content);

    let reply = Comment {
        comment_id: reply_id.clone(),
        post_id: parent_comment.post_id.clone(),
        user_id: user_id.clone(),
        username: user_profile.username,
        content: content.clone(),
        likes_count: 0,
        replies_count: 0,
        parent_comment_id: Some(comment_id.clone()),
        created_at: current_time,
        updated_at: current_time,
        is_pinned: false,
        mentions: mentions.clone(),
    };

    state_handler::insert_comment(reply_id.clone(), reply.clone());

    // Update parent comment's reply count
    let _ = state_handler::update_comment_reply_count(&comment_id, true);

    // Log user activity
    user_management::log_user_activity(
        user_id.clone(),
        ActivityAction::PostCommented,
        Some(parent_comment.post_id.clone()),
        Some("reply".to_string()),
    );

    // Send notification to parent comment owner (if not replying to own comment)
    if parent_comment.user_id != user_id {
        let _ = notifications::notify_comment(
            parent_comment.user_id,
            user_id.clone(),
            parent_comment.post_id.clone(),
            reply_id.clone(),
        );
    }

    // Send notifications to mentioned users
    for mention in mentions {
        if let Some(_mentioned_user) = state_handler::get_user(&mention) {
            let _ = notifications::notify_mention(
                mention,
                user_id.clone(),
                Some(parent_comment.post_id.clone()),
                Some(reply_id.clone()),
            );
        }
    }

    Ok(reply)
}

pub fn get_post_comments(post_id: String, limit: u32, offset: u32) -> Result<Vec<Comment>, String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists and user can view it
    let post = state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    let comments = state_handler::get_post_comments(&post_id);

    // Filter out replies (only show top-level comments)
    let top_level_comments: Vec<Comment> = comments
        .into_iter()
        .filter(|comment| comment.parent_comment_id.is_none())
        .collect();

    // Sort by creation time (oldest first for comments)
    let mut sorted_comments = top_level_comments;
    sorted_comments.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;

    if start >= sorted_comments.len() {
        return Ok(Vec::new());
    }

    let end = end.min(sorted_comments.len());
    Ok(sorted_comments[start..end].to_vec())
}

pub fn get_comment_replies(
    comment_id: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<Comment>, String> {
    let user_id = user_management::authenticate_user()?;

    // Check if parent comment exists
    let parent_comment =
        state_handler::get_comment(&comment_id).ok_or_else(|| "Comment not found".to_string())?;

    // Check if post exists and user can view it
    let post = state_handler::get_post(&parent_comment.post_id)
        .ok_or_else(|| "Post not found".to_string())?;

    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    let all_comments = state_handler::get_post_comments(&parent_comment.post_id);

    // Filter replies to this comment
    let replies: Vec<Comment> = all_comments
        .into_iter()
        .filter(|comment| comment.parent_comment_id == Some(comment_id.clone()))
        .collect();

    // Sort by creation time (oldest first)
    let mut sorted_replies = replies;
    sorted_replies.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;

    if start >= sorted_replies.len() {
        return Ok(Vec::new());
    }

    let end = end.min(sorted_replies.len());
    Ok(sorted_replies[start..end].to_vec())
}

pub fn follow_user(user_id: String) -> Result<(), String> {
    let current_user = user_management::authenticate_user()?;

    if current_user == user_id {
        return Err("Cannot follow yourself".to_string());
    }

    // Check if target user exists
    if !state_handler::user_exists(&user_id) {
        return Err("User not found".to_string());
    }

    // Check if already following to prevent duplicate operations
    if state_handler::is_following(&current_user, &user_id) {
        return Err("Already following this user".to_string());
    }

    // Check if user is blocked
    if user_management::is_user_blocked(&current_user, &user_id) {
        return Err("Cannot follow blocked user".to_string());
    }

    // Add the follow relationship
    state_handler::add_follower(user_id.clone(), current_user.clone())?;

    // Log user activity
    user_management::log_user_activity(
        current_user.clone(),
        ActivityAction::UserFollowed,
        Some(user_id.clone()),
        Some("user".to_string()),
    );

    // Send notification to followed user
    let _ = notifications::notify_follow(user_id, current_user);

    Ok(())
}

pub fn unfollow_user(user_id: String) -> Result<(), String> {
    let current_user = user_management::authenticate_user()?;

    if current_user == user_id {
        return Err("Cannot unfollow yourself".to_string());
    }

    // Check if target user exists
    state_handler::get_user(&user_id).ok_or_else(|| "User not found".to_string())?;

    // Check if currently following
    if !state_handler::is_following(&current_user, &user_id) {
        return Err("Not following this user".to_string());
    }

    state_handler::remove_follower(&user_id, &current_user)?;
    Ok(())
}

pub fn get_followers(user_id: String, limit: u32, offset: u32) -> Result<Vec<UserProfile>, String> {
    let current_user = user_management::authenticate_user().ok();

    // Check if target user exists
    let target_user =
        state_handler::get_user(&user_id).ok_or_else(|| "User not found".to_string())?;

    // Check privacy settings
    if target_user.is_private {
        if let Some(current_user_id) = &current_user {
            if *current_user_id != user_id
                && !state_handler::is_following(current_user_id, &user_id)
            {
                return Err("Private account".to_string());
            }
        } else {
            return Err("Authentication required".to_string());
        }
    }

    let follower_ids = state_handler::get_followers(&user_id);

    let mut followers = Vec::new();
    for follower_id in follower_ids
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
    {
        if let Some(user_profile) = state_handler::get_user(follower_id) {
            followers.push(user_profile);
        }
    }

    Ok(followers)
}

pub fn get_following(user_id: String, limit: u32, offset: u32) -> Result<Vec<UserProfile>, String> {
    let current_user = user_management::authenticate_user().ok();

    // Check if target user exists
    let target_user =
        state_handler::get_user(&user_id).ok_or_else(|| "User not found".to_string())?;

    // Check privacy settings
    if target_user.is_private {
        if let Some(current_user_id) = &current_user {
            if *current_user_id != user_id
                && !state_handler::is_following(current_user_id, &user_id)
            {
                return Err("Private account".to_string());
            }
        } else {
            return Err("Authentication required".to_string());
        }
    }

    let following_ids = state_handler::get_following(&user_id);

    let mut following = Vec::new();
    for following_id in following_ids
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
    {
        if let Some(user_profile) = state_handler::get_user(following_id) {
            following.push(user_profile);
        }
    }

    Ok(following)
}

pub fn save_post(post_id: String, collection_name: Option<String>) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists and user can view it
    let post = state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    // Check if post is already saved
    if state_handler::is_post_saved(&user_id, &post_id) {
        return Err("Post already saved".to_string());
    }

    // Create saved post entry
    let saved_post = SavedPost {
        user_id: user_id.clone(),
        post_id: post_id.clone(),
        collection_name: collection_name.clone(),
        created_at: state_handler::get_current_timestamp(),
    };

    state_handler::save_post(user_id.clone(), saved_post)?;

    // Log user activity
    user_management::log_user_activity(
        user_id,
        ActivityAction::PostLiked, // Using closest available action
        Some(post_id),
        Some("save".to_string()),
    );

    Ok(())
}

pub fn unsave_post(post_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists in saved posts
    if !state_handler::is_post_saved(&user_id, &post_id) {
        return Err("Post not saved".to_string());
    }

    // Remove from saved posts
    state_handler::unsave_post(&user_id, &post_id)?;

    Ok(())
}

pub fn get_saved_posts(limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    let user_id = user_management::authenticate_user()?;

    // Get saved posts for the user
    let saved_posts = state_handler::get_saved_posts(&user_id);

    // Get actual post data
    let mut posts = Vec::new();
    for saved_post in saved_posts
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
    {
        if let Some(post) = state_handler::get_post(&saved_post.post_id) {
            posts.push(post);
        }
    }

    // Sort by save time (newest first)
    posts.sort_by(|a, b| {
        let a_saved = saved_posts.iter().find(|sp| sp.post_id == a.post_id);
        let b_saved = saved_posts.iter().find(|sp| sp.post_id == b.post_id);
        match (a_saved, b_saved) {
            (Some(a_save), Some(b_save)) => b_save.created_at.cmp(&a_save.created_at),
            _ => std::cmp::Ordering::Equal,
        }
    });

    Ok(posts)
}

pub fn share_post(post_id: String, target_user_ids: Vec<String>) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if post exists and user can view it
    let post = state_handler::get_post(&post_id).ok_or_else(|| "Post not found".to_string())?;

    if !can_view_post(&post, Some(&user_id)) {
        return Err("Access denied".to_string());
    }

    // Validate target users exist and can receive the post
    for target_user_id in &target_user_ids {
        if !state_handler::user_exists(target_user_id) {
            return Err(format!("User {} not found", target_user_id));
        }

        // Check if target user is blocked
        if user_management::is_user_blocked(&user_id, target_user_id)
            || user_management::is_user_blocked(target_user_id, &user_id)
        {
            return Err(format!("Cannot share with blocked user {}", target_user_id));
        }
    }

    // Get sharer's profile for message content
    let sharer_profile =
        state_handler::get_user(&user_id).ok_or_else(|| "User profile not found".to_string())?;

    // Share to each target user by creating a message
    for target_user_id in target_user_ids {
        // Create or get conversation
        let conversation_id = state_handler::get_or_create_conversation(&user_id, &target_user_id)?;

        // Create share message
        let message_id = state_handler::generate_id();
        let current_time = state_handler::get_current_timestamp();

        let share_message = Message {
            message_id: message_id.clone(),
            conversation_id: conversation_id.clone(),
            sender_id: user_id.clone(),
            recipient_id: target_user_id.clone(),
            content: format!("Shared a post by @{}", post.username),
            message_type: MessageType::Post,
            media_url: None,
            reply_to: Some(post_id.clone()),
            reactions: std::collections::HashMap::new(),
            is_read: false,
            is_vanish_mode: false,
            created_at: current_time,
            expires_at: None,
        };

        state_handler::insert_message(conversation_id, share_message);

        // Send notification
        let _ = notifications::send_notification(
            target_user_id,
            NotificationType::Message,
            "Shared Post".to_string(),
            format!("{} shared a post with you", sharer_profile.username),
            Some(user_id.clone()),
            Some(post_id.clone()),
            None,
        );
    }

    // Update post shares count
    state_handler::increment_post_shares(&post_id)?;

    // Log user activity
    user_management::log_user_activity(
        user_id,
        ActivityAction::PostLiked, // Using closest available action
        Some(post_id),
        Some("share".to_string()),
    );

    Ok(())
}

// Helper functions
fn can_view_post(post: &Post, current_user_id: Option<&str>) -> bool {
    match post.visibility {
        PostVisibility::Public => true,
        PostVisibility::Private => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id
            } else {
                false
            }
        }
        PostVisibility::Followers => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id || state_handler::is_following(user_id, &post.user_id)
            } else {
                false
            }
        }
        PostVisibility::CloseFriends => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id || state_handler::is_close_friend(&post.user_id, user_id)
            } else {
                false
            }
        }
    }
}

fn extract_mentions(content: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    let words: Vec<&str> = content.split_whitespace().collect();

    for word in words {
        if word.starts_with('@') && word.len() > 1 {
            let username = &word[1..]; // Remove the @ symbol
                                       // Remove any trailing punctuation
            let clean_username =
                username.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '.');
            if !clean_username.is_empty() {
                mentions.push(clean_username.to_string());
            }
        }
    }

    mentions
}

pub fn create_close_friends_list(friend_ids: Vec<String>) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Validate that all friend IDs are valid users that the current user follows
    for friend_id in &friend_ids {
        if !state_handler::user_exists(friend_id) {
            return Err(format!("User {} not found", friend_id));
        }

        if !state_handler::is_following(&user_id, friend_id) {
            return Err(format!(
                "You must follow {} to add them to close friends",
                friend_id
            ));
        }
    }

    // Replace existing close friends list
    state_handler::set_close_friends_list(user_id, friend_ids)?;

    Ok(())
}

pub fn add_to_close_friends(friend_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if user exists and is followed
    if !state_handler::user_exists(&friend_id) {
        return Err("User not found".to_string());
    }

    if !state_handler::is_following(&user_id, &friend_id) {
        return Err("You must follow this user to add them to close friends".to_string());
    }

    // Check if already in close friends
    if state_handler::is_close_friend(&user_id, &friend_id) {
        return Err("User is already in close friends".to_string());
    }

    // Add to close friends list
    let close_friend = CloseFriend {
        user_id: user_id.clone(),
        friend_id: friend_id.clone(),
        created_at: state_handler::get_current_timestamp(),
    };

    state_handler::add_close_friend(user_id, close_friend)?;

    Ok(())
}

pub fn remove_from_close_friends(friend_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;

    // Check if user is in close friends
    if !state_handler::is_close_friend(&user_id, &friend_id) {
        return Err("User is not in close friends".to_string());
    }

    // Remove from close friends list
    state_handler::remove_close_friend(&user_id, &friend_id)?;

    Ok(())
}

pub fn get_close_friends() -> Result<Vec<UserProfile>, String> {
    let user_id = user_management::authenticate_user()?;

    // Get close friends list
    let close_friends = state_handler::get_close_friends(&user_id);

    // Get user profiles for each close friend
    let mut friend_profiles = Vec::new();
    for close_friend in close_friends {
        if let Some(profile) = state_handler::get_user(&close_friend.friend_id) {
            friend_profiles.push(profile);
        }
    }

    // Sort by when they were added (newest first)
    friend_profiles.sort_by(|a, b| {
        let a_close_friend = state_handler::get_close_friends(&user_id)
            .iter()
            .find(|cf| cf.friend_id == a.user_id)
            .map(|cf| cf.created_at)
            .unwrap_or(0);
        let b_close_friend = state_handler::get_close_friends(&user_id)
            .iter()
            .find(|cf| cf.friend_id == b.user_id)
            .map(|cf| cf.created_at)
            .unwrap_or(0);
        b_close_friend.cmp(&a_close_friend)
    });

    Ok(friend_profiles)
}
