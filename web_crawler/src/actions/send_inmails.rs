
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use scraper::{Html, Selector};

use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendInmail;
use crate::structs::error::CustomError;
use playwright::api::Page;

use std::thread::sleep;
use undetected_chromedriver::chrome;

use fantoccini::Locator;



pub async fn send_inmails(entry: EntrySendInmail) -> Result<(), CustomError> {
    
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );

    let subject = entry.subject.clone();

    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: Some(entry.recruiter_session_cookie),
        headless: false,
    };

    let browser = start_browser(browser_info).await?;

    let search_input = browser
        .page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;
    wait(3, 15); // random delay
                 //focus on search input and fill it with text
    match search_input {
        Some(search_input) => {
            search_input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            search_input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            search_input
                .fill_builder(&candidate.fullname)
                .fill()
                .await?; // fill search input with text
            wait(1, 5); // random delay
            search_input.press_builder("Enter").press().await?; // press Enter
            wait(2, 6); // random delay
        }
        None => {
            wait(1, 5); // random delay
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    // go to candidate page
    let mut go_to = browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser
                .page
                .goto_builder(candidate.linkedin.as_str())
                .goto()
                .await;
            if build.is_ok() {
                go_to = build;
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Inmail_regular".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            println!("retrying to load page")
        }
        wait(1, 3);
    }

    wait(3, 15); // random delay
                 //check if View in recruiter is present
                 /*
                   let view_button = browser
                   .page
                   .query_selector("button[class='artdeco-button artdeco-button--2 artdeco-button--secondary ember-view pvs-profile-actions__action']")
                   .await?;

                   match view_button {
                    Some(view_button) => {
                       view_button.hover_builder(); // hover on search input
                        wait(1, 4); // random delay
                        view_button.click_builder().click().await?; // click on search input
                        wait(2, 5); // random delay
                    }
                    None => {
                        wait(1, 5); // random delay
                        browser.page.close(Some(false)).await?;
                        browser.browser.close().await?; // close browser
                        return Err(CustomError::ButtonNotFound("View in recruiter button is not visible".to_string()));
                    } // if search input is not found, means page was not loaded and sessuion cookie is not valid
                 };
                 */

    let entity_urn = find_entity_run(&browser.page).await?;
    println!("entity_urn: {:?}", entity_urn);

    let url = format!(
        "https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER",
        entity_urn
    );
    // go to candidate page
    let mut _go_to = browser.page.goto_builder(url.as_str()).goto().await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.page.goto_builder(url.as_str()).goto().await;
            if build.is_ok() {
                _go_to = build; //_go_to never read, is there some point for it?
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Candidate Recruiter page is not loading/Inmail".to_string(),
                )); // if error means page is not loading
            }            x += 1;
            println!("retrying to load page")
        }
    }

    let nav_bar = browser
        .page
        .query_selector("div[class='global-nav__right']")
        .await?;

    match &nav_bar {
        Some(_) => (),
        None => {
            wait(1, 3);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::RecruiterSessionCookieExpired); // if error when session cookie expired
        }
    }

    wait(6, 16);
    let profile_block = browser
.page
.query_selector("div[class='topcard-condensed__content-top topcard-condensed__content-top--profile-size-7']")
.await?;

    match &profile_block {
        Some(_) => (),
        None => {
            wait(1, 3);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ProfileNotFound);
        }
    }

    wait(2, 4);

    let send_button = browser
.page
.query_selector("button[class='artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--2 artdeco-button--tertiary ember-view profile-item-actions__item']")
.await?;

    match send_button {
        Some(button) => {
            button.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            button.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Page".to_string(),
            ));
        }
    };

////////////////////////////////////////////////////////////////////////////////////////////////////////////

