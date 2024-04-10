use std::sync::{Arc, Mutex};

use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::game::digsites::DigSite;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    iid: String,
    pub user: DiscordUser,
}

impl Connection {
    pub fn new(qs: ConnectionQueryString, user: DiscordUser) -> Self {
        Self { iid: qs.iid, user }
    }

    pub fn room(&self) -> String {
        self.iid.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl DiscordUser {
    pub fn name(&self) -> String {
        return self.global_name.as_ref().unwrap_or(&self.username).clone();
    }
}

// Users will have a Connection availiable to access in the handlers
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionQueryString {
    iid: String,
    aut: String,
}

impl ConnectionQueryString {
    pub fn bearer_token(&self) -> String {
        String::from("Bearer ") + &self.aut.to_string()
    }
}

pub struct Parties(Arc<DashMap<String, Arc<Party>>>);

impl Parties {
    pub fn new() -> Self {
        Parties(Arc::new(DashMap::new()))
    }

    pub fn get(&self, id: String) -> Option<Arc<Party>> {
        let parties = Arc::clone(&self.0);
        parties.get(&id).map(|r| Arc::clone(&r))
    }

    pub fn add_party(&self, p: Party) {
        let parties = Arc::clone(&self.0);
        parties.insert(p.id.clone(), Arc::new(p));
    }

    pub fn ensure_party(&self, id: String, uid: String) {
        let parties = Arc::clone(&self.0);
        let party = parties.entry(id.clone());
        party
            .or_insert(Arc::new(Party::from(id)))
            .players
            .insert(uid);
    }

    /// Returns true if the party was deleted bc of no players
    pub fn on_player_left(&self, id: String, uid: String) -> bool {
        let mut will_delete = false;

        let parties = Arc::clone(&self.0);

        if let Some(party) = parties.get(&id) {
            party.players.remove(&uid);
            if party.players.is_empty() {
                will_delete = true;
            }
        };

        if will_delete {
            parties.remove(&id);
            info!("Party {} deleted", id);
        }

        will_delete
    }
}

impl Default for Parties {
    fn default() -> Self {
        Parties::new()
    }
}

#[derive(Debug)]
pub struct Party {
    pub id: String,
    pub players: DashSet<String>,
    pub game: Arc<Mutex<Option<DigSite>>>,
}

impl From<String> for Party {
    fn from(value: String) -> Self {
        Party {
            id: value,
            players: DashSet::new(),
            game: Arc::new(Mutex::new(None)),
        }
    }
}
