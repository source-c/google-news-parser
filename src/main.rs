use colored::Colorize;
use exitfailure::ExitFailure;
use feed_rs::model::Entry;
use feed_rs::model::Feed;
use feed_rs::parser;
use reqwest::Url;
use std::env;

mod messages;
mod parse;
mod utils;

const MAX_ITEMS: usize = 10;

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let mut tasks = vec![];
    let args: Vec<String> = env::args().collect();
    let url: &str;

    if args.len() < 2 {
        messages::help();
        return Ok(());
    }

    if !args[1].contains("--url") {
        println!("{} {}", "Unknow options:".yellow(), args[1]);
        return Ok(());
    }

    if !args.get(2).is_some() {
        println!("{}", "url params not found".yellow());
        return Ok(());
    }

    url = &args[2];

    utils::system_log("Fetching", url);

    let fetch_url: Url = Url::parse(&*url)?;

    let text: String = reqwest::get(fetch_url).await?.text().await?;
    let feed: Feed = parser::parse(text.as_bytes()).unwrap();
    let entries: Vec<Entry> = feed.entries;

    utils::log("title", &feed.title.unwrap().content);

    let mut i: usize = 0;

    for item in entries {
        tasks.push(tokio::spawn(async move {
            let result = parse::get_article_link(&item.id, None).await;

            match result {
                Ok(link) => {
                    let id = &item.id;

                    utils::log("id", id);
                    utils::log("title", &item.title.unwrap().content);
                    utils::log("published", &item.published.unwrap().to_string());
                    utils::log("link", &link);
                    println!("");
                    return link;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    return "error".to_string();
                }
            }
        }));

        i += 1;

        if i >= MAX_ITEMS {
            break;
        }
    }

    println!("");

    let mut results = vec![];

    for task in tasks {
        results.push(task.await.unwrap());
    }

    // println!("results: {:?}", results);

    Ok(())
}
