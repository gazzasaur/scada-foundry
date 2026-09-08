use std::{collections::HashMap, time::Duration};

use rusty_mms::MmsConnection;
use rusty_mms_service::RustyMmsServiceServer;
use rusty_tpkt::{TcpTpktConnection, TcpTpktServer};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

#[async_trait::async_trait]
pub trait ConnectionReceiver {
    async fn connection_received(id: String, mms_connection: Box<dyn RustyMmsServiceServer>);
}

struct TcpTpktListenerState {
    socket_address: String,
    state_signal: UnboundedSender<()>,
}

pub struct TpktConnectionRegistration {
    pub socket_address: String,
    pub listener: UnboundedSender<TcpTpktConnection>,
}

pub struct TpktConnectionListener {
    tcp_servers: HashMap<String, UnboundedSender<()>>,
}

impl TpktConnectionListener {
    pub fn new() -> Self {
        return Self { tcp_servers: HashMap::new() };
    }

    pub async fn update_registrations(&mut self, registrations: Vec<TpktConnectionRegistration>) {
        self.tcp_servers.clear();

        for registration in registrations {
            let (state_signal_sender, state_signal_receiver) = unbounded_channel();
            self.tcp_servers.insert(registration.socket_address.clone(), state_signal_sender);
            tokio::task::spawn(tpkt_worker_task(state_signal_receiver, registration));
        }
    }
}

async fn tpkt_worker_task(mut state_signal: UnboundedReceiver<()>, registration: TpktConnectionRegistration) {
    loop {
        match try_tpkt_worker_task(&mut state_signal, &registration).await {
            Ok(()) => return,
            Err(_) => tokio::time::sleep(Duration::from_secs(2)).await,
        }
    }
}

async fn try_tpkt_worker_task(state_signal: &mut UnboundedReceiver<()>, registration: &TpktConnectionRegistration) -> Result<(), anyhow::Error> {
    let server = TcpTpktServer::listen(registration.socket_address.parse()?).await?;

    loop {
        let connection = select!(
            _ = state_signal.recv() => return Ok(()),
            connection = server.accept() => connection?,
        );
        if let Err(_) = registration.listener.send(connection) {
            return Ok(());
        }
    }
}
