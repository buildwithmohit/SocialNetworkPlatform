use crate::types::{self, *};
use std::cell::RefCell;
use std::collections::HashMap;

// Global state storage
thread_local! {
    static USERS: RefCell<HashMap<String, UserProfile>> = RefCell::new(HashMap::new());
    static POSTS: RefCell<HashMap<String, Post>> = RefCell::new(HashMap::new());
    static COMMENTS: RefCell<HashMap<String, Comment>> = RefCell::new(HashMap::new());
    static LIKES: RefCell<HashMap<String, Vec<Like>>> = RefCell::new(HashMap::new());
    static RELATIONSHIPS: RefCell<HashMap<String, Vec<Relationship>>> = RefCell::new(HashMap::new());
    static CONVERSATIONS: RefCell<HashMap<String, Conversation>> = RefCell::new(HashMap::new());
    static MESSAGES: RefCell<HashMap<String, Vec<Message>>> = RefCell::new(HashMap::new());
    static STORIES: RefCell<HashMap<String, Story>> = RefCell::new(HashMap::new());
    static HIGHLIGHTS: RefCell<HashMap<String, Highlight>> = RefCell::new(HashMap::new());
    static NOTIFICATIONS: RefCell<HashMap<String, Vec<Notification>>> = RefCell::new(HashMap::new());
    static SAVED_POSTS: RefCell<HashMap<String, Vec<SavedPost>>> = RefCell::new(HashMap::new());
    static SHOPS: RefCell<HashMap<String, Shop>> = RefCell::new(HashMap::new());
    static PRODUCTS: RefCell<HashMap<String, Product>> = RefCell::new(HashMap::new());
    static LIVE_STREAMS: RefCell<HashMap<String, LiveStream>> = RefCell::new(HashMap::new());
    static HASHTAGS: RefCell<HashMap<String, Hashtag>> = RefCell::new(HashMap::new());
    static REPORTS: RefCell<HashMap<String, Report>> = RefCell::new(HashMap::new());
    static CLOSE_FRIENDS: RefCell<HashMap<String, Vec<CloseFriend>>> = RefCell::new(HashMap::new());
    static USER_ACTIVITY: RefCell<HashMap<String, Vec<UserActivity>>> = RefCell::new(HashMap::new());
    static POST_LIKES: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // post_id -> user_ids
    static COMMENT_LIKES: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // comment_id -> user_ids
    static USER_FOLLOWERS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> follower_ids
    static USER_FOLLOWING: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> following_ids
    static USER_POSTS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> post_ids
    static HASHTAG_POSTS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // hashtag -> post_ids
    static BLOCKED_USERS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> blocked_user_ids
    static MUTED_USERS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> muted_user_ids
    static RESTRICTED_USERS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new()); // user_id -> restricted_user_ids
    static ONLINE_STATUS: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new()); // user_id -> is_online
    static PRIVACY_SETTINGS: RefCell<HashMap<String, crate::safety_privacy::PrivacySettings>> = RefCell::new(HashMap::new());
    static COMMENT_CONTROLS: RefCell<HashMap<String, crate::safety_privacy::CommentControls>> = RefCell::new(HashMap::new());
    static KEYWORD_FILTERS: RefCell<HashMap<String, Vec<String>>> = RefCell::new(HashMap::new());
    static SECURITY_SETTINGS: RefCell<HashMap<String, crate::safety_privacy::SecuritySettings>> = RefCell::new(HashMap::new());
    static TIME_LIMIT_SETTINGS: RefCell<HashMap<String, crate::safety_privacy::TimeLimitSettings>> = RefCell::new(HashMap::new());
    static NOTIFICATION_SETTINGS: RefCell<HashMap<String, types::NotificationSettings>> = RefCell::new(HashMap::new());
}

// User operations
pub fn insert_user(user_id: String, user: UserProfile) {
    USERS.with(|users| {
        users.borrow_mut().insert(user_id, user);
    });
}

pub fn get_user(user_id: &str) -> Option<UserProfile> {
    USERS.with(|users| users.borrow().get(user_id).cloned())
}

