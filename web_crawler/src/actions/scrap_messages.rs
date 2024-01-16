use crate::actions::wait::wait;
use crate::structs::browser::BrowserConfig;
use crate::structs::conversation::Conversation;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use crate::structs::message::Message;
use playwright::api::ElementHandle;
use playwright::api::Page;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

pub async fn scrap_message(
    conversation: &Conversation,
    page: &Page,
    focused_inbox: bool,
    browser: &BrowserConfig,
) -> Result<(), CustomError> {
    let conversation_select = match page
        .query_selector(format!("li[id='{}']", conversation.id).as_str())
        .await?
    {
        Some(conversation) => {
            conversation.hover_builder();
            wait(1, 7);
            conversation.click_builder().click().await?;
            conversation
        }
        None => {
            return Err(CustomError::ButtonNotFound(
                "Conversation select not found".to_string(),
            ))
        }
    }; // select the conversation

    wait(7, 19);

    let message_container = if let Some(container) = page
        .query_selector("ul[class='msg-s-message-list-content list-style-none full-width mbA']")
        .await?
    {
        container
    } else {
        return Ok(()); // if there is no message container return
    };
    // select the message container

    let owner_container = page
        .clone()
        .query_selector("div[class='msg-title-bar global-title-container shared-title-bar']")
        .await
        .unwrap()
        .unwrap();

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let mut full_text = String::new(); // Conversation text to push to AI
    let tuple = scrap_each_message(
        owner_container.inner_html().await?.as_str(),
        message_container.inner_html().await?.as_str(),
        &mut full_text,
    );
    let messages = tuple.1; //hashmap for storing all messages
    let conversation_owner_link = tuple.0;

    let candidate_of_sequence = scrap_profile(
        browser,
        conversation_owner_link.as_str(),
        &conversation.api_key,
    )
    .await?; // check if candidate is in sequence

    conversation_select.hover_builder();
    wait(1, 9);
    conversation_select.click_builder().click().await?;

    // checks if the message is new or was scraped before

    let full_name = FullName::split_name(conversation.candidate_name.as_str());

    let mut new_message = false;

    /////////////////loop for create/check message

    for message in messages.values() {
        let check_message = check_message_new_message(message, &full_name, conversation).await;
        if check_message == true && message.received == true {
            new_message = true;
            create_message(message, conversation).await;
            /*
            let autoreply = check_autoreply(message, &full_name, conversation).await;
            if autoreply {
                let responce = get_reply(full_text.clone(), conversation, full_name.clone()).await;
                let message = send_message(page, responce).await;
            }
            */
        }
    }
    // check if the message is new

    //////////////////////////////////loop for create/check message

    if new_message == true && candidate_of_sequence == Some(true) && conversation.enable_ai == true
    {
        let category = evaluate(full_text.as_str(), &conversation.api_key, full_name.clone()).await;
        match category {
            MessageCategory::Interested => {
                mark_star(&conversation_select, focused_inbox).await?;
                if conversation.unread == true {
                    conversation_select.hover_builder();
                    wait(1, 7);
                    conversation_select.click_builder().click().await?;
                    wait(3, 5);
                    mark_unread(&conversation_select, focused_inbox).await?;
                    //println!("Marked as unread/Interested");
                }
            }
            MessageCategory::NotInterested => {
                //println!("Nothing happened/NotInterested");
            }
            MessageCategory::NotFound => {
                //println!("Category NotFound");
            }
        }
    }

    if conversation.enable_ai == false && conversation.unread == true {
        mark_unread(&conversation_select, focused_inbox).await?;
    }

    if conversation.enable_ai == true && conversation.unread == true && new_message == false {
        conversation_select.hover_builder();
        wait(1, 3);
        conversation_select.click_builder().click().await?;
        mark_unread(&conversation_select, focused_inbox).await?;
    }

    Ok(())
}

async fn create_message(message: &Message, conversation: &Conversation) {
    // make an api call to bubble
    let full_name = FullName::split_name(message.sender.as_str());
    let _client = reqwest::Client::new();
    let _payload = json!({
            "message_text": message.message_text,
            "candidate_entity_urn": message.url_send_from,
            "received": message.received,
            "sender": message.sender,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
    });
    println!("payload :{}", _payload);
    let _res = _client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_receive")
        .json(&_payload)
        .send()
        .await
        .unwrap();
}

