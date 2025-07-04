use crate::state_handler;
use crate::types::*;
use crate::user_management;

pub fn search_content(query: String, search_type: SearchType) -> Result<SearchResults, String> {
    let _user_id = user_management::authenticate_user().ok();
    
    let mut results = SearchResults {
        users: Vec::new(),
        posts: Vec::new(),
        hashtags: Vec::new(),
        locations: Vec::new(),
    };
    
    match search_type {
        SearchType::All => {
            results.users = search_users(&query, 10);
            results.posts = search_posts(&query, 10);
            results.hashtags = search_hashtags(&query, 10);
            results.locations = search_locations(&query, 10);
        },
        SearchType::Users => {
            results.users = search_users(&query, 50);
        },
        SearchType::Posts => {
            results.posts = search_posts(&query, 50);
        },
        SearchType::Hashtags => {
            results.hashtags = search_hashtags(&query, 50);
        },
        SearchType::Locations => {
            results.locations = search_locations(&query, 50);
        },
        SearchType::Audio => {
            // Search for posts with audio content
            results.posts = search_audio_posts(&query, 50);
        }
    }
    
    Ok(results)
}

// Helper function for audio search
fn search_audio_posts(query: &str, limit: usize) -> Vec<Post> {
    let all_posts = state_handler::get_all_posts();
    
    let matching_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| {
            matches!(post.visibility, PostVisibility::Public) &&
            (post.caption.to_lowercase().contains(&query.to_lowercase()) ||
             post.hashtags.iter().any(|h| h.to_lowercase().contains(&query.to_lowercase())))
        })
        .take(limit)
        .collect();
    
    matching_posts
}

