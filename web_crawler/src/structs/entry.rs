use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[warn(dead_code)]
pub struct Entry {
    pub message_id: String,
    pub webhook: String,
    pub fullname: String,
    pub linkedin: String,
    pub message: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub session_cookie: String,
    pub user_id: String,
    pub entity_urn: String,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
    pub recruiter_session_cookie: String,
    pub regular: bool,
    pub recruiter: bool,
}



impl Entry {
    #[allow(dead_code)]
    pub fn new(
        message_id: String,
        webhook: String,
        fullname: String,
        linkedin: String,
        message: String,
        ip: String,
        username: String,
        password: String,
        user_agent: String,
        session_cookie: String,
        user_id: String,
        entity_urn: String,
        recruiter_stage_interested: String,
        recruiter_stage_not_interested: String,
        recruiter_session_cookie: String,
        regular: bool,
        recruiter: bool,
    ) -> Self {
        Entry {
            message_id,
            webhook,
            fullname,
            linkedin,
            message,
            ip,
            username,
            password,
            user_agent,
            session_cookie,
            user_id,
            entity_urn,
            recruiter_stage_interested,
            recruiter_stage_not_interested,
            recruiter_session_cookie,
            regular,
            recruiter,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRegular {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub regular: bool,
    pub session_cookie: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRecruiter {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub recruiter: bool,
    pub session_cookie: String,
    pub recruiter_session_cookie: String,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
}