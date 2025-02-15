use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapSearchRegular;
use crate::structs::error::CustomError;
use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use tracing::{error, info};

pub async fn scrap_regular_search(entry: EntryScrapSearchRegular) -> Result<(), CustomError> {
    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: None,
        headless: true,
    };

    let browser = start_browser(browser_info).await?;

    browser.page.goto_builder(&entry.url).goto().await?;
    wait(7, 10);

    const CANDIDATE_NUMBER: &str = "h2.pb2.t-black--light.t-14";
    let number_candidates = browser.page.query_selector(CANDIDATE_NUMBER).await?;
    match number_candidates {
        Some(number) => {
            let text = number.text_content().await?;
            let text = match text {
                Some(text) => text,
                None => String::from("1001"),
            };
            println!("{:?}", text);
            let result = if text.contains(',') || text.contains('K') {
                println!("more than 1k");
                1000
            } else {
                let numeric_text: String = text.chars().filter(|c| c.is_digit(10)).collect();
                println!("numeric {}", numeric_text);
                numeric_text.trim().parse::<i32>().unwrap_or(1002)
            };
            println!("{}", result);
            send_search_number(result, &entry.aisearch).await?
        }
        None => {
            println!("none");
            send_search_number(1003, &entry.aisearch).await?
        }
    };
    let search_container = browser
        .page
        .query_selector("div.search-results-container")
        .await?
        .unwrap();

    let pages_count = count_pages(search_container.inner_html().await?);
    println!("pages count: {}", pages_count);
    for i in 1..=pages_count {
        let mut url_list: Vec<String> = Vec::new();
        let page_number = format!("button[aria-label='Page {}']", i);
        let next_page = browser
            .page
            .query_selector(page_number.as_str())
            .await?
            .unwrap();

        next_page.click_builder().click().await?;

        wait(7, 10);

        let container = browser
            .page
            .query_selector("div.search-results-container")
            .await?
            .unwrap();
        scrap(container.inner_html().await?.as_str(), &mut url_list);
        wait(1, 2);
        send_urls(
            url_list,
            &entry.result_url,
            &entry.aisearch,
            &entry.url_list_id,
        )
        .await?;
        wait(3, 5);
    }

    //println!("url list: {:?}", url_list);

    wait(5, 12);

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?; // close browser
    Ok(())
}

fn scrap(html: &str, url_list: &mut Vec<String>) {
    let document = Html::parse_document(html);

    // Define a selector for the LinkedIn URLs
    let a_selector = Selector::parse("span.entity-result__title-text > a.app-aware-link").unwrap();

    // Extract LinkedIn URLs
    let linkedin_urls: Vec<String> = document
        .select(&a_selector)
        .filter_map(|el| el.value().attr("href"))
        .map(String::from)
        .collect();

    // Print the results
    for url in &linkedin_urls {
        url_list.push(url.to_string());
    }
}

fn count_pages(html: String) -> i32 {
    let document = Html::parse_document(html.as_str());

    // Selector for the last page button
    let last_page_selector =
        Selector::parse("li.artdeco-pagination__indicator--number:last-child button").unwrap();
    let last_page_elem = document.select(&last_page_selector).next().unwrap();
    let aria_label = last_page_elem.value().attr("aria-label").unwrap();
    let total_pages: i32 = aria_label
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap();

    total_pages
}

async fn send_urls(
    urls: Vec<String>,
    target_url: &str,
    ai_search: &str,
    url_list_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({ 
        "urls": urls,
        "ai_search": ai_search,
        "url_list_id": url_list_id     });

    let response = client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!("Send_urls/scrap_regular_search/Ok, {} was done", ai_search),
        Err(error) => {
            error!(error = ?error, "Send_urls/scrap_regular_search/Error {} returned error {}", ai_search, error);
        }
    }

    Ok(())
}
async fn send_search_number(number: i32, ai_search: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let send_json = json!({ 
        "number": number,
        "ai_search": ai_search});
    const TARGET_URL: &str = "
        https://overview.tribe.xyz/api/1.1/wf/tribe_search_number";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(TARGET_URL).json(&send_json).send().await;
    match response {
        Ok(_) => info!(
            "Send_search_number/scrap_recruiter_search/Ok, {} was done",
            ai_search
        ),
        Err(error) => {
            error!(error = ?error, "Send_search_number/scrap_recruiter_search/Error {} returned error {}", ai_search, error);
        }
    }
    //println!("{:?}", response.text().await?);

    Ok(())
}
