mod api;
mod auth;


//비밀번호 해쉬 검증 test
#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn hash_verify_ok() {
        let h = hash_password("1234");
        assert!(verify_password(&h, "1234").unwrap());
    }

    #[test]
    fn hash_verify_fail() {
        let h = hash_password("1234");
        assert!(!verify_password(&h, "5678").unwrap());
    }
}
