use mongodb::{Client, options::ClientOptions, bson::doc};
use anyhow::Result;
use clap::Parser;
use std::time::Duration;
mod cli;
mod db;
mod models;
use models::{Plant};
use cli::{Cli, Commands};

use crate::db::add_plant;
use crate::db::remove_plant;
use crate::db::update_plant;
use crate::db::view_plant;

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

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error al crear cliente MongoDB: {}", e);
            return Err(e.into());
        }
    };

    let db = client.database("arbolitos");
    let collection = db.collection::<Plant>("plants");

    if let Err(e) = db.run_command(doc! { "ping": 1 }).await {
        eprintln!("Error al conectar con MongoDB (ping fallido): {}", e);
        return Err(e.into());
    }

    let cli = Cli::parse();
    
    match cli.command {
        Commands::View { search_param, id, ids } => {
            view_plant(&collection, search_param, id, ids)
                .await?;
        }
        Commands::Add(args) => {
            let inserted_id = add_plant(&collection, args.name, args.species, args.tags, args.notes)
                .await?;
            println!("Planta agregada, ID: {}", inserted_id);
        }
        Commands::Update(args) => {
            update_plant(&collection, &args)
                .await?;
            println!("Updated ... ");
        }
        Commands::Remove { id } => {
            remove_plant(&collection, &id).await?;
            println!("Planta con ID: {} removida.", id);        
        }
    }

    Ok(())
}