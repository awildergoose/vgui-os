use core::fmt::Write;

use alloc::{format, string::String};
use log::{trace, warn};
use smoltcp::{iface::SocketHandle, socket::dns::QueryHandle, wire::IpEndpoint};

use crate::net::NETWORK_STACK;

enum IrcState {
    QueryingDNS { dns_query: QueryHandle },
    Connecting { tcp_socket: SocketHandle },
    Connected { tcp_socket: SocketHandle },
    Acknowledged { tcp_socket: SocketHandle },
}

pub struct Irc {
    state: IrcState,
    recv: String,

    pub text: String,
}

impl Irc {
    pub fn new() -> Self {
        let mut ns = NETWORK_STACK.lock();
        let stack = ns.as_mut().unwrap();
        let query = stack
            .dns_query("irc.libera.chat")
            .expect("dns query failed");

        Self {
            state: IrcState::QueryingDNS { dns_query: query },
            recv: String::new(),
            text: String::new(),
        }
    }

    pub fn send_message(&mut self, text: &str) {
        let mut ns = NETWORK_STACK.lock();
        let stack = ns.as_mut().unwrap();

        match self.state {
            IrcState::QueryingDNS { dns_query: _ }
            | IrcState::Connecting { tcp_socket: _ }
            | IrcState::Connected { tcp_socket: _ } => {}
            IrcState::Acknowledged { tcp_socket } => {
                let data = &(format!("PRIVMSG #testingvgui :{text}\r\n")).into_bytes();
                stack.tcp_send(tcp_socket, data);
                let _ = writeln!(self.text, "You: {text}");
            }
        }
    }

    pub fn update(&mut self) {
        let mut ns = NETWORK_STACK.lock();
        let stack = ns.as_mut().unwrap();

        match self.state {
            IrcState::QueryingDNS { dns_query } => {
                let res = stack.dns_result(dns_query);

                let Ok(res) = res else {
                    let err = res.unwrap_err();

                    match err {
                        smoltcp::socket::dns::GetQueryResultError::Pending => return,
                        smoltcp::socket::dns::GetQueryResultError::Failed => {
                            warn!("dns error: {err:#?}");
                            return;
                        }
                    }
                };

                let Some(address) = res.first() else {
                    warn!("no address found for dns query?");
                    return;
                };

                let handle = stack.tcp_connect(IpEndpoint::new(*address, 6667));
                self.state = IrcState::Connecting { tcp_socket: handle };
            }
            IrcState::Connecting { tcp_socket } => {
                if stack.tcp_may_send(tcp_socket) {
                    trace!("sending nick!");
                    stack.tcp_send(tcp_socket, b"NICK vguios\r\n");
                    stack.tcp_send(tcp_socket, b"USER vguios 0 * :Vguios\r\n");
                    self.state = IrcState::Connected { tcp_socket };
                }
            }
            IrcState::Connected { tcp_socket } => {
                let data = stack.tcp_recv(tcp_socket);
                let data = data.iter().map(|c| *c as char).collect::<String>();
                self.recv += &data;

                if self.recv.contains("\r\n") {
                    if self.recv.contains(" 001 ") {
                        stack.tcp_send(tcp_socket, b"JOIN #testingvgui\r\n");
                        self.state = IrcState::Acknowledged { tcp_socket };
                    }

                    self.recv.clear();
                }
            }
            IrcState::Acknowledged { tcp_socket } => {
                let data = stack.tcp_recv(tcp_socket);
                let data = data.iter().map(|c| *c as char).collect::<String>();
                self.recv += &data;

                if self.recv.contains("\r\n") {
                    for message in self.recv.split("\r\n") {
                        if message.is_empty() {
                            continue;
                        }

                        let mut split = message.split(' ');
                        let Some(sender) = split.next() else {
                            continue;
                        };
                        if sender == "PING" {
                            let Some(token) = split.remainder() else {
                                continue;
                            };
                            stack.tcp_send(tcp_socket, &(format!("PONG {token}\r\n").into_bytes()));
                            continue;
                        }
                        let Some(kind) = split.next() else {
                            continue;
                        };

                        match kind {
                            // recv message
                            "PRIVMSG" => {
                                let Some(channel) = split.next() else {
                                    continue;
                                };
                                let Some(text) = split.remainder() else {
                                    continue;
                                };
                                let text = &text[1..];
                                let _ = writeln!(self.text, "{sender}@{channel}: {text}");
                            }
                            // user list | end of user list
                            "353" | "366" => {}
                            _ => {
                                if message.starts_with(':') {
                                    // oh, just an motd message I guess
                                    self.text += message;
                                    self.text += "\n";
                                    continue;
                                }

                                trace!("unknown message kind: \"{message}\"");
                            }
                        }
                        // :stevedecidestous!~stevedeci@that.is.my.ip PRIVMSG #testingvgui :hello, neighbor!
                    }

                    self.recv.clear();
                }
            }
        }
    }
}

impl Default for Irc {
    fn default() -> Self {
        Self::new()
    }
}
