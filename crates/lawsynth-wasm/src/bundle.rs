use crate::{Event, EventDirection, Expression, WasmError, World};

const MAGIC: &[u8; 6] = b"LSWASM";
const VERSION: u8 = 1;

/// Compact, versioned native bundle encoding for portable worlds and events.
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    pub world: World,
    pub events: Vec<Event>,
}
impl Bundle {
    pub fn new(world: World, events: Vec<Event>) -> Result<Self, WasmError> {
        for event in &events {
            if events
                .iter()
                .filter(|other| other.name == event.name)
                .count()
                != 1
            {
                return Err(WasmError::InvalidBundle(format!(
                    "duplicate event {}",
                    event.name
                )));
            }
        }
        Ok(Self { world, events })
    }
    pub fn encode(&self) -> Result<Vec<u8>, WasmError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.push(VERSION);
        put_u32(&mut bytes, self.world.variables.len())?;
        for index in 0..self.world.variables.len() {
            put_string(&mut bytes, &self.world.variables[index])?;
            bytes.extend_from_slice(&self.world.initial_state[index].to_le_bytes());
            put_string(&mut bytes, &self.world.derivatives[index].source())?;
        }
        put_u32(&mut bytes, self.events.len())?;
        for event in &self.events {
            put_string(&mut bytes, &event.name)?;
            bytes.push(match event.direction {
                EventDirection::Any => 0,
                EventDirection::Rising => 1,
                EventDirection::Falling => 2,
            });
            put_string(&mut bytes, &event.condition.source())?;
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, WasmError> {
        let mut reader = Reader { bytes, at: 0 };
        if reader.take(6)? != MAGIC {
            return Err(WasmError::InvalidBundle("invalid bundle magic".into()));
        }
        if reader.byte()? != VERSION {
            return Err(WasmError::InvalidBundle(
                "unsupported bundle version".into(),
            ));
        }
        let variables_len = reader.u32()? as usize;
        if variables_len == 0 || variables_len > 100_000 {
            return Err(WasmError::InvalidBundle("invalid variable count".into()));
        }
        let mut variables = Vec::with_capacity(variables_len);
        let mut state = Vec::with_capacity(variables_len);
        let mut derivatives = Vec::with_capacity(variables_len);
        for _ in 0..variables_len {
            variables.push(reader.string()?);
            state.push(f64::from_le_bytes(reader.array()?));
            derivatives.push(Expression::parse(&reader.string()?)?);
        }
        let world = World::new(variables, state, derivatives)?;
        let events_len = reader.u32()? as usize;
        if events_len > 100_000 {
            return Err(WasmError::InvalidBundle("invalid event count".into()));
        }
        let mut events = Vec::with_capacity(events_len);
        for _ in 0..events_len {
            let name = reader.string()?;
            let direction = match reader.byte()? {
                0 => EventDirection::Any,
                1 => EventDirection::Rising,
                2 => EventDirection::Falling,
                _ => return Err(WasmError::InvalidBundle("invalid event direction".into())),
            };
            events.push(Event::new(
                name,
                Expression::parse(&reader.string()?)?,
                direction,
            )?);
        }
        if reader.at != bytes.len() {
            return Err(WasmError::InvalidBundle("trailing bundle data".into()));
        }
        Self::new(world, events)
    }
}
fn put_u32(out: &mut Vec<u8>, value: usize) -> Result<(), WasmError> {
    let value = u32::try_from(value)
        .map_err(|_| WasmError::InvalidBundle("bundle length exceeds u32".into()))?;
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}
fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), WasmError> {
    put_u32(out, value.len())?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], WasmError> {
        let end = self
            .at
            .checked_add(length)
            .ok_or_else(|| WasmError::InvalidBundle("integer overflow".into()))?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| WasmError::InvalidBundle("truncated bundle".into()))?;
        self.at = end;
        Ok(slice)
    }
    fn byte(&mut self) -> Result<u8, WasmError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, WasmError> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], WasmError> {
        self.take(N)?
            .try_into()
            .map_err(|_| WasmError::InvalidBundle("invalid fixed-width bundle field".into()))
    }
    fn string(&mut self) -> Result<String, WasmError> {
        let length = self.u32()? as usize;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| WasmError::InvalidBundle("invalid UTF-8".into()))
    }
}
