use argon2::{password_hash, Argon2};
use itertools::Itertools;
use password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
type Result<T, E = password_hash::Error> = std::result::Result<T, E>;

pub fn hash_password<'a>(password: &'a str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(format!(
        "{}.{}",
        password_hash.hash.unwrap(),
        password_hash.salt.unwrap()
    ))
}

pub fn verify_password(hash_password: &str, password: &str) -> Result<()> {
    let (hash, salt): (&str, &str) = hash_password.split('.').collect_tuple().unwrap();
    let has_password = format!("$argon2id$v=19$m=19456,t=2,p=1${salt}${hash}");

    let password_hash = PasswordHash::new(has_password.as_str())?;
    let algs: &[&dyn PasswordVerifier] = &[&Argon2::default()];
    password_hash.verify_password(algs, password)
}
