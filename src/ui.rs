


pub trait Ui: Clone + Send {
    fn dispatch_message(&self, message: Message);
}




pub enum Message {

    MsfsConnected,
    MsfsDisconnected,

    EuroscopeConnected(String),
    EuroscopeDisconnected,
    
    MetarsRetrieved,
    MetarsDisconnected,

    MetarNotFound,
    MetarRetrieved(String),

    VatsimDataRetrieved,
    VatsimDataDisconnected,

}