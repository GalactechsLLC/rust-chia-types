pub enum CLVMType {
    Bytes(Vec<u8>),
    Hex(String),
    String(String),
    G1(G1Affine),
    Null,
}
