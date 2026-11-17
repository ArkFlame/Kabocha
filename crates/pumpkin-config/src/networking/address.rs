use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use toml::{Table, Value};

use super::bedrock::NetherNetConfig;
use super::java::JavaConfig;
use super::management::ManagementServerConfig;
use super::query::QueryConfig;

#[derive(Default)]
pub struct AddressReport {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub patched: bool,
}

enum Address {
    Valid(SocketAddr),
    NoPort(IpAddr),
    BadPort(String),
    Invalid,
}

fn listeners() -> [(&'static str, &'static str, u16); 4] {
    [
        (
            "networking.java",
            "Java Edition",
            JavaConfig::default().address.port(),
        ),
        (
            "networking.bedrock.nethernet",
            "Bedrock Edition",
            NetherNetConfig::default().address.port(),
        ),
        (
            "networking.management",
            "Management server",
            ManagementServerConfig::default().address.port(),
        ),
        (
            "networking.query",
            "Query",
            QueryConfig::default().address.port(),
        ),
    ]
}

pub fn check(raw: &mut Value) -> AddressReport {
    let ignore = raw
        .get("ignore_port_warning")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut report = AddressReport::default();

    for (path, name, default_port) in listeners() {
        let Some(table) = table_mut(raw, path) else {
            continue;
        };

        // "port" is not a setting any more: an older config that sets it binds the
        // default port while the edited one is silently dropped. Fold it into
        // "address" instead so the port the operator asked for is the one used.
        let legacy_port = table.get("port").and_then(|value| match value {
            Value::Integer(port) => Some(*port),
            Value::String(text) => text.trim().parse::<i64>().ok(),
            _ => None,
        });
        if let Some(port) = legacy_port {
            table.remove("port");
            report.patched = true;
            match u16::try_from(port) {
                Ok(port) => {
                    let host = table
                        .get("address")
                        .and_then(Value::as_str)
                        .and_then(|text| match classify(text) {
                            Address::Valid(addr) => Some(addr.ip()),
                            Address::NoPort(ip) => Some(ip),
                            _ => None,
                        })
                        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
                    let addr = SocketAddr::new(host, port);
                    report.warnings.push(format!(
                        "{name} [{path}]: port is not a setting, using {addr}"
                    ));
                    table.insert("address".to_owned(), Value::String(addr.to_string()));
                }
                Err(_) => report.errors.push(format!(
                    "{name} [{path}]: {}",
                    port_problem(&port.to_string())
                )),
            }
        }

        let Some(value) = table.get("address") else {
            continue;
        };
        let Some(text) = value.as_str().map(str::to_owned) else {
            report.errors.push(format!(
                "{name} address in [{path}] must be a string, e.g. \"0.0.0.0:{default_port}\""
            ));
            continue;
        };
        let subject = format!("{name} address \"{text}\" in [{path}]");

        let addr = match classify(&text) {
            Address::Valid(addr) => addr,
            Address::NoPort(ip) => {
                let addr = SocketAddr::new(ip, default_port);
                if !ignore {
                    report
                        .warnings
                        .push(format!("{subject} has no port, using {addr}"));
                }
                table.insert("address".to_owned(), Value::String(addr.to_string()));
                report.patched = true;
                addr
            }
            Address::BadPort(port) => {
                let message = format!("{subject}: {}", port_problem(&port));
                reject_port(table, "address", name, message, ignore, &mut report);
                continue;
            }
            Address::Invalid => {
                let reason = if text.matches(':').count() > 1 && !text.starts_with('[') {
                    "IPv6 with a port needs brackets, e.g. \"[::1]:25565\""
                } else {
                    "hostnames are not resolved"
                };
                report
                    .errors
                    .push(format!("{subject} is not a valid IP address ({reason})"));
                continue;
            }
        };

        if addr.ip().is_loopback() && !ignore {
            report.warnings.push(format!(
                "{name} is bound to loopback {addr}, only local connections can reach it"
            ));
        }
    }

    let path = "networking.lan_broadcast";
    if let Some(table) = table_mut(raw, path)
        && let Some(port) = table.get("port")
        && port.as_integer().is_none_or(|p| u16::try_from(p).is_err())
    {
        let text = port
            .as_str()
            .map_or_else(|| port.to_string(), str::to_owned);
        let message = format!("LAN broadcast in [{path}]: {}", port_problem(&text));
        reject_port(table, "port", "LAN broadcast", message, ignore, &mut report);
    }

    report
}

fn classify(text: &str) -> Address {
    if let Ok(addr) = text.parse::<SocketAddr>() {
        return Address::Valid(addr);
    }
    if let Ok(ip) = text.parse::<IpAddr>() {
        return Address::NoPort(ip);
    }
    if let Some(ip) = bracketed_ipv6(text) {
        return Address::NoPort(ip.into());
    }
    match text.rsplit_once(':') {
        Some((host, port))
            if host.parse::<Ipv4Addr>().is_ok() || bracketed_ipv6(host).is_some() =>
        {
            Address::BadPort(port.to_owned())
        }
        _ => Address::Invalid,
    }
}

fn port_problem(port: &str) -> String {
    let number = port.strip_prefix('-').unwrap_or(port);
    if port.is_empty() {
        "port is empty".to_owned()
    } else if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) {
        format!("port {port} is out of range (0-65535)")
    } else {
        format!("port \"{port}\" is not a number")
    }
}

fn bracketed_ipv6(text: &str) -> Option<Ipv6Addr> {
    text.strip_prefix('[')?.strip_suffix(']')?.parse().ok()
}

fn reject_port(
    table: &mut Table,
    key: &str,
    name: &str,
    message: String,
    ignore: bool,
    report: &mut AddressReport,
) {
    if ignore {
        report
            .warnings
            .push(format!("{message}, {name} is disabled"));
        table.remove(key);
        table.insert("enabled".to_owned(), Value::Boolean(false));
        report.patched = true;
    } else {
        report.errors.push(message);
    }
}

fn table_mut<'a>(raw: &'a mut Value, path: &str) -> Option<&'a mut Table> {
    path.split('.')
        .try_fold(raw, |value, key| value.get_mut(key))?
        .as_table_mut()
}

#[cfg(test)]
mod tests {
    use super::check;

    fn table<'a>(raw: &'a toml::Value, path: &str) -> &'a toml::Value {
        path.split('.')
            .fold(raw, |value, key| value.get(key).expect("table"))
    }

    #[test]
    fn legacy_port_setting_is_folded_into_the_address() {
        let mut raw: toml::Value = "[networking.java]\nport = 7732\n".parse().unwrap();
        let report = check(&mut raw);

        assert!(report.patched);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let java = table(&raw, "networking.java");
        assert_eq!(
            java.get("address").and_then(toml::Value::as_str),
            Some("0.0.0.0:7732")
        );
        assert!(java.get("port").is_none());
        assert_eq!(report.warnings.len(), 1);
    }
}
