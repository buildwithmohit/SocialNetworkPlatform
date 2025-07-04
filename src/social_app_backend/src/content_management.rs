use crate::types::*;
use crate::state_handler;
use crate::user_management;

pub fn create_post(post_data: CreatePostRequest) -> Result<Post, String> {
    let user_id = user_management::authenticate_user()?;
    
    // Get user profile to get username
    let user_profile = state_handler::get_user(&user_id)
        .ok_or_else(|| "User profile not found".to_string())?;
    
    // Validate post data
    if post_data.media_urls.is_empty() && post_data.caption.is_empty() {
        return Err("Post must have either media or caption".to_string());
    }
    
    if post_data.caption.len() > 2200 {
        return Err("Caption must be 2200 characters or less".to_string());
    }
    
    let post_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let post = Post {
        post_id: post_id.clone(),
        user_id: user_id.clone(),
        username: user_profile.username.clone(),
        content_type: post_data.content_type,
        media_urls: post_data.media_urls,
        caption: post_data.caption,
        hashtags: post_data.hashtags.clone(),
        tagged_users: post_data.tagged_users,
        location: post_data.location,
        likes_count: 0,
        comments_count: 0,
        shares_count: 0,
        created_at: current_time,
        updated_at: current_time,
        is_archived: false,
        visibility: post_data.visibility,
        music_info: post_data.music_info,
        product_tags: post_data.product_tags,
        post_type: (),
    };
    
    // Insert hashtags
    for hashtag in post_data.hashtags {
        state_handler::insert_hashtag(hashtag, post_id.clone());
    }
    
    state_handler::insert_post(post_id, post.clone());
    
    // Update user's post count
    let mut updated_user = user_profile;
    updated_user.posts_count += 1;
    updated_user.updated_at = current_time;
    state_handler::update_user(&user_id, updated_user)?;
    
    Ok(post)
}

pub fn get_post(post_id: &str) -> Result<Post, String> {
    let current_user = user_management::authenticate_user().ok();
    
    let post = state_handler::get_post(post_id)
        .ok_or_else(|| "Post not found".to_string())?;
    
    // Check if user can view this post
    if !can_view_post(&post, current_user.as_deref()) {
        return Err("Access denied".to_string());
    }
    
    Ok(post)
}

pub fn update_post(post_id: String, caption: Option<String>, hashtags: Option<Vec<String>>) -> Result<Post, String> {
    let user_id = user_management::authenticate_user()?;
    
    let mut post = state_handler::get_post(&post_id)
        .ok_or_else(|| "Post not found".to_string())?;
    
    // Check if user owns the post
    if post.user_id != user_id {
        return Err("Access denied".to_string());
    }
    
    if let Some(new_caption) = caption {
        if new_caption.len() > 2200 {
            return Err("Caption must be 2200 characters or less".to_string());
        }
        post.caption = new_caption;
    }
    
    if let Some(new_hashtags) = hashtags {
        post.hashtags = new_hashtags.clone();
        // Update hashtag index
        for hashtag in new_hashtags {
            state_handler::insert_hashtag(hashtag, post_id.clone());
        }
    }
    
    post.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_post(post_id, post.clone());
    Ok(post)
}

pub fn delete_post(post_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    let post = state_handler::get_post(&post_id)
        .ok_or_else(|| "Post not found".to_string())?;
    
    // Check if user owns the post
    if post.user_id != user_id {
        return Err("Access denied".to_string());
    }
    
    state_handler::delete_post(&post_id)?;
    
    // Update user's post count
    if let Some(mut user_profile) = state_handler::get_user(&user_id) {
        user_profile.posts_count = user_profile.posts_count.saturating_sub(1);
        user_profile.updated_at = state_handler::get_current_timestamp();
        state_handler::update_user(&user_id, user_profile)?;
    }
    
    Ok(())
}

pub fn get_feed(limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    let user_id = user_management::authenticate_user()?;
    
    // Get users that current user follows
    let following = state_handler::get_following(&user_id);
    
    // Get all posts and filter by following users
    let all_posts = state_handler::get_all_posts();
    let mut feed_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| {
            following.contains(&post.user_id) || post.user_id == user_id
        })
        .filter(|post| can_view_post(post, Some(&user_id)))
        .collect();
    
    // Sort by creation time (newest first)
    feed_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;
    
    if start >= feed_posts.len() {
        return Ok(Vec::new());
    }
    
    let end = end.min(feed_posts.len());
    Ok(feed_posts[start..end].to_vec())
}

pub fn get_user_posts(user_id: &str, limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    let current_user = user_management::authenticate_user().ok();
    
    let user_posts = state_handler::get_user_posts(user_id);
    
    // Filter posts based on visibility and user permissions
    let visible_posts: Vec<Post> = user_posts
        .into_iter()
        .filter(|post| can_view_post(post, current_user.as_deref()))
        .collect();
    
    // Sort by creation time (newest first)
    let mut sorted_posts = visible_posts;
    sorted_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;
    
    if start >= sorted_posts.len() {
        return Ok(Vec::new());
    }
    
    let end = end.min(sorted_posts.len());
    Ok(sorted_posts[start..end].to_vec())
}

pub fn archive_post(post_id: String) -> Result<Post, String> {
    let user_id = user_management::authenticate_user()?;
    
    let mut post = state_handler::get_post(&post_id)
        .ok_or_else(|| "Post not found".to_string())?;
    
    // Check if user owns the post
    if post.user_id != user_id {
        return Err("Access denied".to_string());
    }
    
    post.is_archived = true;
    post.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_post(post_id, post.clone());
    Ok(post)
}

