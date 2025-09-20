use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "my-arbolitos")]
#[command(about = "Una CLI para gestionar el crecimiento de mis árboles y plantas")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Ver plantas (con búsqueda por tag/especie o ID)
    View {
        /// Buscar por tag o especie
        #[arg(long = "search-param")]
        search_param: Option<String>,
        /// ID de la planta (ObjectId)
        #[arg(long = "id")]
        id: Option<String>,

        #[arg(long = "ids")]
        ids: bool
    },
    /// Agregar nueva planta
    Add(AddArgs),
    /// Actualizar planta existente
    Update(UpdateArgs),
    /// Remover planta
    Remove {
        /// ID de la planta a remover (ObjectId)
        #[arg(long)]
        id: String,
    },
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Nombre de la planta
    #[arg(short = 'n', long = "name")]
    pub name: String,
    /// Especie de la planta
    #[arg(short = 's', long = "species")]
    pub species: String,
    /// Tags separados por comas
    #[arg(short = 't', long = "tags")]
    pub tags: String,
    /// Notas iniciales
    #[arg(long, default_value = "")]
    pub notes: String,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// ID de la planta a actualizar (ObjectId)
    #[arg(long)]
    pub id: String,
    /// Nuevo nombre (opcional)
    #[arg(short, long)]
    pub name: Option<String>,
    /// Tag a agregar
    #[arg(long)]
    pub add_tag: Option<String>,
    /// Tag a remover
    #[arg(long)]
    pub remove_tag: Option<String>,
    /// Nueva actualización: altura en cm (opcional)
    #[arg(long)]
    pub height_cm: Option<f32>,
    /// Nueva actualización: URL de imagen (opcional)
    #[arg(long)]
    pub image_url: Option<String>,
    /// Nueva actualización: comentario (opcional)
    #[arg(long)]
    pub comment: Option<String>,
}