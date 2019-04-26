use openssl::pkey::Private;
use openssl::rsa::Rsa;

#[derive(Debug)]
pub struct State {
    pub rsa: Rsa<Private>,
}

impl State {
    pub fn new() -> State {
        let rsa = Rsa::generate(2048).unwrap();

        State { rsa }
    }
}