pub fn update_user(user_id: &str, user: UserProfile) -> Result<(), String> {
    USERS.with(|users| {
        if users.borrow().contains_key(user_id) {
            users.borrow_mut().insert(user_id.to_string(), user);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    })
}

pub fn user_exists(user_id: &str) -> bool {
    USERS.with(|users| users.borrow().contains_key(user_id))
}

pub fn username_exists(username: &str) -> bool {
    USERS.with(|users| {
        users
            .borrow()
            .values()
            .any(|user| user.username == username)
    })
}

// Post operations
pub fn insert_post(post_id: String, post: Post) {
    let user_id = post.user_id.clone();
    POSTS.with(|posts| {
        posts.borrow_mut().insert(post_id.clone(), post);
    });

    // Add to user's posts
    USER_POSTS.with(|user_posts| {
        user_posts
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(post_id);
    });
}

pub fn get_post(post_id: &str) -> Option<Post> {
    POSTS.with(|posts| posts.borrow().get(post_id).cloned())
}

pub fn get_all_posts() -> Vec<Post> {
    POSTS.with(|posts| posts.borrow().values().cloned().collect())
}

pub fn get_user_posts(user_id: &str) -> Vec<Post> {
    USER_POSTS.with(|user_posts| {
        if let Some(post_ids) = user_posts.borrow().get(user_id) {
            POSTS.with(|posts| {
                post_ids
                    .iter()
                    .filter_map(|post_id| posts.borrow().get(post_id).cloned())
                    .collect()
            })
        } else {
            Vec::new()
        }
    })
}

pub fn delete_post(post_id: &str) -> Result<(), String> {
    POSTS.with(|posts| {
        if posts.borrow_mut().remove(post_id).is_some() {
            Ok(())
        } else {
            Err("Post not found".to_string())
        }
    })
}

// Like operations
pub fn add_post_like(post_id: String, user_id: String) -> Result<(), String> {
    // First, add the like
    POST_LIKES.with(|likes| {
        likes
            .borrow_mut()
            .entry(post_id.clone())
            .or_insert_with(Vec::new)
            .push(user_id);
    });

    // Then update post likes count separately
    POSTS.with(|posts| {
        let mut posts_map = posts.borrow_mut();
        if let Some(post) = posts_map.get_mut(&post_id) {
            post.likes_count += 1;
        }
    });

    Ok(())
}

pub fn remove_post_like(post_id: &str, user_id: &str) -> Result<(), String> {
    let mut like_removed = false;

    // First, remove the like
    POST_LIKES.with(|likes| {
        if let Some(user_likes) = likes.borrow_mut().get_mut(post_id) {
            if let Some(pos) = user_likes.iter().position(|id| id == user_id) {
                user_likes.remove(pos);
                like_removed = true;
            }
        }
    });

    if !like_removed {
        return Err("Like not found".to_string());
    }

    // Then update post likes count separately
    POSTS.with(|posts| {
        let mut posts_map = posts.borrow_mut();
        if let Some(post) = posts_map.get_mut(post_id) {
            post.likes_count = post.likes_count.saturating_sub(1);
        }
    });

    Ok(())
}

pub fn has_user_liked_post(post_id: &str, user_id: &str) -> bool {
    POST_LIKES.with(|likes| {
        likes
            .borrow()
            .get(post_id)
            .map(|user_likes| user_likes.contains(&user_id.to_string()))
            .unwrap_or(false)
    })
}

// Comment operations
pub fn insert_comment(comment_id: String, comment: Comment) {
    let post_id = comment.post_id.clone();

    // First, insert the comment
    COMMENTS.with(|comments| {
        comments.borrow_mut().insert(comment_id, comment);
    });

    // Then update post comments count separately
    POSTS.with(|posts| {
        let mut posts_map = posts.borrow_mut();
        if let Some(post) = posts_map.get_mut(&post_id) {
            post.comments_count += 1;
        }
    });
}

pub fn get_comment(comment_id: &str) -> Option<Comment> {
    COMMENTS.with(|comments| comments.borrow().get(comment_id).cloned())
}

pub fn get_post_comments(post_id: &str) -> Vec<Comment> {
    COMMENTS.with(|comments| {
        comments
            .borrow()
            .values()
            .filter(|comment| comment.post_id == post_id)
            .cloned()
            .collect()
    })
}

pub fn update_comment_reply_count(comment_id: &str, increment: bool) -> Result<(), String> {
    COMMENTS.with(|comments| {
        let mut comments_map = comments.borrow_mut();
        if let Some(comment) = comments_map.get_mut(comment_id) {
            if increment {
                comment.replies_count += 1;
            } else {
                comment.replies_count = comment.replies_count.saturating_sub(1);
            }
            Ok(())
        } else {
            Err("Comment not found".to_string())
        }
    })
}

// Relationship operations
pub fn add_follower(user_id: String, follower_id: String) -> Result<(), String> {
    // First, update the follower/following relationships
    USER_FOLLOWERS.with(|followers| {
        followers
            .borrow_mut()
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(follower_id.clone());
    });

    USER_FOLLOWING.with(|following| {
        following
            .borrow_mut()
            .entry(follower_id.clone())
            .or_insert_with(Vec::new)
            .push(user_id.clone());
    });

    // Then update user follower count separately to avoid borrowing conflicts
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        if let Some(user) = users_map.get_mut(&user_id) {
            user.followers_count += 1;
            user.updated_at = get_current_timestamp();
        }

        // Also update the follower's following count
        if let Some(follower) = users_map.get_mut(&follower_id) {
            follower.following_count += 1;
            follower.updated_at = get_current_timestamp();
        }
    });

    Ok(())
}

pub fn remove_follower(user_id: &str, follower_id: &str) -> Result<(), String> {
    // First, update the follower/following relationships
    USER_FOLLOWERS.with(|followers| {
        if let Some(user_followers) = followers.borrow_mut().get_mut(user_id) {
            if let Some(pos) = user_followers.iter().position(|id| id == follower_id) {
                user_followers.remove(pos);
            }
        }
    });

    USER_FOLLOWING.with(|following| {
        if let Some(user_following) = following.borrow_mut().get_mut(follower_id) {
            if let Some(pos) = user_following.iter().position(|id| id == user_id) {
                user_following.remove(pos);
            }
        }
    });

    // Then update user follower counts separately
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        if let Some(user) = users_map.get_mut(user_id) {
            user.followers_count = user.followers_count.saturating_sub(1);
            user.updated_at = get_current_timestamp();
        }

        // Also update the follower's following count
        if let Some(follower) = users_map.get_mut(follower_id) {
            follower.following_count = follower.following_count.saturating_sub(1);
            follower.updated_at = get_current_timestamp();
        }
    });

    Ok(())
}

