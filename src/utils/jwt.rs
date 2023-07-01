use hmac::{Hmac, Mac};
use jwt::{
    AlgorithmType, Error, FromBase64, Header, SignWithKey, Token, Unverified, Verified,
    VerifyWithKey,
};
use sha2::Sha256;

fn secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes()
}

#[allow(dead_code)]
pub fn encode<C>(claims: C) -> Result<String, Error>
where
    C: serde::Serialize,
{
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret())?;
    let header = Header {
        algorithm: AlgorithmType::Hs256,
        ..Default::default()
    };

    Token::new(header, claims)
        .sign_with_key(&key)
        .map(|t| t.as_str().to_owned())
}

#[allow(dead_code)]
pub fn verified_decode<T>(token_str: &str) -> Result<Token<Header, T, Verified>, Error>
where
    T: FromBase64,
{
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret())?;

    VerifyWithKey::verify_with_key(token_str, &key)
}

pub fn decode<T>(token_str: &str) -> Result<Token<Header, T, Unverified>, Error>
where
    T: FromBase64,
{
    Token::parse_unverified(token_str)
}
