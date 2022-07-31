use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use actix_telepathy::prelude::*;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use actix_rt;

#[derive(RemoteMessage, Serialize, Deserialize)]
pub struct MyMessage {}

#[derive(RemoteActor)]
#[remote_messages(MyMessage)]
pub struct MyActor {
    pub state: usize
}

impl Actor for MyActor {
type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.register(ctx.address().recipient());
        // optional subscription to receive cluster news
        self.subscribe_system_async::<ClusterLog>(ctx);
    }
}

impl Handler<MyMessage> for MyActor {
    type Result = ();

    fn handle(&mut self,
              _msg: MyMessage,
              _ctx: &mut Self::Context)
    -> Self::Result {
        println!("MyMessage received");
    }
}

impl Handler<ClusterLog> for MyActor {
    type Result = ();

    fn handle(&mut self,
              msg: ClusterLog,
              _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ClusterLog::NewMember(_ip_addr,
                                  mut remote_addr) => {
                remote_addr.change_id(
                    Self::ACTOR_ID.to_string());
                remote_addr.do_send(MyMessage {});
            },
            ClusterLog::MemberLeft(_ip_addr) => ()
        }
    }
}
impl ClusterListener for MyActor {}