if !entry.file_name.is_empty() {
    add_file().await.unwrap();

    let file_button = browser
        .page
        .query_selector("li-icon[type='paperclip-icon']")
        .await?
        .unwrap();

    
    //let url = "https://8391d9e4954c178fdc23d54008f421f8.cdn.bubble.io/f1685347944533x739141849066435200/Foto_Nemi.png";

    // Fetch content from the URL
    //let response = reqwest::get(url).await?;

    // Get the response content as bytes



   let current_dir = env::current_dir().unwrap();
   let file_name = entry.file_name; // Change to your desired file name
   let file_absolute_path = current_dir.join(file_name);

   // Download the file to the absolute path
   let download_url = &entry.file_url; // Replace with your file URL
   download_file(download_url, file_absolute_path.to_str().unwrap()).await.unwrap();

   // Print out the absolute path for verification
   println!("File saved to: {}", file_absolute_path.to_str().unwrap());

   //file_button.click_builder().click().await?; // click on search input

   





    //et screenshot = browser.page.screenshot_builder().path(current_dir.join("screeb")).screenshot().await?;
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
wait(10000, 20000);
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////

}

    let subject_input = browser
        .page
        .query_selector("input[class='compose-subject__input']")
        .await?;

    match subject_input {
        Some(input) => {
            input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            input.fill_builder(subject.as_str()).fill().await?; // fill input for note;
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Subject input in recruiter is not visible".to_string(),
            ));
        }
    };
    wait(2, 5);

    let text_input = browser
        .page
        .query_selector("textarea[class='compose-textarea__textarea']")
        .await?;

    match text_input {
        Some(input) => {
            input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            input
                .fill_builder(candidate.message.as_str())
                .fill()
                .await?; // fill input for note;
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Subject input in recruiter is not visible".to_string(),
            ));
        }
    };

    let send_button = browser
        .page
        .query_selector("button[class='msg-cmpt__button--small compose-actions__submit-button']")
        .await?;

    match send_button {
        Some(button) => {
            button.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            button.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Text".to_string(),
            ));
        }
    };
    
    wait(2, 4);
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}

async fn find_entity_run(page: &Page) -> Result<String, playwright::Error> {
    let link_selector = Selector::parse("a").unwrap();
    let document = scraper::Html::parse_document(&page.content().await?);
    let mut entity_urn = String::new();

    for link in document.select(&link_selector) {
        let href = link.value().attr("href").unwrap_or_default();
        if href.contains("profileUrn=") {
            let parts: Vec<&str> = href
                .split("?profileUrn=urn%3Ali%3Afsd_profile%3A")
                .collect();
            if parts.len() > 1 {
                entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                if entity_urn.is_empty() {
                    let parts: Vec<&str> = href
                        .split("?profileUrn=urn%3Ali%3Afs_normalized_profile%3A")
                        .collect();
                    if parts.len() > 1 {
                        entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                    }
                }
            }
            if !entity_urn.is_empty() {
                break;
            }
        }
    }

    if entity_urn.is_empty() {
        entity_urn = print_elements_with_datalet_in_id(document.html().as_str());
    }
    Ok(entity_urn)
}

fn print_elements_with_datalet_in_id(html: &str) -> String {
    // Parse the document
    let document = Html::parse_document(html);

    // Create a Selector for elements with an 'id' attribute
    let selector = Selector::parse("[id]").unwrap();

    let mut right_id = String::new();
    // Iterate over elements matching the selector
    for element in document.select(&selector) {
        if let Some(id_attr) = element.value().attr("id") {
            if id_attr.contains("datalet")
                && element
                    .html()
                    .contains("/voyager/api/identity/dash/profile")
            {
                let element_html: String = element.html();
                match element_html.find("bpr-guid-") {
                    Some(start) => match element_html[start..].find("\"") {
                        Some(end) => {
                            let end = end + start;
                            right_id = format!("[id={}]", &element_html[start..end]);
                        }
                        None => println!("Could not find end quote"),
                    },
                    None => println!("Could not find 'bpr-guid-'"),
                }
            }
        }
    }

    let entity_id_selector = Selector::parse(&right_id).unwrap();
    let mut entity_urn = String::new();
    for element in document.select(&entity_id_selector) {
        let text = element.html();
        let text_str = text.as_str();

        if let Some(start) = text_str.find("\"*elements\":[\"urn:li:fsd_profile:") {
            let start = start + "\"*elements\":[\"urn:li:fsd_profile:".len();
            if let Some(end) = text_str[start..].find("\"") {
                let end = start + end;
                entity_urn = text_str[start..end].to_string();
            }
        }
    }

    entity_urn
}

async fn download_file(url: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;

    // Ensure the response is successful before proceeding
    if !response.status().is_success() {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to download. HTTP status: {}", response.status()))));
    }

    let mut file = File::create(destination)?;
    let content = response.bytes().await?;
    file.write_all(&content)?;

    Ok(())
}

async fn add_file() -> Result<(), Box<dyn std::error::Error>> {
    let driver = chrome().await?;

    driver.goto("https://www.linkedin.com/talent/profile/ACoAACPdzLMBW1KTn4sQh5OtZu3w64DCIhL2egk").await?;

    let title = driver.title().await?;
    println!("Title: {}", title);

    sleep(std::time::Duration::from_secs(5));

    //let input = driver.find(Locator::Css(".dz-default.dz-message")).await?;
    //input.click().await?;
    let file_input_locator = Locator::Css("input[type='file']"); // adjust this selector as needed
    let file_input = driver.find(file_input_locator).await?;
    file_input.send_keys("/home/mikhail/linkedin/web_crawler/Dummy%20file%20for%20sending.pdf").await?;
    Ok(())
}