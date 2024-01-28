use std::net::SocketAddr;

use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex, MutexGuard,
    },
};
use tokio_rustls::server::TlsStream;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{packet::Packet, PeerMap, CLIENTS};

pub struct Client {
    login: bool,
    contacts: Vec<Uuid>,
    receiver: Mutex<Receiver<Packet>>,
}

impl Client {
    pub fn new() -> (Self, Sender<Packet>) {
        let (sender, receiver) = channel::<Packet>(32);
        (
            Self {
                login: false,
                contacts: Vec::new(),
                receiver: Mutex::from(receiver),
            },
            sender,
        )
    }

    pub async fn handle_new_client(
        &mut self,
        #[cfg(feature = "wss")] raw_stream: TlsStream<TcpStream>,
        #[cfg(not(feature = "wss"))] raw_stream: TcpStream,
        addr: SocketAddr,
    ) {
        println!("Incoming TCP connection from: {}", addr);

        let ws_stream = tokio_tungstenite::accept_async(raw_stream)
            .await
            .expect("Error during the websocket handshake occurred");
        println!("WebSocket connection established: {}", addr);

        let (tx, rx) = unbounded();

        let (outgoing, incoming) = ws_stream.split();

        let broadcast_incoming = incoming.for_each_concurrent(None, |msg| {
            async {
                if let Ok(msg) = msg {
                    if msg.is_binary() {
                        let mut packet = Packet::new_with_data(&msg.clone().into_data());
                        println!("{} \n{}", packet.read_string(), packet.read_number());
                        if let Some(peer) = CLIENTS.lock().await.get(&addr) {
                            let mut packet = Packet::new();
                            packet
                                .write_string(&String::from("Hello from server"))
                                .write_bool(&true);
                            peer.send(packet).await.unwrap();
                        }
                    } else if msg.is_close() {
                        println!("aaaa");
                        self.receiver.lock().await.close();
                        todo!("Sends kill command to receiver");
                    }

                    /*for recp in broadcast_recipients {
                        if msg.is_binary() {
                            let mut packet = Packet::new_with_data(&msg.clone().into_data());
                            println!("{}", packet.read_string());
                            //recp.unbounded_send(msg.clone()).unwrap();
                        } else if msg.is_close() {
                        }
                    }*/
                }
            }
        });

        let handle_received_messages = async {
            loop {
                if let Some(message) = self.receiver.lock().await.recv().await {
                    tx.unbounded_send(Message::from(message.build())).unwrap();
                } else {
                    tokio::task::yield_now().await;
                }
            }
        };

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(
            broadcast_incoming,
            receive_from_others,
            handle_received_messages
        );
        future::select(
            future::select(broadcast_incoming, receive_from_others),
            handle_received_messages,
        )
        .await;

        println!("{} disconnected", &addr);
    }
}
