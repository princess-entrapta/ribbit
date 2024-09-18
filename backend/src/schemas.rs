use core::fmt::Display;

use serde::{Deserialize, Serialize};
use slug::slugify;

use crate::rest::PublishForm;

#[derive(Debug)]
pub struct AppError {
    pub reason: String,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.as_str())
    }
}

type GroupId = uuid::Uuid;
type AuthorId = String; // Typically, a handle/slug
type UserId = uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub author: AuthorInfo,
    pub body: String,
    pub can_reply: bool, // As a post reader, can i reply to this
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Authorization {
    Admin,
    Member,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthorInfo {
    pub name: String,
    pub profile_picture: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Page<T> {
    pub objects: Vec<T>,
    pub current_page: usize,
    pub per_page: usize,
    pub total_objects: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GroupManagement {
    Open,         // Anyone can join or leave freely
    MemberInvite, // Members can invite other members
    AdminInvite,  // Admins can invite other members
}

// Groups can be created freely and contain any number of members.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupEntity {
    pub id: GroupId,
    // Groups can have very varied ways to manage membership.
    pub management: GroupManagement,
    // If the members are allowed to post in.
    pub allow_member_posting: bool,
    pub face: Option<AuthorId>, // Refers to an AuthorEntity that can post. This is controlled by admins.
    pub admins: Vec<UserId>,    // Always at least one person
    pub members: Vec<UserId>,   // Always at least admins
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthorEntity {
    pub author_id: AuthorId,
    pub name: String,
    pub profile_picture: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEntity {
    pub id: UserId,
    // Groups that i have joined, created, or admin.
    // Note that because groups can have faces, i can create myself aliases this way.
    // I also can create a friend list, a friend group, a community...
    pub groups: Vec<GroupId>,
    // The users i have blocked. They are excluded from ALL my groups. Unlike groups, they need no consent for being joined.
    pub block_list: Vec<UserId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostEntity {
    pub title: String,
    pub slug: String,
    pub author: AuthorId,
    pub search_tags: Vec<String>,
    pub body: String,
    pub space: Option<GroupId>,       // None means public.
    pub reply_scope: Option<GroupId>, // None means space inherited
    pub visibility_scope: Option<GroupId>, // idem
                                      // pub publish_datetime: Chrono....    // TODO
}
impl PostEntity {
    pub fn search_tags(&self) -> Vec<String> {
        self.title
            .split(" ")
            .map(|s| s.to_lowercase())
            .chain(self.search_tags.clone())
            .collect()
    }

    pub fn from_form(form: PublishForm) -> Self {
        Self {
            title: form.title.clone(),
            slug: slugify(form.title),
            author: "Some author".to_string(),
            search_tags: form.tags.split(" ").map(|s| s.to_string()).collect(),
            body: form.body,
            space: None,
            reply_scope: None,
            visibility_scope: None,
        }
    }
}
