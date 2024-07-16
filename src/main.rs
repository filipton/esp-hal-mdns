use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
};

use net2::unix::UnixUdpBuilderExt;
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

    /*
    let mut builder = dns_parser::Builder::new_query(0, false);
    let prefer_unicast = false;
    builder.add_question(
        "_stackmat._tcp.local",
        prefer_unicast,
        dns_parser::QueryType::TXT,
        dns_parser::QueryClass::IN,
    );
    let packet_data = builder.build().unwrap();
    let buf = packet_data
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    println!("{:?}", buf);
    */

    //let data = "546001200001000000000001076578616d706c6503636f6d000001000100002904d000000000000c000a0008d9e95614a58b3b33";
    //let data = "5460012000010000000000010A5F737461636B6D6174045F746370056C6F63616C00000C000100002904D000000000000C000A0008D9E95614A58B3B33";
    let data = "000000000001000000000000095f737461636b6d6174045f746370056c6f63616c00000c0001";
    //let data = "000000000001000100000000095f737461636b6d6174045f746370056c6f63616c00000c0001c00c000c000100001194001310737461636b6d61745f6261636b656e64c00c";
    let data = data
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|x| {
            (u8::from_str_radix(&x[0].to_string(), 16).unwrap() * 16)
                + u8::from_str_radix(&x[1].to_string(), 16).unwrap()
        })
        .collect::<Vec<u8>>();

    let addr = SocketAddr::from(([224, 0, 0, 251], 5353));
    socket.send_to(&data, &addr.into()).unwrap();

    let mut buf: [MaybeUninit<u8>; 4096] = unsafe { MaybeUninit::uninit().assume_init() };
    let (n, _) = socket.recv_from(&mut buf).unwrap();

    let buf = unsafe { std::mem::transmute::<_, [u8; 4096]>(buf) };
    let buf = &buf[0..n];
    let id = u16::from_be_bytes(buf[0..2].try_into().unwrap());
    let flags = u16::from_be_bytes(buf[2..4].try_into().unwrap());
    let questions = u16::from_be_bytes(buf[4..6].try_into().unwrap());
    let answer_prs = u16::from_be_bytes(buf[6..8].try_into().unwrap());
    let authority_prs = u16::from_be_bytes(buf[8..10].try_into().unwrap());
    let additional_prs = u16::from_be_bytes(buf[10..12].try_into().unwrap());

    let mut offset: usize = 12;

    println!("id: {id}");
    println!("flags: {flags}");
    println!("questions: {questions}");
    println!("answer_prs: {answer_prs}");
    println!("authority_prs: {authority_prs}");
    println!("additional_prs: {additional_prs}");

    println!();
    println!();
    for q in 0..questions {
        println!("Question: {q}");

        loop {
            let len = buf[offset] as usize;
            offset += 1;
            if len == 0 {
                break;
            }

            let label = String::from_utf8_lossy(&buf[offset..offset + len]);
            println!("label: {label:?}");

            offset += len;
        }

        let q_type = u16::from_be_bytes(buf[offset..offset + 2].try_into().unwrap());
        let class = u16::from_be_bytes(buf[offset + 2..offset + 4].try_into().unwrap());
        println!("type: {q_type}");
        println!("class: {class}");

        offset += 4;
    }

    println!();
    println!();
    for a in 0..answer_prs {
        println!("Answer: {a}");

        let name = u16::from_be_bytes(buf[offset..offset + 2].try_into().unwrap());
        let a_type = u16::from_be_bytes(buf[offset + 2..offset + 4].try_into().unwrap());
        let class = u16::from_be_bytes(buf[offset + 4..offset + 6].try_into().unwrap());
        let ttl = u32::from_be_bytes(buf[offset + 6..offset + 10].try_into().unwrap());

        let len = u16::from_be_bytes(buf[offset + 10..offset + 12].try_into().unwrap()) as usize;
        let data = &buf[offset + 12..offset + 12 + len];

        println!("name: {name}");
        println!("type: {a_type}");
        println!("class: {class}");
        println!("ttl: {ttl}");
        println!("len: {len}");
        println!("buf: {data:?}");

        offset += len + 12 + 1;
    }

    let buf = buf.iter().map(|x| format!("{:02x}", x)).collect::<String>();
    println!("{:?}", buf);
}
