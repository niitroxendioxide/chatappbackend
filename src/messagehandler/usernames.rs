use std::{collections::HashMap, sync::Mutex};

use super::shared::USER_MAP;

pub fn get_user_map() {
    USER_MAP.get_or_init(|| {
        Mutex::new(HashMap::from([]))
    });
}