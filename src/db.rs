use mongodb::{Collection, bson::oid::ObjectId, bson::doc};
use anyhow::Result;
use chrono::Utc;
use crate::{cli::UpdateArgs, models::Plant, models::Update};
use futures_util::stream::TryStreamExt; // Para try_next

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

pub async fn update_plant(collection: &Collection<Plant>, args: &UpdateArgs) -> Result<()> {
    let oid = match ObjectId::parse_str(&args.id) {
        Ok(oid) => oid,
        Err(e) => {
            eprintln!("Error: ID inválido '{}': {}", args.id, e);
            return Err(e.into());
        }
    };

    let mut update_ops = doc! {};
    let mut set_ops = doc! {};

    if let Some(ref name) = args.name {
        set_ops.insert("name", name);
    }
    if let Some(ref species) = args.species {
        set_ops.insert("species", species);
    }
    if !set_ops.is_empty() {
        update_ops.insert("$set", set_ops);
    }
    if let Some(ref tag) = args.add_tag {
        update_ops.insert("$push", doc! { "tags": tag });
    }
    if let Some(ref tag) = args.remove_tag {
        update_ops.insert("$pull", doc! { "tags": tag });
    }
    if args.height_cm.is_some() || args.image_url.is_some() || args.comment.is_some() {
        let update = Update {
            date: Utc::now(),
            height_cm: args.height_cm.unwrap_or(0.0),
            image_url: args.image_url.as_ref().map_or("", String::as_str).to_string(),
            comment: args.comment.as_ref().map_or("", String::as_str).to_string(),
        };
        update_ops.insert("$push", doc! { "updates": mongodb::bson::to_bson(&update)? });
    }

    if !update_ops.is_empty() {
        match collection.update_one(doc! { "_id": oid }, update_ops).await {
            Ok(_) => println!("Planta ID {} actualizada", args.id),
            Err(e) => {
                eprintln!("Error al actualizar planta: {}", e);
                return Err(e.into());
            }
        }
    } else {
        println!("No se proporcionaron cambios para actualizar");
    }

    Ok(())
}

pub async fn view_plant(collection: &Collection<Plant>, search_param: Option<String>, id: Option<String>, ids: bool) -> Result<()> {
    let filter = match (search_param, id) {
        (Some(param), None) => {
            doc! { "$or": [
                { "tags": { "$regex": &param, "$options": "i" } },
                { "species": { "$regex": &param, "$options": "i" } },
                { "name": { "$regex": &param, "$options": "i" } },
            ]}
        }
        (None, Some(id)) => {
            let oid = match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => {
                    eprintln!("Error: ID inválido '{}': {}", id, e);
                    return Err(e.into());
                }
            };
            doc! { "_id": oid }
        }
        (None, None) => doc! {},
        (Some(_), Some(_)) => {
            eprintln!("Error: No puedes usar search_param e id juntos");
            return Err(anyhow::anyhow!("No puedes usar search_param e id juntos"));
        }
    };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            eprintln!("Error al buscar plantas: {}", e);
            return Err(e.into());
        }
    };
    let mut found = false;
    while let Some(plant) = cursor.try_next().await? {
        found = true;

        if ids {
            println!("{}, '{}'", plant.id.unwrap_or_default(), plant.name);
        } else {
            println!(
                "Name: '{}'\nEspecie: '{}'\nTags: {:?}\nNotas: {}\nID: '{}',",
                plant.name,
                plant.species,
                plant.tags,
                plant.notes,
                // plant.updates.len(),
                plant.id.unwrap_or_default(),
            );

            if plant.updates.is_empty() {
                println!("Updates: Ninguno")
            } else {
                for (i, update) in plant.updates.iter().enumerate() {
                    println!(
                        "  Update {}:\n    Fecha: {}\n    Altura: {} cm\n    Imagen: {}\n    Comentario: '{}'",
                        i + 1,
                        update.date.to_rfc3339(),
                        update.height_cm,
                        update.image_url,
                        update.comment
                    );
                }
            }
            println!();
        }
    }

    if !found {
        println!("No se encontraron plantas");
    }

    Ok(())
}