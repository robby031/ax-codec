use axcodec_derive::{Decode, Encode};
use axcodec_net::codec::AxCodec;
use futures::{SinkExt, StreamExt};
use tokio_util::codec::Framed;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct Packet {
    opcode: u16,
    payload: Vec<u8>,
}

#[tokio::test]
async fn framed_duplex_roundtrip() {
    let (client, server) = tokio::io::duplex(64 * 1024);

    let mut client_framed = Framed::new(client, AxCodec::<Packet>::new());
    let mut server_framed = Framed::new(server, AxCodec::<Packet>::new());

    let sent = Packet {
        opcode: 0x1234,
        payload: vec![1, 2, 3, 4, 5],
    };

    // Client sends
    client_framed.send(sent.clone()).await.unwrap();

    // Server receives
    let received = server_framed.next().await.unwrap().unwrap();
    assert_eq!(sent, received);

    // Server echoes back
    server_framed.send(received).await.unwrap();

    // Client receives echo
    let echoed = client_framed.next().await.unwrap().unwrap();
    assert_eq!(sent, echoed);
}
