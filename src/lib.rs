#![no_std]

use dns_protocol::{Flags, LabelSegment, Message, Question, ResourceRecord, ResourceType};

const MDNS_BUF_SIZE: usize = 4096;

pub struct MdnsQuery<'a> {
    pub query_str: &'a str,
    buff: [u8; MDNS_BUF_SIZE],
    n: usize,

    resend_interval: u64,
    last_mdns_sent: u64,
    curr_time_ms_func: fn() -> u64,
}

impl<'a> MdnsQuery<'a> {
    pub fn new(query: &'a str, resend_interval: u64, curr_time_ms_func: fn() -> u64) -> Self {
        let mut questions = [
            Question::new(query, dns_protocol::ResourceType::Ptr, 1 | 0x8000), //1-IN
                                                                               //0x8000 - prefer unicast
        ];

        let msg = Message::new(
            0,
            *Flags::default().set_recursive(false),
            &mut questions,
            &mut [],
            &mut [],
            &mut [],
        );

        let mut buff = [0; MDNS_BUF_SIZE];
        let n = msg.write(&mut buff).unwrap();

        Self {
            query_str: query,

            buff,
            n,

            resend_interval,
            last_mdns_sent: 0,
            curr_time_ms_func,
        }
    }

    pub fn should_send_mdns_packet(&mut self) -> Option<&[u8]> {
        if (self.curr_time_ms_func)() - self.last_mdns_sent > self.resend_interval {
            self.last_mdns_sent = (self.curr_time_ms_func)();

            return Some(&self.buff[..self.n]);
        }

        None
    }

    pub fn parse_mdns_txt(&mut self, data: &[u8], key: &str) -> Option<heapless::String<255>> {
        let mut answers = [ResourceRecord::default(); 16];
        let mut additional = [ResourceRecord::default(); 16];

        let res = Message::read(&data, &mut [], &mut answers, &mut [], &mut additional);

        if let Ok(res) = res {
            if res.answers().len() > 0 && res.additional().len() > 0 {
                let mut segments = res.answers()[0].name().segments();
                let mut is_ans = true;

                for seg in self.query_str.split(".") {
                    if let Some(segment) = segments.next() {
                        if let LabelSegment::String(segment) = segment {
                            if seg == segment {
                                continue;
                            }
                        }
                    }

                    is_ans = false;
                    break;
                }

                if is_ans {
                    for add in res.additional() {
                        if add.ty() == ResourceType::Txt {
                            let data = add.data();
                            let mut offset = 0;
                            loop {
                                if offset >= data.len() {
                                    break;
                                }

                                let len = data[offset] as usize;
                                offset += 1;

                                let mut splitted =
                                    data[offset..offset + len].splitn(2, |&x| x == 0x3d); // 0x3d "="

                                let sp_key = splitted.next().unwrap_or(&[]);
                                let sp_value = splitted.next().unwrap_or(&[]);

                                if key.as_bytes() == sp_key {
                                    let value = core::str::from_utf8(sp_value).unwrap_or("");
                                    return Some(
                                        value.try_into().unwrap_or(heapless::String::new()),
                                    );
                                }

                                offset += len;
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn parse_mdns_port(&mut self, data: &[u8]) -> Option<u16> {
        let mut answers = [ResourceRecord::default(); 16];
        let mut additional = [ResourceRecord::default(); 16];

        let res = Message::read(&data, &mut [], &mut answers, &mut [], &mut additional);

        if let Ok(res) = res {
            if res.answers().len() > 0 && res.additional().len() > 0 {
                let mut segments = res.answers()[0].name().segments();
                let mut is_ans = true;

                for seg in self.query_str.split(".") {
                    if let Some(segment) = segments.next() {
                        if let LabelSegment::String(segment) = segment {
                            if seg == segment {
                                continue;
                            }
                        }
                    }

                    is_ans = false;
                    break;
                }

                if is_ans {
                    for add in res.additional() {
                        if add.ty() == ResourceType::Srv {
                            let port = u16::from_be_bytes(add.data()[4..6].try_into().unwrap());
                            return Some(port);
                        } else if add.ty() == ResourceType::A {
                            let ip = &add.data()[0..4];
                            log::info!("ip: {ip:?}");
                        }
                    }
                }
            }
        }

        None
    }

    pub fn parse_mdns_ip(&mut self, data: &[u8]) -> Option<[u8; 4]> {
        let mut answers = [ResourceRecord::default(); 16];
        let mut additional = [ResourceRecord::default(); 16];

        let res = Message::read(&data, &mut [], &mut answers, &mut [], &mut additional);

        if let Ok(res) = res {
            if res.answers().len() > 0 && res.additional().len() > 0 {
                let mut segments = res.answers()[0].name().segments();
                let mut is_ans = true;

                for seg in self.query_str.split(".") {
                    if let Some(segment) = segments.next() {
                        if let LabelSegment::String(segment) = segment {
                            if seg == segment {
                                continue;
                            }
                        }
                    }

                    is_ans = false;
                    break;
                }

                if is_ans {
                    for add in res.additional() {
                        if add.ty() == ResourceType::A {
                            let ip = &add.data()[0..4];
                            return Some(ip.try_into().unwrap_or([0; 4]));
                        }
                    }
                }
            }
        }

        None
    }
}
