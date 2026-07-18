use crate::tools::{Property, ToolDefinition};
use anyhow::{Context, Result, bail};
use fff_search::{
    FFFMode, FilePickerOptions, FrecencyTracker, FuzzySearchOptions, GrepMode, GrepSearchOptions,
    Location, PaginationArgs, QueryParser, QueryTracker, SharedFilePicker, SharedFrecency,
    SharedQueryTracker, file_picker::FilePicker, parse_grep_query,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, fs};

#[derive(Debug)]
pub struct Search {
    shared_picker: SharedFilePicker,
    shared_frecency: SharedFrecency,
    shared_query_tracker: SharedQueryTracker,
}

impl Default for Search {
    fn default() -> Self {
        let shared_picker = SharedFilePicker::default();
        let shared_frecency = SharedFrecency::default();
        let shared_query_tracker = SharedQueryTracker::default();
        Self {
            shared_picker,
            shared_frecency,
            shared_query_tracker,
        }
    }
}

impl Search {
    pub fn index_cwd(&mut self) -> Result<()> {
        let dir_path = env::current_dir().context("failed to get current directory")?;

        // Search the workspace; keep fff state under .mimi so it doesn't pollute the project root.
        let mimi_dir = dir_path.join(".mimi");
        fs::create_dir_all(&mimi_dir).context("failed to create .mimi directory")?;

        let frecency = FrecencyTracker::open(mimi_dir.join("frecency"))
            .context("failed to open frecency tracker")?;
        self.shared_frecency
            .init(frecency)
            .context("failed to initialize frecency tracker")?;

        let query_tracker = QueryTracker::open(mimi_dir.join("queries"))
            .context("failed to open query tracker")?;

        self.shared_query_tracker
            .init(query_tracker)
            .context("failed to initialize query tracker")?;

        let dir = dir_path
            .to_str()
            .context("current directory is not valid UTF-8")?
            .to_string();

        FilePicker::new_with_shared_state(
            self.shared_picker.clone(),
            self.shared_frecency.clone(),
            FilePickerOptions {
                base_path: dir,
                mode: FFFMode::Ai,
                ..Default::default()
            },
        )
        .context("failed to initialize file picker")?;

        self.shared_picker
            .wait_for_scan(std::time::Duration::from_secs(10));

        Ok(())
    }

    pub fn search_files(&self, search_query: Value) -> Result<Value> {
        let picker_guard = match self.shared_picker.read() {
            Ok(guard) => guard,
            Err(_) => bail!("failed to acquire file picker read lock"),
        };
        let picker = picker_guard
            .as_ref()
            .context("search index has not been initialized")?;

        let qt_guard = match self.shared_query_tracker.read() {
            Ok(guard) => guard,
            Err(_) => bail!("failed to acquire query tracker read lock"),
        };

        let search_query = search_query
            .get("query")
            .and_then(|v| v.as_str())
            .context("missing query")?;

        let parser = QueryParser::default();
        let query = parser.parse(search_query);

        let search_results = picker.fuzzy_search(
            &query,
            qt_guard.as_ref(),
            FuzzySearchOptions {
                max_threads: 0,
                current_file: None,
                pagination: PaginationArgs {
                    offset: 0,
                    limit: 50,
                },
                ..Default::default()
            },
        );

        let paths: Vec<String> = search_results
            .items
            .iter()
            .map(|file| file.relative_path(picker))
            .collect();

        let results = SearchResult {
            query: search_query.to_string(),
            paths,
            total_matched: search_results.total_matched,
            location: search_results.location.map(Loc::from),
        };

        serde_json::to_value(results).context("failed to serialize file search results")
    }

