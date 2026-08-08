use clap::{ArgAction, Parser, Subcommand, ValueHint, builder::PossibleValuesParser};
use clap_complete::Shell;

use arhiv::{
    ArhivServer,
    definitions::{TRACK_TYPE, get_standard_schema},
};
use baza::entities::Id;
use baza_common::get_crate_version;

#[derive(Parser, Debug)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true, disable_help_subcommand = true)]
#[command(name = "arhiv")]
pub(crate) struct CLIArgs {
    #[clap(subcommand)]
    pub(crate) command: CLICommand,

    /// Increases logging verbosity each use for up to 3 times. Default level is WARN.
    /// Logs are written to stderr.
    #[clap(global= true, short, action = ArgAction::Count)]
    pub(crate) verbose: u8,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CLICommand {
    /// Initialize Arhiv instance on local machine
    Init,
    /// Save an Arhiv storage key into the system keyring
    Login,
    /// Erase the cached Arhiv storage key from the system keyring
    Logout,
    /// Change Arhiv password
    ChangePassword,
    /// Export Arhiv key file.
    ExportKey {
        /// Exported key file name.
        output_file: String,

        /// Encode key file as QR Code SVG image
        #[arg(long)]
        qrcode_svg: bool,
    },
    /// Verify if file is a valid Arhiv key file and can open Arhiv.
    VerifyKey {
        /// Key file to verify.
        #[arg(value_hint = ValueHint::FilePath)]
        key_file: String,
    },
    /// Import Arhiv key file and replace existing key file.
    ImportKey {
        /// Age key file to import.
        #[arg(value_hint = ValueHint::FilePath)]
        key_file: String,
        // TODO import from qrcode img as well
    },
    /// Backup Arhiv data
    Backup {
        /// Directory to store backup.
        #[arg(value_hint = ValueHint::DirPath)]
        backup_dir: String,
    },
    /// Check or apply an Arhiv backup restore
    Restore {
        #[command(subcommand)]
        command: RestoreCommand,
    },
    /// Run server
    Server {
        /// The port to listen on
        #[arg(long, env = "SERVER_PORT", default_value_t = ArhivServer::DEFAULT_PORT)]
        port: u16,

        /// Print server info as JSON. The line will start with @@SERVER_INFO:
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Open in $BROWSER
        #[arg(long, default_value_t = false)]
        browser: bool,
    },
    /// Print current status
    Status,
    /// List document locks
    Locks,
    /// Lock document
    Lock {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Reason why the document is being locked
        #[arg(default_value = "locked by CLI")]
        reason: String,
    },
    /// Unlock document
    Unlock {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Lock key to be checked before unlocking
        #[arg()]
        key: Option<String>,
    },
    /// Commit pending changes
    Commit,
    /// List recent documents
    List {
        /// Restrict results to a document type. Can be used more than once.
        #[arg(long = "type", value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_types: Vec<String>,
        /// Page number, starting at 0
        #[arg(long, default_value_t = 0)]
        page: u8,
        /// Show only conflicted documents
        #[arg(long, default_value_t = false)]
        conflicts: bool,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Search documents
    Search {
        /// Full-text search query
        #[arg(required = true, num_args = 1.., value_name = "QUERY")]
        query: Vec<String>,
        /// Restrict results to a document type. Can be used more than once.
        #[arg(long = "type", value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_types: Vec<String>,
        /// Page number, starting at 0
        #[arg(long, default_value_t = 0)]
        page: u8,
        /// Show only conflicted documents
        #[arg(long, default_value_t = false)]
        conflicts: bool,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// List conflicted documents
    Conflicts {
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Inspect document conflicts
    Conflict {
        #[command(subcommand)]
        command: ConflictCommand,
    },
    /// Discard staged changes
    Reset {
        /// Id of the document to reset
        #[arg()]
        id: Option<Id>,
        /// Reset every staged document
        #[arg(long, default_value_t = false)]
        all: bool,
        /// Lock key to be checked before resetting a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// List committed snapshots for a document
    History {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Work with committed document snapshots
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommand,
    },
    /// Stage a committed snapshot as the current document data
    Revert {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Snapshot revision, as printed by history
        #[arg()]
        rev: String,
        /// Lock key to be checked before updating a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Compare document data
    Diff {
        #[command(subcommand)]
        command: DiffCommand,
    },
    /// Get document by id
    Get {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print raw document head JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Add new document
    Add {
        /// One of known document types
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: String,
        /// JSON object with document props
        #[arg()]
        data: String,
    },
    /// Replace an existing document's JSON data
    Update {
        /// Id of the document
        #[arg()]
        id: Id,
        /// JSON object with document props
        #[arg()]
        data: String,
        /// Lock key to be checked before updating a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Erase document by id
    Erase {
        /// Id of the document
        #[arg()]
        id: Id,
    },
    /// Print schema information
    Schema {
        /// Document type to describe. Prints all types when omitted.
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: Option<String>,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Work with document collections
    Collection {
        #[command(subcommand)]
        command: CollectionCommand,
    },
    /// Work with encrypted assets
    Asset {
        #[command(subcommand)]
        command: AssetCommand,
    },
    /// Import files and create documents.
    Import {
        /// Document type to import
        #[arg(value_parser = PossibleValuesParser::new([TRACK_TYPE]))]
        document_type: String,
        /// Files to import
        #[arg(required = true, num_args = 1.., value_hint = ValueHint::FilePath)]
        file_paths: Vec<String>,
        /// Remove original files
        #[arg(short, default_value_t = false)]
        remove_original_file: bool,
    },
    #[clap(name = "generate-completions", hide = true)]
    GenerateCompletions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum CollectionCommand {
    /// List collections containing a document
    List {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// List members of a collection
    Members {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Add a document to a collection
    Add {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to add
        #[arg()]
        id: Id,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Remove a document from a collection
    Remove {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to remove
        #[arg()]
        id: Id,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Move a collection member to a new zero-based position
    Move {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to move
        #[arg()]
        id: Id,
        /// New zero-based position
        #[arg(long)]
        to: usize,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RestoreCommand {
    /// Validate a backup manifest without mutating live storage
    Check {
        /// Encrypted backup manifest to validate.
        #[arg(value_hint = ValueHint::FilePath)]
        manifest_path: String,
        /// Permit backups that are missing referenced asset blob files.
        #[arg(long, default_value_t = false)]
        allow_missing_blobs: bool,
        /// Decrypt every referenced blob and verify plaintext size and SHA-256.
        #[arg(long, default_value_t = false)]
        deep: bool,
    },
    /// Restore a backup manifest into live storage
    Apply {
        /// Encrypted backup manifest to restore.
        #[arg(value_hint = ValueHint::FilePath)]
        manifest_path: String,
        /// Permit degraded restore when referenced asset blob files are missing.
        #[arg(long, default_value_t = false)]
        allow_missing_blobs: bool,
        /// Decrypt every referenced blob and verify plaintext size and SHA-256 before applying.
        #[arg(long, default_value_t = false)]
        deep: bool,
        /// Permit replacing newer current storage with an older backup.
        #[arg(long, default_value_t = false)]
        allow_rollback: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConflictCommand {
    /// Show conflict branches and the staged resolution for a document
    Show {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SnapshotCommand {
    /// Get a committed snapshot by document id and revision
    Get {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Snapshot revision, as printed by history
        #[arg()]
        rev: String,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum DiffCommand {
    /// Diff a staged working copy against its latest committed original
    Staged {
        /// Id of the document
        #[arg()]
        id: Id,
    },
    /// Diff two committed snapshots of one document
    Snapshots {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Left snapshot revision, as printed by history
        #[arg()]
        left_rev: String,
        /// Right snapshot revision, as printed by history
        #[arg()]
        right_rev: String,
    },
    /// Diff a conflict's staged resolution against each branch, or branch-to-branch
    Conflict {
        /// Id of the conflicted document
        #[arg()]
        id: Id,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum AssetCommand {
    /// Create encrypted asset documents from local files
    Create {
        /// Files to store as encrypted assets
        #[arg(required = true, num_args = 1.., value_hint = ValueHint::FilePath)]
        file_paths: Vec<String>,
        /// Remove original files after assets are saved
        #[arg(short, default_value_t = false)]
        remove_original_file: bool,
    },
    /// Decrypt an asset into a local file
    Export {
        /// Id of the asset document
        #[arg()]
        id: Id,
        /// Output file path
        #[arg(value_hint = ValueHint::FilePath)]
        output_file: String,
    },
}
