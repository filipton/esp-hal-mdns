use std::net::{SocketAddr, UdpSocket};

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 51352));
    let socket = UdpSocket::bind(&[addr].as_slice()).unwrap();
    socket.connect("127.0.0.1:5053").expect("Cant connect");

    let data = "546001200001000000000001076578616d706c6503636f6d000001000100002904d000000000000c000a0008d9e95614a58b3b33";
    let data = data
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|x| {
            (u8::from_str_radix(&x[0].to_string(), 16).unwrap() * 16)
                + u8::from_str_radix(&x[1].to_string(), 16).unwrap()
        })
        .collect::<Vec<u8>>();

    socket.send(&data).unwrap();

    let mut buf = [0; 45];
    socket.recv(&mut buf).unwrap();

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

    let buf = buf.map(|x| format!("{:02x}", x)).join("");
    println!("{:?}", buf);
}