pub fn is_following(user_id: &str, target_user_id: &str) -> bool {
    USER_FOLLOWING.with(|following| {
        following
            .borrow()
            .get(user_id)
            .map(|user_following| user_following.contains(&target_user_id.to_string()))
            .unwrap_or(false)
    })
}

pub fn get_followers(user_id: &str) -> Vec<String> {
    USER_FOLLOWERS.with(|followers| followers.borrow().get(user_id).cloned().unwrap_or_default())
}

pub fn get_following(user_id: &str) -> Vec<String> {
    USER_FOLLOWING.with(|following| following.borrow().get(user_id).cloned().unwrap_or_default())
}

// Message operations
pub fn insert_message(conversation_id: String, message: Message) {
    MESSAGES.with(|messages| {
        messages
            .borrow_mut()
            .entry(conversation_id)
            .or_insert_with(Vec::new)
            .push(message);
    });
}

pub fn get_conversation_messages(conversation_id: &str, limit: u32) -> Vec<Message> {
    MESSAGES.with(|messages| {
        if let Some(conv_messages) = messages.borrow().get(conversation_id) {
            let start = if conv_messages.len() > limit as usize {
                conv_messages.len() - limit as usize
            } else {
                0
            };
            conv_messages[start..].to_vec()
        } else {
            Vec::new()
        }
    })
}

// Conversation operations
pub fn insert_conversation(conversation_id: String, conversation: Conversation) {
    CONVERSATIONS.with(|conversations| {
        conversations
            .borrow_mut()
            .insert(conversation_id, conversation);
    });
}

pub fn get_conversation(conversation_id: &str) -> Option<Conversation> {
    CONVERSATIONS.with(|conversations| conversations.borrow().get(conversation_id).cloned())
}

// Story operations
pub fn insert_story(story_id: String, story: Story) {
    STORIES.with(|stories| {
        stories.borrow_mut().insert(story_id, story);
    });
}

pub fn get_story(story_id: &str) -> Option<Story> {
    STORIES.with(|stories| stories.borrow().get(story_id).cloned())
}

pub fn get_user_stories(user_id: &str) -> Vec<Story> {
    STORIES.with(|stories| {
        stories
            .borrow()
            .values()
            .filter(|story| story.user_id == user_id && !story.is_highlight)
            .cloned()
            .collect()
    })
}

// Hashtag operations
pub fn insert_hashtag(hashtag: String, post_id: String) {
    HASHTAGS.with(|hashtags| {
        hashtags
            .borrow_mut()
            .entry(hashtag.clone())
            .or_insert_with(|| Hashtag {
                name: hashtag.clone(),
                posts_count: 0,
                is_trending: false,
            });
    });

    HASHTAG_POSTS.with(|hashtag_posts| {
        hashtag_posts
            .borrow_mut()
            .entry(hashtag.clone())
            .or_insert_with(Vec::new)
            .push(post_id);
    });

    // Update hashtag posts count
    HASHTAGS.with(|hashtags| {
        if let Some(mut hashtag_data) = hashtags.borrow().get(&hashtag).cloned() {
            hashtag_data.posts_count += 1;
            hashtags.borrow_mut().insert(hashtag, hashtag_data);
        }
    });
}

pub fn search_hashtags(query: &str) -> Vec<Hashtag> {
    HASHTAGS.with(|hashtags| {
        hashtags
            .borrow()
            .values()
            .filter(|hashtag| hashtag.name.to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect()
    })
}

// Notification operations
pub fn add_notification(user_id: String, notification: Notification) {
    NOTIFICATIONS.with(|notifications| {
        notifications
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(notification);
    });
}

pub fn get_user_notifications(user_id: &str, limit: u32) -> Vec<Notification> {
    NOTIFICATIONS.with(|notifications| {
        if let Some(user_notifications) = notifications.borrow().get(user_id) {
            let start = if user_notifications.len() > limit as usize {
                user_notifications.len() - limit as usize
            } else {
                0
            };
            user_notifications[start..].to_vec()
        } else {
            Vec::new()
        }
    })
}

pub fn get_all_users() -> Vec<UserProfile> {
    USERS.with(|users| users.borrow().values().cloned().collect())
}

// Initialize state
pub fn init_state() {
    // Initialize any default data if needed
}

// Save state (for upgrades)
pub fn save_state() {
    // In a real implementation, you would serialize state to stable memory
    // For now, this is a placeholder
}

// Restore state (after upgrades)
pub fn restore_state() {
    // In a real implementation, you would deserialize state from stable memory
    // For now, this is a placeholder
}

// Utility functions
pub fn generate_id() -> String {
    use ic_cdk::api::time;

    // Use IC time as seed for deterministic randomness
    let mut seed = time();
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut result = String::new();

    // Simple linear congruential generator for deterministic randomness
    for _ in 0..16 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let index = (seed % 62) as usize;
        result.push(chars.chars().nth(index).unwrap());
    }

    result
}

// pub fn generate_random_id(prefix: &str) -> String {
//     use ic_cdk::api::time;

//     let mut seed = time();
//     let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
//     let mut result = String::new();
//     for _ in 0..12 {
//         seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
//         let index = (seed % 62) as usize;
//         result.push(chars.chars().nth(index).unwrap());
//     }

//     format!("{}_{}", prefix, result)
// }

pub fn get_current_timestamp() -> u64 {
    ic_cdk::api::time()
}

// User relationship operations
pub fn add_blocked_user(user_id: String, blocked_user_id: String) {
    BLOCKED_USERS.with(|blocked| {
        blocked
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(blocked_user_id);
    });
}