    pub fn search_content(&self, search_query: Value) -> Result<Value> {
        let picker_guard = match self.shared_picker.read() {
            Ok(guard) => guard,
            Err(_) => bail!("failed to acquire file picker read lock"),
        };
        let picker = picker_guard
            .as_ref()
            .context("search index has not been initialized")?;

        let query_str = search_query
            .get("query")
            .and_then(|v| v.as_str())
            .context("missing query")?;

        let mode = match search_query
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("plain")
        {
            "regex" => GrepMode::Regex,
            "fuzzy" => GrepMode::Fuzzy,
            _ => GrepMode::PlainText,
        };

        let query = parse_grep_query(query_str);
        let grep_results = picker.grep(
            &query,
            &GrepSearchOptions {
                mode,
                smart_case: true,
                page_limit: 50,
                max_matches_per_file: 20,
                before_context: 0,
                after_context: 0,
                trim_whitespace: true,
                ..Default::default()
            },
        );

        let matches: Vec<ContentMatch> = grep_results
            .matches
            .iter()
            .filter_map(|m| {
                let file = grep_results.files.get(m.file_index)?;
                Some(ContentMatch {
                    path: file.relative_path(picker),
                    line_number: m.line_number,
                    col: m.col,
                    line_content: m.line_content.clone(),
                    context_before: m.context_before.clone(),
                    context_after: m.context_after.clone(),
                })
            })
            .collect();

        let results = ContentSearchResult {
            query: query_str.to_string(),
            matches,
            files_with_matches: grep_results.files_with_matches,
            total_files_searched: grep_results.total_files_searched,
            regex_fallback_error: grep_results.regex_fallback_error,
        };

        // println!("CONTENT SEARCH RESULT: {results:?}");

        serde_json::to_value(results).context("failed to serialize content search results")
    }

    pub fn def_search_files() -> ToolDefinition {
        let name = "search_files".to_string();
        let description =
            "Fuzzy-search for files by name/path (not file contents). Use search_content to find text inside files."
                .to_string();
        let strict = true;

        let mut properties = HashMap::new();
        let mut query_property = Property::default();
        query_property.description =
            "Filename or path fragment to fuzzy-match (e.g. 'agent_loop', 'src/tools')".to_string();
        properties.insert("query".to_string(), query_property);

        let required = Some(vec![String::from("query")]);

        ToolDefinition::new(name, description, strict, properties, required)
    }

    pub fn def_search_content() -> ToolDefinition {
        let name = "search_content".to_string();
        let description =
            "Search for text/keywords inside file contents (grep). Supports constraints like '*.rs keyword'. Use this when looking for code, symbols, or strings — not filenames."
                .to_string();
        let strict = true;

        let mut query_property = Property::default();
        query_property.description =
            "Text/keyword/regex to find inside files. Optional path filters: '*.rs fn main', 'TODO'."
                .to_string();

        let mode_property = Property {
            description: "Search mode: 'plain' (literal, default), 'regex', or 'fuzzy'".to_string(),
            property_enum: Some(vec![
                "plain".to_string(),
                "regex".to_string(),
                "fuzzy".to_string(),
            ]),
            ..Default::default()
        };

        let properties = HashMap::from([
            ("query".to_string(), query_property),
            ("mode".to_string(), mode_property),
        ]);

        let required = Some(vec![String::from("query"), String::from("mode")]);

        ToolDefinition::new(name, description, strict, properties, required)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    query: String,
    paths: Vec<String>,
    total_matched: usize,
    location: Option<Loc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentSearchResult {
    query: String,
    matches: Vec<ContentMatch>,
    files_with_matches: usize,
    total_files_searched: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    regex_fallback_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentMatch {
    path: String,
    line_number: u64,
    col: usize,
    line_content: String,
    context_before: Vec<String>,
    context_after: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Loc {
    Line(i32),
    Range { start: (i32, i32), end: (i32, i32) },
    Position { line: i32, col: i32 },
}

impl From<Location> for Loc {
    fn from(location: Location) -> Self {
        match location {
            Location::Line(line) => Self::Line(line),
            Location::Range { start, end } => Self::Range { start, end },
            Location::Position { line, col } => Self::Position { line, col },
        }
    }
}