async fn check_message_new_message(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
) -> bool {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message.message_text,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "entity_urn": message.url_send_from,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_message")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    json_response["response"]["new_message"].as_bool().unwrap()
}

async fn check_autoreply(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
) -> bool {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message.message_text,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "entity_urn": message.url_send_from,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_autoreply")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    json_response["response"]["autoreply"].as_bool().unwrap()
}

async fn get_prompt() -> String {
    let client = reqwest::Client::new();
    let payload = json!({});
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_prompt")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    json_response["response"]["prompt"].to_string()
}

async fn send_message(page: &Page, message: String) -> Result<(), CustomError> {
    const INPUT_SELECTOR: &str =
        "div[class='msg-form__msg-content-container--scrollable scrollable relative']";
    let text_input = page.query_selector(INPUT_SELECTOR).await?;
    match text_input {
        Some(input) => {
            input.click_builder().click().await?;
            input.fill_builder(message.as_str()).fill().await?;
            println!("{}", message) // fill input for note;
        }
        None => (),
    }

    Ok(())
}

async fn get_reply(message: String, conversation: &Conversation, full_name: FullName) -> String {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message,
            "api_key": conversation.api_key,
            "first": full_name.first_name,
            "last": full_name.last_name,
    });
    let res = client
        .post("https://overview.tribe.xyz/version-test/api/1.1/wf/tribe_api_get_reply")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    json_response["response"]["reply"].to_string()
}
async fn mark_unread(
    conversation_element: &ElementHandle,
    focused_inbox: bool,
) -> Result<(), CustomError> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => (),
    }

    //find container for the buttons inside dropdown
    let inner_container = conversation_element
        .query_selector("div[class=artdeco-dropdown__content-inner]")
        .await?;
    let inner_container = match inner_container {
        Some(inner_container) => inner_container,
        None => return Ok(()),
    };

    //find mark unread button;

    let mark_unread_button = if focused_inbox == true {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(2)").await?
    } else {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(1)").await?
    };

    //click mark unread button
    match mark_unread_button {
        Some(button) => {
            wait(1, 3);
            button.click_builder().click().await?;
            Ok(())
        }
        None => Err(CustomError::ButtonNotFound(
            "Unread button in dropdown not found".to_string(),
        )),
    }
}

async fn mark_star(
    conversation_element: &ElementHandle,
    focused_inbox: bool,
) -> Result<(), CustomError> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => (),
    }

    //find container for the buttons inside dropdown
    let inner_container = conversation_element
        .query_selector("div[class=artdeco-dropdown__content-inner]")
        .await?;
    let inner_container = match inner_container {
        Some(inner_container) => inner_container,
        None => return Ok(()),
    };

    //find mark unread button;

    let mark_star_button = if focused_inbox == true {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(3)").await?
    } else {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(2)").await?
    };

    //click mark unread button
    match mark_star_button {
        Some(button) => {
            wait(1, 3);
            button.click_builder().click().await?;
            Ok(())
        }
        None => Err(CustomError::ButtonNotFound(
            "Unread button in dropdown not found".to_string(),
        )),
    }
}

async fn _move_other(conversation_element: &ElementHandle) -> Result<(), CustomError> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => (),
    }

    //find container for the buttons inside dropdown
    let inner_container = conversation_element
        .query_selector("div[class=artdeco-dropdown__content-inner]")
        .await?;
    if inner_container.is_none() {
        return Ok(());
    }

    //find move to other button
    let move_other_button = inner_container.unwrap().query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(1)").await?;

    //click move to other button
    match move_other_button {
        Some(button) => {
            wait(1, 3);
            button.click_builder().click().await?;
            Ok(())
        }
        None => Err(CustomError::ButtonNotFound(
            "Move to other in dropdown not found".to_string(),
        )),
    }
}

