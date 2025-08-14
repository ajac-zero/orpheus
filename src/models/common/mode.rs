pub trait Mode {}

#[derive(Debug)]
pub struct Sync(pub reqwest::blocking::RequestBuilder);
impl Mode for Sync {}

#[derive(Debug)]
pub struct Async(pub reqwest::RequestBuilder);
impl Mode for Async {}