pub fn get_explore_content(limit: u32) -> Result<Vec<Post>, String> {
    let current_user = user_management::authenticate_user().ok();

    // Get all public posts
    let all_posts = state_handler::get_all_posts();
    let public_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| matches!(post.visibility, PostVisibility::Public))
        .collect();

    // Sort by engagement and recency
    let mut explore_posts = public_posts;
    explore_posts.sort_by(|a, b| {
        let a_score = calculate_explore_score(a);
        let b_score = calculate_explore_score(b);
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Filter out posts from users the current user follows (to show new content)
    if let Some(user_id) = current_user {
        let following = state_handler::get_following(&user_id);
        explore_posts.retain(|post| !following.contains(&post.user_id) && post.user_id != user_id);
    }

    explore_posts.truncate(limit as usize);
    Ok(explore_posts)
}

pub fn get_trending_hashtags(limit: u32) -> Result<Vec<Hashtag>, String> {
    let hashtags = state_handler::search_hashtags("");

    // Sort by posts count and recent activity
    let mut trending = hashtags;
    trending.sort_by(|a, b| b.posts_count.cmp(&a.posts_count));

    trending.truncate(limit as usize);
    Ok(trending)
}

pub fn get_posts_by_location(
    location: LocationTag,
    limit: u32,
    offset: u32,
) -> Result<Vec<Post>, String> {
    let all_posts = state_handler::get_all_posts();

    let location_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| {
            if let Some(post_location) = &post.location {
                // Simple location matching - in a real app you'd use geospatial queries
                post_location
                    .name
                    .to_lowercase()
                    .contains(&location.name.to_lowercase())
                    || (post_location.latitude - location.latitude).abs() < 0.01
                        && (post_location.longitude - location.longitude).abs() < 0.01
            } else {
                false
            }
        })
        .filter(|post| matches!(post.visibility, PostVisibility::Public))
        .collect();

    // Sort by creation time (newest first)
    let mut sorted_posts = location_posts;
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

pub fn get_suggested_users(limit: u32) -> Result<Vec<UserProfile>, String> {
    let user_id = user_management::authenticate_user()?;

    // Get current user's following list
    let following = state_handler::get_following(&user_id);
    let blocked_users = state_handler::get_blocked_users_list(&user_id);

    // Get all users
    let all_users = state_handler::get_all_users();

    // Filter out current user, already following, and blocked users
    let mut candidate_users: Vec<UserProfile> = all_users
        .into_iter()
        .filter(|user| {
            user.user_id != user_id
                && !following.contains(&user.user_id)
                && !blocked_users.contains(&user.user_id)
        })
        .collect();

    // Get user's followers for mutual connections
    let user_followers = state_handler::get_followers(&user_id);

    // Score users based on recommendation factors
    let mut scored_users: Vec<(UserProfile, f64)> = candidate_users
        .into_iter()
        .map(|user| {
            let mut score = 0.0;

            // Factor 1: Mutual followers (highest weight)
            let user_following = state_handler::get_following(&user.user_id);
            let mutual_connections = user_followers
                .iter()
                .filter(|follower| user_following.contains(follower))
                .count();
            score += mutual_connections as f64 * 3.0;

            // Factor 2: Follower count (popularity)
            score += (user.followers_count as f64).log10() * 0.5;

            // Factor 3: Activity level (posts count)
            score += (user.posts_count as f64).log10() * 0.3;

            // Factor 4: Account age (newer accounts get slight boost for discovery)
            let current_time = state_handler::get_current_timestamp();
            let account_age_days =
                (current_time - user.created_at) as f64 / (1000.0 * 60.0 * 60.0 * 24.0);
            if account_age_days < 30.0 {
                score += 0.5; // Boost for new users
            }

            // Factor 5: Verified users get slight boost
            if user.is_verified {
                score += 0.2;
            }

            (user, score)
        })
        .collect();

    // Sort by score (highest first)
    scored_users.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Extract users and apply limit
    let suggested_users: Vec<UserProfile> = scored_users
        .into_iter()
        .map(|(user, _)| user)
        .take(limit as usize)
        .collect();

    Ok(suggested_users)
}

pub fn get_nearby_locations(
    latitude: f64,
    longitude: f64,
    radius_km: f64,
) -> Result<Vec<LocationTag>, String> {
    // Get all posts to extract location data
    let all_posts = state_handler::get_all_posts();

    // Extract unique locations from posts
    let mut location_map: std::collections::HashMap<String, LocationTag> =
        std::collections::HashMap::new();

    for post in all_posts {
        if let Some(location) = post.location {
            // Calculate distance using Haversine formula
            let distance =
                calculate_distance(latitude, longitude, location.latitude, location.longitude);

            if distance <= radius_km {
                // Use location name as key to avoid duplicates
                let key = format!(
                    "{}_{:.6}_{:.6}",
                    location.name, location.latitude, location.longitude
                );

                if let Some(existing_location) = location_map.get_mut(&key) {
                    // Update post count for existing location
                    existing_location.posts_count += 1;
                } else {
                    // Add new location
                    let mut new_location = location.clone();
                    new_location.posts_count = 1;
                    location_map.insert(key, new_location);
                }
            }
        }
    }

    // Convert to vector and sort by post count (most popular first)
    let mut nearby_locations: Vec<LocationTag> = location_map.into_values().collect();
    nearby_locations.sort_by(|a, b| b.posts_count.cmp(&a.posts_count));

    Ok(nearby_locations)
}

fn search_users(query: &str, limit: usize) -> Vec<UserProfile> {
    state_handler::search_users(query, limit)
}

fn search_posts(query: &str, limit: usize) -> Vec<Post> {
    let all_posts = state_handler::get_all_posts();

    let matching_posts: Vec<Post> = all_posts
        .into_iter()
        .filter(|post| {
            matches!(post.visibility, PostVisibility::Public)
                && (post.caption.to_lowercase().contains(&query.to_lowercase())
                    || post
                        .hashtags
                        .iter()
                        .any(|h| h.to_lowercase().contains(&query.to_lowercase())))
        })
        .take(limit)
        .collect();

    matching_posts
}

fn search_hashtags(query: &str, limit: usize) -> Vec<Hashtag> {
    let hashtags = state_handler::search_hashtags(query);
    hashtags.into_iter().take(limit).collect()
}

fn search_locations(query: &str, limit: usize) -> Vec<LocationTag> {
    // Get all posts to extract location data
    let all_posts = state_handler::get_all_posts();

    // Extract and filter locations based on query
    let mut location_map: std::collections::HashMap<String, LocationTag> =
        std::collections::HashMap::new();

    for post in all_posts {
        if let Some(location) = post.location {
            // Check if location name matches query
            if location.name.to_lowercase().contains(&query.to_lowercase()) {
                let key = format!(
                    "{}_{:.6}_{:.6}",
                    location.name, location.latitude, location.longitude
                );

                if let Some(existing_location) = location_map.get_mut(&key) {
                    existing_location.posts_count += 1;
                } else {
                    let mut new_location = location.clone();
                    new_location.posts_count = 1;
                    location_map.insert(key, new_location);
                }
            }
        }
    }

    // Convert to vector and sort by relevance (exact matches first, then by post count)
    let mut matching_locations: Vec<LocationTag> = location_map.into_values().collect();

    matching_locations.sort_by(|a, b| {
        let a_exact = a.name.to_lowercase() == query.to_lowercase();
        let b_exact = b.name.to_lowercase() == query.to_lowercase();

        match (a_exact, b_exact) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.posts_count.cmp(&a.posts_count),
        }
    });

    matching_locations.into_iter().take(limit).collect()
}

// Helper function to calculate distance between two points using Haversine formula
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

fn calculate_explore_score(post: &Post) -> f64 {
    let current_time = state_handler::get_current_timestamp();
    let age_hours = (current_time - post.created_at) as f64 / (1000.0 * 60.0 * 60.0);

    // Calculate engagement score
    let engagement = post.likes_count + post.comments_count + (post.shares_count * 2);

    // Apply time decay - newer posts get higher scores
    let time_factor = 1.0 / (1.0 + age_hours / 24.0); // Decay over days

    engagement as f64 * time_factor
}
