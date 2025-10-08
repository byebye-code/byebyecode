use std::io::{self, Write};

pub struct IoInterceptor {
    pub buffer: Vec<String>,
}

impl Default for IoInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl IoInterceptor {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        self.buffer.push(line.clone());
        Ok(line)
    }

    pub fn write_line(&self, line: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.write_all(line.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
        Ok(())
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.buffer
    }
}
