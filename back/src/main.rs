use std::thread;
use std::time::Duration;
use std::{collections::HashMap, env, io, net::SocketAddr, sync::Arc};

use client::Client;
use get_if_addrs::get_if_addrs;
use lazy_static::lazy_static;
#[cfg(feature = "wss")]
use rustls::pki_types::CertificateDer;
#[cfg(feature = "wss")]
use rustls::pki_types::PrivateKeyDer;
#[cfg(feature = "wss")]
use std::path::Path;
use tokio::sync::mpsc::Sender;

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use packet::Packet;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
#[cfg(feature = "wss")]
use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tokio_tungstenite::tungstenite::protocol::Message;

pub mod client;
pub mod packet;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

const BUFFER_SIZE: usize = 2048;

/*async fn handle_connection(
    peer_map: PeerMap,
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
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let peers = peer_map.lock().unwrap();

        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);
        if msg.is_binary() {
            let mut packet = Packet::new_with_data(&msg.clone().into_data());
            println!("{} \n{}", packet.read_string(), packet.read_number());
            if let Some(peer) = peers.get(&addr) {
                peer.unbounded_send(Message::from(
                    Packet::new()
                        .write_string(&String::from("Hello from server"))
                        .write_bool(&true)
                        .build(),
                ))
                .unwrap();
            }
        }

        /*for recp in broadcast_recipients {
            if msg.is_binary() {
                let mut packet = Packet::new_with_data(&msg.clone().into_data());
                println!("{}", packet.read_string());
                //recp.unbounded_send(msg.clone()).unwrap();
            } else if msg.is_close() {
            }
        }*/

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}*/

#[cfg(feature = "wss")]
fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    use rustls_pemfile::certs;
    use std::{fs::File, io::BufReader};

    certs(&mut BufReader::new(File::open(path)?)).collect()
}

#[cfg(feature = "wss")]
fn load_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    use rustls_pemfile::pkcs8_private_keys;
    use std::{fs::File, io::BufReader};

    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}

lazy_static! {
    static ref CLIENTS: Mutex<HashMap<SocketAddr, Sender<Packet>>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut host_ip: Option<String> = None;
    if let Ok(interfaces) = get_if_addrs() {
        for interface in interfaces {
            if interface.name == "wlan0" {
                if interface.ip().is_ipv4() {
                    host_ip = Some(interface.ip().to_string());
                }
            }
        }
    }

    let addr = env::args().nth(1).unwrap_or_else(|| {
        let mut hostip = host_ip.expect("Failed to get ip");
        hostip.push_str(":8080");
        println!("{}", hostip);
        hostip
    });
    #[cfg(feature = "wss")]
    let certs = load_certs(&Path::new("server.crt"))?;
    #[cfg(feature = "wss")]
    let key = load_keys(&Path::new("server.key"))?;
    #[cfg(feature = "wss")]
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("");
    #[cfg(feature = "wss")]
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Faild to bind");

    while let Ok((stream, addr)) = listener.accept().await {
        #[cfg(feature = "wss")]
        let stream = acceptor.accept(stream).await?;

        let (client, sender) = Client::new();
        let client = Arc::new(Mutex::new(client));
        tokio::spawn(async move {
            client.lock().await.handle_new_client(stream, addr).await;
        });

        CLIENTS.lock().await.insert(addr, sender);
    }

    Ok(())
}
