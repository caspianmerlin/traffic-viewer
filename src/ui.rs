


pub trait Ui: Clone + Send {
    fn dispatch_message(&self, message: Message);
}


pub enum Message {

}