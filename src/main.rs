use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
};

use socket2::Socket;

fn main() {
    let fd = Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    fd.set_reuse_address(true).unwrap();
    fd.set_reuse_port(true).unwrap();
    fd.set_nonblocking(true).unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 5353);
    fd.bind(&addr.into()).unwrap();

    let interface_ip = Ipv4Addr::new(192, 168, 1, 38);
    fd.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &interface_ip)
        .unwrap();
    fd.set_multicast_if_v4(&interface_ip).unwrap();
    let socket = fd;

    let mut builder = dns_parser::Builder::new_query(0, false);
    let prefer_unicast = false;
    builder.add_question(
        "_stackmat._tcp.local",
        prefer_unicast,
        dns_parser::QueryType::PTR,
        dns_parser::QueryClass::IN,
    );
    let packet_data = builder.build().unwrap();
    let buf = packet_data
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    println!("{:?}", buf);

    let addr = SocketAddr::from(([224, 0, 0, 251], 5353));
    socket.send_to(&packet_data, &addr.into()).unwrap();

    loop {
        let mut buf: [MaybeUninit<u8>; 4096] = unsafe { MaybeUninit::uninit().assume_init() };
        let (n, _) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 4096]>(buf) };
        let res = dns_parser::Packet::parse(&buf[0..n]);
        println!("{res:?}");
    }
}
