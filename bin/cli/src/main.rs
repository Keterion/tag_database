use clap::{Parser, Subcommand};


/// CLI to send commands to the sqlite database
#[derive(Parser, Debug)]
struct Args {
    /// Path of the database
    path: String,
    /// Query the database for matching images
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Query different types of data
    #[clap(short_flag='q')]
    Query {
        #[command(subcommand)]
        q: Query,
    },
    #[clap(short_flag='d')]
    Delete {
        #[command(subcommand)]
        t: Target,
    }
}

#[derive(Subcommand, Debug)]
enum Query {
    /// Get tags of an image
    Tags {
        path: String,
    },
    /// Get images with matching tags
    Images {
        filter: String,
    }
}
#[derive(Subcommand, Debug)]
enum Target {
    All {},
}

fn main() {
    let args = Args::parse();
    let mut conn = rusqlite::Connection::open(&args.path).unwrap();
    db::init::init_tables(&conn).unwrap();
    db::tags::add_tags(vec!["a", "b", "c", "d"], &mut conn);
    db::images::add_image("test.jpg", &conn);
    //db::tags::add_tag_to_img("a", 1, false, &conn);

    match &args.cmd {
        Commands::Query { q } => {
            match &q {
                Query::Tags { path } => {
                    let id = db::utils::get_id("images", &format!("path='{}'", path), &conn).unwrap();
                    let tags = db::images::get_tags_of_img(id, &conn);
                    for tag in tags {
                        println!("{}", tag.1);
                    }
                },
                Query::Images { .. } => {
                    todo!()
                }
            }
        },
        Commands::Delete { t } => {
            match &t {
                Target::All {  } => {
                    db::init::recreate_db(std::path::PathBuf::from(args.path));
                }
            }
        },
        _ => {
            println!("Isn't a query");
        }
    }
}
