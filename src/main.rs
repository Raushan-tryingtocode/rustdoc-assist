use anyhow::{Context, Result};

// this struct is basically just a container for the text we pull from the rust book
#[derive(Debug, Clone)]
struct DocChunk {
    text: String,
    chapter: String,
    section: String,
}

// this is what we show the user at the end when everything actually works
#[derive(Debug)]
struct FinalAnswer {
    answer: String,
    source_chapter: String,
    source_section: String,
}

// hardcoding some fake data here because reading actual files is lowkey too hard right now lol
fn mock_ingest() -> Result<Vec<DocChunk>> {
    Ok(vec![DocChunk {
        text: "Ownership rules: 1. Each value has an owner. 2. One owner at a time. 3. Dropped when out of scope.".to_string(),
        chapter: "4".to_string(),
        section: "4.1 What is Ownership".to_string(),
    }])
}

// this is our temporary database setup
struct LocalStore {
    database: Vec<DocChunk>,
}

impl LocalStore {
    fn new(database: Vec<DocChunk>) -> Self {
        Self { database }
    }

    // looks through the text to see if it matches whatever the user typed in
    fn simple_search(&self, query: &str) -> Result<&DocChunk> {
        self.database
            .iter()
            .find(|chunk| chunk.text.to_lowercase().contains(&query.to_lowercase()))
            .context("dang, couldn't find anything matching that query fr")
    }
}

// this function will eventually talk to the actual AI API
async fn get_answer(store: &LocalStore, query: &str) -> Result<FinalAnswer> {
    // first we find the matching text chunk from our fake database
    let matched_chunk = store.simple_search(query)?;

    // TODO: figure out how to actually call the gemini api here instead of hardcoding this string
    let dummy_response = format!("working on a real answer for: {}", query);

    Ok(FinalAnswer {
        answer: dummy_response,
        source_chapter: matched_chunk.chapter.clone(),
        source_section: matched_chunk.section.clone(),
    })
}

// using tokio main macro because async functions need a runtime or whatever to run
#[tokio::main]
async fn main() -> Result<()> {
    // setting up our fake data store
    let docs = mock_ingest()?;
    let store = LocalStore::new(docs);

    // testing out a search query to see if it works
    let output = get_answer(&store, "ownership").await?;

    // printing the final output to the console so we can see it
    println!(
        "Answer: {}\nSource: Chapter {} - {}",
        output.answer,
        output.source_chapter,
        output.source_section
    );

    Ok(())
}