async fn scrap_profile(
    browser: &BrowserConfig,
    entity_urn: &str,
    api_key: &str,
) -> Result<Option<bool>, CustomError> {
    let page = browser.context.new_page().await?;
    let mut x = 0;
    if page.goto_builder(&entity_urn).goto().await.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = page.goto_builder(&entity_urn).goto().await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(1, 7);
                page.close(Some(false)).await?;
                return Ok(None); // if error means page is not loading
            }
            x += 1;
        }
    }

    wait(5, 12);

    let contact_info = page
        .query_selector("a#top-card-text-details-contact-info")
        .await?
        .unwrap();
    let url = contact_info.get_attribute("href").await?;

    let client = reqwest::Client::new();
    let payload = json!({
            "entity_urn": entity_urn,
            "linkedin": url,
    });
    let _res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/update_entity_urn")
        .json(&payload)
        .send()
        .await
        .unwrap();
    // check if candidate is a part of the sequence
    let client = reqwest::Client::new();
    let payload = json!({
            "api_key": api_key,
            "linkedin": url,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_sequence")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    let candidate_part_of_sequence = json_response["response"]["part_of_sequence"]
        .as_bool()
        .unwrap();

    page.close(Some(false)).await?;
    Ok(Some(candidate_part_of_sequence))
}

#[derive(Debug)]
enum MessageCategory {
    Interested,
    NotInterested,
    NotFound,
}

async fn evaluate(full_text: &str, api: &str, name: FullName) -> MessageCategory {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": full_text,
            "first": name.first_name,
            "last": name.last_name,
            "api_key": api
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/check_inmail")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce
    let category = json_response["response"]["category"].as_str();
    match category {
        Some("Interested") => MessageCategory::Interested,
        Some("Not interested") => MessageCategory::NotInterested,
        Some(_) => MessageCategory::NotFound,
        None => MessageCategory::NotFound,
    }
}

fn scrap_each_message(
    html: &str,
    message_container: &str,
    full_text: &mut String,
) -> (String, HashMap<String, Message>) {
    let owner_container_html = html;
    let owner_document = Html::parse_document(owner_container_html);
    let owner_selector = Selector::parse("a.app-aware-link.msg-thread__link-to-profile").unwrap();

    let conversation_owner_link: String;

    if let Some(conversation_owner) = owner_document.select(&owner_selector).next() {
        conversation_owner_link = conversation_owner
            .value()
            .attr("href")
            .and_then(|href| Some(href.to_owned()))
            .unwrap_or_else(|| String::new());
    } else {
        conversation_owner_link = String::new();
    }

    let mut messages: HashMap<String, Message> = HashMap::new(); // list of messages
                                                                 //let mut full_text = String::new(); // Conversation text to push to AI
                                                                 //let mut new_message = false; // if true and candidate_of_sequence is true evaluate conversation

    // Selectors for the message container
    let message_container_html = message_container;
    let document = Html::parse_document(&message_container_html); // parse html
    let sender_selector =
        Selector::parse(".msg-s-message-group__meta .msg-s-message-group__profile-link").unwrap();
    let timestamp_selector = Selector::parse(".msg-s-message-group__meta time").unwrap();
    let content_selector = Selector::parse(".msg-s-event__content p").unwrap();
    let url_send_from_selector =
        Selector::parse(".msg-s-event-listitem__link[tabindex=\"0\"]").unwrap();
    let url_send_to_selector = Selector::parse(".msg-s-message-group__meta a").unwrap();
    //

    // select the conversation
    // Iterate over the message container and create a message
    for ((((sender, timestamp), content), url_send_from), url_send_to) in document
        .select(&sender_selector)
        .zip(document.select(&timestamp_selector))
        .zip(document.select(&content_selector))
        .zip(document.select(&url_send_from_selector))
        .zip(document.select(&url_send_to_selector))
    {
        let sender = sender.text().collect::<String>().trim().to_owned();
        //let full_name = FullName::split_name(&sender);
        let timestamp = timestamp.text().collect::<String>().trim().to_owned();
        let message_text = content.text().collect::<String>().trim().to_owned();
        let url_send_from = url_send_from.value().attr("href").unwrap().to_owned();
        let url_send_to = url_send_to.value().attr("href").unwrap().to_owned();
        println!("url_send_from: {}", url_send_from);
        println!("conversation_owner_link: {}", conversation_owner_link);
        let received: bool = if conversation_owner_link == url_send_from {
            true
        } else {
            false
        };
        println!("received became: {}", received);
        let message = Message {
            sender,
            timestamp,
            message_text,
            url_send_from,
            url_send_to,
            received,
        };
        println!("received in message: {}", message.received);

        if received == true {
            full_text.push_str(format!("Candidate: {}\n", &message.message_text.clone()).as_str())
        } else {
            full_text.push_str(format!("Recruiter: {}\n", &message.message_text.clone()).as_str())
        }

        messages.insert(format!("message_{}", messages.len() + 1), message);
    } // scrap message container end

    (conversation_owner_link, messages)
}
