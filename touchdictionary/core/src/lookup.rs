use serde::{Deserialize, Serialize};

// API Response Structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupResult {
    pub query: String,
    pub content_type: ContentType,
    pub sections: Sections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Word,
    Entity,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sections {
    pub definitions: Option<Vec<DefinitionSection>>,
    pub wikipedia: Option<WikipediaSection>,
    pub thesaurus: Option<ThesaurusSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionSection {
    pub source: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub part_of_speech: Option<String>,
    pub definition: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikipediaSection {
    pub title: String,
    pub summary: String,
    pub paragraphs: Vec<String>,
    pub image_url: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThesaurusSection {
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub related_terms: Vec<String>,
}

// Core lookup function - the heart of the application
pub async fn lookup(query: &str) -> Result<LookupResult, String> {
    let cleaned_query = clean_query(query);

    if cleaned_query.is_empty() {
        return Err("Empty query".to_string());
    }

    let content_type = classify_content(&cleaned_query);

    let sections = match content_type {
        ContentType::Word => aggregate_word_sources(&cleaned_query).await,
        ContentType::Entity => aggregate_entity_sources(&cleaned_query).await,
        ContentType::Mixed => aggregate_all_sources(&cleaned_query).await,
    };

    match sections {
        Ok(sections) => Ok(LookupResult {
            query: cleaned_query,
            content_type,
            sections,
        }),
        Err(e) => Err(format!("Failed to aggregate lookup results: {}", e)),
    }
}

fn clean_query(query: &str) -> String {
    query.trim()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

fn classify_content(query: &str) -> ContentType {
    // Simple heuristic to classify content type
    // In a real implementation, this would be more sophisticated

    // Check if it's likely a named entity (proper noun)
    if query.chars().next().map_or(false, |c| c.is_uppercase()) {
        return ContentType::Entity;
    }

    // Check for multi-word phrases (likely entities)
    if query.split_whitespace().count() > 2 {
        return ContentType::Entity;
    }

    // Default to word for single lowercase terms
    ContentType::Word
}

async fn aggregate_word_sources(query: &str) -> Result<Sections, String> {
    let mut sections = Sections {
        definitions: None,
        wikipedia: None,
        thesaurus: None,
    };

    // Try dictionary sources first
    match get_dictionary_definitions(query).await {
        Ok(defs) if !defs.is_empty() => {
            sections.definitions = Some(defs);
        }
        Ok(_) => {
            eprintln!("[WARN] [touchdictionary] [dictionary] No definitions found for '{}'", query);
        }
        Err(e) => {
            eprintln!("[ERROR] [touchdictionary] [dictionary] Failed to fetch definitions for '{}': {}", query, e);
        }
    }

    // Try Wikipedia as supplemental
    match get_wikipedia_summary(query).await {
        Ok(wiki) => sections.wikipedia = Some(wiki),
        Err(e) => {
            eprintln!("[WARN] [touchdictionary] [wikipedia] Failed to fetch Wikipedia summary for '{}': {}", query, e);
        }
    }

    Ok(sections)
}

async fn aggregate_entity_sources(query: &str) -> Result<Sections, String> {
    let mut sections = Sections {
        definitions: None,
        wikipedia: None,
        thesaurus: None,
    };

    // Prioritize Wikipedia for entities
    match get_wikipedia_summary(query).await {
        Ok(wiki) => sections.wikipedia = Some(wiki),
        Err(e) => {
            eprintln!("[ERROR] [touchdictionary] [wikipedia] Failed to fetch Wikipedia for entity '{}': {}", query, e);
        }
    }

    Ok(sections)
}

async fn aggregate_all_sources(query: &str) -> Result<Sections, String> {
    let mut sections = Sections {
        definitions: None,
        wikipedia: None,
        thesaurus: None,
    };

    // Try all sources and aggregate results
    match get_dictionary_definitions(query).await {
        Ok(defs) if !defs.is_empty() => {
            sections.definitions = Some(defs);
        }
        Ok(_) => {
            eprintln!("[WARN] [touchdictionary] [dictionary] No definitions found for '{}'", query);
        }
        Err(e) => {
            eprintln!("[ERROR] [touchdictionary] [dictionary] Failed to fetch definitions for '{}': {}", query, e);
        }
    }

    match get_wikipedia_summary(query).await {
        Ok(wiki) => sections.wikipedia = Some(wiki),
        Err(e) => {
            eprintln!("[WARN] [touchdictionary] [wikipedia] Failed to fetch Wikipedia summary for '{}': {}", query, e);
        }
    }

    Ok(sections)
}

async fn get_dictionary_definitions(query: &str) -> Result<Vec<DefinitionSection>, String> {
    eprintln!("[INFO] [touchdictionary] [dictionary] Fetching definitions for '{}' from Free Dictionary API", query);
    
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", query);
    
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let json_text = response.text().await
                    .map_err(|e| format!("Failed to read dictionary response: {}", e))?;
                
                // Log the actual response for debugging
                eprintln!("[DEBUG] [touchdictionary] [dictionary] Raw response: {}", &json_text[0..json_text.len().min(200)]);
                
                match serde_json::from_str::<Vec<DictionaryApiResponse>>(&json_text) {
                    Ok(entries) => {
                        if entries.is_empty() {
                            return Ok(vec![]);
                        }
                        
                        let mut sections = Vec::new();
                        
                        for entry in entries {
                            let source = "Free Dictionary API".to_string();
                            let mut definitions = Vec::new();
                            
                            for meaning in entry.meanings {
                                for def in meaning.definitions {
                                    definitions.push(Definition {
                                        word: entry.word.clone(),
                                        part_of_speech: Some(meaning.part_of_speech.clone()),
                                        definition: def.definition,
                                        example: def.example,
                                    });
                                }
                            }
                            
                            sections.push(DefinitionSection {
                                source,
                                definitions,
                            });
                        }
                        
                        eprintln!("[INFO] [touchdictionary] [dictionary] Successfully fetched {} definitions for '{}'", 
                                 sections.iter().map(|s| s.definitions.len()).sum::<usize>(), query);
                        Ok(sections)
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to parse dictionary response: {}", e);
                        eprintln!("[ERROR] [touchdictionary] [dictionary] {}", err_msg);
                        Err(err_msg)
                    }
                }
            } else if response.status() == 404 {
                eprintln!("[INFO] [touchdictionary] [dictionary] No definitions found for '{}' (404)", query);
                Ok(vec![])
            } else {
                let err_msg = format!("Dictionary API returned status: {}", response.status());
                eprintln!("[ERROR] [touchdictionary] [dictionary] {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to connect to dictionary API: {}", e);
            eprintln!("[ERROR] [touchdictionary] [dictionary] {}", err_msg);
            Err(err_msg)
        }
    }
}

async fn get_wikipedia_summary(query: &str) -> Result<WikipediaSection, String> {
    eprintln!("[INFO] [touchdictionary] [wikipedia] Fetching summary for '{}' from Wikipedia API", query);
    
    let formatted_query = query.replace(" ", "_");
    let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", formatted_query);
    
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", "TouchDictionary/0.1.0 (https://github.com/yourusername/touchdictionary)")
        .send()
        .await 
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<WikipediaApiResponse>().await {
                    Ok(data) => {
                        if data.extract.is_empty() || data.extract.to_lowercase().contains("may refer to") {
                            eprintln!("[WARN] [touchdictionary] [wikipedia] Disambiguation page or no content for '{}'", query);
                            return Err("Disambiguation page or no content".to_string());
                        }
                        
                        eprintln!("[INFO] [touchdictionary] [wikipedia] Successfully fetched summary for '{}'", query);
                        
                        // Parse the summary into paragraphs for better formatting
                        let paragraphs: Vec<String> = data.extract
                            .split("\n")
                            .filter(|p| !p.trim().is_empty())
                            .map(|p| p.trim().to_string())
                            .collect();
                        
                        Ok(WikipediaSection {
                            title: data.title,
                            summary: data.extract,
                            paragraphs,
                            image_url: data.thumbnail.map(|t| t.source),
                            url: data.content_urls.desktop.page,
                        })
                    }
                    Err(e) => {
                        let err_msg = format!("Failed to parse Wikipedia response: {}", e);
                        eprintln!("[ERROR] [touchdictionary] [wikipedia] {}", err_msg);
                        Err(err_msg)
                    }
                }
            } else if response.status() == 404 {
                eprintln!("[INFO] [touchdictionary] [wikipedia] Page not found for '{}' (404)", query);
                Err("Page not found".to_string())
            } else {
                let err_msg = format!("Wikipedia API returned status: {}", response.status());
                eprintln!("[ERROR] [touchdictionary] [wikipedia] {}", err_msg);
                Err(err_msg)
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to connect to Wikipedia API: {}", e);
            eprintln!("[ERROR] [touchdictionary] [wikipedia] {}", err_msg);
            Err(err_msg)
        }
    }
}

async fn get_thesaurus_data(_query: &str) -> Result<ThesaurusSection, String> {
    // TODO: Implement real thesaurus API integration
    // For now, return empty data as we don't have a free thesaurus API integrated
    eprintln!("[INFO] [touchdictionary] [thesaurus] Thesaurus API not yet implemented, returning empty data");
    
    Ok(ThesaurusSection {
        synonyms: vec![],
        antonyms: vec![],
        related_terms: vec![],
    })
}

// API Response Structures

#[derive(Debug, Deserialize)]
struct DictionaryApiResponse {
    word: String,
    meanings: Vec<Meaning>,
    phonetics: Option<Vec<Phonetic>>,
}

#[derive(Debug, Deserialize)]
struct Phonetic {
    text: Option<String>,
    audio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Meaning {
    #[serde(default)]
    part_of_speech: String,
    definitions: Vec<DefinitionResponse>,
}

#[derive(Debug, Deserialize)]
struct DefinitionResponse {
    #[serde(rename = "definition")]
    definition: String,
    example: Option<String>,
    synonyms: Option<Vec<String>>,
    antonyms: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WikipediaApiResponse {
    title: String,
    extract: String,
    thumbnail: Option<Thumbnail>,
    content_urls: ContentUrls,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    source: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct ContentUrls {
    desktop: DesktopUrls,
}

#[derive(Debug, Deserialize)]
struct DesktopUrls {
    page: String,
}
