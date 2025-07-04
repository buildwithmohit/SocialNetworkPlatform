use crate::types::*;
use crate::state_handler;
use crate::user_management;

pub fn send_message(recipient_id: String, content: String, message_type: MessageType) -> Result<Message, String> {
    let sender_id = user_management::authenticate_user()?;
    
    if sender_id == recipient_id {
        return Err("Cannot send message to yourself".to_string());
    }
    
    // Check if recipient exists
    state_handler::get_user(&recipient_id)
        .ok_or_else(|| "Recipient not found".to_string())?;
    
    // Check if sender is blocked by recipient
    if user_management::is_user_blocked(&sender_id, &recipient_id) {
        return Err("Cannot send message to this user".to_string());
    }
    
    // Validate message content
    if content.trim().is_empty() && !matches!(message_type, MessageType::Photo | MessageType::Video | MessageType::Voice | MessageType::Gif | MessageType::Sticker) {
        return Err("Message content cannot be empty".to_string());
    }
    
    if content.len() > 1000 {
        return Err("Message must be 1000 characters or less".to_string());
    }
    
    // Find or create conversation
    let conversation_id = get_or_create_conversation(&sender_id, &recipient_id)?;
    
    let message_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let message = Message {
        message_id: message_id.clone(),
        conversation_id: conversation_id.clone(),
        sender_id: sender_id.clone(),
        recipient_id: recipient_id.clone(),
        content,
        message_type,
        media_url: None,
        reply_to: None,
        reactions: std::collections::HashMap::new(),
        is_read: false,
        is_vanish_mode: false,
        created_at: current_time,
        expires_at: None,
    };
    
    state_handler::insert_message(conversation_id, message.clone());
    
    // TODO: Send notification to recipient
    
    Ok(message)
}

pub fn send_media_message(recipient_id: String, media_url: String, message_type: MessageType, caption: Option<String>) -> Result<Message, String> {
    let sender_id = user_management::authenticate_user()?;
    
    if sender_id == recipient_id {
        return Err("Cannot send message to yourself".to_string());
    }
    
    // Check if recipient exists
    state_handler::get_user(&recipient_id)
        .ok_or_else(|| "Recipient not found".to_string())?;
    
    // Check if sender is blocked by recipient
    if user_management::is_user_blocked(&sender_id, &recipient_id) {
        return Err("Cannot send message to this user".to_string());
    }
    
    // Validate message type for media
    if !matches!(message_type, MessageType::Photo | MessageType::Video | MessageType::Voice | MessageType::Gif) {
        return Err("Invalid message type for media".to_string());
    }
    
    // Find or create conversation
    let conversation_id = get_or_create_conversation(&sender_id, &recipient_id)?;
    
    let message_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let message = Message {
        message_id: message_id.clone(),
        conversation_id: conversation_id.clone(),
        sender_id: sender_id.clone(),
        recipient_id: recipient_id.clone(),
        content: caption.unwrap_or_default(),
        message_type,
        media_url: Some(media_url),
        reply_to: None,
        reactions: std::collections::HashMap::new(),
        is_read: false,
        is_vanish_mode: false,
        created_at: current_time,
        expires_at: None,
    };
    
    state_handler::insert_message(conversation_id, message.clone());
    
    Ok(message)
}

pub fn reply_to_message(original_message_id: String, content: String, message_type: MessageType) -> Result<Message, String> {
    let sender_id = user_management::authenticate_user()?;
    
    // TODO: Get original message and validate access
    // For now, this is a placeholder implementation
    
    let message_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let message = Message {
        message_id: message_id.clone(),
        conversation_id: "temp_conversation".to_string(), // TODO: Get from original message
        sender_id: sender_id.clone(),
        recipient_id: "temp_recipient".to_string(), // TODO: Get from original message
        content,
        message_type,
        media_url: None,
        reply_to: Some(original_message_id),
        reactions: std::collections::HashMap::new(),
        is_read: false,
        is_vanish_mode: false,
        created_at: current_time,
        expires_at: None,
    };
    
    // TODO: Insert message properly
    
    Ok(message)
}