pub fn unarchive_post(post_id: String) -> Result<Post, String> {
    let user_id = user_management::authenticate_user()?;
    
    let mut post = state_handler::get_post(&post_id)
        .ok_or_else(|| "Post not found".to_string())?;
    
    // Check if user owns the post
    if post.user_id != user_id {
        return Err("Access denied".to_string());
    }
    
    post.is_archived = false;
    post.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_post(post_id, post.clone());
    Ok(post)
}

pub fn get_archived_posts(limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    let user_id = user_management::authenticate_user()?;
    
    let user_posts = state_handler::get_user_posts(&user_id);
    
    // Filter for archived posts only
    let archived_posts: Vec<Post> = user_posts
        .into_iter()
        .filter(|post| post.is_archived)
        .collect();
    
    // Sort by creation time (newest first)
    let mut sorted_posts = archived_posts;
    sorted_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;
    
    if start >= sorted_posts.len() {
        return Ok(Vec::new());
    }
    
    let end = end.min(sorted_posts.len());
    Ok(sorted_posts[start..end].to_vec())
}

pub fn create_story(media_url: String, story_type: StoryType, text_overlay: Option<String>, stickers: Vec<Sticker>, music_info: Option<MusicInfo>) -> Result<Story, String> {
    let user_id = user_management::authenticate_user()?;
    
    // Get user profile to get username
    let user_profile = state_handler::get_user(&user_id)
        .ok_or_else(|| "User profile not found".to_string())?;
    
    let story_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    let expires_at = current_time + (24 * 60 * 60 * 1000); // 24 hours in milliseconds
    
    let story = Story {
        story_id: story_id.clone(),
        user_id,
        username: user_profile.username,
        media_url,
        story_type,
        text_overlay,
        stickers,
        music_info,
        viewers: Vec::new(),
        created_at: current_time,
        expires_at,
        is_highlight: false,
        highlight_id: None,
    };
    
    state_handler::insert_story(story_id, story.clone());
    Ok(story)
}

pub fn view_story(story_id: String) -> Result<Story, String> {
    let user_id = user_management::authenticate_user()?;
    
    let mut story = state_handler::get_story(&story_id)
        .ok_or_else(|| "Story not found".to_string())?;
    
    // Check if story has expired
    if story.expires_at < state_handler::get_current_timestamp() && !story.is_highlight {
        return Err("Story has expired".to_string());
    }
    
    // Add viewer if not already in the list
    if !story.viewers.contains(&user_id) {
        story.viewers.push(user_id);
        state_handler::insert_story(story_id, story.clone());
    }
    
    Ok(story)
}

pub fn get_user_stories(user_id: &str) -> Result<Vec<Story>, String> {
    let current_user = user_management::authenticate_user().ok();
    
    // Check if user can view stories (not blocked, etc.)
    if let Some(current_user_id) = &current_user {
        if user_management::is_user_blocked(user_id, current_user_id) {
            return Err("Access denied".to_string());
        }
    }
    
    let stories = state_handler::get_user_stories(user_id);
    let current_time = state_handler::get_current_timestamp();
    
    // Filter out expired stories
    let active_stories: Vec<Story> = stories
        .into_iter()
        .filter(|story| story.expires_at > current_time || story.is_highlight)
        .collect();
    
    Ok(active_stories)
}

fn can_view_post(post: &Post, current_user_id: Option<&str>) -> bool {
    match post.visibility {
        PostVisibility::Public => true,
        PostVisibility::Private => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id
            } else {
                false
            }
        },
        PostVisibility::Followers => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id || state_handler::is_following(user_id, &post.user_id)
            } else {
                false
            }
        },
        PostVisibility::CloseFriends => {
            if let Some(user_id) = current_user_id {
                user_id == post.user_id || state_handler::is_close_friend(&post.user_id, user_id)
            } else {
                false
            }
        }
    }
}

pub fn get_trending_posts(limit: u32) -> Result<Vec<Post>, String> {
    let all_posts = state_handler::get_all_posts();
    
    // Filter public posts only
    let public_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| matches!(post.visibility, PostVisibility::Public))
        .collect();
    
    // Sort by engagement (likes + comments + shares)
    let mut trending_posts = public_posts;
    trending_posts.sort_by(|a, b| {
        let a_engagement = a.likes_count + a.comments_count + a.shares_count;
        let b_engagement = b.likes_count + b.comments_count + b.shares_count;
        b_engagement.cmp(&a_engagement)
    });
    
    trending_posts.truncate(limit as usize);
    Ok(trending_posts)
}

pub fn get_posts_by_hashtag(hashtag: &str, limit: u32, offset: u32) -> Result<Vec<Post>, String> {
    // This would be implemented with proper hashtag indexing in a real system
    let all_posts = state_handler::get_all_posts();
    
    let hashtag_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| {
            post.hashtags.iter().any(|h| h.to_lowercase() == hashtag.to_lowercase()) &&
            matches!(post.visibility, PostVisibility::Public)
        })
        .collect();
    
    // Sort by creation time (newest first)
    let mut sorted_posts = hashtag_posts;
    sorted_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;
    
    if start >= sorted_posts.len() {
        return Ok(Vec::new());
    }
    
    let end = end.min(sorted_posts.len());
    Ok(sorted_posts[start..end].to_vec())
}
