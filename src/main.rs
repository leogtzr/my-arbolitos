use mongodb::{Client, options::ClientOptions, bson::doc};
use anyhow::Result;
use clap::Parser;
use mongodb::bson::oid::ObjectId;
use std::time::Duration;
use futures_util::stream::TryStreamExt; // Para try_next
use chrono::Utc;
mod models;
mod cli;
use models::{Plant, Update};
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to MongoDB:
    let uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017/arbolitos".to_string());

    let mut client_options = match ClientOptions::parse(&uri).await {
        Ok(options) => options,
        Err(e) => {
            eprintln!("Error al parsear URI de MongoDB: {}", e);
            return Err(e.into());
        }
    };
    client_options.server_selection_timeout = Some(Duration::from_secs(10));
    // println!("Opciones de cliente configuradas");

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error al crear cliente MongoDB: {}", e);
            return Err(e.into());
        }
    };

    let db = client.database("arbolitos");
    let collection = db.collection::<Plant>("plants");
    // println!("Conexión a MongoDB establecida, usando DB: arbolitos, colección: plants");

    // Verificar conexión con un ping
    if let Err(e) = db.run_command(doc! { "ping": 1 }).await {
        eprintln!("Error al conectar con MongoDB (ping fallido): {}", e);
        return Err(e.into());
    }
    // println!("Ping a MongoDB exitoso");

    let cli = Cli::parse();
    
    match cli.command {
        Commands::View { search_param, id, ids } => {
            let filter = match (search_param, id) {
                (Some(param), None) => {
                    doc! { "$or": [
                        { "tags": { "$regex": &param, "$options": "i" } },
                        { "species": { "$regex": &param, "$options": "i" } }
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
        }
        Commands::Add(args) => {
            let tags: Vec<String> = args.tags.split(',').map(|s| s.trim().to_string()).collect();
            let plant = Plant {
                id: None,
                name: args.name,
                species: args.species,
                tags,
                notes: args.notes,
                updates: vec![],
                created_at: Utc::now(),
            };
            let result = match collection.insert_one(plant).await {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error al agregar planta: {}", e);
                    return Err(e.into());
                }
            };
            println!("Planta agregada, ID: {}", result.inserted_id.as_object_id().unwrap());
        }
        Commands::Update(args) => {
            let oid = match ObjectId::parse_str(&args.id) {
                Ok(oid) => oid,
                Err(e) => {
                    eprintln!("Error: ID inválido '{}': {}", args.id, e);
                    return Err(e.into());
                }
            };

            let mut update_ops = doc! {};
            let mut set_ops = doc! {};

            if let Some(name) = args.name {
                set_ops.insert("name", name);
            }
            if !set_ops.is_empty() {
                update_ops.insert("$set", set_ops);
            }
            if let Some(tag) = args.add_tag {
                update_ops.insert("$push", doc! { "tags": tag });
            }
            if let Some(tag) = args.remove_tag {
                update_ops.insert("$pull", doc! { "tags": tag });
            }
            if args.height_cm.is_some() || args.image_url.is_some() || args.comment.is_some() {
                let update = Update {
                    date: Utc::now(),
                    height_cm: args.height_cm.unwrap_or(0.0),
                    image_url: args.image_url.unwrap_or_default(),
                    comment: args.comment.unwrap_or_default(),
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
        }
        Commands::Remove { id } => {
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
        }
    }

    Ok(())
}