pub fn remove_blocked_user(user_id: &str, blocked_user_id: &str) {
    BLOCKED_USERS.with(|blocked| {
        if let Some(blocked_list) = blocked.borrow_mut().get_mut(user_id) {
            blocked_list.retain(|id| id != blocked_user_id);
        }
    });
}

pub fn is_user_blocked(user_id: &str, target_user_id: &str) -> bool {
    BLOCKED_USERS.with(|blocked| {
        blocked
            .borrow()
            .get(user_id)
            .map(|blocked_list| blocked_list.contains(&target_user_id.to_string()))
            .unwrap_or(false)
    })
}

pub fn get_blocked_users_list(user_id: &str) -> Vec<String> {
    BLOCKED_USERS.with(|blocked| blocked.borrow().get(user_id).cloned().unwrap_or_default())
}

pub fn add_muted_user(user_id: String, muted_user_id: String) {
    MUTED_USERS.with(|muted| {
        muted
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(muted_user_id);
    });
}

pub fn remove_muted_user(user_id: &str, muted_user_id: &str) {
    MUTED_USERS.with(|muted| {
        if let Some(muted_list) = muted.borrow_mut().get_mut(user_id) {
            muted_list.retain(|id| id != muted_user_id);
        }
    });
}

pub fn is_user_muted(user_id: &str, target_user_id: &str) -> bool {
    MUTED_USERS.with(|muted| {
        muted
            .borrow()
            .get(user_id)
            .map(|muted_list| muted_list.contains(&target_user_id.to_string()))
            .unwrap_or(false)
    })
}

pub fn get_muted_users_list(user_id: &str) -> Vec<String> {
    MUTED_USERS.with(|muted| muted.borrow().get(user_id).cloned().unwrap_or_default())
}

pub fn add_restricted_user(user_id: String, restricted_user_id: String) {
    RESTRICTED_USERS.with(|restricted| {
        restricted
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(restricted_user_id);
    });
}

pub fn remove_restricted_user(user_id: &str, restricted_user_id: &str) {
    RESTRICTED_USERS.with(|restricted| {
        if let Some(restricted_list) = restricted.borrow_mut().get_mut(user_id) {
            restricted_list.retain(|id| id != restricted_user_id);
        }
    });
}

pub fn is_user_restricted(user_id: &str, target_user_id: &str) -> bool {
    RESTRICTED_USERS.with(|restricted| {
        restricted
            .borrow()
            .get(user_id)
            .map(|restricted_list| restricted_list.contains(&target_user_id.to_string()))
            .unwrap_or(false)
    })
}

pub fn get_restricted_users_list(user_id: &str) -> Vec<String> {
    RESTRICTED_USERS.with(|restricted| {
        restricted
            .borrow()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    })
}

pub fn set_user_online_status(user_id: String, is_online: bool) {
    ONLINE_STATUS.with(|status| {
        status.borrow_mut().insert(user_id, is_online);
    });
}

pub fn get_user_online_status(user_id: &str) -> bool {
    ONLINE_STATUS.with(|status| status.borrow().get(user_id).copied().unwrap_or(false))
}

pub fn add_user_activity(user_id: String, activity: UserActivity) {
    USER_ACTIVITY.with(|activities| {
        activities
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(activity);
    });
}

pub fn get_user_activities(user_id: &str) -> Vec<UserActivity> {
    USER_ACTIVITY.with(|activities| {
        activities
            .borrow()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    })
}