pub fn get_messages(conversation_id: String, limit: u32) -> Result<Vec<Message>, String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if user has access to this conversation
    let conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !conversation.participants.contains(&user_id) {
        return Err("Access denied".to_string());
    }
    
    let messages = state_handler::get_conversation_messages(&conversation_id, limit);
    Ok(messages)
}

pub fn get_conversations(limit: u32, offset: u32) -> Result<Vec<Conversation>, String> {
    let user_id = user_management::authenticate_user()?;
    
    // Get all conversations where user is a participant
    let mut user_conversations: Vec<Conversation> = Vec::new();
    
    // We need to iterate through all conversations and filter by participant
    // Since we don't have a direct way to get user conversations, we'll need to add this to state_handler
    // For now, let's implement a basic version that gets all conversations and filters
    
    // This is a temporary implementation - ideally we'd have an index of user conversations
    let all_conversations = state_handler::get_all_conversations();
    
    for conversation in all_conversations {
        if conversation.participants.contains(&user_id) {
            user_conversations.push(conversation);
        }
    }
    
    // Sort by last message timestamp (most recent first)
    user_conversations.sort_by(|a, b| {
        let a_time = a.last_message.as_ref().map(|m| m.created_at).unwrap_or(a.created_at);
        let b_time = b.last_message.as_ref().map(|m| m.created_at).unwrap_or(b.created_at);
        b_time.cmp(&a_time)
    });
    
    // Apply pagination
    let start = offset as usize;
    let end = start + limit as usize;
    
    if start >= user_conversations.len() {
        return Ok(Vec::new());
    }
    
    let end = end.min(user_conversations.len());
    Ok(user_conversations[start..end].to_vec())
}

pub fn mark_message_as_read(message_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Find the message in all conversations
    let mut message_found = false;
    let mut conversation_id = String::new();
    
    // Get all conversations where user is a participant
    let all_conversations = state_handler::get_all_conversations();
    
    for conversation in all_conversations {
        if conversation.participants.contains(&user_id) {
            let messages = state_handler::get_conversation_messages(&conversation.conversation_id, 1000);
            
            for message in messages {
                if message.message_id == message_id {
                    // Check if user is the recipient (can't mark own messages as read)
                    if message.recipient_id == user_id || message.sender_id != user_id {
                        conversation_id = conversation.conversation_id.clone();
                        message_found = true;
                        break;
                    } else {
                        return Err("Cannot mark your own message as read".to_string());
                    }
                }
            }
            
            if message_found {
                break;
            }
        }
    }
    
    if !message_found {
        return Err("Message not found or access denied".to_string());
    }
    
    // Mark the message as read
    state_handler::mark_message_as_read(&conversation_id, &message_id, &user_id)?;
    
    Ok(())
}

pub fn mark_conversation_as_read(conversation_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if user has access to this conversation
    let conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !conversation.participants.contains(&user_id) {
        return Err("Access denied".to_string());
    }
    
    // Mark all messages in conversation as read for this user
    state_handler::mark_conversation_as_read(&conversation_id, &user_id)?;
    
    Ok(())
}

pub fn add_reaction_to_message(message_id: String, emoji: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Validate emoji (basic validation)
    if emoji.trim().is_empty() {
        return Err("Emoji cannot be empty".to_string());
    }
    
    if emoji.len() > 10 {
        return Err("Emoji too long".to_string());
    }
    
    // Find the message and check access
    let mut message_found = false;
    let mut conversation_id = String::new();
    
    let all_conversations = state_handler::get_all_conversations();
    
    for conversation in all_conversations {
        if conversation.participants.contains(&user_id) {
            let messages = state_handler::get_conversation_messages(&conversation.conversation_id, 1000);
            
            for message in messages {
                if message.message_id == message_id {
                    conversation_id = conversation.conversation_id.clone();
                    message_found = true;
                    break;
                }
            }
            
            if message_found {
                break;
            }
        }
    }
    
    if !message_found {
        return Err("Message not found or access denied".to_string());
    }
    
    // Add reaction to message
    state_handler::add_message_reaction(&conversation_id, &message_id, &user_id, emoji)?;
    
    Ok(())
}

