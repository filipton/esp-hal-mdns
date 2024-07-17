#![no_std]

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

    pub fn parse_mdns_response(&mut self, data: &[u8]) -> bool {
        let mut answers = [dns_protocol::ResourceRecord::default(); 16];
        let mut additional = [dns_protocol::ResourceRecord::default(); 16];

        let res =
            dns_protocol::Message::read(&data, &mut [], &mut answers, &mut [], &mut additional);

        if let Ok(res) = res {
            if res.answers().len() > 0 && res.additional().len() > 0 {
                let mut segments = res.answers()[0].name().segments();
                let mut is_ans = true;

                for seg in self.query_str.split(".") {
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
                    log::info!("{:?}", res.additional());
                    return true;
                }
            }
        }

        return false;
    }
}
