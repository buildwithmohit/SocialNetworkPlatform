use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// User Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub profile_picture: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub account_type: AccountType,
    pub is_verified: bool,
    pub is_private: bool,
    pub followers_count: u64,
    pub following_count: u64,
    pub posts_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub links: Vec<String>,
    pub location: Option<String>,
    pub date_of_birth: Option<u64>,
    pub gender: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum AccountType {
    Personal,
    Creator,
    Business,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CreateUserProfileRequest {
    pub username: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub profile_picture: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub date_of_birth: Option<u64>,
    pub location: Option<String>,
    pub account_type: AccountType,
    pub is_private: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UpdateUserProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub profile_picture: Option<String>,
    pub website: Option<String>,
    pub is_private: Option<bool>,
    pub links: Option<Vec<String>>,
    pub location: Option<String>,
}

// Post Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    pub post_id: String,
    pub user_id: String,
    pub username: String,
    pub content_type: ContentType,
    pub media_urls: Vec<String>,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub tagged_users: Vec<String>,
    pub location: Option<LocationTag>,
    pub likes_count: u64,
    pub comments_count: u64,
    pub shares_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_archived: bool,
    pub visibility: PostVisibility,
    pub music_info: Option<MusicInfo>,
    pub product_tags: Vec<ProductTag>,
    pub(crate) post_type: (),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ContentType {
    Photo,
    Video,
    Carousel,
    Reel,
    Story,
    Live,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum PostVisibility {
    Public,
    Private,
    CloseFriends,
    Followers,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CreatePostRequest {
    pub content_type: ContentType,
    pub media_urls: Vec<String>,
    pub caption: String,
    pub hashtags: Vec<String>,
    pub tagged_users: Vec<String>,
    pub location: Option<LocationTag>,
    pub visibility: PostVisibility,
    pub music_info: Option<MusicInfo>,
    pub product_tags: Vec<ProductTag>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LocationTag {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub place_id: Option<String>,
    pub posts_count: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MusicInfo {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub start_time: u32,
    pub duration: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ProductTag {
    pub product_id: String,
    pub product_name: String,
    pub price: Option<String>,
    pub shop_name: String,
    pub x_position: f32,
    pub y_position: f32,
}

// Story Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Story {
    pub story_id: String,
    pub user_id: String,
    pub username: String,
    pub media_url: String,
    pub story_type: StoryType,
    pub text_overlay: Option<String>,
    pub stickers: Vec<Sticker>,
    pub music_info: Option<MusicInfo>,
    pub viewers: Vec<String>,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_highlight: bool,
    pub highlight_id: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum StoryType {
    Photo,
    Video,
    Boomerang,
    Layout,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Sticker {
    pub sticker_type: StickerType,
    pub content: String,
    pub x_position: f32,
    pub y_position: f32,
    pub rotation: f32,
    pub scale: f32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum StickerType {
    Poll,
    Question,
    Quiz,
    Slider,
    Countdown,
    Location,
    Hashtag,
    Mention,
    Gif,
    Emoji,
    AddYours,
}

// Comment Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub comment_id: String,
    pub post_id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub likes_count: u64,
    pub replies_count: u64,
    pub parent_comment_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_pinned: bool,
    pub mentions: Vec<String>,
}

// Message Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub media_url: Option<String>,
    pub reply_to: Option<String>,
    pub reactions: HashMap<String, Vec<String>>, // emoji -> user_ids
    pub is_read: bool,
    pub is_vanish_mode: bool,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum MessageType {
    Text,
    Photo,
    Video,
    Voice,
    Gif,
    Sticker,
    Post,
    Story,
    Reel,
    Location,
    Contact,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub conversation_id: String,
    pub participants: Vec<String>,
    pub conversation_type: ConversationType,
    pub last_message: Option<Message>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_archived: bool,
    pub group_name: Option<String>,
    pub group_photo: Option<String>,
    pub admins: Vec<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ConversationType {
    Direct,
    Group,
}

// Notification Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub notification_id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub action_user_id: Option<String>,
    pub post_id: Option<String>,
    pub comment_id: Option<String>,
    pub is_read: bool,
    pub created_at: u64,
    pub(crate) read_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum NotificationType {
    Like,
    Comment,
    Follow,
    Mention,
    Tag,
    Story,
    Live,
    Message,
    Request,
    System,
}

// Search Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SearchResults {
    pub users: Vec<UserProfile>,
    pub posts: Vec<Post>,
    pub hashtags: Vec<Hashtag>,
    pub locations: Vec<LocationTag>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum SearchType {
    All,
    Users,
    Posts,
    Hashtags,
    Locations,
    Audio,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Hashtag {
    pub name: String,
    pub posts_count: u64,
    pub is_trending: bool,
}

// Relationship Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Relationship {
    pub follower_id: String,
    pub following_id: String,
    pub status: RelationshipStatus,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum RelationshipStatus {
    Following,
    Pending,
    Blocked,
    Muted,
    Restricted,
}

// Like Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Like {
    pub user_id: String,
    pub post_id: Option<String>,
    pub comment_id: Option<String>,
    pub story_id: Option<String>,
    pub created_at: u64,
}

// Saved Posts
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SavedPost {
    pub user_id: String,
    pub post_id: String,
    pub collection_name: Option<String>,
    pub created_at: u64,
}

// Shopping Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Shop {
    pub shop_id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub website: Option<String>,
    pub contact_email: String,
    pub products: Vec<Product>,
    pub is_verified: bool,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Product {
    pub product_id: String,
    pub shop_id: String,
    pub name: String,
    pub description: String,
    pub price: String,
    pub currency: String,
    pub images: Vec<String>,
    pub category: String,
    pub is_available: bool,
    pub inventory_count: Option<u32>,
    pub created_at: u64,
}

// Live Stream Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LiveStream {
    pub stream_id: String,
    pub user_id: String,
    pub title: String,
    pub viewers_count: u64,
    pub comments: Vec<LiveComment>,
    pub is_active: bool,
    pub started_at: u64,
    pub ended_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LiveComment {
    pub comment_id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub created_at: u64,
}

// Highlight Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Highlight {
    pub highlight_id: String,
    pub user_id: String,
    pub title: String,
    pub cover_image: String,
    pub stories: Vec<String>, // story_ids
    pub created_at: u64,
    pub updated_at: u64,
}

// Analytics Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Analytics {
    pub user_id: String,
    pub post_id: Option<String>,
    pub story_id: Option<String>,
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub reach: u64,
    pub impressions: u64,
    pub profile_visits: u64,
    pub website_clicks: u64,
    pub date: u64,
}

// Report Types
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Report {
    pub report_id: String,
    pub reporter_id: String,
    pub reported_user_id: Option<String>,
    pub reported_post_id: Option<String>,
    pub reported_comment_id: Option<String>,
    pub reason: ReportReason,
    pub description: String,
    pub status: ReportStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ReportReason {
    Spam,
    Harassment,
    InappropriateContent,
    Violence,
    Hate,
    IntellectualProperty,
    SelfHarm,
    Other,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ReportStatus {
    Pending,
    UnderReview,
    Resolved,
    Dismissed,
}

// Close Friends
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CloseFriend {
    pub user_id: String,
    pub friend_id: String,
    pub created_at: u64,
}

// User Activity
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserActivity {
    pub user_id: String,
    pub action: ActivityAction,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
    pub created_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ActivityAction {
    Login,
    Logout,
    PostCreated,
    PostLiked,
    PostCommented,
    UserFollowed,
    MessageSent,
    StoryViewed,
    ProfileVisited,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct NotificationSettings {
    pub user_id: String,
    pub likes_enabled: bool,
    pub comments_enabled: bool,
    pub follows_enabled: bool,
    pub mentions_enabled: bool,
    pub messages_enabled: bool,
    pub story_views_enabled: bool,
    pub tags_enabled: bool,
    pub push_notifications: bool,
    pub email_notifications: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ActivityInsights {
    pub user_id: String,
    pub total_time_spent: u64, // in minutes
    pub daily_average: u64,    // in minutes
    pub posts_created: u32,
    pub stories_created: u32,
    pub messages_sent: u32,
    pub likes_given: u32,
    pub comments_made: u32,
    pub most_active_hour: u8, // 0-23
    pub weekly_summary: Vec<DailyActivity>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct DailyActivity {
    pub day: String,
    pub time_spent: u64, // in minutes
    pub posts: u32,
    pub stories: u32,
    pub interactions: u32,
}
