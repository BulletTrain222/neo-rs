//! Opening the hosting port on the home router with UPnP (docs/MULTIPLAYER.md night plan 5,
//! item 23; this build's feature, off by default). Plain std networking, on its own thread:
//! SSDP discovery of an Internet Gateway Device, its description, and the WANIPConnection (or
//! WANPPPConnection) service's AddPortMapping, GetExternalIPAddress and DeletePortMapping. A
//! mapping is leased for two hours, so a game that stops without removing it does not leave the
//! port open for good. NEOPETS_UPNP_SSDP=host:port (diagnostic) sends the search to that
//! address instead of the multicast group (a stand-in router for tests).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

const SEARCH_TARGET: &str = "urn:schemas-upnp-org:device:InternetGatewayDevice:1";
const SERVICES: [&str; 2] = ["urn:schemas-upnp-org:service:WANIPConnection:1", "urn:schemas-upnp-org:service:WANPPPConnection:1"];
/// Two hours.
const LEASE: u32 = 7200;

/// A router's port mapping service: its control address and service type.
#[derive(Clone, Debug, PartialEq)]
pub struct Gateway {
    pub control: String,
    pub service: String,
}

/// What a request to the router came to.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// The port is forwarded; the router's address on the internet, when it said.
    Opened(Option<String>),
    Closed,
    Failed(String),
}

/// Find the router (a 2-second search).
pub fn discover() -> Result<Gateway, String> {
    let to: SocketAddr = std::env::var("NEOPETS_UPNP_SSDP").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| "239.255.255.250:1900".parse().unwrap());
    let s = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(2))).map_err(|e| e.to_string())?;
    let msg = format!("M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: {SEARCH_TARGET}\r\n\r\n");
    s.send_to(msg.as_bytes(), to).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 2048];
    let (n, _) = s.recv_from(&mut buf).map_err(|_| "no router answered (UPnP may be off on it)".to_string())?;
    let text = String::from_utf8_lossy(&buf[..n]);
    let location = header(&text, "location").ok_or("the router's answer had no address")?;
    let desc = http(&location, None)?;
    gateway(&location, &desc).ok_or_else(|| "the router offers no port mapping".to_string())
}

/// A header's value (case ignored).
fn header(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim().to_string())
    })
}

/// The port mapping service in a device description, its control address made whole.
pub fn gateway(location: &str, desc: &str) -> Option<Gateway> {
    let base = between(desc, "<URLBase>", "</URLBase>").map(|b| b.trim().trim_end_matches('/').to_string()).unwrap_or_else(|| origin(location));
    for service in SERVICES {
        let Some(at) = desc.find(service) else { continue };
        let control = between(&desc[at..], "<controlURL>", "</controlURL>")?.trim().to_string();
        let control = if control.starts_with("http://") { control } else { format!("{base}/{}", control.trim_start_matches('/')) };
        return Some(Gateway { control, service: service.to_string() });
    }
    None
}

fn between<'a>(s: &'a str, a: &str, b: &str) -> Option<&'a str> {
    let i = s.find(a)? + a.len();
    let j = s[i..].find(b)? + i;
    Some(&s[i..j])
}

/// "http://host:port" of an address.
fn origin(url: &str) -> String {
    let rest = url.strip_prefix("http://").unwrap_or(url);
    format!("http://{}", rest.split('/').next().unwrap_or(rest))
}

/// An HTTP GET, or a SOAP POST when `soap` holds (service#action, body): the reply's body.
fn http(url: &str, soap: Option<(&str, &str)>) -> Result<String, String> {
    let rest = url.strip_prefix("http://").ok_or("not an http address")?;
    let (host, path) = rest.split_once('/').map(|(h, p)| (h, format!("/{p}"))).unwrap_or((rest, "/".into()));
    let addr = host.to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("no address")?;
    let mut t = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    t.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    let req = match soap {
        None => format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"),
        Some((action, body)) => format!("POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: text/xml; charset=\"utf-8\"\r\nSOAPAction: \"{action}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()),
    };
    t.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let _ = t.read_to_end(&mut out);
    let text = String::from_utf8_lossy(&out).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let ok = head.lines().next().is_some_and(|l| l.contains(" 200"));
    if ok {
        Ok(body.to_string())
    } else {
        Err(between(body, "<errorDescription>", "</errorDescription>").map_or_else(|| head.lines().next().unwrap_or("no answer").to_string(), str::to_string))
    }
}

/// A SOAP call on the gateway's service.
fn call(g: &Gateway, action: &str, args: &[(&str, String)]) -> Result<String, String> {
    let params: String = args.iter().map(|(k, v)| format!("<{k}>{v}</{k}>")).collect();
    let body = format!(
        "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:{action} xmlns:u=\"{}\">{params}</u:{action}></s:Body></s:Envelope>",
        g.service
    );
    http(&g.control, Some((&format!("{}#{action}", g.service), &body)))
}

/// Forward UDP `port` to this computer (`local`) for two hours, and ask the router's own address.
pub fn open(port: u16, local: std::net::IpAddr) -> Outcome {
    let g = match discover() {
        Ok(g) => g,
        Err(e) => return Outcome::Failed(e),
    };
    let args = [
        ("NewRemoteHost", String::new()),
        ("NewExternalPort", port.to_string()),
        ("NewProtocol", "UDP".into()),
        ("NewInternalPort", port.to_string()),
        ("NewInternalClient", local.to_string()),
        ("NewEnabled", "1".into()),
        ("NewPortMappingDescription", "Neopets co-op".into()),
        ("NewLeaseDuration", LEASE.to_string()),
    ];
    if let Err(e) = call(&g, "AddPortMapping", &args) {
        return Outcome::Failed(format!("the router refused ({e})"));
    }
    let ip = call(&g, "GetExternalIPAddress", &[]).ok().and_then(|b| between(&b, "<NewExternalIPAddress>", "</NewExternalIPAddress>").map(|s| s.trim().to_string())).filter(|s| !s.is_empty());
    Outcome::Opened(ip)
}

/// Remove the forward of UDP `port`.
pub fn close(port: u16) -> Outcome {
    match discover().and_then(|g| call(&g, "DeletePortMapping", &[("NewRemoteHost", String::new()), ("NewExternalPort", port.to_string()), ("NewProtocol", "UDP".into())])) {
        Ok(_) => Outcome::Closed,
        Err(e) => Outcome::Failed(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A description's WANIPConnection control address is made whole from the description's
    /// address when it is relative, and its base when it gives one.
    #[test]
    fn finds_the_mapping_service() {
        let desc = "<root><device><serviceList><service><serviceType>urn:schemas-upnp-org:service:Layer3Forwarding:1</serviceType><controlURL>/l3f</controlURL></service><service><serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType><controlURL>/ctl/IPConn</controlURL></service></serviceList></device></root>";
        let g = gateway("http://192.168.1.1:5000/rootDesc.xml", desc).unwrap();
        assert_eq!(g.control, "http://192.168.1.1:5000/ctl/IPConn");
        assert_eq!(g.service, "urn:schemas-upnp-org:service:WANIPConnection:1");
        let based = format!("<URLBase>http://10.0.0.1:80/</URLBase>{desc}");
        assert_eq!(gateway("http://192.168.1.1:5000/x.xml", &based).unwrap().control, "http://10.0.0.1:80/ctl/IPConn");
        assert!(gateway("http://h/x", "<root></root>").is_none());
        assert_eq!(header("HTTP/1.1 200 OK\r\nLOCATION: http://a/b\r\n", "location").as_deref(), Some("http://a/b"));
    }
}
