// use crate::tools;
use crate::tools::{Property, ToolDefinition};
use fff_search::{
    FFFMode, FilePickerOptions, FrecencyTracker, FuzzySearchOptions, Location, PaginationArgs,
    QueryParser, QueryTracker, SharedFilePicker, SharedFrecency, SharedQueryTracker,
    file_picker::FilePicker,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env};

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
    pub fn index_cwd(&mut self) -> Result<(), String> {
        let dir_path = env::current_dir().map_err(|e| e.to_string())?;

        let frecency =
            FrecencyTracker::open(dir_path.join("frecency")).map_err(|e| e.to_string())?;
        self.shared_frecency
            .init(frecency)
            .map_err(|e| e.to_string())?;

        let query_tracker =
            QueryTracker::open(dir_path.join("queries")).map_err(|e| e.to_string())?;

        self.shared_query_tracker
            .init(query_tracker)
            .map_err(|e| e.to_string())?;

        let dir = dir_path
            .to_str()
            .ok_or_else(|| "couldnt convert".to_string())?
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
        .map_err(|e| e.to_string())?;

        self.shared_picker
            .wait_for_scan(std::time::Duration::from_secs(10));

        Ok(())
    }

    pub fn search_files(&self, search_query: Value) -> Result<Value, String> {
        let picker_guard = self.shared_picker.read().map_err(|e| e.to_string())?;
        let picker = picker_guard
            .as_ref()
            .ok_or_else(|| "search index has not been initialized".to_string())?;

        let qt_guard = self
            .shared_query_tracker
            .read()
            .map_err(|e| e.to_string())?;

        let search_query = search_query
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing query".to_string())?;

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

        // println!("Search Result: {results:?}");

        Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
    }

    pub fn def_search_files() -> ToolDefinition {
        let name = "search_files".to_string();
        let description = "Performs fuzzy search with frecency-weighted scoring".to_string();
        let strict = true;

        let mut properties = HashMap::new();
        let mut query_property = Property::default();
        query_property.description = "Provide a query to search".to_string();
        properties.insert("query".to_string(), query_property);

        let required = Some(vec![String::from("query")]);

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
