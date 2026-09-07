use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const PISUGAR_HOST: &str = "127.0.0.1";
const PISUGAR_PORT: u16  = 8423;
const CACHE_SECS:   u64  = 30; // only query every 30 seconds

pub struct Battery {
    percentage:  Option<f32>,
    charging:    bool,
    last_update: Option<Instant>,
}

impl Battery {
    pub fn new() -> Self {
        Battery {
            percentage:  None,
            charging:    false,
            last_update: None,
        }
    }

    /// Get battery percentage — returns cached value if recent
    pub fn percentage(&mut self) -> Option<f32> {
        self.update_if_needed();
        self.percentage
    }

    /// Is the battery charging?
    pub fn is_charging(&mut self) -> bool {
        self.update_if_needed();
        self.charging
    }

    /// Format battery as string for display
    pub fn status_string(&mut self) -> String {
        self.update_if_needed();
        match self.percentage {
            Some(pct) => {
                let icon = if self.charging { "CHG" } else { "BAT" };
                format!("{} {:.0}%", icon, pct)
            }
            None => String::from("BAT N/A"),
        }
    }

    fn update_if_needed(&mut self) {
        let should_update = match self.last_update {
            None    => true,
            Some(t) => t.elapsed() > Duration::from_secs(CACHE_SECS),
        };

        if should_update {
            self.fetch();
            self.last_update = Some(Instant::now());
        }
    }

    fn fetch(&mut self) {
        // Query PiSugar server via TCP
        if let Ok(pct) = self.query("get battery") {
            if let Ok(val) = pct.trim().parse::<f32>() {
                self.percentage = Some(val);
            }
        }

        if let Ok(charging) = self.query("get battery_charging") {
            self.charging = charging.trim() == "true";
        }
    }

    fn query(&self, command: &str) -> Result<String, std::io::Error> {
        let addr    = format!("{}:{}", PISUGAR_HOST, PISUGAR_PORT);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(500),
        )?;

        stream.write_all(format!("{}\n", command).as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        // Response format: "battery: 75.5"
        // Extract just the value after the colon
        let value = response
            .split(':')
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(value)
    }
}