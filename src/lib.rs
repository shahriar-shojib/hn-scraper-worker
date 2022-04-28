use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    title: String,
    url: String,
    points: String,
    age: String,
}

mod utils;

async fn fetch_posts() -> core::result::Result<Vec<Post>, Box<dyn std::error::Error>> {
    let body = reqwest::get("https://news.ycombinator.com/news")
        .await?
        .text()
        .await?;

    let parsed_html = Html::parse_document(&body);

    let selector = Selector::parse("tr.athing").unwrap();

    let posts = parsed_html
        .select(&selector)
        .filter_map(|element| {
            let selector = Selector::parse(".title>a").unwrap();
            let url_element = element.select(&selector).next()?;

            let title = url_element.text().collect::<Vec<_>>().join(" ");
            let url = url_element.value().attr("href").unwrap_or("");

            let post = element
                .next_sibling()
                .and_then(ElementRef::wrap)
                .and_then(|element| {
                    let points = element
                        .select(&Selector::parse(".score").unwrap())
                        .next()?
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ");

                    let age = element
                        .select(&Selector::parse(".age").unwrap())
                        .next()?
                        .value()
                        .attr("title")?;

                    Some(Post {
                        title,
                        url: url.to_string(),
                        points,
                        age: age.to_string(),
                    })
                });

            post
        })
        .collect();

    Ok(posts)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_, _| async move {
            let posts = fetch_posts().await;

            match posts {
                Ok(posts) => Response::from_json(&posts),
                Err(_) => Response::from_json(&json!({
                    "error": "Something Went Wrong when fetching the posts"
                })),
            }
        })
        .run(req, env)
        .await
}
