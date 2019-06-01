use std::fmt;

use chrono::{NaiveDateTime, Utc, TimeZone};
use diesel::{Associations, Identifiable, Queryable, debug_query, prelude::*};
use diesel::pg::{Pg, PgConnection};

pub use model::user_email_activation_state::*;
pub use model::user_email_role::*;
pub use schema::user_emails;

use logger::Logger;
use model::voucher::{ActivationClaims, Claims, VoucherData};
use model::user::User;
use util::generate_random_hash;

const ACTIVATION_HASH_LENGTH: i32 = 128;
const ACTIVATION_HASH_SOURCE: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";

/// NewUserEmail
#[derive(Debug)]
pub struct NewUserEmail {
    pub user_id: i64,
    pub email: String,
    pub role: UserEmailRole,
    pub activation_state: UserEmailActivationState,
}

impl Default for NewUserEmail {
    fn default() -> Self {
        Self {
            user_id: -1,           // validation error
            email: "".to_string(), // validation error
            role: UserEmailRole::General,

            activation_state: UserEmailActivationState::Pending,
        }
    }
}

impl From<User> for NewUserEmail {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            email: user.email,
            role: UserEmailRole::Primary,

            ..Default::default()
        }
    }
}

/// UserEmail
#[derive(Associations, Debug, Identifiable, Queryable)]
#[belongs_to(User)]
#[table_name = "user_emails"]
pub struct UserEmail {
    pub id: i64,
    pub user_id: i64,
    pub email: Option<String>,
    pub role: UserEmailRole,
    pub activation_state: UserEmailActivationState,
    pub activation_token: Option<String>,
    pub activation_token_expires_at: Option<NaiveDateTime>,
    pub activation_token_granted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl fmt::Display for UserEmail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<UserEmail {role}>", role = &self.role)
    }
}

impl UserEmail {
    /// Save a new user_email into user_emails.
    ///
    /// # Note
    ///
    /// `activation_state` is assigned always as pending. And following
    /// columns keep still remaining as NULL until granting voucher later.
    ///
    /// * activation_token
    /// * activation_token_expires_at
    /// * activation_token_granted_at
    pub fn insert(
        user_email: &NewUserEmail,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<Self>
    {
        let q = diesel::insert_into(user_emails::table).values((
            user_emails::user_id.eq(&user_email.user_id),
            Some(user_emails::email.eq(&user_email.email)),
            user_emails::role.eq(UserEmailRole::Primary),
            user_emails::activation_state.eq(UserEmailActivationState::Pending),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(u) => Some(u),
        }
    }

    fn generate_activation_voucher(
        &self,
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> VoucherData
    {
        ActivationClaims::encode(value, issuer, key_id, secret)
    }

    pub fn grant_activation_voucher(
        &self,
        issuer: &str,
        key_id: &str,
        secret: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Option<VoucherData>
    {
        // TODO: check duplication
        let activation_token = generate_random_hash(
            ACTIVATION_HASH_SOURCE,
            ACTIVATION_HASH_LENGTH,
        );

        let voucher_data = self.generate_activation_voucher(
            activation_token.to_owned(),
            &issuer,
            &key_id,
            &secret,
        );

        // FIXME: save activation_token_xxxx fields
        let q = diesel::update(self).set((
            user_emails::activation_token.eq(activation_token),
            // from VoucherData
            user_emails::activation_token_expires_at
                .eq(Utc.timestamp(voucher_data.expires_at, 0).naive_utc()),
        ));

        info!(logger, "{}", debug_query::<Pg, _>(&q).to_string());

        match q.get_result::<Self>(conn) {
            Err(e) => {
                error!(logger, "err: {}", e);
                None
            },
            Ok(_) => Some(voucher_data),
        }
    }
}
