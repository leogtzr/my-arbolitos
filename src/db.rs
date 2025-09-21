use mongodb::{Collection, bson::oid::ObjectId, bson::doc};
use anyhow::Result;
use chrono::Utc;
use crate::models::Plant;

pub async fn add_plant(
    collection: &Collection<Plant>,
    name: String,
    species: String,
    tags: String,
    notes: String,
) -> Result<ObjectId> {
    let tags: Vec<String> = tags.split(',').map(|s| s.trim().to_string()).collect();
    let plant = Plant {
        id: None,
        name,
        species,
        tags,
        notes,
        updates: vec![],
        created_at: Utc::now(),
    };
    let result = collection.insert_one(plant).await?;
    Ok(result.inserted_id.as_object_id().unwrap())
}

pub async fn remove_plant(collection: &Collection<Plant>, id: &String) -> Result<()> {
    let oid = match ObjectId::parse_str(&id) {
        Ok(oid) => oid,
        Err(e) => {
            eprintln!("Error: ID inválido '{}': {}", id, e);
            return Err(e.into());
        }
    };
    let result = match collection.delete_one(doc! { "_id": oid }).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error al remover planta: {}", e);
            return Err(e.into());
        }
    };
    if result.deleted_count > 0 {
        println!("Planta ID {} removida", id);
    } else {
        println!("No se encontró planta con ID {}", id);
    }

    Ok(())
}