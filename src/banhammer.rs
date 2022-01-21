use crate::de::{RelayerInput, Token, TransactionError};
use ethereum_types::Address;
use serde::{
    de::{self, Error, Visitor},
    Deserialize, Deserializer,
};
use std::{
    collections::HashMap,
    fmt::{self},
    net::IpAddr,
    time::{Duration, Instant},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BanProgress {
    incorrect_nonce: u32,
    max_gas: u32,
    revert: Vec<String>,
    excessive_gas: u32,
}

#[derive(Debug, Default)]
pub struct BanList {
    pub clients: HashMap<IpAddr, UserClient>,
    pub tokens: HashMap<Token, UserToken>,
    pub froms: HashMap<Address, UserFrom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BanKind {
    IncorrectNonce,
    MaxGas,
    Revert(String),
    ExcessiveGas(u32),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct UserClient {
    tokens: Vec<Token>,
    froms: Vec<Address>,
    ban_progress: BanProgress,
    banned: Option<BanKind>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct UserFrom {
    clients: Vec<IpAddr>,
    tokens: Vec<Token>,
    ban_progress: BanProgress,
    banned: Option<BanKind>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct UserToken {
    clients: Vec<IpAddr>,
    froms: Vec<Address>,
    ban_progress: BanProgress,
    banned: Option<BanKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserKind {
    Client(UserClient),
    From(UserFrom),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BannedUserKind {
    Client(IpAddr),
    Token(Token),
    From(Address),
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("transaction as hex string")
        }

        fn visit_u8<E>(self, duration: u8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }

        fn visit_u16<E>(self, duration: u16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }

        fn visit_u64<E>(self, duration: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs(duration))
        }

        fn visit_i8<E>(self, duration: i8) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }

        fn visit_i16<E>(self, duration: i16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }

        fn visit_i32<E>(self, duration: i32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }

        fn visit_i64<E>(self, duration: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            self.visit_u64(duration as u64)
        }
    }

    deserializer.deserialize_u64(DurationVisitor)
}

struct User {
    client: IpAddr,
    from: Address,
    token: Option<Token>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_duration")]
    timeframe: Duration,
    incorrect_nonce_threshold: u32,
    max_gas_threshold: u32,
    revert_threshold: u32,
    // excessive_gas_threshold: u32, // TODO
    token_multiplier: u32,
}

#[derive(Debug)]
pub struct Banhammer {
    next_check: Duration,
    user_clients: HashMap<IpAddr, UserClient>,
    user_froms: HashMap<Address, UserFrom>,
    user_tokens: HashMap<Token, UserToken>,
    ban_list: BanList,
    config: Config,
}

fn check_error_ban(
    ban_progress: &mut BanProgress,
    config: &Config,
    token: Option<&Token>,
    maybe_error: Option<&TransactionError>,
) -> bool {
    let error = if let Some(error) = maybe_error {
        error
    } else {
        return false;
    };
    match error {
        TransactionError::ErrIncorrectNonce => {
            let threshold = {
                if token.is_some() {
                    config.incorrect_nonce_threshold * config.token_multiplier
                } else {
                    config.incorrect_nonce_threshold
                }
            };
            ban_progress.incorrect_nonce += 1;
            ban_progress.incorrect_nonce >= threshold
        }
        TransactionError::MaxGas => {
            let threshold = {
                if token.is_some() {
                    config.max_gas_threshold * config.token_multiplier
                } else {
                    config.max_gas_threshold
                }
            };
            ban_progress.max_gas += 1;
            ban_progress.max_gas >= threshold
        }
        TransactionError::Revert(msg) => {
            let threshold = {
                if token.is_some() {
                    config.revert_threshold * config.token_multiplier
                } else {
                    config.revert_threshold
                }
            };
            ban_progress.revert.push(msg.clone());
            ban_progress.max_gas >= threshold
        }
        TransactionError::Relayer(_) => false,
    }
}

impl Banhammer {
    pub fn new(config: Config) -> Self {
        Self {
            next_check: config.timeframe,
            user_clients: HashMap::default(),
            user_froms: HashMap::default(),
            user_tokens: HashMap::default(),
            ban_list: BanList::default(),
            config,
        }
    }

    pub fn tick(&mut self, time: Instant) {
        if time.elapsed() > self.next_check {
            for (_, client) in self.user_clients.iter_mut() {
                client.ban_progress.excessive_gas = 0;
                client.ban_progress.max_gas = 0;
                client.ban_progress.incorrect_nonce = 0;
                client.ban_progress.revert = Vec::new();
            }
            self.next_check += self.config.timeframe;
        }
    }

    fn ban_progression(
        &mut self,
        user: &User,
        token: Option<&Token>,
        maybe_error: Option<&TransactionError>,
        // gas: // TODO when added to relayer
    ) -> (bool, bool, bool) {
        // TODO excessive gas
        let is_client_banned = if !self.ban_list.clients.contains_key(&user.client) {
            let client_progress = &mut self
                .user_clients
                .get_mut(&user.client)
                .expect("`UserClient` missing.")
                .ban_progress;
            check_error_ban(client_progress, &self.config, token, maybe_error)
        } else {
            false
        };

        let is_from_banned = if !self.ban_list.froms.contains_key(&user.from) {
            let from_progress = &mut self
                .user_froms
                .get_mut(&user.from)
                .expect("`UserFrom' missing.")
                .ban_progress;
            check_error_ban(from_progress, &self.config, token, maybe_error)
        } else {
            false
        };

        let is_token_banned = {
            if let Some(token) = token {
                if !self.ban_list.tokens.contains_key(token) {
                    let token_progress = &mut self
                        .user_tokens
                        .get_mut(token)
                        .expect("'UserToken' missing.")
                        .ban_progress;
                    check_error_ban(token_progress, &self.config, Some(token), maybe_error)
                } else {
                    false
                }
            } else {
                false
            }
        };

        (is_client_banned, is_from_banned, is_token_banned)
    }

    fn associate_with_user_client(
        &mut self,
        client: IpAddr,
        from: Address,
        maybe_token: Option<Token>,
    ) {
        let user_client = self
            .user_clients
            .entry(client)
            .or_insert_with(UserClient::default);

        if !user_client.froms.contains(&from) {
            user_client.froms.push(from)
        }

        if let Some(token) = maybe_token {
            if !user_client.tokens.contains(&token) {
                user_client.tokens.push(token);
            }
        }
    }

    fn associate_with_user_from(
        &mut self,
        from: Address,
        client: IpAddr,
        maybe_token: Option<Token>,
    ) {
        let user_from = self
            .user_froms
            .entry(from)
            .or_insert_with(UserFrom::default);

        if !user_from.clients.contains(&client) {
            user_from.clients.push(client);
        }

        if let Some(token) = maybe_token {
            if !user_from.tokens.contains(&token) {
                user_from.tokens.push(token);
            }
        }
    }

    fn associate_with_user_token(&mut self, token: Token, client: IpAddr, from: Address) {
        let user_token = self
            .user_tokens
            .entry(token)
            .or_insert_with(UserToken::default);

        if !user_token.clients.contains(&client) {
            user_token.clients.push(client);
        }

        if !user_token.froms.contains(&from) {
            user_token.froms.push(from);
        }
    }

    pub fn read_input(&mut self, input: &RelayerInput) {
        let maybe_error = input.error.as_ref();
        let user = User {
            client: input.client,
            from: input.params.from,
            token: input.token.clone(),
        };

        self.associate_with_user_client(user.client, user.from, user.token.clone());
        self.associate_with_user_from(user.from, user.client, user.token.clone());
        if let Some(token) = user.token.clone() {
            self.associate_with_user_token(token, user.client, user.from);
        }

        let (is_client_banned, is_from_banned, is_token_banned) =
            self.ban_progression(&user, user.token.as_ref(), maybe_error);

        if is_client_banned {
            println!(
                "BANNED client: {}, reason: {:?}",
                user.client,
                maybe_error.expect("Error expected")
            );
            let user_client = self
                .user_clients
                .remove(&user.client)
                .expect("`UserClient` missing.");
            self.ban_list.clients.insert(user.client, user_client);
        }
        if is_from_banned {
            println!(
                "BANNED from: {:?}, reason: {:?}",
                user.from,
                maybe_error.expect("Error expected")
            );
            let user_from = self
                .user_froms
                .remove(&user.from)
                .expect("`UserFrom` missing.");
            self.ban_list.froms.insert(user.from, user_from);
        }
        if is_token_banned {
            let token = user.token.expect("'Token' missing.");
            println!(
                "BANNED token: {:?}, reason: {:?}",
                token,
                maybe_error.expect("Error expected")
            );
            let user_token = self
                .user_tokens
                .remove(&token)
                .expect("`UserToken` missing.");
            self.ban_list.tokens.insert(token, user_token);
        }
    }

    pub fn ban_list(&self) -> &BanList {
        &self.ban_list
    }
}
