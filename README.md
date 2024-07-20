# esp-hal-mdns
Simple library for mdns request's for esp-hal.

## Example
This exampleis using embassy-net, but you can use non-async esp-wifi.
If you want to see full example, check `./examples` dir.

```rust
// create udp socket
let mut sock = UdpSocket::new(&stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer,);

// these values can be consts i guess
let ip_addr = IpAddress::v4(224, 0, 0, 251);
let ip_endpoint: IpEndpoint = IpEndpoint::new(ip_addr, 5353);

// bind on port 5353
_ = sock.bind(5353);

// join multicast group
_ = stack.join_multicast_group(ip_addr).await;

// create mdns query struct with given label
let mut mdns = MdnsQuery::new("_stackmat._tcp.local", 2500, esp_wifi::current_millis);

let mut data_buf = [0; 4096];
loop {
    // non blocking function returning mdns packet data every 2500ms (configurable in ::new)
    if let Some(data) = mdns.should_send_mdns_packet() {
        _ = sock.send_to(&data, ip_endpoint).await;
    }

    // its best not to block your thread i guess
    if sock.may_recv() {
        let res = sock.recv_from(&mut data_buf).await;
        if let Ok((n, _endpoint)) = res {
            // this "ws" is txt name
            let resp = mdns.parse_mdns_query(&data_buf[..n], Some("ws"));
            log::info!("{resp:?}");

            // response is (ip, port, Option<Txt-Value>)
            if resp.2.is_some() {
                // if txt value is found, break from the loop
                break;
            }
        }
    }

    Timer::after(Duration::from_millis(50)).await;
}

// leave multicast group
_ = stack.leave_multicast_group(ip_addr).await;
```
