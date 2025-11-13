use std::env;

use crate::lookup;

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let query = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        println!("TouchDictionary - Modern Dictionary Lookup");
        println!("Usage: touchdictionary <word>");
        println!("       touchdictionary --selection");
        return Ok(());
    };

    if query == "--selection" {
        match crate::clipboard::get_selected_text() {
            Some(text) => {
                println!("Looking up selected text: '{}'", text);
                match lookup::lookup(&text).await {
                    Ok(result) => print_lookup_result(&result),
                    Err(e) => {
                        eprintln!("[ERROR] [touchdictionary] [lookup] Failed to lookup '{}': {}", text, e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                eprintln!("[ERROR] [touchdictionary] [clipboard] No text selected or could not access clipboard");
                std::process::exit(1);
            }
        }
    } else {
        println!("Looking up: '{}'", query);
        match lookup::lookup(&query).await {
            Ok(result) => print_lookup_result(&result),
            Err(e) => {
                eprintln!("[ERROR] [touchdictionary] [lookup] Failed to lookup '{}': {}", query, e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_lookup_result(result: &lookup::LookupResult) {
    println!("\n=== TouchDictionary Result ===");
    println!("Query: {}", result.query);
    println!("Content Type: {:?}", result.content_type);
    println!();

    // Print definitions
    if let Some(definitions) = &result.sections.definitions {
        for section in definitions {
            println!("[DEFINITION] Source: {}", section.source);
            for def in &section.definitions {
                // Only show part of speech if it's not empty
                if let Some(pos) = &def.part_of_speech {
                    if !pos.is_empty() {
                        println!("  - ({}): {}", pos, def.definition);
                    } else {
                        println!("  - {}", def.definition);
                    }
                } else {
                    println!("  - {}", def.definition);
                }
                if let Some(example) = &def.example {
                    println!("    Example: {}", example);
                }
            }
            println!();
        }
    }

    // Print Wikipedia section
    if let Some(wiki) = &result.sections.wikipedia {
        println!("[WIKIPEDIA] {}", wiki.title);
        println!("{}", wiki.summary);
        if !wiki.url.is_empty() {
            println!("URL: {}", wiki.url);
        }
        println!();
    }

    // Print thesaurus
    if let Some(thesaurus) = &result.sections.thesaurus {
        println!("[THESAURUS]");
        if !thesaurus.synonyms.is_empty() {
            println!("  Synonyms: {}", thesaurus.synonyms.join(", "));
        }
        if !thesaurus.antonyms.is_empty() {
            println!("  Antonyms: {}", thesaurus.antonyms.join(", "));
        }
        if !thesaurus.related_terms.is_empty() {
            println!("  Related: {}", thesaurus.related_terms.join(", "));
        }
        println!();
    }

    println!("========================");
}
