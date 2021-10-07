// TODO: We need to determine which fields, structs and enums need to be public.
#[derive(Debug)]
pub enum ConnectionStatus {
    Succeeded,
    Failed,
}

#[derive(Debug)]
pub struct AirbyteConnectionStatus {
    pub status: ConnectionStatus,
    pub message: String,
}

// pub struct AirbyteCatalog<T> {
//     streams: Vec<T>,
// }

// pub struct AirbyteRecordMessage<T> {
//     stream: String,
//     data: Vec<T>,
//     emitted_at: u32,
// }