pub fn search_users(query: &str, limit: usize) -> Vec<UserProfile> {
    let query_lower = query.to_lowercase();

    USERS.with(|users| {
        let users_map = users.borrow();
        let mut results: Vec<UserProfile> = users_map
            .values()
            .filter(|user| {
                user.username.to_lowercase().contains(&query_lower)
                    || user.display_name.to_lowercase().contains(&query_lower)
                    || user.bio.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        // Sort by relevance (exact username match first, then partial matches)
        results.sort_by(|a, b| {
            let a_exact = a.username.to_lowercase() == query_lower;
            let b_exact = b.username.to_lowercase() == query_lower;

            if a_exact && !b_exact {
                std::cmp::Ordering::Less
            } else if !a_exact && b_exact {
                std::cmp::Ordering::Greater
            } else {
                // Sort by follower count for equal relevance
                b.followers_count.cmp(&a.followers_count)
            }
        });

        results.truncate(limit);
        results
    })
}

// Saved posts operations
pub fn is_post_saved(user_id: &str, post_id: &str) -> bool {
    SAVED_POSTS.with(|saved_posts| {
        saved_posts
            .borrow()
            .get(user_id)
            .map(|user_saved_posts| {
                user_saved_posts
                    .iter()
                    .any(|saved_post| saved_post.post_id == post_id)
            })
            .unwrap_or(false)
    })
}

pub fn save_post(user_id: String, saved_post: SavedPost) -> Result<(), String> {
    SAVED_POSTS.with(|saved_posts| {
        saved_posts
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(saved_post);
    });
    Ok(())
}

pub fn unsave_post(user_id: &str, post_id: &str) -> Result<(), String> {
    SAVED_POSTS.with(|saved_posts| {
        if let Some(user_saved_posts) = saved_posts.borrow_mut().get_mut(user_id) {
            let initial_len = user_saved_posts.len();
            user_saved_posts.retain(|saved_post| saved_post.post_id != post_id);

            if user_saved_posts.len() < initial_len {
                Ok(())
            } else {
                Err("Post not found in saved posts".to_string())
            }
        } else {
            Err("No saved posts found for user".to_string())
        }
    })
}

pub fn get_saved_posts(user_id: &str) -> Vec<SavedPost> {
    SAVED_POSTS.with(|saved_posts| {
        saved_posts
            .borrow()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    })
}

// Conversation operations
pub fn get_or_create_conversation(user1_id: &str, user2_id: &str) -> Result<String, String> {
    // First try to find existing direct conversation between these two users
    let existing_conversation_id = CONVERSATIONS.with(|conversations| {
        conversations
            .borrow()
            .values()
            .find(|conversation| {
                matches!(conversation.conversation_type, ConversationType::Direct)
                    && conversation.participants.len() == 2
                    && conversation.participants.contains(&user1_id.to_string())
                    && conversation.participants.contains(&user2_id.to_string())
            })
            .map(|conversation| conversation.conversation_id.clone())
    });

    if let Some(conversation_id) = existing_conversation_id {
        return Ok(conversation_id);
    }

    // Create new direct conversation
    let conversation_id = generate_id();
    let current_time = get_current_timestamp();

    let new_conversation = Conversation {
        conversation_id: conversation_id.clone(),
        participants: vec![user1_id.to_string(), user2_id.to_string()],
        conversation_type: ConversationType::Direct,
        last_message: None,
        created_at: current_time,
        updated_at: current_time,
        is_archived: false,
        group_name: None,
        group_photo: None,
        admins: Vec::new(),
    };

    CONVERSATIONS.with(|conversations| {
        conversations
            .borrow_mut()
            .insert(conversation_id.clone(), new_conversation);
    });

    Ok(conversation_id)
}

// Post shares operations
pub fn increment_post_shares(post_id: &str) -> Result<(), String> {
    POSTS.with(|posts| {
        let mut posts_map = posts.borrow_mut();
        if let Some(post) = posts_map.get_mut(post_id) {
            post.shares_count += 1;
            post.updated_at = get_current_timestamp();
            Ok(())
        } else {
            Err("Post not found".to_string())
        }
    })
}

// Close friends operations
pub fn is_close_friend(user_id: &str, friend_id: &str) -> bool {
    CLOSE_FRIENDS.with(|close_friends| {
        close_friends
            .borrow()
            .get(user_id)
            .map(|user_close_friends| {
                user_close_friends
                    .iter()
                    .any(|close_friend| close_friend.friend_id == friend_id)
            })
            .unwrap_or(false)
    })
}

pub fn set_close_friends_list(user_id: String, friend_ids: Vec<String>) -> Result<(), String> {
    let current_time = get_current_timestamp();

    let close_friends_list: Vec<CloseFriend> = friend_ids
        .into_iter()
        .map(|friend_id| CloseFriend {
            user_id: user_id.clone(),
            friend_id,
            created_at: current_time,
        })
        .collect();

    CLOSE_FRIENDS.with(|close_friends| {
        close_friends
            .borrow_mut()
            .insert(user_id, close_friends_list);
    });

    Ok(())
}

pub fn add_close_friend(user_id: String, close_friend: CloseFriend) -> Result<(), String> {
    CLOSE_FRIENDS.with(|close_friends| {
        close_friends
            .borrow_mut()
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(close_friend);
    });
    Ok(())
}

pub fn remove_close_friend(user_id: &str, friend_id: &str) -> Result<(), String> {
    CLOSE_FRIENDS.with(|close_friends| {
        if let Some(user_close_friends) = close_friends.borrow_mut().get_mut(user_id) {
            let initial_len = user_close_friends.len();
            user_close_friends.retain(|close_friend| close_friend.friend_id != friend_id);

            if user_close_friends.len() < initial_len {
                Ok(())
            } else {
                Err("Friend not found in close friends list".to_string())
            }
        } else {
            Err("No close friends found for user".to_string())
        }
    })
}

pub fn get_close_friends(user_id: &str) -> Vec<CloseFriend> {
    CLOSE_FRIENDS.with(|close_friends| {
        close_friends
            .borrow()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    })
}

// Report operations
pub fn insert_report(report_id: String, report: Report) -> Result<(), String> {
    REPORTS.with(|reports| {
        reports.borrow_mut().insert(report_id, report);
    });
    Ok(())
}

// Privacy settings operations
pub fn update_privacy_settings(user_id: String, settings: crate::safety_privacy::PrivacySettings) -> Result<(), String> {
    PRIVACY_SETTINGS.with(|privacy| {
        privacy.borrow_mut().insert(user_id, settings);
    });
    Ok(())
}

pub fn get_privacy_settings(user_id: &str) -> Option<crate::safety_privacy::PrivacySettings> {
    PRIVACY_SETTINGS.with(|privacy| {
        privacy.borrow().get(user_id).cloned()
    })
}

// Comment controls operations
pub fn update_comment_controls(user_id: String, controls: crate::safety_privacy::CommentControls) -> Result<(), String> {
    COMMENT_CONTROLS.with(|controls_map| {
        controls_map.borrow_mut().insert(user_id, controls);
    });
    Ok(())
}

pub fn get_comment_controls(user_id: &str) -> Option<crate::safety_privacy::CommentControls> {
    COMMENT_CONTROLS.with(|controls| {
        controls.borrow().get(user_id).cloned()
    })
}
pub fn update_security_settings(user_id: String, settings: crate::safety_privacy::SecuritySettings) -> Result<(), String> {
    SECURITY_SETTINGS.with(|security| {
        security.borrow_mut().insert(user_id, settings);
    });
    Ok(())
}

pub fn get_security_settings(user_id: &str) -> Option<crate::safety_privacy::SecuritySettings> {
    SECURITY_SETTINGS.with(|security| {
        security.borrow().get(user_id).cloned()
    })
}

// Time limit settings operations
pub fn update_time_limit_settings(user_id: String, settings: crate::safety_privacy::TimeLimitSettings) -> Result<(), String> {
    TIME_LIMIT_SETTINGS.with(|time_limits| {
        time_limits.borrow_mut().insert(user_id, settings);
    });
    Ok(())
}

pub fn get_time_limit_settings(user_id: &str) -> Option<crate::safety_privacy::TimeLimitSettings> {
    TIME_LIMIT_SETTINGS.with(|time_limits| {
        time_limits.borrow().get(user_id).cloned()
    })
}


// Keyword filters operations
pub fn update_keyword_filters(user_id: String, keywords: Vec<String>) -> Result<(), String> {
    KEYWORD_FILTERS.with(|filters| {
        filters.borrow_mut().insert(user_id, keywords);
    });
    Ok(())
}

pub fn get_keyword_filters(user_id: &str) -> Vec<String> {
    KEYWORD_FILTERS.with(|filters| {
        filters.borrow().get(user_id).cloned().unwrap_or_default()
    })
}


pub fn get_posts_by_user(user_id: &str) -> Vec<Post> {
    USER_POSTS.with(|user_posts| {
        if let Some(post_ids) = user_posts.borrow().get(user_id) {
            POSTS.with(|posts| {
                post_ids
                    .iter()
                    .filter_map(|post_id| posts.borrow().get(post_id).cloned())
                    .collect()
            })
        } else {
            Vec::new()
        }
    })
}

pub fn delete_post_comments(post_id: &str) -> Result<(), String> {
    let comment_ids: Vec<String> = COMMENTS.with(|comments| {
        comments
            .borrow()
            .iter()
            .filter(|(_, comment)| comment.post_id == post_id)
            .map(|(comment_id, _)| comment_id.clone())
            .collect()
    });

    COMMENTS.with(|comments| {
        let mut comments_map = comments.borrow_mut();
        for comment_id in comment_ids {
            comments_map.remove(&comment_id);
        }
    });

    // Reset post comments count
    POSTS.with(|posts| {
        let mut posts_map = posts.borrow_mut();
        if let Some(post) = posts_map.get_mut(post_id) {
            post.comments_count = 0;
        }
    });

    Ok(())
}

pub fn delete_comments_by_user(user_id: &str) -> Result<(), String> {
    let comment_ids: Vec<String> = COMMENTS.with(|comments| {
        comments
            .borrow()
            .iter()
            .filter(|(_, comment)| comment.user_id == user_id)
            .map(|(comment_id, _)| comment_id.clone())
            .collect()
    });

    COMMENTS.with(|comments| {
        let mut comments_map = comments.borrow_mut();
        for comment_id in comment_ids {
            comments_map.remove(&comment_id);
        }
    });

    Ok(())
}

pub fn unfollow(follower_id: &str, following_id: &str) -> Result<(), String> {
    // Remove from follower's following list
    USER_FOLLOWING.with(|following| {
        if let Some(user_following) = following.borrow_mut().get_mut(follower_id) {
            user_following.retain(|id| id != following_id);
        }
    });

    // Remove from following user's followers list
    USER_FOLLOWERS.with(|followers| {
        if let Some(user_followers) = followers.borrow_mut().get_mut(following_id) {
            user_followers.retain(|id| id != follower_id);
        }
    });

    // Update user counts
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        
        // Update follower's following count
        if let Some(follower) = users_map.get_mut(follower_id) {
            follower.following_count = follower.following_count.saturating_sub(1);
            follower.updated_at = get_current_timestamp();
        }

        // Update following user's followers count
        if let Some(following_user) = users_map.get_mut(following_id) {
            following_user.followers_count = following_user.followers_count.saturating_sub(1);
            following_user.updated_at = get_current_timestamp();
        }
    });

    Ok(())
}

