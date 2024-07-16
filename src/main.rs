use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use socket2::Socket;

fn main() {
    let fd = Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    fd.set_reuse_address(true).unwrap();
    fd.set_reuse_port(true).unwrap();
    //fd.set_nonblocking(true).unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 5353);
    fd.bind(&addr.into()).unwrap();

    let interface_ip = Ipv4Addr::new(192, 168, 1, 38);
    fd.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &interface_ip)
        .unwrap();
    fd.set_multicast_if_v4(&interface_ip).unwrap();
    let socket = fd;

    let query = "_stackmat._tcp.local";
    let mut questions = [
        dns_protocol::Question::new(query, dns_protocol::ResourceType::Ptr, 1 | 0x8000), //1-IN
                                                                                         //0x8000 - prefer unicast
    ];

    let msg = dns_protocol::Message::new(
        0,
        *dns_protocol::Flags::default().set_recursive(false),
        &mut questions,
        &mut [],
        &mut [],
        &mut [],
    );

    let mut buf = [0; 4096];
    let n = msg.write(&mut buf).unwrap();

    let addr = SocketAddr::from(([224, 0, 0, 251], 5353));
    socket.send_to(&buf[..n], &addr.into()).unwrap();

    loop {
        let mut buf: [MaybeUninit<u8>; 4096] = unsafe { MaybeUninit::uninit().assume_init() };
        let (n, _) = socket.recv_from(&mut buf).unwrap();

        let buf = unsafe { std::mem::transmute::<_, [u8; 4096]>(buf) };

        let mut answers = [dns_protocol::ResourceRecord::default(); 16];
        let mut additional = [dns_protocol::ResourceRecord::default(); 16];

        let res =
            dns_protocol::Message::read(&buf[..n], &mut [], &mut answers, &mut [], &mut additional);

        println!("");
        println!("{res:?}");
        println!("");

        if let Ok(res) = res {
            if res.answers().len() > 0 {
                let mut segments = res.answers()[0].name().segments();
                let mut is_ans = true;

                for seg in query.split(".") {
                    if let Some(segment) = segments.next() {
                        if let dns_protocol::LabelSegment::String(segment) = segment {
                            if seg == segment {
                                continue;
                            }
                        }
                    }

                    is_ans = false;
                    break;
                }

                if is_ans {
                    println!("{:?}", res.additional());
                }
            }
        }
    }
}
