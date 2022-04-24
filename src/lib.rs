use scraper::{ElementRef, Html, Selector};
use serde_json::{json, Value};
use worker::*;

mod utils;

async fn fetch_posts() -> Option<Vec<Value>> {
    let res = reqwest::get("https://news.ycombinator.com/news").await;

    if let Ok(res) = res {
        if let Ok(body) = res.text().await {
            let parsed_html = Html::parse_document(&body);

            let posts = parsed_html
                .select(&Selector::parse("tr.athing").unwrap())
                .map(|element| {
                    let url_element = element
                        .select(&Selector::parse(".title>a").unwrap())
                        .next()
                        .unwrap();

                    let title = url_element.text().collect::<Vec<_>>().join(" ");
                    let url = url_element.value().attr("href").unwrap_or("");

                    console_log!("{}", title);

                    let next_element = element.next_sibling().and_then(ElementRef::wrap).unwrap();

                    let points = next_element
                        .select(&Selector::parse(".score").unwrap())
                        .next()
                        .and_then(|e| Some(e.text().collect::<Vec<_>>().join(" ")));

                    let age = next_element
                        .select(&Selector::parse(".age").unwrap())
                        .next()
                        .unwrap()
                        .value()
                        .attr("title")
                        .unwrap_or("");

                    json!({
                        "title": title,
                        "url": url,
                        "points": points,
                        "age": age,
                    })
                })
                .collect::<Vec<_>>();

            return Some(posts);
        }
    }

    None
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::set_panic_hook();

    Router::new()
        .get_async("/", |_, _| async move {
            let posts = fetch_posts().await;

            match posts {
                Some(posts) => Response::from_json(&posts),
                None => Response::from_json(&json!({
                    "error": "failed to fetch posts"
                })),
            }
        })
        .run(req, env)
        .await
}