pub fn delete_user_messages(user_id: &str) -> Result<(), String> {
    // Get all conversations the user is part of
    let user_conversations: Vec<String> = CONVERSATIONS.with(|conversations| {
        conversations
            .borrow()
            .iter()
            .filter(|(_, conversation)| conversation.participants.contains(&user_id.to_string()))
            .map(|(conversation_id, _)| conversation_id.clone())
            .collect()
    });

    // Delete messages from those conversations
    MESSAGES.with(|messages| {
        let mut messages_map = messages.borrow_mut();
        for conversation_id in &user_conversations {
            messages_map.remove(conversation_id);
        }
    });

    // Remove user from conversations or delete conversations if they become empty
    CONVERSATIONS.with(|conversations| {
        let mut conversations_map = conversations.borrow_mut();
        let mut conversations_to_remove = Vec::new();

        for conversation_id in &user_conversations {
            if let Some(conversation) = conversations_map.get_mut(conversation_id) {
                conversation.participants.retain(|participant| participant != user_id);
                
                // If conversation has no participants left, mark for deletion
                if conversation.participants.is_empty() {
                    conversations_to_remove.push(conversation_id.clone());
                }
            }
        }

        // Remove empty conversations
        for conversation_id in conversations_to_remove {
            conversations_map.remove(&conversation_id);
        }
    });

    Ok(())
}

