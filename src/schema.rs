table! {
    use diesel::sql_types::*;
    use model::message::{ELogFormat, ELogLevel};

    messages (id) {
        id -> Int8,
        code -> Nullable<Varchar>,
        lang -> Varchar,
        level -> ELogLevel,
        format -> ELogFormat,
        title -> Varchar,
        content -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel::pg::types::sql_types::Uuid;
    use model::user::{EUserState, EUserResetPasswordState};

    users (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Varchar,
        password -> Bytea,
        state -> EUserState,
        reset_password_state -> EUserResetPasswordState,
        reset_password_token -> Nullable<Varchar>,
        reset_password_token_expires_at -> Nullable<Timestamp>,
        reset_password_token_granted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use model::user_email::{EUserEmailVerificationState, EUserEmailRole};

    user_emails (id) {
        id -> Int8,
        user_id -> Int8,
        email -> Nullable<Varchar>,
        role -> EUserEmailRole,
        verification_state -> EUserEmailVerificationState,
        verification_token -> Nullable<Varchar>,
        verification_token_expires_at -> Nullable<Timestamp>,
        verification_token_granted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(user_emails -> users (user_id));
allow_tables_to_appear_in_same_query!(users, user_emails);
