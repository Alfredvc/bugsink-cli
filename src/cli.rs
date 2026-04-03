use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "bugsink",
    version,
    about = "Agent-friendly CLI for Bugsink error tracking"
)]
pub struct Cli {
    /// Bugsink instance URL (overrides config and BUGSINK_URL env var)
    #[arg(long, global = true)]
    pub url: Option<String>,

    /// API token (overrides config and BUGSINK_TOKEN env var)
    #[arg(long, global = true)]
    pub token: Option<String>,

    /// Force JSON output
    #[arg(long, global = true)]
    pub json: bool,

    /// Filter output to specific fields (comma-separated)
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Fetch all pages of results (default: first page only)
    #[arg(long, global = true)]
    pub all: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Manage teams
    Teams {
        #[command(subcommand)]
        command: TeamsCommands,
    },
    /// Manage projects
    Projects {
        #[command(subcommand)]
        command: ProjectsCommands,
    },
    /// Manage issues
    Issues {
        #[command(subcommand)]
        command: IssuesCommands,
    },
    /// Manage events
    Events {
        #[command(subcommand)]
        command: EventsCommands,
    },
    /// Manage releases
    Releases {
        #[command(subcommand)]
        command: ReleasesCommands,
    },
    /// Fetch API schema for agent discoverability
    Describe,
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with a Bugsink instance
    Login,
    /// Show current authentication status
    Status {
        /// Verify credentials by making an API call
        #[arg(long)]
        verify: bool,
    },
    /// Remove stored credentials
    Logout,
}

#[derive(Subcommand)]
pub enum TeamsCommands {
    /// List all teams
    List,
    /// Get details for a specific team
    Get {
        /// Team ID
        id: u64,
    },
}

#[derive(Subcommand)]
pub enum ProjectsCommands {
    /// List projects
    List {
        /// Filter by team ID
        #[arg(long)]
        team: Option<u64>,
    },
    /// Get details for a specific project
    Get {
        /// Project ID
        id: u64,
    },
    /// Create a new project
    Create {
        /// Team ID to create the project in
        #[arg(long)]
        team: u64,
        /// Project name
        #[arg(long)]
        name: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum SortField {
    #[value(name = "digest_order")]
    DigestOrder,
    #[value(name = "last_seen")]
    LastSeen,
}

impl SortField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DigestOrder => "digest_order",
            Self::LastSeen => "last_seen",
        }
    }
}

#[derive(Clone, ValueEnum)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Subcommand)]
pub enum IssuesCommands {
    /// List issues for a project
    List {
        /// Project ID (required)
        #[arg(long)]
        project: u64,
        /// Sort by: digest_order (default) or last_seen
        #[arg(long, default_value = "digest_order")]
        sort: SortField,
        /// Order: asc (default) or desc
        #[arg(long, default_value = "asc")]
        order: SortOrder,
    },
    /// Get details for a specific issue
    Get {
        /// Issue ID
        id: u64,
    },
}

#[derive(Subcommand)]
pub enum EventsCommands {
    /// List events for an issue
    List {
        /// Issue ID (required)
        #[arg(long)]
        issue: u64,
        /// Order: asc or desc (default)
        #[arg(long, default_value = "desc")]
        order: SortOrder,
    },
    /// Get details for a specific event
    Get {
        /// Event ID
        id: String,
    },
    /// Get the stacktrace for a specific event as markdown
    Stacktrace {
        /// Event ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ReleasesCommands {
    /// List releases for a project
    List {
        /// Project ID (required)
        #[arg(long)]
        project: u64,
    },
    /// Get details for a specific release
    Get {
        /// Release ID
        id: u64,
    },
    /// Create a new release
    Create {
        /// Project ID
        #[arg(long)]
        project: u64,
        /// Release version string
        #[arg(long)]
        version: String,
    },
}