pub fn delete_user_stories(user_id: &str) -> Result<(), String> {
    let story_ids: Vec<String> = STORIES.with(|stories| {
        stories
            .borrow()
            .iter()
            .filter(|(_, story)| story.user_id == user_id)
            .map(|(story_id, _)| story_id.clone())
            .collect()
    });

    STORIES.with(|stories| {
        let mut stories_map = stories.borrow_mut();
        for story_id in story_ids {
            stories_map.remove(&story_id);
        }
    });

    Ok(())
}

pub fn delete_user_saved_posts(user_id: &str) -> Result<(), String> {
    SAVED_POSTS.with(|saved_posts| {
        saved_posts.borrow_mut().remove(user_id);
    });
    Ok(())
}

pub fn delete_user_notifications(user_id: &str) -> Result<(), String> {
    NOTIFICATIONS.with(|notifications| {
        notifications.borrow_mut().remove(user_id);
    });
    Ok(())
}

pub fn remove_user_blocks(user_id: &str) -> Result<(), String> {
    // Remove user from blocked users lists
    BLOCKED_USERS.with(|blocked| {
        blocked.borrow_mut().remove(user_id);
    });

    // Remove user from other users' blocked lists
    BLOCKED_USERS.with(|blocked| {
        let mut blocked_map = blocked.borrow_mut();
        for (_, blocked_list) in blocked_map.iter_mut() {
            blocked_list.retain(|id| id != user_id);
        }
    });

    // Remove user from muted users lists
    MUTED_USERS.with(|muted| {
        muted.borrow_mut().remove(user_id);
    });

    // Remove user from other users' muted lists
    MUTED_USERS.with(|muted| {
        let mut muted_map = muted.borrow_mut();
        for (_, muted_list) in muted_map.iter_mut() {
            muted_list.retain(|id| id != user_id);
        }
    });

    // Remove user from restricted users lists
    RESTRICTED_USERS.with(|restricted| {
        restricted.borrow_mut().remove(user_id);
    });

    // Remove user from other users' restricted lists
    RESTRICTED_USERS.with(|restricted| {
        let mut restricted_map = restricted.borrow_mut();
        for (_, restricted_list) in restricted_map.iter_mut() {
            restricted_list.retain(|id| id != user_id);
        }
    });

    Ok(())
}

pub fn delete_user_analytics(user_id: &str) -> Result<(), String> {
    // Remove user from activity tracking
    USER_ACTIVITY.with(|activities| {
        activities.borrow_mut().remove(user_id);
    });

    // Remove user's online status
    ONLINE_STATUS.with(|status| {
        status.borrow_mut().remove(user_id);
    });

    // Remove user's privacy settings
    PRIVACY_SETTINGS.with(|privacy| {
        privacy.borrow_mut().remove(user_id);
    });

    // Remove user's comment controls
    COMMENT_CONTROLS.with(|controls| {
        controls.borrow_mut().remove(user_id);
    });

    // Remove user's security settings
    SECURITY_SETTINGS.with(|security| {
        security.borrow_mut().remove(user_id);
    });

    // Remove user's time limit settings
    TIME_LIMIT_SETTINGS.with(|time_limits| {
        time_limits.borrow_mut().remove(user_id);
    });

    // Remove user's keyword filters
    KEYWORD_FILTERS.with(|filters| {
        filters.borrow_mut().remove(user_id);
    });

    Ok(())
}