pub fn remove_reaction_from_message(message_id: String, emoji: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Find the message and check access
    let mut message_found = false;
    let mut conversation_id = String::new();
    
    let all_conversations = state_handler::get_all_conversations();
    
    for conversation in all_conversations {
        if conversation.participants.contains(&user_id) {
            let messages = state_handler::get_conversation_messages(&conversation.conversation_id, 1000);
            
            for message in messages {
                if message.message_id == message_id {
                    conversation_id = conversation.conversation_id.clone();
                    message_found = true;
                    break;
                }
            }
            
            if message_found {
                break;
            }
        }
    }
    
    if !message_found {
        return Err("Message not found or access denied".to_string());
    }
    
    // Remove reaction from message
    state_handler::remove_message_reaction(&conversation_id, &message_id, &user_id, emoji)?;
    
    Ok(())
}

pub fn delete_message(message_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Find the message and check ownership/admin rights
    let mut message_found = false;
    let mut conversation_id = String::new();
    let mut can_delete = false;
    
    let all_conversations = state_handler::get_all_conversations();
    
    for conversation in all_conversations {
        if conversation.participants.contains(&user_id) {
            let messages = state_handler::get_conversation_messages(&conversation.conversation_id, 1000);
            
            for message in messages {
                if message.message_id == message_id {
                    conversation_id = conversation.conversation_id.clone();
                    message_found = true;
                    
                    // Check if user can delete this message
                    if message.sender_id == user_id {
                        // User owns the message
                        can_delete = true;
                    } else if matches!(conversation.conversation_type, ConversationType::Group) {
                        // Check if user is admin of the group
                        if conversation.admins.contains(&user_id) {
                            can_delete = true;
                        }
                    }
                    
                    break;
                }
            }
            
            if message_found {
                break;
            }
        }
    }
    
    if !message_found {
        return Err("Message not found or access denied".to_string());
    }
    
    if !can_delete {
        return Err("You don't have permission to delete this message".to_string());
    }
    
    // Delete the message
    state_handler::delete_message(&conversation_id, &message_id)?;
    
    Ok(())
}

pub fn create_group_chat(participants: Vec<String>, group_name: String, group_photo: Option<String>) -> Result<Conversation, String> {
    let creator_id = user_management::authenticate_user()?;
    
    // Validate participants
    if participants.len() < 2 {
        return Err("Group chat must have at least 2 participants".to_string());
    }
    
    if participants.len() > 50 {
        return Err("Group chat cannot have more than 50 participants".to_string());
    }
    
    // Check if all participants exist
    for participant_id in &participants {
        if !state_handler::user_exists(participant_id) {
            return Err(format!("User {} not found", participant_id));
        }
    }
    
    // Validate group name
    if group_name.trim().is_empty() {
        return Err("Group name cannot be empty".to_string());
    }
    
    if group_name.len() > 50 {
        return Err("Group name must be 50 characters or less".to_string());
    }
    
    let conversation_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let mut all_participants = participants;
    if !all_participants.contains(&creator_id) {
        all_participants.push(creator_id.clone());
    }
    
    let conversation = Conversation {
        conversation_id: conversation_id.clone(),
        participants: all_participants,
        conversation_type: ConversationType::Group,
        last_message: None,
        created_at: current_time,
        updated_at: current_time,
        is_archived: false,
        group_name: Some(group_name),
        group_photo,
        admins: vec![creator_id], // Creator is the initial admin
    };
    
    state_handler::insert_conversation(conversation_id, conversation.clone());
    
    Ok(conversation)
}

