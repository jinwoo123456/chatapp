use argon2::{
    Argon2,
    PasswordHasher,
    PasswordVerifier,
    password_hash::{PasswordHash, SaltString, rand_core::OsRng},
};

#[test]
fn hash_and_verify_success() {
    // given
    let password = "MyS3cret!";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // when: 해시 생성 (PHC 문자열)
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing should succeed")
        .to_string();

    // then: 동일한 비밀번호는 검증 성공
    let parsed_hash = PasswordHash::new(&hash).expect("valid phc string");
    assert!(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok());
}

#[test]
fn hash_and_verify_failure_with_wrong_password() {
    // given
    let real_password = "MyS3cret!";
    let wrong_password = "NotTheSame";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // when
    let hash = argon2
        .hash_password(real_password.as_bytes(), &salt)
        .expect("hashing should succeed")
        .to_string();

    // then: 다른 비밀번호는 검증 실패
    let parsed_hash = PasswordHash::new(&hash).expect("valid phc string");
    assert!(argon2.verify_password(wrong_password.as_bytes(), &parsed_hash).is_err());
}
