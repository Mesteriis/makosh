//! Implicit-TLS SMTP protocol adapter owned exclusively by Mail.

use std::time::Duration;

use async_native_tls::{Certificate, TlsConnector};
use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
use makosh_mail_api::{OutgoingMailV1, SmtpEndpointV1, valid_host, valid_mailbox, valid_smtp_port};

pub const PACKAGE: &str = "makosh-mail-smtp";
const MAX_RESPONSE_LINE_BYTES: usize = 4 * 1024;
const SMTP_OPERATION_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SmtpAcceptedReceiptV1 {
    pub response_code: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmtpAdapterErrorV1 {
    InvalidRequest,
    Unavailable,
    Protocol,
    Rejected,
}

pub async fn send_implicit_tls(
    endpoint: &SmtpEndpointV1,
    message: &OutgoingMailV1,
    password: &str,
    rfc822_message: &str,
) -> Result<SmtpAcceptedReceiptV1, SmtpAdapterErrorV1> {
    async_std::future::timeout(
        SMTP_OPERATION_TIMEOUT,
        send_implicit_tls_inner(endpoint, message, password, rfc822_message),
    )
    .await
    .map_err(|_| SmtpAdapterErrorV1::Unavailable)?
}

async fn send_implicit_tls_inner(
    endpoint: &SmtpEndpointV1,
    message: &OutgoingMailV1,
    password: &str,
    rfc822_message: &str,
) -> Result<SmtpAcceptedReceiptV1, SmtpAdapterErrorV1> {
    if !valid_host(&endpoint.host)
        || !valid_smtp_port(endpoint.port)
        || password.is_empty()
        || rfc822_message.is_empty()
        || !valid_mailbox(&endpoint.from_address)
        || message.recipients.is_empty()
        || message
            .recipients
            .iter()
            .chain(&message.cc_recipients)
            .chain(&message.bcc_recipients)
            .any(|value| !valid_mailbox(value))
    {
        return Err(SmtpAdapterErrorV1::InvalidRequest);
    }
    let tcp = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
    let connector = endpoint
        .ca_certificate_pem
        .as_deref()
        .map(|pem| {
            Certificate::from_pem(pem.as_bytes())
                .map(|certificate| TlsConnector::new().add_root_certificate(certificate))
                .map_err(|_| SmtpAdapterErrorV1::InvalidRequest)
        })
        .transpose()?
        .unwrap_or_default();
    let mut stream = connector
        .connect(&endpoint.host, tcp)
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;

    expect_response(&mut stream, 220).await?;
    send_line(&mut stream, "EHLO makosh.local").await?;
    expect_response(&mut stream, 250).await?;
    let credentials = base64_encode(format!("\0{}\0{password}", endpoint.username).as_bytes());
    send_line(&mut stream, &format!("AUTH PLAIN {credentials}")).await?;
    expect_response(&mut stream, 235).await?;
    send_line(
        &mut stream,
        &format!("MAIL FROM:<{}>", endpoint.from_address),
    )
    .await?;
    expect_response(&mut stream, 250).await?;
    for recipient in message
        .recipients
        .iter()
        .chain(&message.cc_recipients)
        .chain(&message.bcc_recipients)
    {
        send_line(&mut stream, &format!("RCPT TO:<{recipient}>")).await?;
        expect_response(&mut stream, 250).await?;
    }
    send_line(&mut stream, "DATA").await?;
    expect_response(&mut stream, 354).await?;
    stream
        .write_all(dot_stuff(rfc822_message).as_bytes())
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
    stream
        .flush()
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
    let response_code = expect_response(&mut stream, 250).await?;
    send_line(&mut stream, "QUIT").await?;
    let _ = expect_response(&mut stream, 221).await;
    Ok(SmtpAcceptedReceiptV1 { response_code })
}

async fn send_line<S: WriteExt + Unpin>(
    stream: &mut S,
    line: &str,
) -> Result<(), SmtpAdapterErrorV1> {
    if line.contains(['\r', '\n', '\0']) {
        return Err(SmtpAdapterErrorV1::InvalidRequest);
    }
    stream
        .write_all(line.as_bytes())
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
    stream
        .write_all(b"\r\n")
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
    stream
        .flush()
        .await
        .map_err(|_| SmtpAdapterErrorV1::Unavailable)
}

async fn expect_response<S: ReadExt + Unpin>(
    stream: &mut S,
    expected: u16,
) -> Result<u16, SmtpAdapterErrorV1> {
    let mut response = read_response_line(stream).await?;
    let (code, continued) = parse_response_line(&response)?;
    if continued {
        loop {
            response = read_response_line(stream).await?;
            let (next_code, next_continued) = parse_response_line(&response)?;
            if next_code != code {
                return Err(SmtpAdapterErrorV1::Protocol);
            }
            if !next_continued {
                break;
            }
        }
    }
    (code == expected)
        .then_some(code)
        .ok_or(SmtpAdapterErrorV1::Rejected)
}

async fn read_response_line<S: ReadExt + Unpin>(
    stream: &mut S,
) -> Result<String, SmtpAdapterErrorV1> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let read = stream
            .read(&mut byte)
            .await
            .map_err(|_| SmtpAdapterErrorV1::Unavailable)?;
        if read == 0 || bytes.len() >= MAX_RESPONSE_LINE_BYTES {
            return Err(SmtpAdapterErrorV1::Protocol);
        }
        bytes.push(byte[0]);
        if bytes.ends_with(b"\r\n") {
            break;
        }
    }
    String::from_utf8(bytes).map_err(|_| SmtpAdapterErrorV1::Protocol)
}

fn parse_response_line(line: &str) -> Result<(u16, bool), SmtpAdapterErrorV1> {
    let bytes = line.as_bytes();
    if bytes.len() < 5 || !line.ends_with("\r\n") || !bytes[..3].iter().all(u8::is_ascii_digit) {
        return Err(SmtpAdapterErrorV1::Protocol);
    }
    let code = line[..3]
        .parse::<u16>()
        .map_err(|_| SmtpAdapterErrorV1::Protocol)?;
    match bytes[3] {
        b' ' => Ok((code, false)),
        b'-' => Ok((code, true)),
        _ => Err(SmtpAdapterErrorV1::Protocol),
    }
}

fn dot_stuff(message: &str) -> String {
    let normalized = message.replace("\r\n", "\n").replace('\r', "\n");
    let body = normalized
        .split('\n')
        .map(|line| {
            if line.starts_with('.') {
                format!(".{line}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    format!("{body}\r\n.\r\n")
}

fn base64_encode(value: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(value.len().div_ceil(3) * 4);
    for chunk in value.chunks(3) {
        let packed = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(char::from(ALPHABET[((packed >> 18) & 0x3f) as usize]));
        encoded.push(char::from(ALPHABET[((packed >> 12) & 0x3f) as usize]));
        encoded.push(if chunk.len() > 1 {
            char::from(ALPHABET[((packed >> 6) & 0x3f) as usize])
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            char::from(ALPHABET[(packed & 0x3f) as usize])
        } else {
            '='
        });
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_response_and_dot_stuffs_body() {
        assert_eq!(parse_response_line("250-mail.example\r\n"), Ok((250, true)));
        assert_eq!(parse_response_line("250 accepted\r\n"), Ok((250, false)));
        assert_eq!(
            dot_stuff("header\r\n\r\n.leading"),
            "header\r\n\r\n..leading\r\n.\r\n"
        );
    }

    #[test]
    fn encodes_auth_plain_without_external_helper() {
        assert_eq!(base64_encode(b"\0user\0password"), "AHVzZXIAcGFzc3dvcmQ=");
    }
}
