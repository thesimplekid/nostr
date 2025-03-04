// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{Contact as ContactSdk, UncheckedUrl};

use crate::PublicKey;

pub struct Contact {
    contact: ContactSdk,
}

impl Deref for Contact {
    type Target = ContactSdk;
    fn deref(&self) -> &Self::Target {
        &self.contact
    }
}

impl Contact {
    pub fn new(pk: Arc<PublicKey>, relay_url: Option<String>, alias: Option<String>) -> Self {
        let relay_url = relay_url.map(|relay_url| UncheckedUrl::from(&relay_url));
        Self {
            contact: ContactSdk::new(*pk.as_ref().deref(), relay_url, alias),
        }
    }

    pub fn alias(&self) -> Option<String> {
        self.contact.alias.clone()
    }

    pub fn public_key(&self) -> Arc<PublicKey> {
        Arc::new(self.contact.pk.into())
    }

    pub fn relay_url(&self) -> Option<String> {
        self.contact.relay_url.clone().map(|u| u.to_string())
    }
}
