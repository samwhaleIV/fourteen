use crate::app::WimpyIO;

#[derive(Default)]
pub struct KeyValueStore {
    //todo... dictionary structure
}

pub enum StorageValueType {
    String,
    Integer,
    Flag,
}

pub enum KeyValueStoreError {

}

impl KeyValueStore {
    pub fn delete_all(&mut self) {
        todo!()
    }

    pub fn set_string(&mut self,_key: &'static str,_value: &'static str) {
        todo!()
    }

    pub fn set_u32(&mut self,_key: &'static str,_value: u32) {
        todo!()
    }

    pub fn set_flag(&mut self,_key: &'static str) {
        todo!()
    }

    pub fn delete(&mut self,_key: &'static str) -> Result<(),KeyValueStoreError> {
        todo!()
    }

    pub fn get_string(&self,_key: &'static str) -> Result<(),&'static str> {
        todo!()
    }

    pub fn get_u32(&self,_key: &'static str) -> Result<u32,KeyValueStoreError> {
        todo!()
    }

    pub fn has_key(&self,_key: &'static str) -> bool {
        todo!()
    }

    pub async fn import<IO: WimpyIO>(&mut self) {
        todo!();
    }

    pub async fn export<IO: WimpyIO>(&self) {
        todo!();
    }
}