pub fn delete_user(user_id: &str) -> Result<(), String> {
    // Remove user from main users storage
    USERS.with(|users| {
        if users.borrow_mut().remove(user_id).is_some() {
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    })?;

    // Clean up user's posts tracking
    USER_POSTS.with(|user_posts| {
        user_posts.borrow_mut().remove(user_id);
    });

    // Clean up user's followers tracking
    USER_FOLLOWERS.with(|followers| {
        followers.borrow_mut().remove(user_id);
    });

    // Clean up user's following tracking
    USER_FOLLOWING.with(|following| {
        following.borrow_mut().remove(user_id);
    });

    // Clean up user's close friends
    CLOSE_FRIENDS.with(|close_friends| {
        close_friends.borrow_mut().remove(user_id);
    });

    // Remove user from other users' close friends lists
    CLOSE_FRIENDS.with(|close_friends| {
        let mut close_friends_map = close_friends.borrow_mut();
        for (_, friends_list) in close_friends_map.iter_mut() {
            friends_list.retain(|friend| friend.friend_id != user_id);
        }
    });

    Ok(())
}

// Notification settings operations
pub fn save_notification_settings(user_id: String, settings: types::NotificationSettings) -> Result<(), String> {
    NOTIFICATION_SETTINGS.with(|notification_settings| {
        notification_settings.borrow_mut().insert(user_id, settings);
    });
    Ok(())
}

pub fn get_notification_settings(user_id: &str) -> Option<types::NotificationSettings> {
    NOTIFICATION_SETTINGS.with(|notification_settings| {
        notification_settings.borrow().get(user_id).cloned()
    })
}

pub fn update_notification_settings(user_id: String, settings: types::NotificationSettings) -> Result<(), String> {
    NOTIFICATION_SETTINGS.with(|notification_settings| {
        notification_settings.borrow_mut().insert(user_id, settings);
    });
    Ok(())
}

pub fn delete_notification(user_id: &str, notification_id: &str) -> Result<(), String> {
    NOTIFICATIONS.with(|notifications| {
        if let Some(user_notifications) = notifications.borrow_mut().get_mut(user_id) {
            let initial_len = user_notifications.len();
            user_notifications.retain(|notification| notification.notification_id != notification_id);
            
            if user_notifications.len() < initial_len {
                Ok(())
            } else {
                Err("Notification not found".to_string())
            }
        } else {
            Err("No notifications found for user".to_string())
        }
    })
}

pub fn mark_notification_as_read(user_id: &str, notification_id: &str) -> Result<(), String> {
    NOTIFICATIONS.with(|notifications| {
        if let Some(user_notifications) = notifications.borrow_mut().get_mut(user_id) {
            if let Some(notification) = user_notifications.iter_mut().find(|n| n.notification_id == notification_id) {
                notification.is_read = true;
                notification.read_at = Some(get_current_timestamp());
                Ok(())
            } else {
                Err("Notification not found".to_string())
            }
        } else {
            Err("No notifications found for user".to_string())
        }
    })
}

pub fn mark_all_notifications_as_read(user_id: &str) -> Result<(), String> {
    NOTIFICATIONS.with(|notifications| {
        if let Some(user_notifications) = notifications.borrow_mut().get_mut(user_id) {
            let current_time = get_current_timestamp();
            for notification in user_notifications.iter_mut() {
                if !notification.is_read {
                    notification.is_read = true;
                    notification.read_at = Some(current_time);
                }
            }
            Ok(())
        } else {
            Err("No notifications found for user".to_string())
        }
    })
}



// Additional messaging functions
pub fn get_all_conversations() -> Vec<Conversation> {
    CONVERSATIONS.with(|conversations| {
        conversations.borrow().values().cloned().collect()
    })
}

pub fn mark_message_as_read(conversation_id: &str, message_id: &str, user_id: &str) -> Result<(), String> {
    MESSAGES.with(|messages| {
        if let Some(conversation_messages) = messages.borrow_mut().get_mut(conversation_id) {
            if let Some(message) = conversation_messages.iter_mut().find(|m| m.message_id == message_id) {
                // Only mark as read if user is the recipient
                if message.recipient_id == user_id || message.sender_id != user_id {
                    message.is_read = true;
                    Ok(())
                } else {
                    Err("Cannot mark your own message as read".to_string())
                }
            } else {
                Err("Message not found".to_string())
            }
        } else {
            Err("Conversation not found".to_string())
        }
    })
}

pub fn mark_conversation_as_read(conversation_id: &str, user_id: &str) -> Result<(), String> {
    MESSAGES.with(|messages| {
        if let Some(conversation_messages) = messages.borrow_mut().get_mut(conversation_id) {
            for message in conversation_messages.iter_mut() {
                // Only mark messages as read if user is the recipient
                if message.recipient_id == user_id || message.sender_id != user_id {
                    message.is_read = true;
                }
            }
            Ok(())
        } else {
            Err("Conversation not found".to_string())
        }
    })
}

pub fn add_message_reaction(conversation_id: &str, message_id: &str, user_id: &str, emoji: String) -> Result<(), String> {
    MESSAGES.with(|messages| {
        if let Some(conversation_messages) = messages.borrow_mut().get_mut(conversation_id) {
            if let Some(message) = conversation_messages.iter_mut().find(|m| m.message_id == message_id) {
                // Add or update reaction
                let reaction_count = message.reactions.entry(emoji).or_insert(Vec::new());
                if !reaction_count.contains(&user_id.to_string()) {
                    reaction_count.push(user_id.to_string());
                }
                Ok(())
            } else {
                Err("Message not found".to_string())
            }
        } else {
            Err("Conversation not found".to_string())
        }
    })
}

pub fn remove_message_reaction(conversation_id: &str, message_id: &str, user_id: &str, emoji: String) -> Result<(), String> {
    MESSAGES.with(|messages| {
        if let Some(conversation_messages) = messages.borrow_mut().get_mut(conversation_id) {
            if let Some(message) = conversation_messages.iter_mut().find(|m| m.message_id == message_id) {
                if let Some(reaction_users) = message.reactions.get_mut(&emoji) {
                    reaction_users.retain(|id| id != user_id);
                    // Remove emoji entry if no users have this reaction
                    if reaction_users.is_empty() {
                        message.reactions.remove(&emoji);
                    }
                }
                Ok(())
            } else {
                Err("Message not found".to_string())
            }
        } else {
            Err("Conversation not found".to_string())
        }
    })
}

pub fn delete_message(conversation_id: &str, message_id: &str) -> Result<(), String> {
    MESSAGES.with(|messages| {
        if let Some(conversation_messages) = messages.borrow_mut().get_mut(conversation_id) {
            let initial_len = conversation_messages.len();
            conversation_messages.retain(|m| m.message_id != message_id);
            
            if conversation_messages.len() < initial_len {
                // Update conversation's last message if the deleted message was the last one
                CONVERSATIONS.with(|conversations| {
                    if let Some(conversation) = conversations.borrow_mut().get_mut(conversation_id) {
                        // If there are still messages, update last_message to the most recent one
                        if let Some(last_message) = conversation_messages.last() {
                            conversation.last_message = Some(last_message.clone());
                        } else {
                            // No messages left in conversation
                            conversation.last_message = None;
                        }
                        conversation.updated_at = get_current_timestamp();
                    }
                });
                Ok(())
            } else {
                Err("Message not found".to_string())
            }
        } else {
            Err("Conversation not found".to_string())
        }
    })
}