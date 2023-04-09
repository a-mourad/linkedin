
use crate::actions::wait::wait;
use crate::actions::start_browser::start_browser;
use crate::structs::entry::Entry;
use crate::structs::conversation::Conversation;
use scraper::{Html, Selector};
use std::{collections::HashMap};

use crate::actions::scrap_messages::scrap_message;

pub async fn scrap(entry: Entry) -> Result<(), playwright::Error> {
    
    let api_key = entry.user_id.clone();
    
    let browser = start_browser(entry).await?;
    println!("Wait starts");
    wait(3,7);
    println!("Wait ends");
    let messaging_button = browser.page
        .query_selector("a.global-nav__primary-link:has-text('Messaging')")
        .await
        .unwrap();
    match messaging_button {
        Some(messaging_button) => {
            println!("messaging button is ok");
            messaging_button.click_builder().click().await.unwrap();
        }
        None => {
            println!("messaging button is not ok");
        }
    }

    wait(5, 10);
    

    let conversation_list = match browser.page.query_selector("ul[class='list-style-none msg-conversations-container__conversations-list']").await?{
        Some(conversation_list) => conversation_list,
        None => {
            println!("Conversation list is None");
            browser.browser.close().await?;
            return Err(playwright::Error::ObjectNotFound);
        }
    };

    let document = Html::parse_document(&conversation_list.inner_html().await.unwrap()); // parse html
                                                                                         //selectors (which part of html to parse to get the specific data)
    let conversation_selector = Selector::parse("li.msg-conversation-listitem").unwrap();
    

    let participant_name_selector =
        Selector::parse("h3.msg-conversation-listitem__participant-names").unwrap();
    let timestamp_selector = Selector::parse("time.msg-conversation-listitem__time-stamp").unwrap();
    let message_snippet_selector =
        Selector::parse("p.msg-conversation-card__message-snippet").unwrap();
    let thread_url_selector = Selector::parse("a.msg-conversation-listitem__link").unwrap();
    let unread_selector = Selector::parse(".msg-conversation-card__unread-count").unwrap();

  

    let mut conversations = HashMap::new(); // hashmap to store conversations

    for convo in document.select(&conversation_selector) {
        let id = convo.value().attr("id").unwrap().to_string();
        //println!("id: {}", id) ;
        //once conversation thread url is not found, break the loop, that means it was the last convo
        if convo.select(&thread_url_selector).next() == None {
            break;
        }
        let thread_url = convo
            .select(&thread_url_selector)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .to_string();

        //println!("convo: {:?}", thread_url);
        let candidate_name = convo
            .select(&participant_name_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        //println!("convo: {:?}", candidate_name);
        let timestamp = convo
            .select(&timestamp_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        //println!("convo: {:?}", timestamp);
        let message_snippet = convo
            .select(&message_snippet_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();
        //println!("convo: {:?}", message_snippet);

        let unread = match convo.select(&unread_selector).next() {
            Some(_) => true,
            None => false,
        };
        //println!("unread: {:?}", unread);

        let conversation = Conversation {
            id: id.clone(),
            thread_url: thread_url,
            candidate_name,
            timestamp,
            message_snippet,
            unread: unread,
            api_key: api_key.clone(),
        };

        conversations.insert(id, conversation);
    }

    //println!("{:#?}", conversations);
    println!("{:?}", conversations.len());

    for conversation in conversations.values() {
        scrap_message(conversation, &browser.page).await?;
        
    }
    //"li[class=search-global-typeahead__input]"
    browser.browser.close().await?;
    Ok(())
}
