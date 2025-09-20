use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub date: DateTime<Utc>,
    pub height_cm: f32,
    pub image_url: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Plant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub species: String,
    pub tags: Vec<String>,
    pub notes: String,
    pub updates: Vec<Update>,
    pub created_at: DateTime<Utc>,
}