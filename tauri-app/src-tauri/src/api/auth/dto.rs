use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignupReq {
    pub userid : String,
    pub password : String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignupRes {
    pub userid : String,
    pub password : String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginReq {
    pub userid : String,
    pub password : String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginRes {
    pub userid : String,
    pub password : String,
}