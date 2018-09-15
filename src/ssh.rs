use std;

use futures;
use std::sync::Arc;
use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh_keys;
use thrussh_keys::*;

#[derive(Clone)]
pub struct Server {

}

impl server::Server for Server {
   type Handler = Self;
   fn new(&self) -> Self {
       self.clone()
   }
}

impl server::Handler for Server {
   type Error = std::io::Error;
   type FutureAuth = futures::Finished<(Self, server::Auth), Self::Error>;
   type FutureUnit = futures::Finished<(Self, server::Session), Self::Error>;
   type FutureBool = futures::Finished<(Self, server::Session, bool), Self::Error>;

   fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
       futures::finished((self, auth))
   }
   fn finished_bool(self, session: Session, b: bool) -> Self::FutureBool {
       futures::finished((self, session, b))
   }
   fn finished(self, session: Session) -> Self::FutureUnit {
       futures::finished((self, session))
   }

   fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
       futures::finished((self, server::Auth::Accept))
   }
   fn data(self, channel: ChannelId, data: &[u8], mut session: server::Session) -> Self::FutureUnit {
       println!("data on channel {:?}: {:?}", channel, std::str::from_utf8(data));
       session.data(channel, None, data);
       futures::finished((self, session))
   }
}
