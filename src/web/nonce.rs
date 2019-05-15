use iron::prelude::*;
use iron::{BeforeMiddleware, typemap};
use textnonce::TextNonce;

pub struct Nonce;

impl typemap::Key for Nonce {
    type Value = TextNonce;
}

impl BeforeMiddleware for Nonce {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let nonce = TextNonce::sized_urlsafe(32).unwrap();
        req.extensions.insert::<Nonce>(nonce);
        Ok(())
    }
}