pub fn add_participant_to_group(conversation_id: String, participant_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if conversation exists and is a group
    let mut conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !matches!(conversation.conversation_type, ConversationType::Group) {
        return Err("Not a group conversation".to_string());
    }
    
    // Check if user is admin of the group
    if !conversation.admins.contains(&user_id) {
        return Err("Only group admins can add participants".to_string());
    }
    
    // Check if participant exists
    if !state_handler::user_exists(&participant_id) {
        return Err("User not found".to_string());
    }
    
    // Check if participant is already in the group
    if conversation.participants.contains(&participant_id) {
        return Err("User is already a participant".to_string());
    }
    
    conversation.participants.push(participant_id);
    conversation.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_conversation(conversation_id, conversation);
    
    Ok(())
}

pub fn remove_participant_from_group(conversation_id: String, participant_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if conversation exists and is a group
    let mut conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !matches!(conversation.conversation_type, ConversationType::Group) {
        return Err("Not a group conversation".to_string());
    }
    
    // Check if user is admin of the group or removing themselves
    if !conversation.admins.contains(&user_id) && user_id != participant_id {
        return Err("Only group admins can remove participants".to_string());
    }
    
    // Check if participant is in the group
    if !conversation.participants.contains(&participant_id) {
        return Err("User is not a participant".to_string());
    }
    
    // Remove participant
    conversation.participants.retain(|id| id != &participant_id);
    
    // If participant was an admin, remove from admins too
    conversation.admins.retain(|id| id != &participant_id);
    
    conversation.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_conversation(conversation_id, conversation);
    
    Ok(())
}

pub fn leave_group(conversation_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    remove_participant_from_group(conversation_id, user_id)
}

pub fn make_group_admin(conversation_id: String, participant_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if conversation exists and is a group
    let mut conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !matches!(conversation.conversation_type, ConversationType::Group) {
        return Err("Not a group conversation".to_string());
    }
    
    // Check if user is admin of the group
    if !conversation.admins.contains(&user_id) {
        return Err("Only group admins can promote other users".to_string());
    }
    
    // Check if participant is in the group
    if !conversation.participants.contains(&participant_id) {
        return Err("User is not a participant".to_string());
    }
    
    // Check if participant is already an admin
    if conversation.admins.contains(&participant_id) {
        return Err("User is already an admin".to_string());
    }
    
    conversation.admins.push(participant_id);
    conversation.updated_at = state_handler::get_current_timestamp();
    
    state_handler::insert_conversation(conversation_id, conversation);
    
    Ok(())
}

pub fn enable_vanish_mode(conversation_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if user has access to this conversation
    let conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !conversation.participants.contains(&user_id) {
        return Err("Access denied".to_string());
    }
    
    // TODO: Implement vanish mode state tracking
    
    Ok(())
}

pub fn disable_vanish_mode(conversation_id: String) -> Result<(), String> {
    let user_id = user_management::authenticate_user()?;
    
    // Check if user has access to this conversation
    let conversation = state_handler::get_conversation(&conversation_id)
        .ok_or_else(|| "Conversation not found".to_string())?;
    
    if !conversation.participants.contains(&user_id) {
        return Err("Access denied".to_string());
    }
    
    // TODO: Implement vanish mode state tracking
    
    Ok(())
}

// Helper function to get or create a direct conversation between two users
fn get_or_create_conversation(user1_id: &str, user2_id: &str) -> Result<String, String> {
    // TODO: Check if conversation already exists between these users
    // For now, create a new conversation ID
    
    let conversation_id = state_handler::generate_id();
    let current_time = state_handler::get_current_timestamp();
    
    let conversation = Conversation {
        conversation_id: conversation_id.clone(),
        participants: vec![user1_id.to_string(), user2_id.to_string()],
        conversation_type: ConversationType::Direct,
        last_message: None,
        created_at: current_time,
        updated_at: current_time,
        is_archived: false,
        group_name: None,
        group_photo: None,
        admins: Vec::new(), // Direct conversations don't have admins
    };
    
    state_handler::insert_conversation(conversation_id.clone(), conversation);
    
    Ok(conversation_id)
}
