pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const IPPROTO_UDP: u8 = 17;
pub const UDP_ECHO_PORT: u16 = 7;

pub fn is_udp_echo_port(port: u16) -> bool {
    port == UDP_ECHO_PORT
}
