#[derive(Default)]
pub struct KeyValueStore {
    //todo... dictionary structure
}

pub enum StorageValueType {
    String,
    Integer,
    Flag,
}

pub trait KeyValueStoreIO {
    fn export<'a>(&self) -> &'a [u8];
    fn import(&mut self,data: &[u8]);
}

impl KeyValueStoreIO for KeyValueStore {
    fn export<'a>(&self) -> &'a [u8] {
        todo!();
    }
    fn import(&mut self,data: &[u8]) {
        self.delete_all();
        todo!();
    }
}

pub enum KeyValueStoreError {

}

impl KeyValueStore {
    pub fn delete_all(&mut self) {
        todo!()
    }

    pub fn set_string(&mut self,key: &'static str,value: &'static str) {
        todo!()
    }

    pub fn set_u32(&mut self,key: &'static str,value: u32) {
        todo!()
    }

    pub fn set_flag(&mut self,key: &'static str) {
        todo!()
    }

    pub fn delete(&mut self,key: &'static str) -> Result<(),KeyValueStoreError> {
        todo!()
    }

    pub fn get_string(&self,key: &'static str) -> Result<(),&'static str> {
        todo!()
    }

    pub fn get_u32(&self,key: &'static str) -> Result<u32,KeyValueStoreError> {
        todo!()
    }

    pub fn has_key(&self,key: &'static str) -> bool {
        todo!()
    }
}
