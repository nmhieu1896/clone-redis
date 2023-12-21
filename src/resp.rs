use bytes::BytesMut;
use tokio::net::TcpStream;

pub enum Value {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Value>),
}

impl Value {
    pub fn serialize(self) -> String {
        match &self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            _ => panic!("Not implemented"),
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> RespHandler {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Value> {}

    pub async fn write_value(&mut self, value: Value) -> Result<()> {}
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '$' => parse_bulk_string(buffer),
        '*' => parse_array(buffer),
        _ => Err(anyhow::anyhow!("Invalid value type")),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    match read_until_crlf(&buffer[1..]) {
        Some((line, idx)) => {
            let string = String::from_utf8(line.to_vec()).unwrap();
            return Ok((Value::SimpleString(string), idx + 1));
        }
        None => Err(anyhow::anyhow!("Invalid simple string")),
    }
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (arr_length, bytes_consumed) = if let Some((line, idx)) = read_until_crlf(&buffer[1..]) {
        let arr_length = parse_int(line)?;
        (arr_length, idx + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array length"));
    };

    let mut items = vec![];
    for _ in 0..arr_length {
        let (arr_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(arr_item);
        bytes_consumed += len;
    }

    return Ok((Value::Array(items), bytes_consumed));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_string_length, bytes_consumed) =
        if let Some((line, idx)) = read_until_crlf(&buffer[1..]) {
            let bulk_string_length = parse_int(line)?;
            (bulk_string_length, idx + 1)
        } else {
            return Err(anyhow::anyhow!("Invalid array length"));
        };

    let mut items = vec![];
    for _ in 0..bulk_string_length {
        let (arr_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(arr_item);
        bytes_consumed += len;
    }

    return Ok((Value::Array(items), bytes_consumed));
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[..i - 1], i + 1));
        }
    }

    return None;
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    String::from_utf8(buffer.to_vec())?.parse::<i64>()
}
