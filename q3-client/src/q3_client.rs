use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::net::UdpSocket;
use std::time::Duration;

pub struct Q3Client {
    hostname: String,
    options: Q3ClientOptions,
}

pub struct Q3ClientOptions {
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub clean_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    pub keys: HashMap<String, String>,
    pub players: Vec<Player>,
}

impl Q3Client {
    pub fn new(hostname: String) -> Self {
        return Q3Client::new_with_options(
            hostname,
            Q3ClientOptions {
                read_timeout: Duration::from_secs(5),
                write_timeout: Duration::from_secs(5),
            },
        );
    }

    pub fn new_with_options(hostname: String, options: Q3ClientOptions) -> Self {
        Self { hostname, options }
    }

    pub fn get_status(self: &Self) -> Result<ServerStatus, Box<dyn Error>> {
        let mut status = ServerStatus {
            keys: HashMap::new(),
            players: Vec::new(),
        };

        let socket = UdpSocket::bind("0.0.0.0:0");
        if let Err(e) = socket {
            return Err(e.into());
        }

        let socket = socket.unwrap();
        let timeout_result = socket.set_read_timeout(Some(self.options.read_timeout));
        if let Err(e) = timeout_result {
            return Err(e.into());
        }
        let timeout_result = socket.set_write_timeout(Some(self.options.write_timeout));
        if let Err(e) = timeout_result {
            return Err(e.into());
        }

        // join 0xff, 0xff, 0xff, 0xff and getstatus (as string)
        let prefix: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
        let getstatus = String::from("getstatus");
        let buf = [&prefix, getstatus.as_bytes()].concat();

        let send_result = socket.send_to(&buf, &self.hostname);
        if let Err(e) = send_result {
            return Err(e.into());
        }

        let mut buf: [u8; 1024] = [0; 1024];
        let receive_result = socket.recv_from(&mut buf);
        if let Err(e) = receive_result {
            return Err(e.into());
        }

        let (bytes_read, _) = receive_result.unwrap();
        let response = String::from_utf8_lossy(&buf[..bytes_read]);

        let rows = response.split("\n").collect::<Vec<&str>>();

        let keys = rows[1].split("\\").skip(1).collect::<Vec<&str>>();
        let mut current_key = keys[1].to_string();
        for val in &keys[1..] {
            if current_key == "" {
                current_key = val.to_string();
                continue;
            }

            status.keys.insert(current_key, val.to_string());
            current_key = String::from("");
        }

        let players = rows[2..rows.len() - 1]
            .iter()
            .map(|row| Q3Client::parse_player_name(row))
            .collect::<Vec<String>>();

        status.players = players
            .iter()
            .map(|player| Player {
                name: player.to_string(),
                clean_name: string_utils::sanitize_string(player),
            })
            .collect::<Vec<Player>>();

        return Ok(status);
    }

    fn parse_player_name(get_status_player: &str) -> String {
        return get_status_player.split("\"").collect::<Vec<&str>>()[1].to_string();
    }
}

mod string_utils {
    // sanitize_string removes all color codes from a original string
    pub fn sanitize_string(orig_string: &str) -> String {
        let mut cleaned_string = String::from("");

        let mut i = 0;
        while i < orig_string.len() {
            let c = orig_string.chars().nth(i).unwrap();
            if c == '^' {
                if orig_string.chars().nth(i + 1) == Some('^') {
                    cleaned_string += "^";
                    i += 1;
                    continue;
                }
                i += 2;
                continue;
            }

            cleaned_string += &c.to_string();
            i += 1;
        }

        return cleaned_string;
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_sanitize_string() {
            let player_name = "^1Player^7Name";
            let expected = "PlayerName";

            assert_eq!(sanitize_string(player_name), expected);
        }

        #[test]
        fn test_sanitize_string_with_double_caret() {
            let player_name = "^1Player^^7Name";
            let expected = "Player^Name";

            assert_eq!(sanitize_string(player_name), expected);
        }

        #[test]
        fn test_sanitize_string_with_no_caret() {
            let player_name = "PlayerName";
            let expected = "PlayerName";

            assert_eq!(sanitize_string(player_name), expected);
        }

        #[test]
        fn test_sanitize_string_with_triple_caret() {
            let player_name = "^1Player^^^7Name";
            let expected = "Player^^Name";

            assert_eq!(sanitize_string(player_name), expected);
        }
    }
}
